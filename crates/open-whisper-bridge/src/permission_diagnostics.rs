use open_whisper_core::{AppSettings, DiagnosticItemDto, DiagnosticStatus, DiagnosticsDto};

use crate::{dictation::DictationController, hotkey::HotKeyController, model_manager};

pub fn collect(
    settings: &AppSettings,
    dictation: &DictationController,
    hotkey: Option<&HotKeyController>,
    autostart_summary: &str,
) -> DiagnosticsDto {
    let mut items = Vec::new();
    let available_devices = dictation.available_input_devices();

    if available_devices.is_empty() {
        items.push(item(
            "Mikrofon",
            DiagnosticStatus::Error,
            "Es wurde kein Eingabegeraet erkannt.",
            "Pruefe Mikrofonfreigabe und ob mindestens ein Eingabegeraet angeschlossen ist.",
        ));
    } else {
        items.push(item(
            "Mikrofon",
            DiagnosticStatus::Ok,
            &format!("{} Eingabegeraet(e) erkannt.", available_devices.len()),
            "Keine Aktion noetig.",
        ));
    }

    let selected_device_ok = settings.input_device_name == "System Default"
        || available_devices
            .iter()
            .any(|device| device == &settings.input_device_name);
    if selected_device_ok {
        items.push(item(
            "Eingabegeraet",
            DiagnosticStatus::Ok,
            &format!(
                "Das ausgewaehlte Geraet '{}' ist verfuegbar.",
                settings.input_device_name
            ),
            "Keine Aktion noetig.",
        ));
    } else {
        items.push(item(
            "Eingabegeraet",
            DiagnosticStatus::Warning,
            &format!(
                "Das ausgewaehlte Geraet '{}' ist aktuell nicht verfuegbar.",
                settings.input_device_name
            ),
            "Waehle im Onboarding oder in den Einstellungen ein anderes Mikrofon aus.",
        ));
    }

    match model_manager::resolve_model_path(settings) {
        Ok(path) if path.exists() => items.push(item(
            "Lokales Modell",
            DiagnosticStatus::Ok,
            &model_ready_message(settings),
            "Das lokale Diktat ist einsatzbereit.",
        )),
        Ok(_) => items.push(item(
            "Lokales Modell",
            DiagnosticStatus::Warning,
            &format!(
                "{} ist noch nicht heruntergeladen.",
                settings.local_model.display_label()
            ),
            "Lade das ausgewaehlte Modell vor dem ersten Diktat herunter.",
        )),
        Err(err) => items.push(item(
            "Lokales Modell",
            DiagnosticStatus::Error,
            &err,
            "Pruefe den Modellpfad oder waehle eines der integrierten Presets neu aus.",
        )),
    }

    if let Some(hotkey) = hotkey {
        if hotkey.is_registered() {
            items.push(item(
                "Globaler Hotkey",
                DiagnosticStatus::Ok,
                &format!("Hotkey '{}' ist registriert.", settings.hotkey),
                "Keine Aktion noetig.",
            ));
        } else {
            items.push(item(
                "Globaler Hotkey",
                DiagnosticStatus::Warning,
                &format!(
                    "Hotkey '{}' ist noch nicht aktiv.",
                    settings.hotkey
                ),
                "Pruefe die Hotkey-Kombination und erteile auf macOS bei Bedarf Accessibility- oder Input-Monitoring-Rechte.",
            ));
        }
    } else {
        items.push(item(
            "Globaler Hotkey",
            DiagnosticStatus::Error,
            "Die Hotkey-Integration konnte nicht initialisiert werden.",
            "Starte die App neu und pruefe, ob die Kombination bereits von einer anderen App belegt ist.",
        ));
    }

    items.push(item(
        "Autostart",
        DiagnosticStatus::Info,
        autostart_summary,
        "Passe das Verhalten im Onboarding oder in den Einstellungen an.",
    ));

    #[cfg(target_os = "macos")]
    {
        items.push(item(
            "macOS Datenschutz",
            DiagnosticStatus::Info,
            "Fuer Mikrofon, globalen Hotkey und das Einfuegen in andere Apps kann macOS Datenschutz-Rechte verlangen.",
            "Oeffne bei Problemen System Settings > Privacy & Security und pruefe Microphone, Accessibility und Input Monitoring.",
        ));
    }

    #[cfg(target_os = "linux")]
    {
        items.push(item(
            "Linux Sitzung",
            DiagnosticStatus::Info,
            "Globale Hotkeys und simuliertes Paste sind unter X11 meist robuster als unter Wayland.",
            "Falls der Hotkey oder das Einfuegen nicht funktioniert, teste eine X11-Sitzung oder desktop-spezifische Rechte.",
        ));
    }

    #[cfg(target_os = "windows")]
    {
        items.push(item(
            "Windows Fokus",
            DiagnosticStatus::Info,
            "Einige Apps blockieren simulierte Eingaben, solange UAC-Dialoge oder Sicherheitssoftware aktiv sind.",
            "Teste das Einfuegen in eine normale Text-App und pruefe ggf. Fokus oder Sicherheitssoftware.",
        ));
    }

    let error_count = items
        .iter()
        .filter(|item| item.status == DiagnosticStatus::Error)
        .count();
    let warning_count = items
        .iter()
        .filter(|item| item.status == DiagnosticStatus::Warning)
        .count();

    let summary = match (error_count, warning_count) {
        (0, 0) => "Diagnose: keine offenen Probleme erkannt.".to_owned(),
        (0, warnings) => format!("Diagnose: {warnings} Warnung(en), keine Fehler."),
        (errors, warnings) => format!("Diagnose: {errors} Fehler, {warnings} Warnung(en)."),
    };

    DiagnosticsDto { summary, items }
}

fn item(
    title: &str,
    status: DiagnosticStatus,
    problem: &str,
    recommendation: &str,
) -> DiagnosticItemDto {
    DiagnosticItemDto {
        title: title.to_owned(),
        status,
        problem: problem.to_owned(),
        recommendation: recommendation.to_owned(),
    }
}

fn model_ready_message(settings: &AppSettings) -> String {
    format!("{} ist lokal verfuegbar.", settings.local_model.display_label())
}

#[cfg(test)]
mod tests {
    use super::*;
    use open_whisper_core::AppSettings;

    #[test]
    fn diagnostics_summary_reports_errors_and_warnings() {
        let settings = AppSettings::default();
        let dictation = DictationController::new();
        let report = collect(&settings, &dictation, None, "Autostart unklar");

        assert!(report.summary.contains("Fehler"));
    }
}
