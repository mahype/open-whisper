//! Application lifecycle — startup & activate handlers.

use std::time::Duration;

use adw::prelude::*;
use glib::ControlFlow;

use crate::bridge;
use crate::state::{AppSnapshot, AppState};
use crate::ui::main_window;

/// 350 ms matches the macOS AppModel polling cadence. Short enough for
/// the waveform/status to feel live, long enough to avoid waking the
/// bridge on every frame.
const POLL_INTERVAL: Duration = Duration::from_millis(350);

pub fn on_startup(app: &adw::Application) {
    tracing::info!(app_id = %app.application_id().unwrap_or_default(), "startup");

    // Hook the runtime into a single per-process state holder. Creating the
    // state here (rather than on every activate) guarantees that both the
    // tray (Linux only) and the window speak to the same snapshot.
    let initial = AppSnapshot {
        settings: bridge::load_settings(),
        runtime: bridge::runtime_status(),
        model: bridge::model_status(),
        diagnostics: bridge::diagnostics(),
    };
    let state = AppState::new(initial);

    // Drive the poll loop on the GTK main thread. All bridge calls must
    // happen here — the BridgeRuntime is thread-local.
    let poll_state = state.clone();
    glib::timeout_add_local(POLL_INTERVAL, move || {
        poll_state.update(|snapshot| {
            snapshot.runtime = bridge::runtime_status();
            snapshot.model = bridge::model_status();
        });
        ControlFlow::Continue
    });

    // Store the state on the Application using its GObject data slot so
    // activate handlers can retrieve it. `set_data` is unsafe because it
    // type-erases; we balance that with a single getter in `app_state()`.
    unsafe {
        app.set_data("open-whisper-state", state);
    }

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
