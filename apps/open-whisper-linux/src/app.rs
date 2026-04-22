//! Application lifecycle — startup & activate handlers.

use std::time::{Duration, Instant};

use adw::prelude::*;
use gio::SimpleAction;
use glib::ControlFlow;

use crate::bridge;
use crate::i18n::tr;
use crate::state::{AppSnapshot, AppState};
use crate::ui::main_window;

/// Runtime poll cadence. macOS polls at roughly 1 s — runtime flags change
/// only on user input (recording toggle, hotkey fire) so this is plenty,
/// and leaves the GTK main loop with headroom to service Wayland pings.
const RUNTIME_POLL: Duration = Duration::from_millis(1000);

/// Model status updates at download time (multi-second granularity) so we
/// poll less aggressively and save the filesystem checks it performs.
const MODEL_POLL: Duration = Duration::from_millis(3000);

/// Any single bridge call taking longer than this logs a warning — this is
/// the threshold at which the main loop starts to feel sluggish.
const SLOW_BRIDGE_THRESHOLD_MS: u128 = 100;

pub fn on_startup(app: &adw::Application) {
    let startup_started = Instant::now();
    tracing::info!(app_id = %app.application_id().unwrap_or_default(), "startup begin");

    // First bridge call lazily initialises the thread-local runtime — time
    // it so we notice when ALSA enumeration or hotkey portal registration
    // regresses. Diagnostics are not used in Stage 1 and are deliberately
    // excluded from startup to avoid blocking on audio-device probing.
    let settings = timed_bridge("load_settings", bridge::load_settings);
    let runtime = timed_bridge("runtime_status", bridge::runtime_status);
    let model = timed_bridge("model_status", bridge::model_status);

    let initial = AppSnapshot {
        settings,
        runtime,
        model,
        diagnostics: Default::default(),
    };
    let state = AppState::new(initial);

    // Runtime poll — fast-changing flags (recording, status text, hotkey
    // binding). Wrapped in `timed_bridge` so a regression shows up in logs.
    let runtime_state = state.clone();
    glib::timeout_add_local(RUNTIME_POLL, move || {
        let rt = timed_bridge("runtime_status", bridge::runtime_status);
        runtime_state.update(|snapshot| snapshot.runtime = rt);
        ControlFlow::Continue
    });

    // Model poll — slower-changing (only meaningful during an active
    // download) so we spare the filesystem checks it performs.
    let model_state = state.clone();
    glib::timeout_add_local(MODEL_POLL, move || {
        let m = timed_bridge("model_status", bridge::model_status);
        model_state.update(|snapshot| snapshot.model = m);
        ControlFlow::Continue
    });

    // Store the state on the Application using its GObject data slot so
    // activate handlers can retrieve it. `set_data` is unsafe because it
    // type-erases; we balance that with a single getter in `app_state()`.
    unsafe {
        app.set_data("open-whisper-state", state);
    }

    install_actions(app);

    // Linux tray is **opt-in** for now: `ksni` 0.2 integration deadlocks the
    // GTK main thread on some GNOME setups (confirmed reproducible on
    // AnduinOS / Ubuntu 25.10 + NVIDIA), even with our StatusNotifierWatcher
    // probe guarding the spawn. Stage 5 replaces the bridge with a proper
    // portal-aware tray; until then, `OW_ENABLE_TRAY=1` re-enables it for
    // desktops that are known to work (KDE, Xfce, Cinnamon, Budgie, MATE).
    #[cfg(target_os = "linux")]
    if std::env::var_os("OW_ENABLE_TRAY").is_some() {
        tracing::info!("tray enabled via OW_ENABLE_TRAY; spawning");
        crate::tray::spawn(app.clone(), app_state(app));
        tracing::info!("tray spawn returned");
    } else {
        tracing::debug!("tray disabled by default; set OW_ENABLE_TRAY=1 to opt in");
    }

    // On Wayland the bridge skips its built-in hotkey binding; drive it
    // via the XDG GlobalShortcuts portal instead. Installer returns fast
    // — the portal loop runs as a local task on the GLib main context.
    crate::hotkey::install(app_state(app));

    tracing::info!(
        elapsed_ms = startup_started.elapsed().as_millis() as u64,
        "on_startup complete"
    );
}

pub fn on_activate(app: &adw::Application) {
    // Re-use the same window if the user reopens from the tray / secondary
    // invocation. GNOME recommends a single primary window per app.
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }

    let state = app_state(app);
    let window = main_window::build(app, state);
    window.present();
}

/// Handle a secondary invocation relayed to the primary instance.
///
/// With `ApplicationFlags::HANDLES_COMMAND_LINE` set, GIO dispatches
/// every invocation through this signal. When a user triggers a GNOME
/// custom keyboard shortcut that runs
/// `open-whisper-linux --dictate-toggle`, GIO:
///
///  1. Sees that a primary instance already holds `APP_ID` on the bus.
///  2. Forwards the command-line to the primary over D-Bus.
///  3. The secondary process exits immediately (with the value we
///     return here).
///
/// That's the workaround for GNOME still stubbing out
/// `GlobalShortcuts.BindShortcuts`. Desktop environments that handle
/// portals properly (KDE/Plasma) don't need this path.
pub fn on_command_line(
    app: &adw::Application,
    cmd_line: &gio::ApplicationCommandLine,
) -> i32 {
    let args: Vec<String> = cmd_line
        .arguments()
        .into_iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();

    let mut matched = false;
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--dictate-toggle" => {
                tracing::info!("CLI: --dictate-toggle");
                log_bridge_result("toggle", bridge::hotkey_external_triggered());
                matched = true;
            }
            "--dictate-start" => {
                tracing::info!("CLI: --dictate-start");
                log_bridge_result("start", bridge::start_dictation());
                matched = true;
            }
            "--dictate-stop" => {
                tracing::info!("CLI: --dictate-stop");
                log_bridge_result("stop", bridge::stop_dictation());
                matched = true;
            }
            "--help" | "-h" => {
                print_help();
                return 0;
            }
            other if other.starts_with("--") => {
                tracing::warn!(flag = %other, "CLI: unknown flag, ignored");
            }
            _ => {}
        }
    }

    if !matched {
        app.activate();
    }
    0
}

fn log_bridge_result(label: &str, outcome: Result<String, String>) {
    match outcome {
        Ok(msg) => tracing::info!(action = label, message = %msg, "bridge action ok"),
        Err(err) => tracing::warn!(action = label, %err, "bridge action failed"),
    }
}

fn print_help() {
    eprintln!(
        "open-whisper-linux — local dictation shell\n\n\
         Flags:\n  \
         --dictate-toggle   Toggle dictation in the running instance\n  \
         --dictate-start    Start dictation in the running instance\n  \
         --dictate-stop     Stop dictation in the running instance\n  \
         --help, -h         Show this help\n\n\
         Bind any of the flags to a system-wide keyboard shortcut (e.g. GNOME\n\
         Settings → Keyboard → Custom Shortcuts) to drive dictation without\n\
         focusing the window. See docs/LINUX.md for details.\n"
    );
}

/// Retrieve the per-application shared state installed in `on_startup`.
/// Panics if called before startup — by design, since every UI entry point
/// lives downstream of the application lifecycle.
pub fn app_state(app: &adw::Application) -> AppState {
    unsafe {
        app.data::<AppState>("open-whisper-state")
            .expect("AppState must be installed during application startup")
            .as_ref()
            .clone()
    }
}

/// Install the `app.*` `gio::SimpleAction`s reachable from accelerators,
/// from the Help tab, and from the optional tray. `app.settings` is no
/// longer needed — settings live inside the main window, so any secondary
/// caller can simply `app.activate()` to bring the window forward.
fn install_actions(app: &adw::Application) {
    let restart_onboarding_action = SimpleAction::new("restart_onboarding", None);
    restart_onboarding_action.connect_activate(glib::clone!(
        #[weak]
        app,
        move |_, _| {
            show_placeholder_dialog(&app);
        }
    ));
    app.add_action(&restart_onboarding_action);

    let about_action = SimpleAction::new("about", None);
    about_action.connect_activate(glib::clone!(
        #[weak]
        app,
        move |_, _| {
            show_about_window(&app);
        }
    ));
    app.add_action(&about_action);

    let quit_action = SimpleAction::new("quit", None);
    quit_action.connect_activate(glib::clone!(
        #[weak]
        app,
        move |_, _| {
            app.quit();
        }
    ));
    app.add_action(&quit_action);
    app.set_accels_for_action("app.quit", &["<Primary>q"]);
}

fn show_about_window(app: &adw::Application) {
    let lang = app_state(app).with(|snap| snap.settings.ui_language);

    let about = adw::AboutWindow::builder()
        .application_name(tr("app.title", lang))
        .application_icon("audio-input-microphone-symbolic")
        .developer_name(tr("about.developer", lang))
        .version(env!("CARGO_PKG_VERSION"))
        .comments(tr("about.comments", lang))
        .license_type(gtk::License::MitX11)
        .build();
    if let Some(active) = app.active_window() {
        about.set_transient_for(Some(&active));
    }
    about.present();
}

/// Call `f` and log a warning if it exceeds `SLOW_BRIDGE_THRESHOLD_MS`.
/// Every bridge call in this process runs on the GTK main thread, so any
/// slow one translates directly into a missed Wayland ping and an "App not
/// responding" banner from the compositor. Keeping a trace makes the
/// offender visible in the logs.
fn timed_bridge<T>(label: &'static str, f: impl FnOnce() -> T) -> T {
    let started = Instant::now();
    let value = f();
    let elapsed_ms = started.elapsed().as_millis();
    if elapsed_ms > SLOW_BRIDGE_THRESHOLD_MS {
        tracing::warn!(bridge_call = label, elapsed_ms, "slow bridge call");
    } else {
        tracing::debug!(bridge_call = label, elapsed_ms, "bridge call");
    }
    value
}

fn show_placeholder_dialog(app: &adw::Application) {
    let lang = app_state(app).with(|snap| snap.settings.ui_language);
    let dialog = adw::MessageDialog::builder()
        .heading(tr("dialog.restart_onboarding.title", lang))
        .body(tr("dialog.restart_onboarding.body", lang))
        .default_response("close")
        .close_response("close")
        .build();
    dialog.add_response("close", &tr("dialog.close", lang));
    if let Some(active) = app.active_window() {
        dialog.set_transient_for(Some(&active));
    }
    dialog.present();
}
