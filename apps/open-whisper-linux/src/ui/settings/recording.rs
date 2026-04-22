//! Settings → *Recording* tab.
//!
//! Mirrors the macOS *Recording* pane: three grouped sections (audio
//! source, trigger, text output). Stage 2 wires the simple controls;
//! the hotkey recorder is deferred to Stage 3, so the hotkey row is
//! display-only for now and the actual edit flow still lives in the
//! header-bar menu's placeholder dialog.

use adw::prelude::*;
use glib::clone;

use open_whisper_core::{AppSettings, TriggerMode};

use crate::bridge;
use crate::i18n::tr;
use crate::state::AppState;
use crate::ui::settings::{hotkey_recorder, persist_settings};

pub fn build(state: AppState) -> adw::PreferencesPage {
    let lang = state.with(|snap| snap.settings.ui_language);

    let page = adw::PreferencesPage::builder()
        .title(tr("settings.tab.recording", lang))
        .icon_name("audio-input-microphone-symbolic")
        .name("recording")
        .build();

    page.add(&audio_source_group(&state, lang));
    page.add(&trigger_group(&state, lang));
    page.add(&text_output_group(&state, lang));

    page
}

fn audio_source_group(
    state: &AppState,
    lang: open_whisper_core::UiLanguage,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.recording.audio_source.title", lang))
        .description(tr("settings.recording.audio_source.description", lang))
        .build();

    group.add(&mic_row(state, lang));
    group.add(&language_row(state, lang));

    group
}

/// Microphone picker — lists input devices exposed by `cpal` / ALSA. Falls
/// back to a single "System default" entry if enumeration fails.
fn mic_row(state: &AppState, lang: open_whisper_core::UiLanguage) -> adw::ComboRow {
    let devices = bridge::list_input_devices();
    let current_name = state.with(|snap| snap.settings.input_device_name.clone());

    let model = gtk::StringList::new(&[]);
    let mut names: Vec<String> = Vec::with_capacity(devices.len().max(1));

    if devices.is_empty() {
        // cpal enumeration failed or no devices — still show a default entry
        // so the row remains interactive.
        let default_label = tr("settings.recording.mic.system_default", lang);
        model.append(&default_label);
        names.push("System Default".to_owned());
    } else {
        for device in &devices {
            model.append(&device.name);
            names.push(device.name.clone());
        }
    }

    // Find the stored device in the list; if missing, append a disabled-
    // looking entry preserving the user's configuration so a USB mic that's
    // currently unplugged doesn't silently reset to something else.
    let selected = names
        .iter()
        .position(|n| n == &current_name)
        .unwrap_or_else(|| {
            if !current_name.is_empty() {
                model.append(&current_name);
                names.push(current_name.clone());
                names.len() - 1
            } else {
                0
            }
        });

    let row = adw::ComboRow::builder()
        .title(tr("settings.recording.mic.title", lang))
        .model(&model)
        .selected(selected as u32)
        .build();

    row.connect_selected_notify(clone!(
        #[strong]
        state,
        #[strong]
        names,
        move |row| {
            let idx = row.selected() as usize;
            if let Some(name) = names.get(idx).cloned() {
                persist_settings(&state, |s: &mut AppSettings| s.input_device_name = name);
            }
        }
    ));

    row
}

/// Transcription language picker. Whisper auto-detects when set to "auto";
/// otherwise the ISO 639-1 code biases recognition toward that language.
fn language_row(state: &AppState, lang: open_whisper_core::UiLanguage) -> adw::ComboRow {
    const ENTRIES: &[(&str, &str)] = &[
        ("auto", "settings.recording.language.auto"),
        ("en", "settings.recording.language.en"),
        ("de", "settings.recording.language.de"),
        ("es", "settings.recording.language.es"),
        ("fr", "settings.recording.language.fr"),
        ("it", "settings.recording.language.it"),
        ("nl", "settings.recording.language.nl"),
        ("pt", "settings.recording.language.pt"),
        ("pl", "settings.recording.language.pl"),
        ("ja", "settings.recording.language.ja"),
        ("zh", "settings.recording.language.zh"),
    ];

    let model = gtk::StringList::new(&[]);
    for (_, label_key) in ENTRIES {
        model.append(&tr(label_key, lang));
    }

    let current = state.with(|snap| snap.settings.transcription_language.clone());
    let selected = ENTRIES
        .iter()
        .position(|(code, _)| *code == current.as_str())
        .unwrap_or(0);

    let row = adw::ComboRow::builder()
        .title(tr("settings.recording.language.title", lang))
        .model(&model)
        .selected(selected as u32)
        .build();

    row.connect_selected_notify(clone!(
        #[strong]
        state,
        move |row| {
            let idx = row.selected() as usize;
            if let Some((code, _)) = ENTRIES.get(idx) {
                let value = (*code).to_owned();
                persist_settings(&state, move |s: &mut AppSettings| {
                    s.transcription_language = value;
                });
            }
        }
    ));

    row
}

fn trigger_group(state: &AppState, lang: open_whisper_core::UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.recording.trigger.title", lang))
        .description(tr("settings.recording.trigger.description", lang))
        .build();

    group.add(&trigger_mode_row(state, lang));
    group.add(&hotkey_recorder::build(state.clone(), lang));

    group
}

fn trigger_mode_row(state: &AppState, lang: open_whisper_core::UiLanguage) -> adw::ComboRow {
    let model = gtk::StringList::new(&[]);
    // Display order matches macOS: Toggle first, then Push-to-talk.
    const ORDER: &[TriggerMode] = &[TriggerMode::Toggle, TriggerMode::PushToTalk];
    for variant in ORDER {
        let key = match variant {
            TriggerMode::Toggle => "settings.recording.trigger_mode.toggle",
            TriggerMode::PushToTalk => "settings.recording.trigger_mode.push_to_talk",
        };
        model.append(&tr(key, lang));
    }

    let current = state.with(|snap| snap.settings.trigger_mode);
    let selected = ORDER.iter().position(|m| *m == current).unwrap_or(0);

    let row = adw::ComboRow::builder()
        .title(tr("settings.recording.trigger_mode.title", lang))
        .model(&model)
        .selected(selected as u32)
        .build();

    row.connect_selected_notify(clone!(
        #[strong]
        state,
        move |row| {
            let idx = row.selected() as usize;
            if let Some(mode) = ORDER.get(idx).copied() {
                persist_settings(&state, move |s: &mut AppSettings| s.trigger_mode = mode);
            }
        }
    ));

    row
}

/// Hotkey display row. Read-only in Stage 2; Stage 3 replaces the subtitle
/// with a dedicated recorder widget hooked up to `bridge_api::validate_hotkey`.
fn text_output_group(
    state: &AppState,
    lang: open_whisper_core::UiLanguage,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.recording.text_output.title", lang))
        .description(tr("settings.recording.text_output.description", lang))
        .build();

    group.add(&insert_text_row(state, lang));
    group.add(&restore_clipboard_row(state, lang));
    group.add(&insert_delay_row(state, lang));

    group
}

fn insert_text_row(state: &AppState, lang: open_whisper_core::UiLanguage) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(tr("settings.recording.insert_text.title", lang))
        .subtitle(tr("settings.recording.insert_text.subtitle", lang))
        .active(state.with(|snap| snap.settings.insert_text_automatically))
        .build();

    row.connect_active_notify(clone!(
        #[strong]
        state,
        move |row| {
            let on = row.is_active();
            persist_settings(&state, move |s: &mut AppSettings| {
                s.insert_text_automatically = on;
            });
        }
    ));

    row
}

fn restore_clipboard_row(state: &AppState, lang: open_whisper_core::UiLanguage) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(tr("settings.recording.restore_clipboard.title", lang))
        .subtitle(tr("settings.recording.restore_clipboard.subtitle", lang))
        .active(state.with(|snap| snap.settings.restore_clipboard_after_insert))
        .build();

    row.connect_active_notify(clone!(
        #[strong]
        state,
        move |row| {
            let on = row.is_active();
            persist_settings(&state, move |s: &mut AppSettings| {
                s.restore_clipboard_after_insert = on;
            });
        }
    ));

    row
}

fn insert_delay_row(state: &AppState, lang: open_whisper_core::UiLanguage) -> adw::SpinRow {
    let current = state.with(|snap| snap.settings.insert_delay_ms);
    let adjustment = gtk::Adjustment::new(current as f64, 0.0, 2000.0, 10.0, 100.0, 0.0);

    let row = adw::SpinRow::builder()
        .title(tr("settings.recording.insert_delay.title", lang))
        .subtitle(tr("settings.recording.insert_delay.subtitle", lang))
        .adjustment(&adjustment)
        .numeric(true)
        .build();

    row.connect_value_notify(clone!(
        #[strong]
        state,
        move |row| {
            let value = row.value().round().clamp(0.0, u32::MAX as f64) as u32;
            persist_settings(&state, move |s: &mut AppSettings| s.insert_delay_ms = value);
        }
    ));

    row
}
