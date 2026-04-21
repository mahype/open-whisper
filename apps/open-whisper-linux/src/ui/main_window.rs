//! Skeleton main window.
//!
//! Phase 1 only has to prove the lifecycle — bridge load → state → window
//! present → periodic poll updates visible status text. Settings, HUD,
//! onboarding, and tray menus arrive in later phases.

use adw::prelude::*;
use glib::clone;

use crate::bridge;
use crate::i18n::tr;
use crate::state::AppState;

pub fn build(app: &adw::Application, state: AppState) -> adw::ApplicationWindow {
    let settings_snapshot = state.with(|snap| snap.settings.clone());
    let lang = settings_snapshot.ui_language;

    let header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new(
            &tr("app.title", lang),
            &tr("window.main.subtitle", lang),
        ))
        .build();

    let status_label = gtk::Label::builder()
        .label(state.with(|snap| snap.runtime.last_status.clone()))
        .halign(gtk::Align::Center)
        .wrap(true)
        .build();
    status_label.add_css_class("title-2");

    let dictate_button = gtk::Button::builder()
        .label(tr("button.start_dictation", lang))
        .halign(gtk::Align::Center)
        .build();
    dictate_button.add_css_class("pill");
    dictate_button.add_css_class("suggested-action");

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

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(48)
        .margin_end(48)
        .valign(gtk::Align::Center)
        .build();
    content.append(&status_label);
    content.append(&dictate_button);

    let root = adw::ToolbarView::new();
    root.add_top_bar(&header);
    root.set_content(Some(&content));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(720)
        .default_height(480)
        .content(&root)
        .build();

    // Repaint the status + button whenever the poll loop refreshes state.
    glib::timeout_add_local(
        std::time::Duration::from_millis(350),
        clone!(
            #[weak]
            status_label,
            #[weak]
            dictate_button,
            #[strong]
            state,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                let snap = state.snapshot();
                status_label.set_label(&snap.runtime.last_status);
                dictate_button.set_label(&if snap.runtime.is_recording {
                    tr("button.stop_dictation", snap.settings.ui_language)
                } else {
                    tr("button.start_dictation", snap.settings.ui_language)
                });
                glib::ControlFlow::Continue
            }
        ),
    );

    window
}
