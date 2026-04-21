//! Open Whisper — GTK4/libadwaita shell for Linux.
//!
//! The Rust core (`open-whisper-bridge`) carries every stateful subsystem
//! (settings, dictation, whisper inference, post-processing). This binary
//! hosts the user-visible desktop shell: main window, settings window,
//! recording HUD, tray, and the wiring that drives them from the bridge.

mod app;
mod bridge;
mod hotkey;
mod i18n;
mod state;
mod tray;
mod ui;

use adw::prelude::*;
use anyhow::Result;
use gio::ApplicationFlags;
use tracing_subscriber::{EnvFilter, fmt};

const APP_ID: &str = "com.openwhisper.OpenWhisper";

fn main() -> Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    // libadwaita pulls in GTK and sets the theme/stylesheet. Must be called
    // before any widget lookup.
    adw::init()?;

    // NON_UNIQUE skips the single-instance bus-name registration. This is both
    // pragmatic (Flatpak-proxied session buses refuse to register arbitrary
    // well-known names) and matches the menu-bar-only UX: re-running the
    // binary spawns a fresh process rather than focussing an existing one.
    let application = adw::Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::NON_UNIQUE)
        .build();

    application.connect_activate(app::on_activate);
    application.connect_startup(app::on_startup);

    // gtk/gio converts the integer exit code; propagate a non-zero on failure.
    let exit_code = application.run();
    if exit_code.value() != 0 {
        std::process::exit(exit_code.value());
    }
    Ok(())
}
