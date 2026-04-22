//! Settings → *Dashboard* page (the default landing view).
//!
//! Keeps the macOS menu-bar-menu vocabulary — status, active mode, active
//! transcription model, registered hotkey, dictation toggle — but wrapped
//! in the same `PreferencesPage` chrome the other tabs use, so it sits
//! naturally at the top of the sidebar instead of living in a separate
//! window.

use adw::prelude::*;
use glib::clone;

use open_whisper_core::UiLanguage;

use crate::bridge;
use crate::i18n::tr;
use crate::state::{AppSnapshot, AppState};

const UI_REFRESH_MS: u64 = 500;

pub fn build(state: AppState) -> adw::PreferencesPage {
    let lang = state.with(|snap| snap.settings.ui_language);
    let initial = state.snapshot();

    let page = adw::PreferencesPage::builder()
        .title(tr("settings.tab.dashboard", lang))
        .icon_name("user-home-symbolic")
        .name("dashboard")
        .build();

    page.add(&status_group(&state, &initial, lang));
    page.add(&info_group(&state, &initial, lang));
    page.add(&actions_group(&state, lang));

    page
}

/// Big centered status badge. `PreferencesGroup` with no title/description
/// gives a nice padded card — same look as other groups without the
/// labeled header that would undermine the "at-a-glance" feel.
fn status_group(
    state: &AppState,
    initial: &AppSnapshot,
    lang: UiLanguage,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();

    let status_label = gtk::Label::builder()
        .label(derive_status(initial, lang))
        .halign(gtk::Align::Center)
        .wrap(true)
        .margin_top(24)
        .margin_bottom(24)
        .build();
    status_label.add_css_class("title-1");

    group.add(&status_label);

    // The dashboard has a live poll — re-reads AppState (cheap) on the same
    // cadence as the previous standalone main window so the status badge
    // reacts to `is_recording`/`is_transcribing` in near-real-time.
    glib::timeout_add_local(
        std::time::Duration::from_millis(UI_REFRESH_MS),
        clone!(
            #[weak]
            status_label,
            #[strong]
            state,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                let snap = state.snapshot();
                status_label.set_label(&derive_status(&snap, snap.settings.ui_language));
                glib::ControlFlow::Continue
            }
        ),
    );

    group
}

fn info_group(state: &AppState, initial: &AppSnapshot, lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.dashboard.current.title", lang))
        .description(tr("settings.dashboard.current.description", lang))
        .build();

    let mode_row = adw::ActionRow::builder()
        .title(tr("card.mode", lang))
        .subtitle(mode_subtitle(initial, lang))
        .build();
    group.add(&mode_row);

    let model_row = adw::ActionRow::builder()
        .title(tr("card.model", lang))
        .subtitle(model_subtitle(initial, lang))
        .build();
    group.add(&model_row);

    let hotkey_row = adw::ActionRow::builder()
        .title(tr("card.hotkey", lang))
        .subtitle(hotkey_subtitle(initial, lang))
        .build();
    group.add(&hotkey_row);

    // Keep the three cards in sync with runtime changes.
    glib::timeout_add_local(
        std::time::Duration::from_millis(UI_REFRESH_MS),
        clone!(
            #[weak]
            mode_row,
            #[weak]
            model_row,
            #[weak]
            hotkey_row,
            #[strong]
            state,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                let snap = state.snapshot();
                let lang = snap.settings.ui_language;
                mode_row.set_subtitle(&mode_subtitle(&snap, lang));
                model_row.set_subtitle(&model_subtitle(&snap, lang));
                hotkey_row.set_subtitle(&hotkey_subtitle(&snap, lang));
                glib::ControlFlow::Continue
            }
        ),
    );

    group
}

fn actions_group(state: &AppState, lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();

    let button = gtk::Button::builder()
        .label(dictate_label(&state.snapshot(), lang))
        .halign(gtk::Align::Center)
        .margin_top(12)
        .margin_bottom(12)
        .build();
    button.add_css_class("pill");
    button.add_css_class("suggested-action");

    button.connect_clicked(clone!(
        #[strong]
        state,
        move |_| {
            let currently_recording = state.with(|snap| snap.runtime.is_recording);
            let outcome = if currently_recording {
                bridge::stop_dictation()
            } else {
                bridge::start_dictation()
            };
            if let Err(err) = outcome {
                tracing::warn!(%err, "dictation toggle failed");
            }
        }
    ));

    // Keep the button label in sync — "Diktat starten" ↔ "Diktat stoppen".
    glib::timeout_add_local(
        std::time::Duration::from_millis(UI_REFRESH_MS),
        clone!(
            #[weak]
            button,
            #[strong]
            state,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                let snap = state.snapshot();
                button.set_label(&dictate_label(&snap, snap.settings.ui_language));
                glib::ControlFlow::Continue
            }
        ),
    );

    group.add(&button);
    group
}

fn derive_status(snap: &AppSnapshot, lang: UiLanguage) -> String {
    if snap.runtime.is_recording {
        tr("status.recording", lang)
    } else if snap.runtime.is_transcribing {
        tr("status.transcribing", lang)
    } else if snap.runtime.is_post_processing {
        tr("status.post_processing", lang)
    } else if snap.runtime.dictation_blocked_by_missing_model
        || (!snap.model.is_downloaded && !snap.model.preset_label.is_empty())
    {
        tr("status.model_loading", lang)
    } else {
        tr("status.ready", lang)
    }
}

fn dictate_label(snap: &AppSnapshot, lang: UiLanguage) -> String {
    if snap.runtime.is_recording {
        tr("button.stop_dictation", lang)
    } else {
        tr("button.start_dictation", lang)
    }
}

fn mode_subtitle(snap: &AppSnapshot, lang: UiLanguage) -> String {
    if snap.runtime.active_mode_name.is_empty() {
        tr("card.mode.default", lang)
    } else {
        snap.runtime.active_mode_name.clone()
    }
}

fn model_subtitle(snap: &AppSnapshot, lang: UiLanguage) -> String {
    if !snap.model.preset_label.is_empty() {
        snap.model.preset_label.clone()
    } else {
        tr("card.model.unknown", lang)
    }
}

fn hotkey_subtitle(snap: &AppSnapshot, lang: UiLanguage) -> String {
    let text = if !snap.runtime.hotkey_text.is_empty() {
        snap.runtime.hotkey_text.clone()
    } else {
        snap.settings.hotkey.clone()
    };
    if text.is_empty() {
        tr("card.hotkey.unset", lang)
    } else {
        text
    }
}
