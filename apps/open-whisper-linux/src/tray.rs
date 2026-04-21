//! StatusNotifierItem tray integration (Linux-only).
//!
//! Phase 2 replaces this stub with a real `ksni::Tray` implementation.
//! For now `spawn` is a no-op so `app.rs` can reference it unconditionally
//! on Linux; non-Linux platforms never call it (guarded by `cfg`).

#![allow(dead_code)]

#[cfg(target_os = "linux")]
pub fn spawn(_app: adw::Application, _state: crate::state::AppState) {
    tracing::debug!("tray::spawn placeholder — real ksni wiring arrives in Phase 2");
}
