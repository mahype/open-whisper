//! Main window — GNOME-Settings-style navigation.
//!
//! An `adw::NavigationSplitView` hosts a permanent left sidebar listing
//! every page, and an `adw::ViewStack` in the content area renders the
//! selected page. This matches the layout of GNOME Settings (and most
//! adaptive libadwaita apps): on desktop widths both panes are visible
//! side-by-side, on narrow widths the split view collapses into a
//! push-navigation automatically.
//!
//! Every page is still an `adw::PreferencesPage` — embedding a
//! PreferencesPage in a `GtkStack` is explicitly supported and keeps the
//! padding/title chrome each tab expects.

use adw::prelude::*;
use glib::clone;

use crate::i18n::tr;
use crate::state::AppState;
use crate::ui::settings;

/// Descriptor for a single sidebar entry.
struct Page {
    id: &'static str,
    title: String,
    icon: &'static str,
    content: gtk::Widget,
}

pub fn build(app: &adw::Application, state: AppState) -> adw::ApplicationWindow {
    let lang = state.with(|snap| snap.settings.ui_language);

    let pages: Vec<Page> = vec![
        Page {
            id: "dashboard",
            title: tr("settings.tab.dashboard", lang),
            // user-home-symbolic ships with libadwaita's bundled icons, so
            // it renders consistently even when the user's system icon theme
            // (e.g. Fluent-dark) doesn't inherit Adwaita. `view-dashboard-*`
            // was not bundled and fell back to a multi-colour stand-in.
            icon: "user-home-symbolic",
            content: settings::dashboard::build(state.clone()).upcast(),
        },
        Page {
            id: "recording",
            title: tr("settings.tab.recording", lang),
            icon: "audio-input-microphone-symbolic",
            content: settings::recording::build(state.clone()).upcast(),
        },
        Page {
            id: "post-processing",
            title: tr("settings.tab.post_processing", lang),
            icon: "text-editor-symbolic",
            content: settings::placeholder_page(
                "post-processing",
                &tr("settings.tab.post_processing", lang),
                "text-editor-symbolic",
                lang,
            )
            .upcast(),
        },
        Page {
            id: "language-models",
            title: tr("settings.tab.language_models", lang),
            icon: "folder-download-symbolic",
            content: settings::language_models::build(state.clone()).upcast(),
        },
        Page {
            id: "start-behavior",
            title: tr("settings.tab.start_behavior", lang),
            icon: "preferences-system-symbolic",
            content: settings::start_behavior::build(state.clone()).upcast(),
        },
        Page {
            id: "updates",
            title: tr("settings.tab.updates", lang),
            icon: "software-update-available-symbolic",
            content: settings::updates_page(lang).upcast(),
        },
        Page {
            id: "diagnostics",
            title: tr("settings.tab.diagnostics", lang),
            // system-run-symbolic is part of libadwaita's bundled set and
            // reads as "run checks / diagnose" well; dialog-information-*
            // was inconsistent on themes that don't inherit Adwaita.
            icon: "system-run-symbolic",
            content: settings::diagnostics::build(state.clone()).upcast(),
        },
        Page {
            id: "help",
            title: tr("settings.tab.help", lang),
            icon: "help-about-symbolic",
            content: settings::help::build(state).upcast(),
        },
    ];

    // Build the ViewStack first so the sidebar selection callback can
    // retarget it by name.
    let stack = adw::ViewStack::new();
    for page in &pages {
        stack.add_named(&page.content, Some(page.id));
    }

    // Sidebar = ListBox of rows, one per page.
    let sidebar_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .build();
    sidebar_list.add_css_class("navigation-sidebar");

    for page in &pages {
        sidebar_list.append(&build_sidebar_row(page));
    }

    // Wire selection changes to the stack and to the content header title.
    let titles: Vec<(String, String)> = pages
        .iter()
        .map(|p| (p.id.to_owned(), p.title.clone()))
        .collect();

    let content_title = adw::WindowTitle::new(&pages[0].title, "");

    sidebar_list.connect_row_selected(clone!(
        #[weak]
        stack,
        #[weak]
        content_title,
        move |_, row| {
            let Some(row) = row else {
                return;
            };
            let name = row.widget_name();
            stack.set_visible_child_name(&name);
            if let Some((_, title)) = titles.iter().find(|(id, _)| *id == name.as_str()) {
                content_title.set_title(title);
            }
        }
    ));

    // Pick the first entry on open so the stack has a visible child.
    if let Some(first) = sidebar_list.row_at_index(0) {
        sidebar_list.select_row(Some(&first));
    }

    // Sidebar toolbar: app title at the top, scrollable list below.
    let sidebar_header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new(
            &tr("app.title", lang),
            &tr("window.main.subtitle", lang),
        ))
        .show_end_title_buttons(false)
        .build();

    let sidebar_scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&sidebar_list)
        .build();

    let sidebar_toolbar = adw::ToolbarView::new();
    sidebar_toolbar.add_top_bar(&sidebar_header);
    sidebar_toolbar.set_content(Some(&sidebar_scroller));

    let sidebar_page = adw::NavigationPage::builder()
        .title(tr("app.title", lang))
        .tag("sidebar")
        .child(&sidebar_toolbar)
        .build();

    // Content toolbar: dynamic title reflects the active sidebar page.
    let content_header = adw::HeaderBar::builder()
        .title_widget(&content_title)
        .build();

    let content_toolbar = adw::ToolbarView::new();
    content_toolbar.add_top_bar(&content_header);
    content_toolbar.set_content(Some(&stack));

    let content_page = adw::NavigationPage::builder()
        .title(tr("app.title", lang))
        .tag("content")
        .child(&content_toolbar)
        .build();

    let split = adw::NavigationSplitView::new();
    split.set_sidebar(Some(&sidebar_page));
    split.set_content(Some(&content_page));
    split.set_min_sidebar_width(220.0);
    split.set_max_sidebar_width(280.0);

    adw::ApplicationWindow::builder()
        .application(app)
        .default_width(920)
        .default_height(640)
        .title(tr("app.title", lang))
        .content(&split)
        .build()
}

fn build_sidebar_row(page: &Page) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(page.id);

    let icon = gtk::Image::builder()
        .icon_name(page.icon)
        .pixel_size(18)
        .build();

    let label = gtk::Label::builder()
        .label(&page.title)
        .xalign(0.0)
        .hexpand(true)
        .build();

    let inner = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();
    inner.append(&icon);
    inner.append(&label);

    row.set_child(Some(&inner));
    row
}
