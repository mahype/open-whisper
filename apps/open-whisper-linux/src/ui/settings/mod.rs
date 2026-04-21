//! Settings sub-pages.
//!
//! Each module here owns one tab of the `PreferencesWindow` and builds an
//! `adw::PreferencesPage` that mirrors the corresponding macOS section.
//! Stage 2 covers the simple tabs (Recording, Start & behavior, Help) —
//! hotkey recorder, mode editor, model manager, and diagnostics follow in
//! Stage 3.

pub mod help;
pub mod recording;
pub mod start_behavior;
