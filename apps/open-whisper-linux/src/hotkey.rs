//! Global hotkey handling.
//!
//! On X11 the bridge binds the shortcut itself through the `global-hotkey`
//! crate. On Wayland that crate has no functional path (no X keygrab
//! available under a native compositor), so we route through the XDG
//! `org.freedesktop.portal.GlobalShortcuts` portal via `ashpd`.
//!
//! The portal is user-mediated: the desktop shell shows its own
//! confirmation dialog when we try to bind a shortcut, and delivers
//! activated/deactivated signals over D-Bus. We dispatch those into the
//! bridge through `hotkey_external_triggered` / `_released`, which
//! respects `TriggerMode::Toggle` vs. `TriggerMode::PushToTalk` for us.

#![allow(dead_code)]

#[cfg(target_os = "linux")]
pub fn is_wayland_session() -> bool {
    matches!(std::env::var("XDG_SESSION_TYPE").as_deref(), Ok("wayland"))
}

#[cfg(not(target_os = "linux"))]
pub fn is_wayland_session() -> bool {
    false
}

#[cfg(target_os = "linux")]
pub fn install(state: crate::state::AppState) {
    linux::install(state);
}

#[cfg(not(target_os = "linux"))]
pub fn install(_state: crate::state::AppState) {
    // no-op — macOS / Windows take a different path.
}

#[cfg(target_os = "linux")]
mod linux {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    use ashpd::desktop::Session;
    use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
    use futures_util::future::FutureExt;
    use futures_util::stream::StreamExt;

    use crate::bridge;
    use crate::state::AppState;

    /// Settings-change poll interval. The hotkey-recorder writes changes
    /// synchronously via `persist_settings`, but we pick them up by
    /// polling the shared state so there is no cross-crate signal wiring.
    const SETTINGS_POLL: Duration = Duration::from_secs(2);

    pub fn install(state: AppState) {
        if !super::is_wayland_session() {
            tracing::debug!("hotkey: not a Wayland session, portal path disabled");
            return;
        }

        // Entire portal conversation runs on a single GLib-main-context
        // task. `ashpd` with the `gtk4` feature uses the current GLib
        // context as its executor, so no separate Tokio runtime is needed.
        glib::MainContext::default().spawn_local(async move {
            if let Err(err) = run_portal(state).await {
                tracing::warn!(?err, "hotkey portal loop terminated");
            }
        });
    }

    async fn run_portal(state: AppState) -> Result<(), ashpd::Error> {
        let portal = match GlobalShortcuts::new().await {
            Ok(p) => p,
            Err(err) => {
                tracing::warn!(%err, "GlobalShortcuts portal unavailable");
                return Err(err);
            }
        };
        tracing::info!("hotkey: portal proxy connected");

        let session = portal.create_session().await?;
        tracing::info!("hotkey: portal session created");

        let current = Rc::new(RefCell::new(String::new()));
        let initial_hotkey = state.with(|s| s.settings.hotkey.clone());
        bind(&portal, &session, &initial_hotkey).await?;
        *current.borrow_mut() = initial_hotkey;

        let mut activated = portal.receive_activated().await?.boxed_local();
        let mut deactivated = portal.receive_deactivated().await?.boxed_local();

        loop {
            // `futures_util::select!` polls each future once per loop
            // iteration; whichever resolves first wins. We don't use
            // `select_biased!` — activation events and rebind checks are
            // equal priority.
            futures_util::select! {
                event = activated.next().fuse() => {
                    let Some(event) = event else { break };
                    tracing::info!(shortcut = %event.shortcut_id(), "portal activated");
                    if let Err(err) = bridge::hotkey_external_triggered() {
                        tracing::warn!(%err, "bridge::hotkey_external_triggered failed");
                    }
                }
                event = deactivated.next().fuse() => {
                    let Some(event) = event else { break };
                    tracing::debug!(shortcut = %event.shortcut_id(), "portal deactivated");
                    let _ = bridge::hotkey_external_released();
                }
                _ = glib::timeout_future(SETTINGS_POLL).fuse() => {
                    let latest = state.with(|s| s.settings.hotkey.clone());
                    if latest != *current.borrow() {
                        tracing::info!(
                            from = %*current.borrow(),
                            to = %latest,
                            "hotkey setting changed, rebinding",
                        );
                        match bind(&portal, &session, &latest).await {
                            Ok(()) => *current.borrow_mut() = latest,
                            Err(err) => tracing::warn!(%err, "portal rebind failed"),
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn bind<'a>(
        portal: &GlobalShortcuts<'a>,
        session: &Session<'a, GlobalShortcuts<'a>>,
        hotkey: &str,
    ) -> Result<(), ashpd::Error> {
        let trigger = to_portal_trigger(hotkey);
        let shortcut = NewShortcut::new("dictate_toggle", "Toggle dictation")
            .preferred_trigger(trigger.as_deref());

        let request = portal.bind_shortcuts(session, &[shortcut], None).await?;
        match request.response() {
            Ok(bound) => {
                let actual_trigger = bound
                    .shortcuts()
                    .first()
                    .map(|s| s.trigger_description().to_owned())
                    .unwrap_or_default();
                tracing::info!(
                    preferred = %hotkey,
                    bound = %actual_trigger,
                    "portal shortcut bound",
                );
                Ok(())
            }
            Err(err) => {
                // Common case on GNOME 49 and older: the session is
                // created successfully, but `BindShortcuts` on the
                // `org.gnome.Settings.GlobalShortcutsProvider` backend
                // is still a stub and returns `Response::Other`. KDE's
                // Plasma portal implements it fully, and the upstream
                // GNOME implementation is in progress. This message
                // surfaces the situation so the user doesn't think the
                // code is at fault.
                tracing::warn!(
                    %err,
                    "GlobalShortcuts portal rejected bind. On GNOME this is a known \
                     upstream limitation — BindShortcuts is not yet implemented by \
                     gnome-settings-daemon. Dictation can still be toggled from the \
                     main window until the upstream portal support lands."
                );
                Err(err)
            }
        }
    }

    /// Convert our internal hotkey text (e.g. `"Ctrl+Shift+Space"`) to the
    /// format the GlobalShortcuts portal expects (e.g. `"CTRL+SHIFT+space"`).
    /// Modifier labels are canonicalised to upper-case; the main key stays
    /// lower-case so `Space` (a GDK name) becomes `space`, matching the
    /// portal's documented convention.
    fn to_portal_trigger(hotkey: &str) -> Option<String> {
        let trimmed = hotkey.trim();
        if trimmed.is_empty() {
            return None;
        }
        let parts: Vec<String> = trimmed
            .split('+')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(|part| match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => "CTRL".to_owned(),
                "shift" => "SHIFT".to_owned(),
                "alt" => "ALT".to_owned(),
                // Portal nomenclature for the "Windows/Command" key.
                "super" | "meta" | "win" | "windows" => "LOGO".to_owned(),
                _ => part.to_ascii_lowercase(),
            })
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("+"))
        }
    }
}
