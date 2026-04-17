use std::{env, path::PathBuf};

use auto_launch::AutoLaunchBuilder;
use open_whisper_core::{AppSettings, StartupBehavior};

pub const START_HIDDEN_FLAG: &str = "--hidden";

pub struct AutostartManager {
    launcher: Option<auto_launch::AutoLaunch>,
    status: String,
    enabled: Option<bool>,
}

impl AutostartManager {
    pub fn new() -> Self {
        match build_launcher() {
            Ok(launcher) => {
                let enabled = launcher.is_enabled().ok();
                Self {
                    launcher: Some(launcher),
                    status: "Systemstart noch nicht synchronisiert.".to_owned(),
                    enabled,
                }
            }
            Err(err) => Self {
                launcher: None,
                status: format!("Systemstart nicht verfuegbar: {err}"),
                enabled: None,
            },
        }
    }

    pub fn sync_saved_settings(
        &mut self,
        settings: &AppSettings,
    ) -> Result<Option<String>, String> {
        let Some(launcher) = &self.launcher else {
            return Err(self.status.clone());
        };

        match settings.startup_behavior {
            StartupBehavior::AskOnFirstLaunch => {
                let enabled = launcher.is_enabled().map_err(|err| {
                    format!("Systemstart-Status konnte nicht gelesen werden: {err}")
                })?;
                self.enabled = Some(enabled);
                self.status = if enabled {
                    "Systemstart ist aktiv. 'Beim ersten Start fragen' laesst den OS-Status unveraendert.".to_owned()
                } else {
                    "Systemstart ist inaktiv. 'Beim ersten Start fragen' laesst den OS-Status unveraendert.".to_owned()
                };
                Ok(None)
            }
            StartupBehavior::LaunchAtLogin => {
                if !launcher.is_enabled().map_err(|err| {
                    format!("Systemstart-Status konnte nicht gelesen werden: {err}")
                })? {
                    launcher.enable().map_err(|err| {
                        format!("Systemstart konnte nicht aktiviert werden: {err}")
                    })?;
                }

                let enabled = launcher
                    .is_enabled()
                    .map_err(|err| format!("Systemstart konnte nicht bestaetigt werden: {err}"))?;
                self.enabled = Some(enabled);
                self.status = if enabled {
                    "Systemstart ist aktiv und startet die App verborgen.".to_owned()
                } else {
                    "Systemstart sollte aktiv sein, konnte aber nicht bestaetigt werden.".to_owned()
                };
                Ok(Some(if enabled {
                    "Systemstart aktiviert.".to_owned()
                } else {
                    "Systemstart konnte nicht bestaetigt werden.".to_owned()
                }))
            }
            StartupBehavior::ManualLaunch => {
                if launcher.is_enabled().map_err(|err| {
                    format!("Systemstart-Status konnte nicht gelesen werden: {err}")
                })? {
                    launcher.disable().map_err(|err| {
                        format!("Systemstart konnte nicht deaktiviert werden: {err}")
                    })?;
                }

                let enabled = launcher
                    .is_enabled()
                    .map_err(|err| format!("Systemstart konnte nicht bestaetigt werden: {err}"))?;
                self.enabled = Some(enabled);
                self.status = if enabled {
                    "Systemstart sollte deaktiviert sein, ist aber noch aktiv.".to_owned()
                } else {
                    "Systemstart ist deaktiviert.".to_owned()
                };
                Ok(Some(if enabled {
                    "Systemstart ist noch aktiv.".to_owned()
                } else {
                    "Systemstart deaktiviert.".to_owned()
                }))
            }
        }
    }

    pub fn summary(&self) -> &str {
        &self.status
    }
    pub fn start_hidden_requested() -> bool {
        env::args().any(|arg| arg == START_HIDDEN_FLAG)
    }
}

fn build_launcher() -> Result<auto_launch::AutoLaunch, String> {
    let executable_path = current_executable_path()?;
    let app_path = executable_path
        .to_str()
        .ok_or_else(|| "Pfad zur aktuellen App ist nicht UTF-8.".to_owned())?;

    let mut builder = AutoLaunchBuilder::new();
    builder
        .set_app_name("open-whisper")
        .set_app_path(app_path)
        .set_args(&[START_HIDDEN_FLAG]);

    #[cfg(target_os = "linux")]
    {
        use auto_launch::LinuxLaunchMode;
        builder.set_linux_launch_mode(LinuxLaunchMode::XdgAutostart);
    }

    #[cfg(target_os = "macos")]
    {
        use auto_launch::MacOSLaunchMode;
        builder.set_macos_launch_mode(MacOSLaunchMode::LaunchAgent);
    }

    #[cfg(target_os = "windows")]
    {
        use auto_launch::WindowsEnableMode;
        builder.set_windows_enable_mode(WindowsEnableMode::CurrentUser);
    }

    builder.build().map_err(|err| err.to_string())
}

fn current_executable_path() -> Result<PathBuf, String> {
    let path = env::current_exe().map_err(|err| format!("Pfad zur App nicht verfuegbar: {err}"))?;
    if !path.is_absolute() {
        return Err("Pfad zur App ist nicht absolut.".to_owned());
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_flag_constant_matches_cli_expectation() {
        assert_eq!(START_HIDDEN_FLAG, "--hidden");
    }
}
