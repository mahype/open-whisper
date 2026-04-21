//! Application lifecycle — startup & activate handlers.

use std::time::{Duration, Instant};

use adw::prelude::*;
use gio::SimpleAction;
use glib::ControlFlow;

use crate::bridge;
use crate::i18n::tr;
use crate::state::{AppSnapshot, AppState};
use crate::ui::{main_window, settings_window};

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

    // Linux tray attaches on startup and stays alive for the app lifetime.
    #[cfg(target_os = "linux")]
    crate::tray::spawn(app.clone(), app_state(app));
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

/// Install the `app.*` `gio::SimpleAction`s that the hamburger menu binds
/// to. Keeping action handling on the application (rather than the window)
/// means the tray menu — or future CLI hooks — can fire the same actions.
fn install_actions(app: &adw::Application) {
    let settings_action = SimpleAction::new("settings", None);
    settings_action.connect_activate(glib::clone!(
        #[weak]
        app,
        move |_, _| {
            open_settings(&app);
        }
    ));
    app.add_action(&settings_action);

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

fn open_settings(app: &adw::Application) {
    let lang = app_state(app).with(|snap| snap.settings.ui_language);

    // Focus an existing settings window rather than stacking duplicates.
    for window in app.windows() {
        if window.is::<adw::PreferencesWindow>() {
            window.present();
            return;
        }
    }

    let state = app_state(app);
    let window = settings_window::build(app, state);
    if let Some(active) = app.active_window() {
        window.set_transient_for(Some(&active));
    }
    window.set_title(Some(&tr("settings.window.title", lang)));
    window.present();
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
