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

    // On Wayland `global-hotkey` can't bind a system-wide shortcut (it
    // only knows the X11 grab path). Tell the bridge to skip its
    // built-in registration so we can drive the hotkey through the XDG
    // GlobalShortcuts portal instead. This must run *before* any bridge
    // call — `BridgeRuntime::new` reads the env once during lazy init.
    if hotkey::is_wayland_session() && std::env::var_os("OW_EXTERNAL_HOTKEY").is_none() {
        // SAFETY: single-threaded context — `main` before any spawn.
        unsafe {
            std::env::set_var("OW_EXTERNAL_HOTKEY", "1");
        }
        tracing::info!("Wayland detected — hotkey will be bound via the GlobalShortcuts portal");
    }

    // libadwaita pulls in GTK and sets the theme/stylesheet. Must be called
    // before any widget lookup.
    adw::init()?;

    // When running inside a Flatpak sandbox the session-bus proxy refuses
    // to register arbitrary well-known names, so we skip the single-
    // instance handshake. On the real host we want the registration:
    //
    //  - The XDG GlobalShortcuts portal uses the D-Bus app_id derived
    //    from the well-known name to route `BindShortcuts`.
    //  - `HANDLES_COMMAND_LINE` turns the primary instance into an
    //    inbox for `--dictate-toggle` / `--dictate-start` /
    //    `--dictate-stop` launched by a GNOME custom shortcut; the
    //    secondary invocation is relayed via D-Bus so the user's
    //    keypress arrives at the running app without running whisper a
    //    second time.
    let flags = if std::env::var_os("FLATPAK_ID").is_some() {
        ApplicationFlags::NON_UNIQUE
    } else {
        ApplicationFlags::HANDLES_COMMAND_LINE
    };
    let application = adw::Application::builder()
        .application_id(APP_ID)
        .flags(flags)
        .build();

    application.connect_activate(app::on_activate);
    application.connect_startup(app::on_startup);
    application.connect_command_line(app::on_command_line);

    // gtk/gio converts the integer exit code; propagate a non-zero on failure.
    let exit_code = application.run();
    if exit_code.value() != 0 {
        std::process::exit(exit_code.value());
    }
    Ok(())
}
