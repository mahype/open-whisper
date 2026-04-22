//! Settings → *Language models* tab.
//!
//! Mirrors the macOS pane: two groups (Whisper transcription model, Gemma
//! 4 post-processing LLM) each with a preset picker plus a **Manage…**
//! action that opens a modal sheet listing every preset with download /
//! delete buttons and live progress.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use glib::clone;

use open_whisper_core::{
    AppSettings, LlmModelStatusDto, LlmPreset, ModelPreset, ModelStatusDto, UiLanguage,
};

use crate::bridge;
use crate::i18n::tr;
use crate::state::AppState;
use crate::ui::settings::persist_settings;

/// Refresh cadence for the Manage sheet. Downloads emit progress every
/// few hundred ms; 500 ms keeps the bar smooth without hammering the
/// filesystem checks `model_status_list` performs.
const MANAGE_REFRESH_MS: u64 = 500;

pub fn build(state: AppState) -> adw::PreferencesPage {
    let lang = state.with(|snap| snap.settings.ui_language);

    let page = adw::PreferencesPage::builder()
        .title(tr("settings.tab.language_models", lang))
        .icon_name("folder-download-symbolic")
        .name("language-models")
        .build();

    page.add(&transcription_group(&state, lang));
    page.add(&post_processing_group(&state, lang));
    page.add(&advanced_group(&state, lang));

    page
}

// ---------------------------------------------------------------------------
// Transcription (Whisper)
// ---------------------------------------------------------------------------

fn transcription_group(state: &AppState, lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.models.transcription.title", lang))
        .description(tr("settings.models.transcription.description", lang))
        .build();

    group.add(&whisper_preset_row(state, lang));
    group.add(&manage_row(
        state,
        lang,
        tr("settings.models.manage.transcription.title", lang),
        ManageKind::Whisper,
    ));

    group
}

fn whisper_preset_row(state: &AppState, lang: UiLanguage) -> adw::ComboRow {
    let model = gtk::StringList::new(&[]);
    for preset in ModelPreset::ALL {
        model.append(preset.display_label());
    }

    let current = state.with(|snap| snap.settings.local_model);
    let selected = ModelPreset::ALL
        .iter()
        .position(|p| *p == current)
        .unwrap_or(0);

    let row = adw::ComboRow::builder()
        .title(tr("settings.models.active_preset", lang))
        .model(&model)
        .selected(selected as u32)
        .build();

    row.connect_selected_notify(clone!(
        #[strong]
        state,
        move |row| {
            let idx = row.selected() as usize;
            if let Some(preset) = ModelPreset::ALL.get(idx).copied() {
                persist_settings(&state, move |s: &mut AppSettings| s.local_model = preset);
            }
        }
    ));

    row
}

// ---------------------------------------------------------------------------
// Post-processing (Gemma 4)
// ---------------------------------------------------------------------------

fn post_processing_group(state: &AppState, lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.models.post_processing.title", lang))
        .description(tr("settings.models.post_processing.description", lang))
        .build();

    group.add(&llm_preset_row(state, lang));
    group.add(&manage_row(
        state,
        lang,
        tr("settings.models.manage.post_processing.title", lang),
        ManageKind::Llm,
    ));

    group
}

fn llm_preset_row(state: &AppState, lang: UiLanguage) -> adw::ComboRow {
    let model = gtk::StringList::new(&[]);
    for preset in LlmPreset::ALL {
        model.append(preset.display_label());
    }

    let current = state.with(|snap| snap.settings.local_llm);
    let selected = LlmPreset::ALL
        .iter()
        .position(|p| *p == current)
        .unwrap_or(0);

    let row = adw::ComboRow::builder()
        .title(tr("settings.models.active_preset", lang))
        .model(&model)
        .selected(selected as u32)
        .build();

    row.connect_selected_notify(clone!(
        #[strong]
        state,
        move |row| {
            let idx = row.selected() as usize;
            if let Some(preset) = LlmPreset::ALL.get(idx).copied() {
                persist_settings(&state, move |s: &mut AppSettings| s.local_llm = preset);
            }
        }
    ));

    row
}

// ---------------------------------------------------------------------------
// Advanced (LLM idle-unload)
// ---------------------------------------------------------------------------

fn advanced_group(state: &AppState, lang: UiLanguage) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("settings.models.advanced.title", lang))
        .description(tr("settings.models.advanced.description", lang))
        .build();

    let current = state.with(|snap| snap.settings.local_llm_auto_unload_secs);
    // 0..=3600 — 0 means "never", step by 30 s which matches the coarse
    // granularity the setting needs.
    let adjustment = gtk::Adjustment::new(current as f64, 0.0, 3600.0, 30.0, 60.0, 0.0);

    let row = adw::SpinRow::builder()
        .title(tr("settings.models.llm_unload.title", lang))
        .subtitle(tr("settings.models.llm_unload.subtitle", lang))
        .adjustment(&adjustment)
        .numeric(true)
        .build();

    row.connect_value_notify(clone!(
        #[strong]
        state,
        move |row| {
            let value = row.value().round().clamp(0.0, u32::MAX as f64) as u32;
            persist_settings(&state, move |s: &mut AppSettings| {
                s.local_llm_auto_unload_secs = value;
            });
        }
    ));

    group.add(&row);
    group
}

// ---------------------------------------------------------------------------
// Manage sheet — opened by the "Verwalten" action row.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum ManageKind {
    Whisper,
    Llm,
}

fn manage_row(
    state: &AppState,
    lang: UiLanguage,
    sheet_title: String,
    kind: ManageKind,
) -> adw::ActionRow {
    let button = gtk::Button::builder()
        .label(tr("settings.models.manage", lang))
        .valign(gtk::Align::Center)
        .build();
    button.add_css_class("pill");

    let row = adw::ActionRow::builder()
        .title(tr("settings.models.manage", lang))
        .subtitle(tr("settings.models.manage.subtitle", lang))
        .build();
    row.add_suffix(&button);
    row.set_activatable_widget(Some(&button));

    button.connect_clicked(clone!(
        #[strong]
        state,
        move |btn| {
            let parent = btn.root().and_downcast::<gtk::Window>();
            present_manage_sheet(state.clone(), parent.as_ref(), &sheet_title, kind);
        }
    ));

    row
}

fn present_manage_sheet(
    state: AppState,
    parent: Option<&gtk::Window>,
    title: &str,
    kind: ManageKind,
) {
    let lang = state.with(|snap| snap.settings.ui_language);

    let window = adw::Window::builder()
        .title(title)
        .default_width(560)
        .default_height(520)
        .modal(true)
        .build();
    if let Some(parent) = parent {
        window.set_transient_for(Some(parent));
    }

    let header = adw::HeaderBar::new();
    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();
    page.add(&group);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&page));
    window.set_content(Some(&toolbar));

    // Build one row per preset; keep handles in a shared vec so the refresh
    // tick can update subtitles / button labels in place.
    let rows: Rc<RefCell<Vec<ManagedRow>>> = Rc::new(RefCell::new(Vec::new()));

    match kind {
        ManageKind::Whisper => {
            for preset in ModelPreset::ALL {
                let row = build_whisper_row(&state, preset, lang);
                group.add(&row.row);
                rows.borrow_mut().push(row);
            }
        }
        ManageKind::Llm => {
            for preset in LlmPreset::ALL {
                let row = build_llm_row(&state, preset, lang);
                group.add(&row.row);
                rows.borrow_mut().push(row);
            }
        }
    }

    // Refresh on a timer. The closure holds a weak reference to the sheet
    // window so the timer auto-cancels when the user closes it.
    let weak_window = window.downgrade();
    glib::timeout_add_local(
        std::time::Duration::from_millis(MANAGE_REFRESH_MS),
        clone!(
            #[strong]
            rows,
            move || {
                if weak_window.upgrade().is_none() {
                    return glib::ControlFlow::Break;
                }
                refresh_rows(&rows, kind, lang);
                glib::ControlFlow::Continue
            }
        ),
    );

    // Seed the visible state before the first tick.
    refresh_rows(&rows, kind, lang);

    window.present();
}

/// Per-row handles kept alive for the refresh timer.
struct ManagedRow {
    row: adw::ActionRow,
    button: gtk::Button,
    /// Monotonic index into `ModelPreset::ALL` or `LlmPreset::ALL`; decouples
    /// the row from the enum type so the refresh loop can stay generic over
    /// `ManageKind`.
    preset_index: usize,
}

fn build_whisper_row(state: &AppState, preset: ModelPreset, lang: UiLanguage) -> ManagedRow {
    let preset_index = ModelPreset::ALL
        .iter()
        .position(|p| *p == preset)
        .expect("preset must be in ALL");

    let row = adw::ActionRow::builder()
        .title(preset.display_label())
        .subtitle(preset.description())
        .build();

    let button = gtk::Button::builder()
        .label(tr("settings.models.action.download", lang))
        .valign(gtk::Align::Center)
        .build();
    button.add_css_class("pill");

    button.connect_clicked(clone!(
        #[strong]
        state,
        move |btn| {
            // `is_downloaded()` on the DTO is only available after a bridge
            // round-trip, so we introspect the current button label — it
            // already reflects the row's rendered state.
            let label = btn.label().unwrap_or_default().to_string();
            let outcome = if label == tr("settings.models.action.delete", lang) {
                bridge::delete_model(Some(preset))
            } else if label == tr("settings.models.action.download", lang) {
                bridge::start_model_download(Some(preset))
            } else {
                return; // button disabled during "Running…" state
            };
            if let Err(err) = outcome {
                tracing::warn!(%err, "whisper action failed");
            }
            // Trigger a faster model poll so the user sees progress begin.
            let _ = bridge::model_status_list();
            let _ = state; // keep strong-ref alive
        }
    ));

    row.add_suffix(&button);

    ManagedRow {
        row,
        button,
        preset_index,
    }
}

fn build_llm_row(state: &AppState, preset: LlmPreset, lang: UiLanguage) -> ManagedRow {
    let preset_index = LlmPreset::ALL
        .iter()
        .position(|p| *p == preset)
        .expect("preset must be in ALL");

    let row = adw::ActionRow::builder()
        .title(preset.display_label())
        .subtitle(preset.description())
        .build();

    let button = gtk::Button::builder()
        .label(tr("settings.models.action.download", lang))
        .valign(gtk::Align::Center)
        .build();
    button.add_css_class("pill");

    button.connect_clicked(clone!(
        #[strong]
        state,
        move |btn| {
            let label = btn.label().unwrap_or_default().to_string();
            let outcome = if label == tr("settings.models.action.delete", lang) {
                bridge::delete_llm_model(preset)
            } else if label == tr("settings.models.action.download", lang) {
                bridge::start_llm_download(preset)
            } else {
                return;
            };
            if let Err(err) = outcome {
                tracing::warn!(%err, "llm action failed");
            }
            let _ = bridge::llm_status_list();
            let _ = state;
        }
    ));

    row.add_suffix(&button);

    ManagedRow {
        row,
        button,
        preset_index,
    }
}

fn refresh_rows(rows: &Rc<RefCell<Vec<ManagedRow>>>, kind: ManageKind, lang: UiLanguage) {
    match kind {
        ManageKind::Whisper => {
            let list = bridge::model_status_list();
            for managed in rows.borrow().iter() {
                if let Some(status) = list.get(managed.preset_index) {
                    apply_whisper_state(managed, status, lang);
                }
            }
        }
        ManageKind::Llm => {
            let list = bridge::llm_status_list();
            for managed in rows.borrow().iter() {
                if let Some(status) = list.get(managed.preset_index) {
                    apply_llm_state(managed, status, lang);
                }
            }
        }
    }
}

fn apply_whisper_state(managed: &ManagedRow, status: &ModelStatusDto, lang: UiLanguage) {
    if status.is_downloading {
        managed
            .row
            .set_subtitle(&downloading_subtitle(status.progress_basis_points, lang));
        managed
            .button
            .set_label(&tr("settings.models.action.downloading", lang));
        managed.button.set_sensitive(false);
    } else if status.is_downloaded {
        managed
            .row
            .set_subtitle(&tr("settings.models.state.ready", lang));
        managed
            .button
            .set_label(&tr("settings.models.action.delete", lang));
        managed.button.set_sensitive(true);
        managed.button.remove_css_class("suggested-action");
        managed.button.add_css_class("destructive-action");
    } else {
        managed
            .row
            .set_subtitle(&tr("settings.models.state.not_downloaded", lang));
        managed
            .button
            .set_label(&tr("settings.models.action.download", lang));
        managed.button.set_sensitive(true);
        managed.button.remove_css_class("destructive-action");
        managed.button.add_css_class("suggested-action");
    }
}

fn apply_llm_state(managed: &ManagedRow, status: &LlmModelStatusDto, lang: UiLanguage) {
    if status.is_downloading {
        managed
            .row
            .set_subtitle(&downloading_subtitle(status.progress_basis_points, lang));
        managed
            .button
            .set_label(&tr("settings.models.action.downloading", lang));
        managed.button.set_sensitive(false);
    } else if status.is_downloaded {
        managed
            .row
            .set_subtitle(&tr("settings.models.state.ready", lang));
        managed
            .button
            .set_label(&tr("settings.models.action.delete", lang));
        managed.button.set_sensitive(true);
        managed.button.remove_css_class("suggested-action");
        managed.button.add_css_class("destructive-action");
    } else {
        managed
            .row
            .set_subtitle(&tr("settings.models.state.not_downloaded", lang));
        managed
            .button
            .set_label(&tr("settings.models.action.download", lang));
        managed.button.set_sensitive(true);
        managed.button.remove_css_class("destructive-action");
        managed.button.add_css_class("suggested-action");
    }
}

fn downloading_subtitle(progress_basis_points: Option<u16>, lang: UiLanguage) -> String {
    let percent = progress_basis_points
        .map(|bp| (bp as f32 / 100.0).round() as u32)
        .unwrap_or(0);
    let template = tr("settings.models.state.progress_percent", lang);
    let filled = template.replace("{}", &percent.to_string());
    format!(
        "{} \u{2022} {}",
        tr("settings.models.state.downloading", lang),
        filled
    )
}
