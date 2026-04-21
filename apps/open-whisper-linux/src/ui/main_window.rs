//! Main window — dashboard shell.
//!
//! macOS is menu-bar-first; on Linux we don't have that UX across every
//! desktop, so the GTK window itself acts as the anchor. The layout is a
//! deliberate port of macOS's status-bar menu ("what mode am I in, what
//! model is loaded, which hotkey am I listening to") as a dashboard with
//! three info cards, a derived status badge, and the two actions the user
//! performs from here: toggle dictation and open settings.

use adw::prelude::*;
use glib::clone;

use crate::bridge;
use crate::i18n::tr;
use crate::state::{AppSnapshot, AppState};

/// Widget refresh cadence. `app.rs` polls the bridge at 1 Hz; this re-reads
/// the resulting `AppSnapshot` twice as often so widget updates feel
/// immediate after a state change, without the cost of a bridge call.
const UI_REFRESH_MS: u64 = 500;

pub fn build(app: &adw::Application, state: AppState) -> adw::ApplicationWindow {
    let lang = state.with(|snap| snap.settings.ui_language);
    let initial = state.snapshot();

    let header = build_header(lang);

    // Status badge: large title that reflects derived phase (Ready /
    // Recording / Transcribing / Loading model).
    let status_label = gtk::Label::builder()
        .label(derive_status(&initial, lang))
        .halign(gtk::Align::Center)
        .wrap(true)
        .build();
    status_label.add_css_class("title-1");

    // Info cards. Using `adw::PreferencesGroup` gives proper card styling
    // even on brew-libadwaita where raw `gtk::Box` backgrounds are often
    // transparent and disappear into the window.
    let info_group = adw::PreferencesGroup::new();
    info_group.set_margin_start(24);
    info_group.set_margin_end(24);

    let mode_row = adw::ActionRow::builder()
        .title(tr("card.mode", lang))
        .subtitle(mode_subtitle(&initial, lang))
        .build();
    info_group.add(&mode_row);

    let model_row = adw::ActionRow::builder()
        .title(tr("card.model", lang))
        .subtitle(model_subtitle(&initial, lang))
        .build();
    info_group.add(&model_row);

    let hotkey_row = adw::ActionRow::builder()
        .title(tr("card.hotkey", lang))
        .subtitle(hotkey_subtitle(&initial, lang))
        .build();
    info_group.add(&hotkey_row);

    // Primary + secondary actions. `suggested-action` paints the primary
    // button in the accent colour; the neutral settings button is a plain
    // pill so hierarchy is obvious without reading labels.
    let dictate_button = gtk::Button::builder()
        .label(dictate_label(&initial, lang))
        .build();
    dictate_button.add_css_class("pill");
    dictate_button.add_css_class("suggested-action");

    let settings_button = gtk::Button::builder()
        .label(tr("button.settings", lang))
        .build();
    settings_button.add_css_class("pill");
    settings_button.set_action_name(Some("app.settings"));

    dictate_button.connect_clicked(clone!(
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

    let button_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .build();
    button_row.append(&dictate_button);
    button_row.append(&settings_button);

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(36)
        .margin_bottom(36)
        .margin_start(24)
        .margin_end(24)
        .build();
    content.append(&status_label);
    content.append(&info_group);
    content.append(&button_row);

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&content)
        .build();

    let root = adw::ToolbarView::new();
    root.add_top_bar(&header);
    root.set_content(Some(&scroller));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(620)
        .default_height(560)
        .content(&root)
        .build();

    // Repaint status label, card subtitles and the dictate button whenever
    // the poll loop refreshes state.
    glib::timeout_add_local(
        std::time::Duration::from_millis(UI_REFRESH_MS),
        clone!(
            #[weak]
            status_label,
            #[weak]
            dictate_button,
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
                status_label.set_label(&derive_status(&snap, lang));
                dictate_button.set_label(&dictate_label(&snap, lang));
                mode_row.set_subtitle(&mode_subtitle(&snap, lang));
                model_row.set_subtitle(&model_subtitle(&snap, lang));
                hotkey_row.set_subtitle(&hotkey_subtitle(&snap, lang));
                glib::ControlFlow::Continue
            }
        ),
    );

    window
}

fn build_header(lang: open_whisper_core::UiLanguage) -> adw::HeaderBar {
    let header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new(
            &tr("app.title", lang),
            &tr("window.main.subtitle", lang),
        ))
        .build();

    let menu = gio::Menu::new();
    menu.append(Some(&tr("menu.settings", lang)), Some("app.settings"));
    menu.append(
        Some(&tr("menu.restart_onboarding", lang)),
        Some("app.restart_onboarding"),
    );
    menu.append(Some(&tr("menu.about", lang)), Some("app.about"));
    menu.append(Some(&tr("menu.quit", lang)), Some("app.quit"));

    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .primary(true)
        .build();
    header.pack_end(&menu_button);

    header
}

fn derive_status(snap: &AppSnapshot, lang: open_whisper_core::UiLanguage) -> String {
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

fn dictate_label(snap: &AppSnapshot, lang: open_whisper_core::UiLanguage) -> String {
    if snap.runtime.is_recording {
        tr("button.stop_dictation", lang)
    } else {
        tr("button.start_dictation", lang)
    }
}

fn mode_subtitle(snap: &AppSnapshot, lang: open_whisper_core::UiLanguage) -> String {
    if snap.runtime.active_mode_name.is_empty() {
        tr("card.mode.default", lang)
    } else {
        snap.runtime.active_mode_name.clone()
    }
}

fn model_subtitle(snap: &AppSnapshot, lang: open_whisper_core::UiLanguage) -> String {
    if !snap.model.preset_label.is_empty() {
        snap.model.preset_label.clone()
    } else {
        tr("card.model.unknown", lang)
    }
}

fn hotkey_subtitle(snap: &AppSnapshot, lang: open_whisper_core::UiLanguage) -> String {
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
