//! Main window.
//!
//! A single `adw::PreferencesWindow` hosts the entire shell: the Dashboard
//! is the first sidebar page and replaces what used to be a separate
//! hamburger-driven main view; the seven pages after it mirror the macOS
//! settings tabs. Quit stays on `Ctrl+Q` (installed as `app.quit`
//! accelerator in `app.rs`), About is reachable via the Help tab.

use adw::prelude::*;

use crate::i18n::tr;
use crate::state::AppState;
use crate::ui::settings;

pub fn build(app: &adw::Application, state: AppState) -> adw::PreferencesWindow {
    let lang = state.with(|snap| snap.settings.ui_language);

    let window = adw::PreferencesWindow::builder()
        .application(app)
        .title(tr("app.title", lang))
        .default_width(920)
        .default_height(640)
        .build();

    // Order in the sidebar == order of `add(...)`. Dashboard is the
    // intentional landing page — libadwaita selects the first added page
    // by default.
    window.add(&settings::dashboard::build(state.clone()));
    window.add(&settings::recording::build(state.clone()));
    window.add(&settings::placeholder_page(
        "post-processing",
        &tr("settings.tab.post_processing", lang),
        "text-editor-symbolic",
        lang,
    ));
    window.add(&settings::language_models::build(state.clone()));
    window.add(&settings::start_behavior::build(state.clone()));
    window.add(&settings::updates_page(lang));
    window.add(&settings::placeholder_page(
        "diagnostics",
        &tr("settings.tab.diagnostics", lang),
        "dialog-information-symbolic",
        lang,
    ));
    window.add(&settings::help::build(state));

    window
}
