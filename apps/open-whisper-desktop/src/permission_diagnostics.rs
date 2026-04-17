use std::path::PathBuf;

use crate::{
    desktop_integration::DesktopIntegration, dictation::DictationController,
    model_manager::resolve_model_path,
};
use open_whisper_core::AppSettings;

pub struct PermissionReport {
    items: Vec<PermissionItem>,
}

impl PermissionReport {
    pub fn collect(
        settings: &AppSettings,
        dictation: &DictationController,
        desktop: &DesktopIntegration,
    ) -> Self {
        let mut items = Vec::new();

        let available_devices = dictation.available_input_devices();
        if available_devices.is_empty() {
            items.push(PermissionItem::error(
                "Mikrofon",
                "Es wurde kein Eingabegeraet gefunden. Das ist oft ein Hinweis auf fehlende Mikrofon-Freigabe oder auf ein nicht angeschlossenes Geraet.",
            ));
        } else {
            items.push(PermissionItem::ok(
                "Mikrofon",
                format!("{} Eingabegeraet(e) erkannt.", available_devices.len()),
            ));
        }

        if available_devices
            .iter()
            .any(|device| device == &settings.input_device_name)
        {
            items.push(PermissionItem::ok(
                "Eingabegeraet",
                format!(
                    "Ausgewaehltes Geraet '{}' ist verfuegbar.",
                    settings.input_device_name
                ),
            ));
        } else {
            items.push(PermissionItem::warning(
                "Eingabegeraet",
                format!(
                    "Das ausgewaehlte Geraet '{}' ist aktuell nicht verfuegbar. Bitte waehle es im Setup neu aus.",
                    settings.input_device_name
                ),
            ));
        }

        match resolve_model_path(settings) {
            Ok(path) if path.exists() => items.push(PermissionItem::ok(
                "Lokales Modell",
                model_ready_message(path),
            )),
            Ok(path) => items.push(PermissionItem::warning(
                "Lokales Modell",
                format!(
                    "Das ausgewaehlte Modell liegt noch nicht unter {}. Lade es im Setup herunter, bevor du diktierst.",
                    path.display()
                ),
            )),
            Err(err) => items.push(PermissionItem::error("Lokales Modell", err)),
        }

        if desktop.tray_active() {
            items.push(PermissionItem::ok(
                "Tray",
                "Tray-Integration ist aktiv.".to_owned(),
            ));
        } else {
            items.push(PermissionItem::warning(
                "Tray",
                "Tray ist noch nicht aktiv. Auf manchen Linux-Desktops fehlen dafuer Systemkomponenten.",
            ));
        }

        if desktop.hotkey_active() {
            items.push(PermissionItem::ok(
                "Globaler Hotkey",
                format!("Hotkey '{}' ist registriert.", settings.hotkey),
            ));
        } else {
            let detail = desktop
                .initialization_message()
                .map(|message| message.to_owned())
                .unwrap_or_else(|| {
                    "Der globale Hotkey ist noch nicht aktiv. Das kann an OS-Rechten oder an einer ungueltigen Hotkey-Konfiguration liegen.".to_owned()
                });
            items.push(PermissionItem::warning("Globaler Hotkey", detail));
        }

        #[cfg(target_os = "macos")]
        {
            items.push(PermissionItem::info(
                "macOS Rechte",
                "Fuer globalen Hotkey und Einfuegen in andere Apps kann macOS Accessibility und Input Monitoring verlangen. Wenn Aufnahme oder Paste scheitern, pruefe Systemeinstellungen > Datenschutz & Sicherheit.",
            ));
        }

        #[cfg(target_os = "linux")]
        {
            if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                items.push(PermissionItem::warning(
                    "Wayland",
                    "Wayland kann globale Hotkeys und simuliertes Einfuegen einschraenken. Falls der Hotkey oder das Paste in Fremd-Apps nicht geht, teste eine X11-Sitzung oder die Desktop-spezifischen Rechte.",
                ));
            } else {
                items.push(PermissionItem::info(
                    "Linux Sitzung",
                    "Keine Wayland-Sitzung erkannt. Globale Hotkeys und simuliertes Paste sind unter X11 meist robuster.",
                ));
            }
        }

        #[cfg(target_os = "windows")]
        {
            items.push(PermissionItem::info(
                "Windows Eingabe",
                "Wenn das Einfuegen in andere Apps blockiert wird, pruefe Fokus, UAC-Prompts oder Sicherheitssoftware.",
            ));
        }

        Self { items }
    }

    pub fn items(&self) -> &[PermissionItem] {
        &self.items
    }

    pub fn has_errors(&self) -> bool {
        self.items
            .iter()
            .any(|item| item.status == PermissionStatus::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.items
            .iter()
            .any(|item| item.status == PermissionStatus::Warning)
    }

    pub fn summary(&self) -> String {
        let errors = self
            .items
            .iter()
            .filter(|item| item.status == PermissionStatus::Error)
            .count();
        let warnings = self
            .items
            .iter()
            .filter(|item| item.status == PermissionStatus::Warning)
            .count();

        match (errors, warnings) {
            (0, 0) => "Diagnose: keine offenen Probleme erkannt.".to_owned(),
            (0, warnings) => format!("Diagnose: {warnings} Warnung(en), keine Fehler."),
            (errors, warnings) => {
                format!("Diagnose: {errors} Fehler, {warnings} Warnung(en).")
            }
        }
    }
}

pub struct PermissionItem {
    title: &'static str,
    detail: String,
    status: PermissionStatus,
}

impl PermissionItem {
    fn ok(title: &'static str, detail: impl Into<String>) -> Self {
        Self {
            title,
            detail: detail.into(),
            status: PermissionStatus::Ok,
        }
    }

    fn info(title: &'static str, detail: impl Into<String>) -> Self {
        Self {
            title,
            detail: detail.into(),
            status: PermissionStatus::Info,
        }
    }

    fn warning(title: &'static str, detail: impl Into<String>) -> Self {
        Self {
            title,
            detail: detail.into(),
            status: PermissionStatus::Warning,
        }
    }

    fn error(title: &'static str, detail: impl Into<String>) -> Self {
        Self {
            title,
            detail: detail.into(),
            status: PermissionStatus::Error,
        }
    }

    pub fn title(&self) -> &'static str {
        self.title
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn status(&self) -> PermissionStatus {
        self.status
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    Ok,
    Info,
    Warning,
    Error,
}

impl PermissionStatus {
    pub fn badge(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Info => "Hinweis",
            Self::Warning => "Warnung",
            Self::Error => "Fehler",
        }
    }
}

fn model_ready_message(path: PathBuf) -> String {
    format!("Lokales Modell gefunden unter {}.", path.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_report_detects_errors() {
        let report = PermissionReport {
            items: vec![PermissionItem::error("Test", "Fehler")],
        };

        assert!(report.has_errors());
        assert!(!report.has_warnings());
    }

    #[test]
    fn status_badges_are_stable() {
        assert_eq!(PermissionStatus::Warning.badge(), "Warnung");
        assert_eq!(PermissionStatus::Ok.badge(), "OK");
    }
}
