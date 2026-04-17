use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use directories::ProjectDirs;
use open_whisper_core::{AppSettings, ModelPreset};
use reqwest::blocking::Client;

const USER_AGENT: &str = "open-whisper-desktop/0.1";
const DOWNLOAD_BUFFER_SIZE: usize = 64 * 1024;
const DOWNLOAD_PROGRESS_INTERVAL: Duration = Duration::from_millis(150);

pub struct ModelDownloadManager {
    state: ModelDownloadState,
    download_rx: Option<Receiver<DownloadEvent>>,
}

impl ModelDownloadManager {
    pub fn new() -> Self {
        Self {
            state: ModelDownloadState::Idle,
            download_rx: None,
        }
    }

    pub fn start_download(&mut self, settings: &AppSettings) -> Result<String, String> {
        if self.is_downloading() {
            return Err("Ein Modelldownload laeuft bereits.".to_owned());
        }

        let target_path = resolve_model_path(settings)?;
        if target_path.exists() {
            self.state = ModelDownloadState::Ready {
                path: target_path.clone(),
            };
            return Ok(format!(
                "Modell ist bereits vorhanden: {}",
                target_path.display()
            ));
        }

        let preset = settings.local_model;
        let download_url = preset.download_url().to_owned();
        let download_path = target_path.clone();
        let temp_path = temporary_download_path(&target_path);
        let (tx, rx) = mpsc::channel();

        self.download_rx = Some(rx);
        self.state = ModelDownloadState::Downloading {
            preset,
            path: target_path.clone(),
            downloaded_bytes: 0,
            total_bytes: None,
            started_at: Instant::now(),
        };

        thread::spawn(move || {
            let result = download_model_file(&download_url, &download_path, &temp_path, &tx);
            if let Err(err) = result {
                let _ = cleanup_temp_file(&temp_path);
                let _ = tx.send(DownloadEvent::Failed(err));
            }
        });

        Ok(format!(
            "Modelldownload fuer '{}' gestartet.",
            preset.label()
        ))
    }

    pub fn delete_downloaded_model(&mut self, settings: &AppSettings) -> Result<String, String> {
        if self.is_downloading() {
            return Err(
                "Ein laufender Download kann nicht gleichzeitig geloescht werden.".to_owned(),
            );
        }

        let path = resolve_model_path(settings)?;
        if !path.exists() {
            self.state = ModelDownloadState::Missing { path: path.clone() };
            return Ok(format!(
                "Kein lokales Modell unter {} gefunden.",
                path.display()
            ));
        }

        fs::remove_file(&path)
            .map_err(|err| format!("Modell konnte nicht geloescht werden: {err}"))?;
        self.state = ModelDownloadState::Missing { path: path.clone() };

        Ok(format!("Lokales Modell geloescht: {}", path.display()))
    }

    pub fn poll(&mut self) -> Vec<String> {
        let mut messages = Vec::new();

        if let Some(rx) = &self.download_rx {
            loop {
                match rx.try_recv() {
                    Ok(DownloadEvent::Progress {
                        downloaded_bytes,
                        total_bytes,
                    }) => {
                        if let ModelDownloadState::Downloading {
                            downloaded_bytes: current_downloaded,
                            total_bytes: current_total,
                            ..
                        } = &mut self.state
                        {
                            *current_downloaded = downloaded_bytes;
                            *current_total = total_bytes;
                        }
                    }
                    Ok(DownloadEvent::Completed {
                        path,
                        downloaded_bytes,
                    }) => {
                        self.download_rx = None;
                        self.state = ModelDownloadState::Ready { path: path.clone() };
                        messages.push(format!(
                            "Modelldownload abgeschlossen: {} ({})",
                            path.display(),
                            human_readable_size(downloaded_bytes)
                        ));
                        break;
                    }
                    Ok(DownloadEvent::Failed(err)) => {
                        self.download_rx = None;
                        self.state = ModelDownloadState::Failed {
                            message: err.clone(),
                        };
                        messages.push(err);
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.download_rx = None;
                        self.state = ModelDownloadState::Failed {
                            message: "Download-Worker wurde unerwartet beendet.".to_owned(),
                        };
                        messages.push("Download-Worker wurde unerwartet beendet.".to_owned());
                        break;
                    }
                }
            }
        }

        messages
    }

    pub fn refresh_local_state(&mut self, settings: &AppSettings) {
        if self.is_downloading() {
            return;
        }

        if let Ok(path) = resolve_model_path(settings) {
            self.state = if path.exists() {
                ModelDownloadState::Ready { path }
            } else {
                ModelDownloadState::Missing { path }
            };
        }
    }

    pub fn is_downloading(&self) -> bool {
        matches!(self.state, ModelDownloadState::Downloading { .. })
    }

    pub fn progress_fraction(&self) -> Option<f32> {
        match &self.state {
            ModelDownloadState::Downloading {
                downloaded_bytes,
                total_bytes: Some(total_bytes),
                ..
            } if *total_bytes > 0 => Some(*downloaded_bytes as f32 / *total_bytes as f32),
            _ => None,
        }
    }

    pub fn summary(&self, settings: &AppSettings) -> String {
        match &self.state {
            ModelDownloadState::Idle => summary_for_path(resolve_model_path(settings).ok()),
            ModelDownloadState::Missing { path } => {
                format!("Lokales Modell fehlt unter {}.", path.display())
            }
            ModelDownloadState::Ready { path } => summary_for_existing_path(path),
            ModelDownloadState::Downloading {
                preset,
                path,
                downloaded_bytes,
                total_bytes,
                started_at,
            } => {
                let progress = match total_bytes {
                    Some(total_bytes) if *total_bytes > 0 => format!(
                        "{} von {}",
                        human_readable_size(*downloaded_bytes),
                        human_readable_size(*total_bytes)
                    ),
                    _ => format!("{} geladen", human_readable_size(*downloaded_bytes)),
                };
                format!(
                    "Download fuer '{}' nach {} laeuft seit {} ({progress}).",
                    preset.label(),
                    path.display(),
                    human_readable_duration(started_at.elapsed())
                )
            }
            ModelDownloadState::Failed { message } => {
                format!("Letzter Modelldownload fehlgeschlagen: {message}")
            }
        }
    }
}

enum ModelDownloadState {
    Idle,
    Missing {
        path: PathBuf,
    },
    Ready {
        path: PathBuf,
    },
    Downloading {
        preset: ModelPreset,
        path: PathBuf,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        started_at: Instant,
    },
    Failed {
        message: String,
    },
}

enum DownloadEvent {
    Progress {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    Completed {
        path: PathBuf,
        downloaded_bytes: u64,
    },
    Failed(String),
}

pub fn resolve_model_path(settings: &AppSettings) -> Result<PathBuf, String> {
    if !settings.local_model_path.trim().is_empty() {
        return Ok(PathBuf::from(settings.local_model_path.trim()));
    }

    default_model_path(settings.local_model)
}

pub fn default_model_path(preset: ModelPreset) -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("dev", "awesome", "open-whisper")
        .ok_or_else(|| "Config-Verzeichnis fuer Modelle nicht verfuegbar.".to_owned())?;
    Ok(project_dirs
        .config_dir()
        .join("models")
        .join(preset.default_filename()))
}

fn download_model_file(
    url: &str,
    target_path: &Path,
    temp_path: &Path,
    tx: &mpsc::Sender<DownloadEvent>,
) -> Result<(), String> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Modellverzeichnis konnte nicht erstellt werden: {err}"))?;
    }

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .build()
        .map_err(|err| format!("HTTP-Client fuer Modelldownload fehlgeschlagen: {err}"))?;

    let mut response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| format!("Modelldownload fehlgeschlagen: {err}"))?;

    let total_bytes = response.content_length();
    let mut file = fs::File::create(temp_path)
        .map_err(|err| format!("Temporaere Modelldatei konnte nicht erstellt werden: {err}"))?;
    let mut buffer = [0_u8; DOWNLOAD_BUFFER_SIZE];
    let mut downloaded_bytes = 0_u64;
    let mut last_progress = Instant::now() - DOWNLOAD_PROGRESS_INTERVAL;

    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|err| format!("Lesefehler waehrend des Downloads: {err}"))?;
        if read == 0 {
            break;
        }

        file.write_all(&buffer[..read]).map_err(|err| {
            format!("Modell konnte nicht auf die Platte geschrieben werden: {err}")
        })?;
        downloaded_bytes += read as u64;

        if last_progress.elapsed() >= DOWNLOAD_PROGRESS_INTERVAL {
            let _ = tx.send(DownloadEvent::Progress {
                downloaded_bytes,
                total_bytes,
            });
            last_progress = Instant::now();
        }
    }

    file.sync_all()
        .map_err(|err| format!("Modelldatei konnte nicht finalisiert werden: {err}"))?;
    fs::rename(temp_path, target_path)
        .map_err(|err| format!("Modelldatei konnte nicht aktiviert werden: {err}"))?;

    let _ = tx.send(DownloadEvent::Completed {
        path: target_path.to_path_buf(),
        downloaded_bytes,
    });

    Ok(())
}

fn temporary_download_path(target_path: &Path) -> PathBuf {
    let file_name = target_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("model.bin");
    target_path.with_file_name(format!("{file_name}.part"))
}

fn cleanup_temp_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path)
            .map_err(|err| format!("Temp-Datei konnte nicht entfernt werden: {err}"))?;
    }

    Ok(())
}

fn summary_for_path(path: Option<PathBuf>) -> String {
    match path {
        Some(path) if path.exists() => summary_for_existing_path(&path),
        Some(path) => format!("Lokales Modell fehlt unter {}.", path.display()),
        None => "Lokaler Modellpfad ist aktuell nicht aufloesbar.".to_owned(),
    }
}

fn summary_for_existing_path(path: &Path) -> String {
    match fs::metadata(path) {
        Ok(metadata) => format!(
            "Lokales Modell bereit: {} ({})",
            path.display(),
            human_readable_size(metadata.len())
        ),
        Err(_) => format!("Lokales Modell bereit: {}", path.display()),
    }
}

fn human_readable_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

    let mut value = bytes as f64;
    let mut unit_index = 0_usize;
    while value >= 1024.0 && unit_index + 1 < UNITS.len() {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {}", UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

fn human_readable_duration(duration: Duration) -> String {
    if duration.as_secs() < 60 {
        format!("{}s", duration.as_secs())
    } else {
        format!("{}m {}s", duration.as_secs() / 60, duration.as_secs() % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_download_path_keeps_original_name() {
        let path = temporary_download_path(Path::new("/tmp/ggml-small.bin"));
        assert!(path.ends_with("ggml-small.bin.part"));
    }

    #[test]
    fn human_readable_size_uses_expected_units() {
        assert_eq!(human_readable_size(900), "900 B");
        assert_eq!(human_readable_size(2_048), "2.0 KB");
    }
}
