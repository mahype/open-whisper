//! Settings window (`adw::PreferencesWindow`).
//!
//! libadwaita's `PreferencesWindow` is the GTK-native equivalent of macOS's
//! tabbed `NSTabView`: a side-navigation of icon-and-title pages, each built
//! from `PreferencesGroup`s. The macOS app exposes seven tabs; we mirror that
//! structure here. Stage 2 ships real content for Recording, Start &
//! behavior, and Help — Post-processing, Language models, and Diagnostics
//! stay on the placeholder scaffold until Stage 3.

use adw::prelude::*;

use crate::bridge;
use crate::i18n::tr;
use crate::state::AppState;
use crate::ui::settings;

/// Build the preferences window and all seven pages.
pub fn build(app: &adw::Application, state: AppState) -> adw::PreferencesWindow {
    let lang = state.with(|snap| snap.settings.ui_language);

    let window = adw::PreferencesWindow::builder()
        .application(app)
        .title(tr("settings.window.title", lang))
        .default_width(820)
        .default_height(640)
        .modal(true)
        .build();

    window.add(&settings::recording::build(state.clone()));
    window.add(&stub_page(
        "post-processing",
        &tr("settings.tab.post_processing", lang),
        "text-editor-symbolic",
        lang,
    ));
    window.add(&stub_page(
        "language-models",
        &tr("settings.tab.language_models", lang),
        "folder-download-symbolic",
        lang,
    ));
    window.add(&settings::start_behavior::build(state.clone()));
    window.add(&updates_page(lang));
    window.add(&stub_page(
        "diagnostics",
        &tr("settings.tab.diagnostics", lang),
        "dialog-information-symbolic",
        lang,
    ));
    window.add(&settings::help::build(state));

    window
}

/// Placeholder page used for every tab whose real contents live in a later
/// stage. Keeps navigation structure stable so users see the full surface
/// area from day one.
fn stub_page(
    tag: &str,
    title: &str,
    icon_name: &str,
    lang: open_whisper_core::UiLanguage,
) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .title(title)
        .icon_name(icon_name)
        .name(tag)
        .build();

    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.placeholder.title", lang))
        .description(tr("settings.placeholder.body", lang))
        .build();
    page.add(&group);

    page
}

/// Updates page — on Linux we deliberately don't ship Sparkle; users receive
/// updates through their distribution channel. One static info card is all
/// the page needs.
fn updates_page(lang: open_whisper_core::UiLanguage) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .title(tr("settings.tab.updates", lang))
        .icon_name("software-update-available-symbolic")
        .name("updates")
        .build();

    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.updates.title", lang))
        .description(tr("settings.updates.body", lang))
        .build();
    page.add(&group);

    page
}

/// Persist a `mutate`-driven change to `AppSettings` through the bridge.
///
/// The tab modules use this whenever a switch, combo, or spin-row value
/// changes. Keeping the write path in one function means every edit goes
/// through `bridge::save_settings` — the same call macOS uses — so the two
/// shells stay in lockstep on settings-JSON semantics.
pub fn persist_settings<F>(state: &AppState, mutate: F)
where
    F: FnOnce(&mut open_whisper_core::AppSettings),
{
    let mut settings = state.with(|snap| snap.settings.clone());
    mutate(&mut settings);
    state.update(|snap| snap.settings = settings.clone());
    if let Err(err) = bridge::save_settings(settings) {
        tracing::warn!(%err, "failed to persist settings");
    }
}
