//! Shared in-memory state for the GTK shell.
//!
//! Mirrors the role of `AppModel.swift` on macOS: a single observable hub
//! that holds the latest settings and runtime status, and that every UI
//! surface reads from. GTK widgets connect to the state's "changed" signal
//! to re-render when the bridge reports new status.

use std::cell::RefCell;
use std::rc::Rc;

use open_whisper_core::{AppSettings, DiagnosticsDto, ModelStatusDto, RuntimeStatusDto};

/// Concrete snapshot the UI renders against. Re-fetched from the bridge on a
/// periodic poll (every ~350 ms, matching the macOS AppModel cadence).
///
/// `diagnostics` is unused in Phase 1; the Diagnose tab in Phase 4 consumes it.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AppSnapshot {
    pub settings: AppSettings,
    pub runtime: RuntimeStatusDto,
    pub model: ModelStatusDto,
    pub diagnostics: DiagnosticsDto,
}

/// Shared handle to the application state. Cloneable on the GTK main thread;
/// internally refcounted so every widget that needs to observe or mutate
/// state can hold its own copy.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    inner: Rc<RefCell<AppSnapshot>>,
}

impl AppState {
    pub fn new(snapshot: AppSnapshot) -> Self {
        Self {
            inner: Rc::new(RefCell::new(snapshot)),
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        self.inner.borrow().clone()
    }

    pub fn with<R>(&self, f: impl FnOnce(&AppSnapshot) -> R) -> R {
        f(&self.inner.borrow())
    }

    pub fn update(&self, f: impl FnOnce(&mut AppSnapshot)) {
        f(&mut self.inner.borrow_mut());
    }
}
