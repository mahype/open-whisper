//! Settings → *Diagnostics* tab.
//!
//! Two groups mirror the macOS Diagnostics pane:
//!
//! - **Overview** shows the top-level summary line from the bridge plus a
//!   *Refresh* button that re-runs `bridge::diagnostics()` on demand.
//! - **Details** lists every `DiagnosticItemDto` as an `adw::ActionRow`
//!   with a coloured status pill (OK / Info / Warning / Error). The
//!   problem and recommendation are stacked in a multi-line subtitle so
//!   the user sees the full context without a second click.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use glib::clone;

use open_whisper_core::{DiagnosticItemDto, DiagnosticStatus, UiLanguage};

use crate::bridge;
use crate::i18n::tr;
use crate::state::AppState;

pub fn build(state: AppState) -> adw::PreferencesPage {
    let lang = state.with(|snap| snap.settings.ui_language);

    let page = adw::PreferencesPage::builder()
        .title(tr("settings.tab.diagnostics", lang))
        .icon_name("system-run-symbolic")
        .name("diagnostics")
        .build();

    // Overview group: summary + refresh button on a single action row.
    let summary_row = adw::ActionRow::builder()
        .title(tr("settings.diagnostics.overview.title", lang))
        .subtitle(tr("settings.diagnostics.summary.unknown", lang))
        .subtitle_lines(4)
        .build();

    let refresh_button = gtk::Button::builder()
        .label(tr("settings.diagnostics.refresh", lang))
        .valign(gtk::Align::Center)
        .build();
    refresh_button.add_css_class("pill");
    refresh_button.add_css_class("suggested-action");
    summary_row.add_suffix(&refresh_button);

    let overview_group = adw::PreferencesGroup::builder()
        .title(tr("settings.diagnostics.overview.title", lang))
        .description(tr("settings.diagnostics.overview.subtitle", lang))
        .build();
    overview_group.add(&summary_row);

    // Details group: populated on refresh.
    let details_group = adw::PreferencesGroup::builder()
        .title(tr("settings.diagnostics.details.title", lang))
        .description(tr("settings.diagnostics.details.description", lang))
        .build();

    // Track dynamically-added rows so Refresh can remove them cleanly.
    let rows: Rc<RefCell<Vec<adw::ActionRow>>> = Rc::new(RefCell::new(Vec::new()));

    // Initial populate.
    refresh(&state, &summary_row, &details_group, &rows, lang);

    refresh_button.connect_clicked(clone!(
        #[strong]
        state,
        #[weak]
        summary_row,
        #[weak]
        details_group,
        #[strong]
        rows,
        move |_| {
            refresh(&state, &summary_row, &details_group, &rows, lang);
        }
    ));

    page.add(&overview_group);
    page.add(&details_group);
    page
}

fn refresh(
    state: &AppState,
    summary_row: &adw::ActionRow,
    details_group: &adw::PreferencesGroup,
    rows: &Rc<RefCell<Vec<adw::ActionRow>>>,
    lang: UiLanguage,
) {
    // Clear previous rows before re-populating.
    for old in rows.borrow().iter() {
        details_group.remove(old);
    }
    rows.borrow_mut().clear();

    let diagnostics = bridge::diagnostics();
    state.update(|snap| snap.diagnostics = diagnostics.clone());

    let summary_text = if diagnostics.summary.is_empty() {
        tr("settings.diagnostics.summary.unknown", lang)
    } else {
        diagnostics.summary.clone()
    };
    summary_row.set_subtitle(&summary_text);

    if diagnostics.items.is_empty() {
        let empty_row = adw::ActionRow::builder()
            .title(tr("settings.diagnostics.empty", lang))
            .build();
        details_group.add(&empty_row);
        rows.borrow_mut().push(empty_row);
        return;
    }

    for item in &diagnostics.items {
        let row = build_item_row(item, lang);
        details_group.add(&row);
        rows.borrow_mut().push(row);
    }
}

fn build_item_row(item: &DiagnosticItemDto, lang: UiLanguage) -> adw::ActionRow {
    // Compose subtitle: problem on the first line, recommendation on the
    // second (prefixed so it reads as a hint, not as a restatement).
    let mut subtitle = item.problem.clone();
    if !item.recommendation.trim().is_empty() {
        if !subtitle.is_empty() {
            subtitle.push('\n');
        }
        subtitle.push_str(&tr("settings.diagnostics.recommendation_prefix", lang));
        subtitle.push(' ');
        subtitle.push_str(&item.recommendation);
    }
    if subtitle.trim().is_empty() {
        subtitle = tr(status_i18n_key(item.status), lang);
    }

    let row = adw::ActionRow::builder()
        .title(&item.title)
        .subtitle(subtitle)
        .subtitle_lines(6)
        .build();
    row.add_prefix(&status_icon(item.status));
    row.add_suffix(&status_badge(item.status, lang));
    row
}

fn status_i18n_key(status: DiagnosticStatus) -> &'static str {
    match status {
        DiagnosticStatus::Ok => "settings.diagnostics.status.ok",
        DiagnosticStatus::Info => "settings.diagnostics.status.info",
        DiagnosticStatus::Warning => "settings.diagnostics.status.warning",
        DiagnosticStatus::Error => "settings.diagnostics.status.error",
    }
}

fn status_icon(status: DiagnosticStatus) -> gtk::Image {
    let (icon_name, css_class) = match status {
        DiagnosticStatus::Ok => ("emblem-ok-symbolic", "success"),
        DiagnosticStatus::Info => ("dialog-information-symbolic", "accent"),
        DiagnosticStatus::Warning => ("dialog-warning-symbolic", "warning"),
        DiagnosticStatus::Error => ("dialog-error-symbolic", "error"),
    };
    let image = gtk::Image::builder()
        .icon_name(icon_name)
        .pixel_size(20)
        .margin_end(4)
        .build();
    image.add_css_class(css_class);
    image
}

/// Coloured text badge mirroring the macOS diagnostic card header. We use
/// the standard libadwaita semantic CSS classes so the tint follows the
/// user's accent / dark-mode preference.
fn status_badge(status: DiagnosticStatus, lang: UiLanguage) -> gtk::Label {
    let css_class = match status {
        DiagnosticStatus::Ok => "success",
        DiagnosticStatus::Info => "accent",
        DiagnosticStatus::Warning => "warning",
        DiagnosticStatus::Error => "error",
    };
    let label = gtk::Label::builder()
        .label(tr(status_i18n_key(status), lang))
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    label.add_css_class(css_class);
    label.add_css_class("caption-heading");
    label
}
