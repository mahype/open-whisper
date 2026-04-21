//! Settings sub-pages.
//!
//! Each module here owns one tab of the main `PreferencesWindow` and
//! builds an `adw::PreferencesPage`. The Dashboard is the landing
//! page — keeps the "at-a-glance" view macOS has in the menu-bar menu,
//! just inside the same sidebar the other pages live in.

pub mod dashboard;
pub mod help;
pub mod language_models;
pub mod recording;
pub mod start_behavior;

use adw::prelude::*;

use crate::bridge;
use crate::i18n::tr;
use crate::state::AppState;

/// Placeholder page used for every tab whose real contents live in a later
/// stage. Keeps navigation structure stable so the user sees the full
/// surface area from day one.
pub fn placeholder_page(
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

/// Updates page — on Linux we deliberately don't ship Sparkle; users
/// receive updates through their distribution channel. One static info
/// card is all the page needs.
pub fn updates_page(lang: open_whisper_core::UiLanguage) -> adw::PreferencesPage {
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
/// Every settings-row change handler funnels through here, so Linux and
/// macOS share the same `bridge::save_settings` contract.
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
