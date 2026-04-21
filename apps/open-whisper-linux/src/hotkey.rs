//! Global hotkey handling.
//!
//! Phase 3 will flesh this out: X11 sessions use the `global-hotkey`
//! crate directly, Wayland sessions route through the XDG
//! `org.freedesktop.portal.GlobalShortcuts` portal (via `ashpd`).
//! Phase 1 only needs the module to exist so `main.rs` compiles.

#![allow(dead_code)]

/// Runtime probe for the active session; used to pick between portal and
/// direct grab paths in Phase 3.
#[cfg(target_os = "linux")]
pub fn is_wayland_session() -> bool {
    matches!(std::env::var("XDG_SESSION_TYPE").as_deref(), Ok("wayland"))
}

#[cfg(not(target_os = "linux"))]
pub fn is_wayland_session() -> bool {
    false
}
