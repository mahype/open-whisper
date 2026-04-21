//! Global-hotkey recorder row.
//!
//! `adw::ActionRow` with a suffix button that captures live key events.
//! The workflow mirrors macOS's `HotkeyRecorderField`:
//!
//!  1. Row shows the currently-registered hotkey; subtitle explains the
//!     interaction.
//!  2. Clicking the button enters *capture* mode — background tint + hint
//!     label, key-events grabbed via a `gtk::EventControllerKey`.
//!  3. While holding modifiers the button shows a live preview (e.g.
//!     `Ctrl+Shift+…`). Pressing a non-modifier commits, `Escape` cancels.
//!  4. The committed combination is validated through the bridge
//!     (`bridge::validate_hotkey`). On success the normalized text
//!     (matching what the hotkey controller expects) is persisted to
//!     `settings.hotkey`. On validation error we show the bridge's
//!     message in the subtitle and stay in capture mode.

use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use glib::clone;
use gtk::gdk;

use open_whisper_core::{AppSettings, UiLanguage};

use crate::bridge;
use crate::i18n::tr;
use crate::state::AppState;
use crate::ui::settings::persist_settings;

pub fn build(state: AppState, lang: UiLanguage) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(tr("settings.recording.hotkey.title", lang))
        .subtitle(tr("settings.recording.hotkey.subtitle.ready", lang))
        .build();

    let button = gtk::Button::builder()
        .label(button_label_for_current(&state, lang))
        .valign(gtk::Align::Center)
        .focusable(true)
        .build();
    button.add_css_class("pill");
    row.add_suffix(&button);

    let capturing = Rc::new(Cell::new(false));

    // Key controller attached to the button — it only fires while the
    // button has focus, which `grab_focus` below ensures after a click.
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(clone!(
        #[strong]
        state,
        #[weak]
        button,
        #[weak]
        row,
        #[strong]
        capturing,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_, keyval, _keycode, modifiers| {
            if !capturing.get() {
                return glib::Propagation::Proceed;
            }

            // Escape bails without touching settings.
            if keyval == gdk::Key::Escape {
                exit_capture(&button, &row, &state, lang, false);
                capturing.set(false);
                return glib::Propagation::Stop;
            }

            if is_modifier_key(keyval) {
                // Live preview of held modifiers, so the user sees which
                // keys GDK is registering.
                let preview = format_modifier_prefix(modifiers);
                button.set_label(&format!(
                    "{}{}",
                    preview,
                    tr("settings.recording.hotkey.press", lang)
                ));
                return glib::Propagation::Stop;
            }

            // Non-modifier pressed — commit attempt.
            let raw = format_hotkey(modifiers, keyval);
            match bridge::validate_hotkey(&raw) {
                Ok(normalized) => {
                    let saved = normalized.clone();
                    button.set_label(&normalized);
                    button.remove_css_class("accent");
                    row.set_subtitle(&tr("settings.recording.hotkey.subtitle.saved", lang));
                    capturing.set(false);
                    persist_settings(&state, move |s: &mut AppSettings| {
                        s.hotkey = saved;
                    });
                }
                Err(message) => {
                    // Stay in capture mode so the user can retry; show the
                    // bridge's message verbatim (it already localises enough
                    // to be actionable, e.g. "Hotkey is already in use").
                    row.set_subtitle(&message);
                    button.set_label(&tr("settings.recording.hotkey.press", lang));
                }
            }
            glib::Propagation::Stop
        }
    ));
    button.add_controller(key_controller);

    button.connect_clicked(clone!(
        #[weak]
        row,
        #[strong]
        state,
        #[strong]
        capturing,
        move |btn| {
            if capturing.get() {
                // Second click cancels the pending capture.
                exit_capture(btn, &row, &state, lang, true);
                capturing.set(false);
                return;
            }
            capturing.set(true);
            btn.add_css_class("accent");
            btn.set_label(&tr("settings.recording.hotkey.press", lang));
            row.set_subtitle(&tr("settings.recording.hotkey.subtitle.capture", lang));
            btn.grab_focus();
        }
    ));

    row
}

fn exit_capture(
    button: &gtk::Button,
    row: &adw::ActionRow,
    state: &AppState,
    lang: UiLanguage,
    cancelled: bool,
) {
    button.remove_css_class("accent");
    button.set_label(&button_label_for_current(state, lang));
    let subtitle_key = if cancelled {
        "settings.recording.hotkey.cancelled"
    } else {
        "settings.recording.hotkey.subtitle.ready"
    };
    row.set_subtitle(&tr(subtitle_key, lang));
}

fn button_label_for_current(state: &AppState, lang: UiLanguage) -> String {
    let current = state.with(|snap| snap.settings.hotkey.clone());
    if current.trim().is_empty() {
        tr("settings.recording.hotkey.empty", lang)
    } else {
        current
    }
}

fn is_modifier_key(key: gdk::Key) -> bool {
    matches!(
        key.name().as_deref(),
        Some(
            "Control_L"
                | "Control_R"
                | "Alt_L"
                | "Alt_R"
                | "Shift_L"
                | "Shift_R"
                | "Super_L"
                | "Super_R"
                | "Meta_L"
                | "Meta_R"
                | "Hyper_L"
                | "Hyper_R"
                | "ISO_Level3_Shift"
                | "ISO_Level5_Shift",
        )
    )
}

/// Build the `Ctrl+Shift+…` prefix for live preview.
fn format_modifier_prefix(modifiers: gdk::ModifierType) -> String {
    let mut parts = Vec::with_capacity(4);
    if modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
        parts.push("Ctrl");
    }
    if modifiers.contains(gdk::ModifierType::ALT_MASK) {
        parts.push("Alt");
    }
    if modifiers.contains(gdk::ModifierType::SHIFT_MASK) {
        parts.push("Shift");
    }
    if modifiers.contains(gdk::ModifierType::SUPER_MASK) {
        parts.push("Super");
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("{}+", parts.join("+"))
    }
}

/// Raw formatting — the bridge's `validate_hotkey` will normalize further.
/// We deliberately pass the GDK key *name* verbatim (e.g. `space`,
/// `Return`, `F5`) because the hotkey controller shares its parser with
/// macOS which also accepts these GDK-style names.
fn format_hotkey(modifiers: gdk::ModifierType, key: gdk::Key) -> String {
    let mut text = format_modifier_prefix(modifiers);
    let name = key
        .name()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_owned());

    // Single ASCII letters look more hotkey-ish uppercased ("a" → "A").
    let normalized = if name.len() == 1 && name.chars().all(|c| c.is_ascii_alphabetic()) {
        name.to_uppercase()
    } else {
        name
    };
    text.push_str(&normalized);
    text
}
