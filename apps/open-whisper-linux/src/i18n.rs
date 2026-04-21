//! Localization entry point.
//!
//! The core already owns a `UiLanguage` enum (System / En / De); the
//! macOS app translates it via Swift's `Localizable.xcstrings`. Here we
//! defer to a hand-rolled lookup until gettext-rs + `.po` files land in
//! a later phase. Every string that appears in the UI goes through
//! `tr!` so the switch is mechanical when we add gettext.

use open_whisper_core::UiLanguage;

/// Resolve `UiLanguage::System` to the preferred language code from the
/// environment (`LANG`, `LC_ALL`, `LC_MESSAGES`). Returns `"en"` or `"de"`
/// — any other value falls back to English.
pub fn resolved_code(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::En => "en",
        UiLanguage::De => "de",
        UiLanguage::System => {
            let env = std::env::var("LC_ALL")
                .or_else(|_| std::env::var("LC_MESSAGES"))
                .or_else(|_| std::env::var("LANG"))
                .unwrap_or_default();
            if env.to_lowercase().starts_with("de") {
                "de"
            } else {
                "en"
            }
        }
    }
}

/// Temporary message catalog. Will be replaced by `gettext` lookups once
/// `.po` files are generated from the shared xcstrings catalog.
pub fn tr(key: &str, lang: UiLanguage) -> String {
    let code = resolved_code(lang);
    match (code, key) {
        ("de", "app.title") => "Open Whisper",
        ("de", "window.main.subtitle") => {
            "Diktat \u{2022} Transkription \u{2022} KI-Nachbearbeitung"
        }
        ("de", "button.start_dictation") => "Diktat starten",
        ("de", "button.stop_dictation") => "Diktat stoppen",
        ("de", "button.open_settings") => "Einstellungen \u{2026}",
        ("de", "status.ready") => "Bereit",
        (_, "app.title") => "Open Whisper",
        (_, "window.main.subtitle") => {
            "Dictation \u{2022} Transcription \u{2022} AI post-processing"
        }
        (_, "button.start_dictation") => "Start dictation",
        (_, "button.stop_dictation") => "Stop dictation",
        (_, "button.open_settings") => "Settings\u{2026}",
        (_, "status.ready") => "Ready",
        _ => key,
    }
    .to_owned()
}
