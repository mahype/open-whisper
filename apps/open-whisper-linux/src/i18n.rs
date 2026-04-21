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
        // App / main window
        ("de", "app.title") => "Open Whisper",
        ("de", "window.main.subtitle") => {
            "Diktat \u{2022} Transkription \u{2022} KI-Nachbearbeitung"
        }
        ("de", "button.start_dictation") => "Diktat starten",
        ("de", "button.stop_dictation") => "Diktat stoppen",
        ("de", "button.settings") => "Einstellungen \u{2026}",
        ("de", "button.open_settings") => "Einstellungen \u{2026}",

        // Derived status
        ("de", "status.ready") => "Bereit",
        ("de", "status.recording") => "Nimmt auf \u{2026}",
        ("de", "status.transcribing") => "Transkribiert \u{2026}",
        ("de", "status.post_processing") => "Nachbearbeitung \u{2026}",
        ("de", "status.model_loading") => "Modell wird geladen \u{2026}",

        // Info cards
        ("de", "card.mode") => "Modus",
        ("de", "card.mode.default") => "Standard",
        ("de", "card.model") => "Modell",
        ("de", "card.model.unknown") => "Unbekannt",
        ("de", "card.hotkey") => "Hotkey",
        ("de", "card.hotkey.unset") => "Nicht festgelegt",

        // Hamburger menu
        ("de", "menu.settings") => "Einstellungen \u{2026}",
        ("de", "menu.restart_onboarding") => "Onboarding neu starten",
        ("de", "menu.about") => "\u{00dc}ber Open Whisper",
        ("de", "menu.quit") => "Beenden",

        // Settings window
        ("de", "settings.window.title") => "Einstellungen",
        ("de", "settings.tab.dashboard") => "\u{00dc}bersicht",
        ("de", "settings.tab.recording") => "Aufnahme",
        ("de", "settings.tab.post_processing") => "Nachbearbeitung",
        ("de", "settings.tab.language_models") => "Sprachmodelle",
        ("de", "settings.tab.start_behavior") => "Start & Verhalten",
        ("de", "settings.tab.updates") => "Updates",
        ("de", "settings.tab.diagnostics") => "Diagnose",
        ("de", "settings.tab.help") => "Hilfe",
        ("de", "settings.placeholder.title") => "Noch nicht verf\u{00fc}gbar",
        ("de", "settings.placeholder.body") => "Dieser Bereich folgt in einer sp\u{00e4}teren Ausbaustufe.",
        ("de", "settings.updates.title") => "Updates",
        ("de", "settings.updates.body") => {
            "Unter Linux verwaltet Dein System-Paketmanager (Flatpak, AppImage-Hub oder die Distro-Repos) die Updates dieser App."
        }

        // Settings -> Recording
        ("de", "settings.recording.audio_source.title") => "Audioquelle",
        ("de", "settings.recording.audio_source.description") => {
            "Mikrofon und Sprache f\u{00fc}r die Transkription"
        }
        ("de", "settings.recording.mic.title") => "Mikrofon",
        ("de", "settings.recording.mic.system_default") => "Systemstandard",
        ("de", "settings.recording.language.title") => "Sprache",
        ("de", "settings.recording.language.auto") => "Automatisch",
        ("de", "settings.recording.language.en") => "Englisch",
        ("de", "settings.recording.language.de") => "Deutsch",
        ("de", "settings.recording.language.es") => "Spanisch",
        ("de", "settings.recording.language.fr") => "Franz\u{00f6}sisch",
        ("de", "settings.recording.language.it") => "Italienisch",
        ("de", "settings.recording.language.nl") => "Niederl\u{00e4}ndisch",
        ("de", "settings.recording.language.pt") => "Portugiesisch",
        ("de", "settings.recording.language.pl") => "Polnisch",
        ("de", "settings.recording.language.ja") => "Japanisch",
        ("de", "settings.recording.language.zh") => "Chinesisch",

        ("de", "settings.recording.trigger.title") => "Ausl\u{00f6}ser",
        ("de", "settings.recording.trigger.description") => {
            "Wie Du Diktate startest und stoppst"
        }
        ("de", "settings.recording.trigger_mode.title") => "Modus",
        ("de", "settings.recording.trigger_mode.toggle") => "Umschalten (starten/stoppen)",
        ("de", "settings.recording.trigger_mode.push_to_talk") => "Push-to-Talk (halten)",
        ("de", "settings.recording.hotkey.title") => "Globaler Hotkey",
        ("de", "settings.recording.hotkey.subtitle") => {
            "Bearbeiten folgt in einer sp\u{00e4}teren Ausbaustufe"
        }

        ("de", "settings.recording.text_output.title") => "Textausgabe",
        ("de", "settings.recording.text_output.description") => {
            "Wohin der fertige Transkript-Text geht"
        }
        ("de", "settings.recording.insert_text.title") => "Text automatisch einf\u{00fc}gen",
        ("de", "settings.recording.insert_text.subtitle") => {
            "Simuliert einen Einf\u{00fc}gen-Tastendruck in die fokussierte Anwendung"
        }
        ("de", "settings.recording.restore_clipboard.title") => "Zwischenablage wiederherstellen",
        ("de", "settings.recording.restore_clipboard.subtitle") => {
            "Stellt den urspr\u{00fc}nglichen Zwischenablage-Inhalt nach dem Einf\u{00fc}gen wieder her"
        }
        ("de", "settings.recording.insert_delay.title") => "Verz\u{00f6}gerung vor Einf\u{00fc}gen",
        ("de", "settings.recording.insert_delay.subtitle") => {
            "Wartezeit in Millisekunden bevor der Paste-Tastendruck gesendet wird"
        }

        // Settings -> Start & behavior
        ("de", "settings.start.startup.title") => "Startverhalten",
        ("de", "settings.start.startup.description") => {
            "Wie Open Whisper startet"
        }
        ("de", "settings.start.startup_behavior.title") => "Automatisch starten",
        ("de", "settings.start.startup_behavior.ask") => "Beim ersten Start fragen",
        ("de", "settings.start.startup_behavior.launch") => "Bei Anmeldung starten",
        ("de", "settings.start.startup_behavior.manual") => "Nur manuell starten",
        ("de", "settings.start.app_language.title") => "App-Sprache",
        ("de", "settings.start.app_language.system") => "System",
        ("de", "settings.start.app_language.en") => "Englisch",
        ("de", "settings.start.app_language.de") => "Deutsch",

        ("de", "settings.start.vad.title") => "Sprachaktivit\u{00e4}tserkennung",
        ("de", "settings.start.vad.description") => {
            "Stoppt Aufnahme automatisch bei Stille"
        }
        ("de", "settings.start.vad_enabled.title") => "Aktivieren",
        ("de", "settings.start.vad_enabled.subtitle") => {
            "Beendet das Diktat, wenn du eine Weile nicht mehr sprichst"
        }
        ("de", "settings.start.vad_silence.title") => "Stille-Dauer",
        ("de", "settings.start.vad_silence.subtitle") => {
            "Wie lange du schweigen musst, bevor die Aufnahme endet (ms)"
        }

        ("de", "settings.start.status.title") => "Aktueller Zustand",
        ("de", "settings.start.status.description") => "Nur lesbar \u{2014} aus dem Laufzeit-Status",
        ("de", "settings.start.status.startup") => "Autostart",
        ("de", "settings.start.status.hotkey_registered") => "Hotkey registriert",
        ("de", "settings.start.status.active_mode") => "Aktiver Modus",
        ("de", "settings.start.status.yes") => "Ja",
        ("de", "settings.start.status.no") => "Nein",

        // Settings -> Help
        ("de", "settings.help.about.title") => "\u{00dc}ber",
        ("de", "settings.help.version.title") => "Version",
        ("de", "settings.help.bundle_id.title") => "Bundle-ID",
        ("de", "settings.help.actions.title") => "Aktionen",
        ("de", "settings.help.about_action.title") => "\u{00dc}ber Open Whisper",
        ("de", "settings.help.about_action.subtitle") => "Version, Autor, Lizenz",
        ("de", "settings.help.about_action.button") => "Anzeigen",
        ("de", "settings.help.restart_onboarding.title") => "Onboarding neu starten",
        ("de", "settings.help.restart_onboarding.subtitle") => {
            "F\u{00fc}hrt dich erneut durch Einrichtung"
        }
        ("de", "settings.help.restart_onboarding.button") => "Starten",

        // Dashboard (first sidebar page)
        ("de", "settings.dashboard.current.title") => "Aktueller Zustand",
        ("de", "settings.dashboard.current.description") => {
            "Modus, Modell und Hotkey auf einen Blick"
        }

        // Settings -> Language models
        ("de", "settings.models.transcription.title") => "Transkription",
        ("de", "settings.models.transcription.description") => {
            "Whisper-Modell, das Deine Aufnahmen in Text umwandelt"
        }
        ("de", "settings.models.post_processing.title") => "Nachbearbeitung",
        ("de", "settings.models.post_processing.description") => {
            "Gemma 4 Sprachmodell, das den rohen Transkript aufbereitet"
        }
        ("de", "settings.models.active_preset") => "Aktives Modell",
        ("de", "settings.models.manage") => "Verwalten",
        ("de", "settings.models.manage.subtitle") => {
            "Modelle herunterladen, l\u{00f6}schen oder aktualisieren"
        }

        ("de", "settings.models.manage.transcription.title") => {
            "Whisper-Modelle verwalten"
        }
        ("de", "settings.models.manage.post_processing.title") => {
            "Gemma 4 Modelle verwalten"
        }
        ("de", "settings.models.state.ready") => "Bereit",
        ("de", "settings.models.state.downloading") => "Wird heruntergeladen",
        ("de", "settings.models.state.not_downloaded") => "Nicht heruntergeladen",
        ("de", "settings.models.state.progress_percent") => "Fortschritt: {}%",
        ("de", "settings.models.action.download") => "Download",
        ("de", "settings.models.action.delete") => "L\u{00f6}schen",
        ("de", "settings.models.action.downloading") => "L\u{00e4}uft \u{2026}",
        ("de", "settings.models.advanced.title") => "Erweitert",
        ("de", "settings.models.advanced.description") => {
            "Speichermanagement f\u{00fc}r lokal geladene Modelle"
        }
        ("de", "settings.models.llm_unload.title") => "Modell nach Leerlauf entladen",
        ("de", "settings.models.llm_unload.subtitle") => {
            "Sekunden Inaktivit\u{00e4}t, bevor das Sprachmodell RAM freigibt (0 = nie)"
        }

        // Dialogs
        ("de", "dialog.restart_onboarding.title") => "Onboarding neu starten",
        ("de", "dialog.restart_onboarding.body") => {
            "Der Onboarding-Wizard steht auf Linux noch nicht zur Verf\u{00fc}gung. Er folgt in einer sp\u{00e4}teren Ausbaustufe."
        }
        ("de", "dialog.close") => "Schlie\u{00df}en",

        // About window
        ("de", "about.comments") => "Lokales Diktat f\u{00fc}r jede Anwendung \u{2014} 100 % privat.",
        ("de", "about.developer") => "Sven Wagener",

        // Tray (already present)
        ("de", "tray.start_dictation") => "Diktat starten",
        ("de", "tray.stop_dictation") => "Diktat stoppen",
        ("de", "tray.open_settings") => "Einstellungen \u{2026}",
        ("de", "tray.quit") => "Beenden",
        ("de", "tray.mode") => "Modus",
        ("de", "tray.tooltip.idle") => "Bereit \u{2013} Hotkey oder Klick zum Starten",
        ("de", "tray.tooltip.recording") => "Aufnahme l\u{00e4}uft \u{2026}",

        // English defaults
        (_, "app.title") => "Open Whisper",
        (_, "window.main.subtitle") => {
            "Dictation \u{2022} Transcription \u{2022} AI post-processing"
        }
        (_, "button.start_dictation") => "Start dictation",
        (_, "button.stop_dictation") => "Stop dictation",
        (_, "button.settings") => "Settings\u{2026}",
        (_, "button.open_settings") => "Settings\u{2026}",

        (_, "status.ready") => "Ready",
        (_, "status.recording") => "Recording\u{2026}",
        (_, "status.transcribing") => "Transcribing\u{2026}",
        (_, "status.post_processing") => "Post-processing\u{2026}",
        (_, "status.model_loading") => "Loading model\u{2026}",

        (_, "card.mode") => "Mode",
        (_, "card.mode.default") => "Default",
        (_, "card.model") => "Model",
        (_, "card.model.unknown") => "Unknown",
        (_, "card.hotkey") => "Hotkey",
        (_, "card.hotkey.unset") => "Not set",

        (_, "menu.settings") => "Settings\u{2026}",
        (_, "menu.restart_onboarding") => "Restart onboarding",
        (_, "menu.about") => "About Open Whisper",
        (_, "menu.quit") => "Quit",

        (_, "settings.window.title") => "Settings",
        (_, "settings.tab.dashboard") => "Overview",
        (_, "settings.tab.recording") => "Recording",
        (_, "settings.tab.post_processing") => "Post-processing",
        (_, "settings.tab.language_models") => "Language models",
        (_, "settings.tab.start_behavior") => "Start & behavior",
        (_, "settings.tab.updates") => "Updates",
        (_, "settings.tab.diagnostics") => "Diagnostics",
        (_, "settings.tab.help") => "Help",
        (_, "settings.placeholder.title") => "Not available yet",
        (_, "settings.placeholder.body") => "This section arrives in a later stage.",
        (_, "settings.updates.title") => "Updates",
        (_, "settings.updates.body") => {
            "On Linux your system package manager (Flatpak, AppImage Hub, or your distro repos) delivers updates for this app."
        }

        // Settings -> Recording
        (_, "settings.recording.audio_source.title") => "Audio source",
        (_, "settings.recording.audio_source.description") => {
            "Microphone and language for transcription"
        }
        (_, "settings.recording.mic.title") => "Microphone",
        (_, "settings.recording.mic.system_default") => "System default",
        (_, "settings.recording.language.title") => "Language",
        (_, "settings.recording.language.auto") => "Automatic",
        (_, "settings.recording.language.en") => "English",
        (_, "settings.recording.language.de") => "German",
        (_, "settings.recording.language.es") => "Spanish",
        (_, "settings.recording.language.fr") => "French",
        (_, "settings.recording.language.it") => "Italian",
        (_, "settings.recording.language.nl") => "Dutch",
        (_, "settings.recording.language.pt") => "Portuguese",
        (_, "settings.recording.language.pl") => "Polish",
        (_, "settings.recording.language.ja") => "Japanese",
        (_, "settings.recording.language.zh") => "Chinese",

        (_, "settings.recording.trigger.title") => "Trigger",
        (_, "settings.recording.trigger.description") => {
            "How dictation starts and stops"
        }
        (_, "settings.recording.trigger_mode.title") => "Mode",
        (_, "settings.recording.trigger_mode.toggle") => "Toggle (start/stop)",
        (_, "settings.recording.trigger_mode.push_to_talk") => "Push-to-talk (hold)",
        (_, "settings.recording.hotkey.title") => "Global hotkey",
        (_, "settings.recording.hotkey.subtitle") => {
            "Editing arrives in a later stage"
        }

        (_, "settings.recording.text_output.title") => "Text output",
        (_, "settings.recording.text_output.description") => {
            "Where the finished transcript lands"
        }
        (_, "settings.recording.insert_text.title") => "Insert text automatically",
        (_, "settings.recording.insert_text.subtitle") => {
            "Simulates a paste keystroke into the focused application"
        }
        (_, "settings.recording.restore_clipboard.title") => "Restore clipboard",
        (_, "settings.recording.restore_clipboard.subtitle") => {
            "Puts the previous clipboard content back after pasting"
        }
        (_, "settings.recording.insert_delay.title") => "Insert delay",
        (_, "settings.recording.insert_delay.subtitle") => {
            "Milliseconds to wait before sending the paste keystroke"
        }

        // Settings -> Start & behavior
        (_, "settings.start.startup.title") => "Startup",
        (_, "settings.start.startup.description") => "How Open Whisper launches",
        (_, "settings.start.startup_behavior.title") => "Auto-start",
        (_, "settings.start.startup_behavior.ask") => "Ask on first launch",
        (_, "settings.start.startup_behavior.launch") => "Launch at login",
        (_, "settings.start.startup_behavior.manual") => "Launch manually only",
        (_, "settings.start.app_language.title") => "App language",
        (_, "settings.start.app_language.system") => "System",
        (_, "settings.start.app_language.en") => "English",
        (_, "settings.start.app_language.de") => "German",

        (_, "settings.start.vad.title") => "Voice activity detection",
        (_, "settings.start.vad.description") => "Stops recording automatically on silence",
        (_, "settings.start.vad_enabled.title") => "Enabled",
        (_, "settings.start.vad_enabled.subtitle") => {
            "Ends dictation when you stop speaking"
        }
        (_, "settings.start.vad_silence.title") => "Silence duration",
        (_, "settings.start.vad_silence.subtitle") => {
            "How long you have to stay silent before recording stops (ms)"
        }

        (_, "settings.start.status.title") => "Current status",
        (_, "settings.start.status.description") => "Read-only \u{2014} from the runtime",
        (_, "settings.start.status.startup") => "Auto-start",
        (_, "settings.start.status.hotkey_registered") => "Hotkey registered",
        (_, "settings.start.status.active_mode") => "Active mode",
        (_, "settings.start.status.yes") => "Yes",
        (_, "settings.start.status.no") => "No",

        // Settings -> Help
        (_, "settings.help.about.title") => "About",
        (_, "settings.help.version.title") => "Version",
        (_, "settings.help.bundle_id.title") => "Bundle ID",
        (_, "settings.help.actions.title") => "Actions",
        (_, "settings.help.about_action.title") => "About Open Whisper",
        (_, "settings.help.about_action.subtitle") => "Version, author, license",
        (_, "settings.help.about_action.button") => "Show",
        (_, "settings.help.restart_onboarding.title") => "Restart onboarding",
        (_, "settings.help.restart_onboarding.subtitle") => {
            "Walks you through the setup again"
        }
        (_, "settings.help.restart_onboarding.button") => "Start",

        // Dashboard (first sidebar page)
        (_, "settings.dashboard.current.title") => "Current state",
        (_, "settings.dashboard.current.description") => {
            "Mode, model and hotkey at a glance"
        }

        // Settings -> Language models
        (_, "settings.models.transcription.title") => "Transcription",
        (_, "settings.models.transcription.description") => {
            "Whisper model that turns your recordings into text"
        }
        (_, "settings.models.post_processing.title") => "Post-processing",
        (_, "settings.models.post_processing.description") => {
            "Gemma 4 language model that cleans up the raw transcript"
        }
        (_, "settings.models.active_preset") => "Active model",
        (_, "settings.models.manage") => "Manage",
        (_, "settings.models.manage.subtitle") => {
            "Download, delete, or update models"
        }

        (_, "settings.models.manage.transcription.title") => "Manage Whisper models",
        (_, "settings.models.manage.post_processing.title") => "Manage Gemma 4 models",
        (_, "settings.models.state.ready") => "Ready",
        (_, "settings.models.state.downloading") => "Downloading",
        (_, "settings.models.state.not_downloaded") => "Not downloaded",
        (_, "settings.models.state.progress_percent") => "Progress: {}%",
        (_, "settings.models.action.download") => "Download",
        (_, "settings.models.action.delete") => "Delete",
        (_, "settings.models.action.downloading") => "Running \u{2026}",
        (_, "settings.models.advanced.title") => "Advanced",
        (_, "settings.models.advanced.description") => {
            "Memory management for loaded models"
        }
        (_, "settings.models.llm_unload.title") => "Unload model after idle",
        (_, "settings.models.llm_unload.subtitle") => {
            "Seconds of inactivity before the language model releases RAM (0 = never)"
        }

        (_, "dialog.restart_onboarding.title") => "Restart onboarding",
        (_, "dialog.restart_onboarding.body") => {
            "The onboarding wizard is not available on Linux yet. It will arrive in a later stage."
        }
        (_, "dialog.close") => "Close",

        (_, "about.comments") => "Local dictation for any application \u{2014} 100% private.",
        (_, "about.developer") => "Sven Wagener",

        (_, "tray.start_dictation") => "Start dictation",
        (_, "tray.stop_dictation") => "Stop dictation",
        (_, "tray.open_settings") => "Settings\u{2026}",
        (_, "tray.quit") => "Quit",
        (_, "tray.mode") => "Mode",
        (_, "tray.tooltip.idle") => "Idle \u{2014} press hotkey or click to start",
        (_, "tray.tooltip.recording") => "Recording\u{2026}",

        _ => key,
    }
    .to_owned()
}
