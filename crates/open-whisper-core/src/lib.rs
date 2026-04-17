use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StartupBehavior {
    AskOnFirstLaunch,
    LaunchAtLogin,
    ManualLaunch,
}

impl StartupBehavior {
    pub const ALL: [Self; 3] = [
        Self::AskOnFirstLaunch,
        Self::LaunchAtLogin,
        Self::ManualLaunch,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::AskOnFirstLaunch => "Beim ersten Start fragen",
            Self::LaunchAtLogin => "Mit dem System starten",
            Self::ManualLaunch => "Nur manuell starten",
        }
    }
}

impl Default for StartupBehavior {
    fn default() -> Self {
        Self::AskOnFirstLaunch
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerMode {
    PushToTalk,
    Toggle,
}

impl TriggerMode {
    pub const ALL: [Self; 2] = [Self::PushToTalk, Self::Toggle];

    pub fn label(self) -> &'static str {
        match self {
            Self::PushToTalk => "Push-to-talk",
            Self::Toggle => "Toggle",
        }
    }
}

impl Default for TriggerMode {
    fn default() -> Self {
        Self::Toggle
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelPreset {
    Light,
    Standard,
    Quality,
}

impl ModelPreset {
    pub const ALL: [Self; 3] = [Self::Light, Self::Standard, Self::Quality];

    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Klein",
            Self::Standard => "Mittel",
            Self::Quality => "Gross",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Light => "Whisper Base (klein)",
            Self::Standard => "Whisper Small (mittel)",
            Self::Quality => "Whisper Medium (gross)",
        }
    }

    pub fn whisper_model(self) -> &'static str {
        match self {
            Self::Light => "base",
            Self::Standard => "small",
            Self::Quality => "medium",
        }
    }

    pub fn default_filename(self) -> &'static str {
        match self {
            Self::Light => "ggml-base.bin",
            Self::Standard => "ggml-small.bin",
            Self::Quality => "ggml-medium.bin",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Light => "Kleines lokales Modell fuer schwache Rechner und schnelle Reaktion.",
            Self::Standard => {
                "Mittleres lokales Modell als guter Standard fuer Alltag und Genauigkeit."
            }
            Self::Quality => {
                "Grosses lokales Modell mit hoeherer Genauigkeit, aber mehr CPU/RAM-Bedarf."
            }
        }
    }

    pub fn download_url(self) -> &'static str {
        match self {
            Self::Light => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
            }
            Self::Standard => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
            }
            Self::Quality => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
            }
        }
    }
}

impl Default for ModelPreset {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    LocalWhisper,
    Ollama,
    LmStudio,
}

impl ProviderKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalWhisper => "Local Whisper",
            Self::Ollama => "Ollama",
            Self::LmStudio => "LM Studio",
        }
    }
}

impl Default for ProviderKind {
    fn default() -> Self {
        Self::LocalWhisper
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostProcessingProvider {
    Disabled,
    Ollama,
    LmStudio,
}

impl PostProcessingProvider {
    pub const ALL: [Self; 3] = [Self::Disabled, Self::Ollama, Self::LmStudio];

    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "Aus",
            Self::Ollama => "Ollama",
            Self::LmStudio => "LM Studio",
        }
    }
}

impl Default for PostProcessingProvider {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalProviderSettings {
    pub endpoint: String,
    pub model_name: String,
}

impl ExternalProviderSettings {
    pub fn ollama_defaults() -> Self {
        Self {
            endpoint: "http://127.0.0.1:11434".to_owned(),
            model_name: "whisper".to_owned(),
        }
    }

    pub fn lm_studio_defaults() -> Self {
        Self {
            endpoint: "http://127.0.0.1:1234".to_owned(),
            model_name: "openai/whisper-small".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessingMode {
    pub id: String,
    pub name: String,
    pub post_processing_enabled: bool,
    pub post_processing_provider: PostProcessingProvider,
    pub prompt: String,
}

impl ProcessingMode {
    pub fn standard() -> Self {
        Self {
            id: "standard".to_owned(),
            name: "Standard".to_owned(),
            post_processing_enabled: false,
            post_processing_provider: PostProcessingProvider::Disabled,
            prompt: String::new(),
        }
    }

    pub fn post_processing_summary(&self) -> &'static str {
        if !self.post_processing_enabled {
            return "Direktes Diktat ohne Nachverarbeitung";
        }

        match self.post_processing_provider {
            PostProcessingProvider::Disabled => "Direktes Diktat ohne Nachverarbeitung",
            PostProcessingProvider::Ollama => "Nachverarbeitung ueber Ollama",
            PostProcessingProvider::LmStudio => "Nachverarbeitung ueber LM Studio",
        }
    }
}

impl Default for ProcessingMode {
    fn default() -> Self {
        Self::standard()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppSettings {
    pub onboarding_completed: bool,
    pub startup_behavior: StartupBehavior,
    pub input_device_name: String,
    pub hotkey: String,
    pub trigger_mode: TriggerMode,
    pub transcription_language: String,
    pub insert_text_automatically: bool,
    pub insert_delay_ms: u32,
    pub restore_clipboard_after_insert: bool,
    pub vad_enabled: bool,
    pub vad_threshold: f32,
    pub vad_silence_ms: u32,
    pub local_model: ModelPreset,
    pub local_model_path: String,
    pub active_provider: ProviderKind,
    pub ollama: ExternalProviderSettings,
    pub lm_studio: ExternalProviderSettings,
    pub modes: Vec<ProcessingMode>,
    pub active_mode_id: String,
}

impl AppSettings {
    pub fn normalize(&mut self) {
        if self.modes.is_empty() {
            self.modes.push(ProcessingMode::standard());
        }

        if !self.modes.iter().any(|mode| mode.id == "standard") {
            self.modes.insert(0, ProcessingMode::standard());
        }

        if self.active_mode_id.trim().is_empty()
            || !self.modes.iter().any(|mode| mode.id == self.active_mode_id)
        {
            self.active_mode_id = self
                .modes
                .first()
                .map(|mode| mode.id.clone())
                .unwrap_or_else(|| "standard".to_owned());
        }

        for mode in &mut self.modes {
            if mode.name.trim().is_empty() {
                mode.name = "Neuer Modus".to_owned();
            }
        }
    }

    pub fn active_mode(&self) -> &ProcessingMode {
        self.modes
            .iter()
            .find(|mode| mode.id == self.active_mode_id)
            .or_else(|| self.modes.first())
            .expect("normalized settings must always contain at least one mode")
    }

    pub fn active_mode_name(&self) -> &str {
        &self.active_mode().name
    }

    pub fn active_mode_provider(&self) -> PostProcessingProvider {
        let mode = self.active_mode();
        if !mode.post_processing_enabled {
            PostProcessingProvider::Disabled
        } else {
            mode.post_processing_provider
        }
    }

    pub fn active_provider_summary(&self) -> String {
        let mode = self.active_mode();
        match self.active_mode_provider() {
            PostProcessingProvider::Disabled => format!(
                "Lokales Whisper mit {}",
                self.local_model.display_label()
            ),
            PostProcessingProvider::Ollama => format!(
                "Lokales Whisper + Ollama im Modus '{}'",
                mode.name
            ),
            PostProcessingProvider::LmStudio => format!(
                "Lokales Whisper + LM Studio im Modus '{}'",
                mode.name
            ),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            onboarding_completed: false,
            startup_behavior: StartupBehavior::default(),
            input_device_name: "System Default".to_owned(),
            hotkey: "Ctrl+Shift+Space".to_owned(),
            trigger_mode: TriggerMode::default(),
            transcription_language: "auto".to_owned(),
            insert_text_automatically: true,
            insert_delay_ms: 120,
            restore_clipboard_after_insert: true,
            vad_enabled: false,
            vad_threshold: 0.014,
            vad_silence_ms: 900,
            local_model: ModelPreset::default(),
            local_model_path: String::new(),
            active_provider: ProviderKind::default(),
            ollama: ExternalProviderSettings::ollama_defaults(),
            lm_studio: ExternalProviderSettings::lm_studio_defaults(),
            modes: vec![ProcessingMode::standard()],
            active_mode_id: "standard".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceDto {
    pub name: String,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelStatusDto {
    pub preset_label: String,
    pub backend_model_name: String,
    pub path: String,
    pub summary: String,
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub progress_basis_points: Option<u16>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticStatus {
    Ok,
    Info,
    Warning,
    Error,
}

impl DiagnosticStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Info => "Hinweis",
            Self::Warning => "Warnung",
            Self::Error => "Fehler",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticItemDto {
    pub title: String,
    pub status: DiagnosticStatus,
    pub problem: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticsDto {
    pub summary: String,
    pub items: Vec<DiagnosticItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeStatusDto {
    pub is_recording: bool,
    pub is_transcribing: bool,
    pub is_post_processing: bool,
    pub last_status: String,
    pub last_transcript: String,
    pub dictation_trigger_count: u64,
    pub hotkey_registered: bool,
    pub hotkey_text: String,
    pub startup_summary: String,
    pub provider_summary: String,
    pub active_mode_name: String,
    pub onboarding_completed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_select_local_whisper() {
        let settings = AppSettings::default();

        assert_eq!(settings.active_provider, ProviderKind::LocalWhisper);
        assert_eq!(settings.local_model, ModelPreset::Standard);
        assert!(!settings.onboarding_completed);
        assert!(settings.insert_text_automatically);
        assert!(settings.restore_clipboard_after_insert);
        assert_eq!(settings.trigger_mode, TriggerMode::Toggle);
        assert!(!settings.vad_enabled);
        assert_eq!(settings.active_mode_name(), "Standard");
    }

    #[test]
    fn quality_maps_to_medium_model() {
        assert_eq!(ModelPreset::Quality.whisper_model(), "medium");
    }

    #[test]
    fn standard_preset_uses_small_model_filename() {
        assert_eq!(ModelPreset::Standard.default_filename(), "ggml-small.bin");
    }

    #[test]
    fn light_preset_uses_expected_download_url() {
        assert!(ModelPreset::Light.download_url().contains("ggml-base.bin"));
    }

    #[test]
    fn quality_label_maps_to_gross() {
        assert_eq!(ModelPreset::Quality.label(), "Gross");
    }

    #[test]
    fn remote_provider_summary_uses_endpoint_and_model() {
        let mut settings = AppSettings::default();
        settings.modes.push(ProcessingMode {
            id: "dev".to_owned(),
            name: "Entwickler".to_owned(),
            post_processing_enabled: true,
            post_processing_provider: PostProcessingProvider::Ollama,
            prompt: "Arbeite wie ein Entwickler.".to_owned(),
        });
        settings.active_mode_id = "dev".to_owned();

        assert!(settings.active_provider_summary().contains("Ollama"));
        assert!(settings.active_provider_summary().contains("Entwickler"));
    }

    #[test]
    fn diagnostics_status_has_stable_label() {
        assert_eq!(DiagnosticStatus::Warning.label(), "Warnung");
    }

    #[test]
    fn device_dto_marks_selection() {
        let dto = DeviceDto {
            name: "Mic".to_owned(),
            is_selected: true,
        };

        assert!(dto.is_selected);
    }

    #[test]
    fn normalize_recovers_missing_modes() {
        let mut settings = AppSettings {
            modes: Vec::new(),
            active_mode_id: String::new(),
            ..AppSettings::default()
        };

        settings.normalize();

        assert_eq!(settings.modes.len(), 1);
        assert_eq!(settings.active_mode_id, "standard");
    }
}
