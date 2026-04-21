//! Settings → *Start & behavior* tab.
//!
//! Ports macOS's third pane: how the app launches, how voice-activity
//! detection trims the tail of a recording, and a read-only summary of
//! runtime state so the user can confirm their current binding works.

use adw::prelude::*;
use glib::clone;

use open_whisper_core::{AppSettings, StartupBehavior, UiLanguage};

use crate::i18n::tr;
use crate::state::AppState;
use crate::ui::settings::persist_settings;

pub fn build(state: AppState) -> adw::PreferencesPage {
    let lang = state.with(|snap| snap.settings.ui_language);

    let page = adw::PreferencesPage::builder()
        .title(tr("settings.tab.start_behavior", lang))
        .icon_name("preferences-system-symbolic")
        .name("start-behavior")
        .build();

    page.add(&startup_group(&state, lang));
    page.add(&vad_group(&state, lang));
    page.add(&status_group(&state, lang));

    page
}

fn startup_group(state: &AppState, lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.start.startup.title", lang))
        .description(tr("settings.start.startup.description", lang))
        .build();

    group.add(&startup_behavior_row(state, lang));
    group.add(&app_language_row(state, lang));

    group
}

fn startup_behavior_row(state: &AppState, lang: UiLanguage) -> adw::ComboRow {
    const ORDER: &[StartupBehavior] = &[
        StartupBehavior::AskOnFirstLaunch,
        StartupBehavior::LaunchAtLogin,
        StartupBehavior::ManualLaunch,
    ];

    let model = gtk::StringList::new(&[]);
    for variant in ORDER {
        let key = match variant {
            StartupBehavior::AskOnFirstLaunch => "settings.start.startup_behavior.ask",
            StartupBehavior::LaunchAtLogin => "settings.start.startup_behavior.launch",
            StartupBehavior::ManualLaunch => "settings.start.startup_behavior.manual",
        };
        model.append(&tr(key, lang));
    }

    let current = state.with(|snap| snap.settings.startup_behavior);
    let selected = ORDER.iter().position(|b| *b == current).unwrap_or(0);

    let row = adw::ComboRow::builder()
        .title(tr("settings.start.startup_behavior.title", lang))
        .model(&model)
        .selected(selected as u32)
        .build();

    row.connect_selected_notify(clone!(
        #[strong]
        state,
        move |row| {
            let idx = row.selected() as usize;
            if let Some(behavior) = ORDER.get(idx).copied() {
                persist_settings(&state, move |s: &mut AppSettings| {
                    s.startup_behavior = behavior;
                });
            }
        }
    ));

    row
}

fn app_language_row(state: &AppState, lang: UiLanguage) -> adw::ComboRow {
    const ORDER: &[UiLanguage] = &[UiLanguage::System, UiLanguage::En, UiLanguage::De];

    let model = gtk::StringList::new(&[]);
    for variant in ORDER {
        let key = match variant {
            UiLanguage::System => "settings.start.app_language.system",
            UiLanguage::En => "settings.start.app_language.en",
            UiLanguage::De => "settings.start.app_language.de",
        };
        model.append(&tr(key, lang));
    }

    let current = state.with(|snap| snap.settings.ui_language);
    let selected = ORDER.iter().position(|u| *u == current).unwrap_or(0);

    let row = adw::ComboRow::builder()
        .title(tr("settings.start.app_language.title", lang))
        .model(&model)
        .selected(selected as u32)
        .build();

    row.connect_selected_notify(clone!(
        #[strong]
        state,
        move |row| {
            let idx = row.selected() as usize;
            if let Some(new_lang) = ORDER.get(idx).copied() {
                persist_settings(&state, move |s: &mut AppSettings| {
                    s.ui_language = new_lang;
                });
            }
        }
    ));

    row
}

fn vad_group(state: &AppState, lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.start.vad.title", lang))
        .description(tr("settings.start.vad.description", lang))
        .build();

    let silence_row = vad_silence_row(state, lang);
    let enabled_row = vad_enabled_row(state, lang, &silence_row);

    group.add(&enabled_row);
    group.add(&silence_row);

    group
}

fn vad_enabled_row(
    state: &AppState,
    lang: UiLanguage,
    silence_row: &adw::SpinRow,
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(tr("settings.start.vad_enabled.title", lang))
        .subtitle(tr("settings.start.vad_enabled.subtitle", lang))
        .active(state.with(|snap| snap.settings.vad_enabled))
        .build();

    // Disabled VAD makes the silence spin-row meaningless; mirror macOS by
    // greying it out rather than hiding it — keeps layout stable.
    silence_row.set_sensitive(row.is_active());

    row.connect_active_notify(clone!(
        #[strong]
        state,
        #[weak]
        silence_row,
        move |row| {
            let on = row.is_active();
            silence_row.set_sensitive(on);
            persist_settings(&state, move |s: &mut AppSettings| s.vad_enabled = on);
        }
    ));

    row
}

fn vad_silence_row(state: &AppState, lang: UiLanguage) -> adw::SpinRow {
    let current = state.with(|snap| snap.settings.vad_silence_ms);
    // macOS range: 300..=2500ms with 50ms step.
    let adjustment = gtk::Adjustment::new(current as f64, 300.0, 2500.0, 50.0, 100.0, 0.0);

    let row = adw::SpinRow::builder()
        .title(tr("settings.start.vad_silence.title", lang))
        .subtitle(tr("settings.start.vad_silence.subtitle", lang))
        .adjustment(&adjustment)
        .numeric(true)
        .build();

    row.connect_value_notify(clone!(
        #[strong]
        state,
        move |row| {
            let value = row.value().round().clamp(0.0, u32::MAX as f64) as u32;
            persist_settings(&state, move |s: &mut AppSettings| s.vad_silence_ms = value);
        }
    ));

    row
}

fn status_group(state: &AppState, lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.start.status.title", lang))
        .description(tr("settings.start.status.description", lang))
        .build();

    // Snapshot at build time. Good enough for Stage 2 — the macOS pane
    // also re-renders when the Settings sheet is opened rather than on a
    // continuous subscription.
    let snap = state.snapshot();

    let startup_row = adw::ActionRow::builder()
        .title(tr("settings.start.status.startup", lang))
        .subtitle(if snap.runtime.startup_summary.is_empty() {
            tr("card.model.unknown", lang)
        } else {
            snap.runtime.startup_summary.clone()
        })
        .build();

    let hotkey_row = adw::ActionRow::builder()
        .title(tr("settings.start.status.hotkey_registered", lang))
        .subtitle(if snap.runtime.hotkey_registered {
            tr("settings.start.status.yes", lang)
        } else {
            tr("settings.start.status.no", lang)
        })
        .build();

    let mode_row = adw::ActionRow::builder()
        .title(tr("settings.start.status.active_mode", lang))
        .subtitle(if snap.runtime.active_mode_name.is_empty() {
            tr("card.mode.default", lang)
        } else {
            snap.runtime.active_mode_name.clone()
        })
        .build();

    group.add(&startup_row);
    group.add(&hotkey_row);
    group.add(&mode_row);

    group
}
