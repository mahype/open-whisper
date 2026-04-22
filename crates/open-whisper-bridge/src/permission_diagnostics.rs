use open_whisper_core::{AppSettings, DiagnosticItemDto, DiagnosticStatus, DiagnosticsDto};

use crate::{dictation::DictationController, hotkey::HotKeyController, model_manager};

pub fn collect(
    settings: &AppSettings,
    dictation: &DictationController,
    hotkey: Option<&HotKeyController>,
    autostart_summary: &str,
    external_hotkey: bool,
) -> DiagnosticsDto {
    let mut items = Vec::new();
    let available_devices = dictation.available_input_devices();

    if available_devices.is_empty() {
        items.push(item(
            "Microphone",
            DiagnosticStatus::Error,
            "No input device was detected.",
            "Check microphone permissions and that at least one input device is connected.",
        ));
    } else {
        items.push(item(
            "Microphone",
            DiagnosticStatus::Ok,
            &format!("{} input device(s) detected.", available_devices.len()),
            "No action needed.",
        ));
    }

    let selected_device_ok = settings.input_device_name == "System Default"
        || available_devices
            .iter()
            .any(|device| device == &settings.input_device_name);
    if selected_device_ok {
        items.push(item(
            "Input device",
            DiagnosticStatus::Ok,
            &format!(
                "The selected device '{}' is available.",
                settings.input_device_name
            ),
            "No action needed.",
        ));
    } else {
        items.push(item(
            "Input device",
            DiagnosticStatus::Warning,
            &format!(
                "The selected device '{}' is not currently available.",
                settings.input_device_name
            ),
            "Pick a different microphone in Onboarding or Settings.",
        ));
    }

    match model_manager::resolve_model_path(settings) {
        Ok(path) if path.exists() => items.push(item(
            "Local model",
            DiagnosticStatus::Ok,
            &model_ready_message(settings),
            "Local dictation is ready to use.",
        )),
        Ok(_) => items.push(item(
            "Local model",
            DiagnosticStatus::Warning,
            &format!(
                "{} has not been downloaded yet.",
                settings.local_model.display_label()
            ),
            "Download the selected model before your first dictation.",
        )),
        Err(err) => items.push(item(
            "Local model",
            DiagnosticStatus::Error,
            &err,
            "Check the model path or pick one of the built-in presets again.",
        )),
    }

    if external_hotkey {
        items.push(item(
            "Global hotkey",
            DiagnosticStatus::Info,
            &format!(
                "Hotkey '{}' is managed by the XDG GlobalShortcuts portal (Wayland).",
                settings.hotkey
            ),
            "On KDE/Plasma the portal accepts the binding directly. On GNOME 49 the \
             portal backend is still a stub (`BindShortcuts` not implemented \
             upstream) — until that lands, start dictation from the main window or \
             bind an in-shell custom keybinding to the app as a workaround.",
        ));
    } else if let Some(hotkey) = hotkey {
        if hotkey.is_registered() {
            items.push(item(
                "Global hotkey",
                DiagnosticStatus::Ok,
                &format!("Hotkey '{}' is registered.", settings.hotkey),
                "No action needed.",
            ));
        } else {
            items.push(item(
                "Global hotkey",
                DiagnosticStatus::Warning,
                &format!(
                    "Hotkey '{}' is not active yet.",
                    settings.hotkey
                ),
                "Check the hotkey combination and, on macOS, grant Accessibility or Input Monitoring permission if needed.",
            ));
        }
    } else {
        items.push(item(
            "Global hotkey",
            DiagnosticStatus::Error,
            "The hotkey integration could not be initialized.",
            "Restart the app and check whether the combination is already in use by another app.",
        ));
    }

    items.push(item(
        "Autostart",
        DiagnosticStatus::Info,
        autostart_summary,
        "Adjust the behavior in Onboarding or Settings.",
    ));

    #[cfg(target_os = "macos")]
    {
        items.push(item(
            "macOS privacy",
            DiagnosticStatus::Info,
            "macOS may require privacy permissions for the microphone, global hotkey, and pasting into other apps.",
            "If you run into issues, open System Settings > Privacy & Security and check Microphone, Accessibility, and Input Monitoring.",
        ));
    }

    #[cfg(target_os = "linux")]
    {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into());
        let current_desktop =
            std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".into());

        items.push(item(
            "Linux session",
            DiagnosticStatus::Info,
            &format!(
                "Session: {session_type} on {current_desktop}."
            ),
            match session_type.as_str() {
                "wayland" => "On Wayland, hotkey binding goes through the XDG GlobalShortcuts portal and paste through the RemoteDesktop portal. A portal dialog will ask for permission on first use.",
                "x11" => "On X11, global-hotkey and libei/XTest work directly without portals.",
                _ => "Unable to detect the session type — set XDG_SESSION_TYPE if autodetection fails.",
            },
        ));

        // Audio server probe — PipeWire is default on Fedora/Ubuntu 24.04+; PulseAudio on older.
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
        let has_pipewire = !runtime_dir.is_empty()
            && std::path::Path::new(&format!("{runtime_dir}/pipewire-0")).exists();
        let has_pulse = !runtime_dir.is_empty()
            && std::path::Path::new(&format!("{runtime_dir}/pulse/native")).exists();
        let audio_summary = match (has_pipewire, has_pulse) {
            (true, _) => "PipeWire detected.",
            (false, true) => "PulseAudio detected (PipeWire not running).",
            (false, false) => "Neither PipeWire nor PulseAudio socket was found.",
        };
        let audio_status = if has_pipewire || has_pulse {
            DiagnosticStatus::Ok
        } else {
            DiagnosticStatus::Warning
        };
        items.push(item(
            "Audio server",
            audio_status,
            audio_summary,
            "Install pipewire or pulseaudio so the microphone is accessible via cpal.",
        ));

        // Tray availability (StatusNotifierItem host). We do a cheap filesystem
        // heuristic — a real D-Bus probe happens in the GTK shell's onboarding.
        let snw_hint = match current_desktop.to_lowercase().as_str() {
            d if d.contains("kde") || d.contains("plasma") => {
                "KDE/Plasma ships a StatusNotifier host out of the box."
            }
            d if d.contains("xfce")
                || d.contains("cinnamon")
                || d.contains("budgie")
                || d.contains("mate") =>
            {
                "Your desktop supports StatusNotifierItem by default."
            }
            d if d.contains("gnome") => {
                "Vanilla GNOME does not show tray icons; install the 'AppIndicator and KStatusNotifierItem Support' extension."
            }
            _ => {
                "If the tray icon does not appear, your desktop may not implement StatusNotifierItem."
            }
        };
        items.push(item(
            "System tray",
            DiagnosticStatus::Info,
            &format!("Desktop: {current_desktop}."),
            snw_hint,
        ));
    }

    #[cfg(target_os = "windows")]
    {
        items.push(item(
            "Windows focus",
            DiagnosticStatus::Info,
            "Some apps block simulated input while UAC dialogs or security software are active.",
            "Test pasting in a plain text app and check focus or security software if needed.",
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
        (0, 0) => "Diagnostics: no open issues detected.".to_owned(),
        (0, warnings) => format!("Diagnostics: {warnings} warning(s), no errors."),
        (errors, warnings) => format!("Diagnostics: {errors} error(s), {warnings} warning(s)."),
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
    format!(
        "{} is available locally.",
        settings.local_model.display_label()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use open_whisper_core::AppSettings;

    #[test]
    fn diagnostics_summary_reports_errors_and_warnings() {
        let settings = AppSettings::default();
        let dictation = DictationController::new();
        let report = collect(&settings, &dictation, None, "Autostart unknown");

        assert!(report.summary.contains("error"));
    }
}
