//! Settings → *Help* tab.
//!
//! Mirrors the macOS *Help* pane: version and bundle identifier (read-only),
//! plus an action to re-run onboarding. Onboarding itself is a Stage 5
//! deliverable, so the button activates the existing `app.restart_onboarding`
//! action which currently shows a "coming later" placeholder dialog.

use adw::prelude::*;

use open_whisper_core::UiLanguage;

use crate::i18n::tr;
use crate::state::AppState;

const APP_ID: &str = "com.openwhisper.OpenWhisper";

pub fn build(state: AppState) -> adw::PreferencesPage {
    let lang = state.with(|snap| snap.settings.ui_language);

    let page = adw::PreferencesPage::builder()
        .title(tr("settings.tab.help", lang))
        .icon_name("help-about-symbolic")
        .name("help")
        .build();

    page.add(&about_group(lang));
    page.add(&actions_group(lang));

    page
}

fn about_group(lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.help.about.title", lang))
        .build();

    let version_row = adw::ActionRow::builder()
        .title(tr("settings.help.version.title", lang))
        .subtitle(env!("CARGO_PKG_VERSION"))
        .build();

    let bundle_row = adw::ActionRow::builder()
        .title(tr("settings.help.bundle_id.title", lang))
        .subtitle(APP_ID)
        .build();

    group.add(&version_row);
    group.add(&bundle_row);

    group
}

fn actions_group(lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.help.actions.title", lang))
        .build();

    // Action rows triggering existing `app.*` actions. Wiring via
    // action-name keeps the behaviour identical whether the user hits the
    // button here, an accelerator, or (future) a tray entry.
    let about_button = gtk::Button::builder()
        .label(tr("settings.help.about_action.button", lang))
        .valign(gtk::Align::Center)
        .action_name("app.about")
        .build();
    about_button.add_css_class("pill");

    let about_row = adw::ActionRow::builder()
        .title(tr("settings.help.about_action.title", lang))
        .subtitle(tr("settings.help.about_action.subtitle", lang))
        .build();
    about_row.add_suffix(&about_button);
    about_row.set_activatable_widget(Some(&about_button));

    let onboarding_button = gtk::Button::builder()
        .label(tr("settings.help.restart_onboarding.button", lang))
        .valign(gtk::Align::Center)
        .action_name("app.restart_onboarding")
        .build();
    onboarding_button.add_css_class("pill");

    let onboarding_row = adw::ActionRow::builder()
        .title(tr("settings.help.restart_onboarding.title", lang))
        .subtitle(tr("settings.help.restart_onboarding.subtitle", lang))
        .build();
    onboarding_row.add_suffix(&onboarding_button);
    onboarding_row.set_activatable_widget(Some(&onboarding_button));

    group.add(&about_row);
    group.add(&onboarding_row);

    group
}
