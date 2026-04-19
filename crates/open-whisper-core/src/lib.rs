use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum StartupBehavior {
    #[default]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TriggerMode {
    PushToTalk,
    #[default]
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WaveformStyle {
    #[default]
    CenteredBars,
    Line,
    Envelope,
}

impl WaveformStyle {
    pub const ALL: [Self; 3] = [Self::CenteredBars, Self::Line, Self::Envelope];

    pub fn label(self) -> &'static str {
        match self {
            Self::CenteredBars => "Zentrierte Balken",
            Self::Line => "Linie",
            Self::Envelope => "Welle",
        }
    }
}

impl<'de> serde::Deserialize<'de> for WaveformStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "centered_bars" => Self::CenteredBars,
            "line" => Self::Line,
            "envelope" => Self::Envelope,
            _ => Self::default(),
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WaveformColor {
    #[default]
    Accent,
    Blue,
    Green,
    Teal,
    Orange,
    Red,
    Pink,
    Purple,
}

impl WaveformColor {
    pub const ALL: [Self; 8] = [
        Self::Accent,
        Self::Blue,
        Self::Green,
        Self::Teal,
        Self::Orange,
        Self::Red,
        Self::Pink,
        Self::Purple,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Accent => "Systemfarbe",
            Self::Blue => "Blau",
            Self::Green => "Gruen",
            Self::Teal => "Tuerkis",
            Self::Orange => "Orange",
            Self::Red => "Rot",
            Self::Pink => "Pink",
            Self::Purple => "Violett",
        }
    }
}

impl<'de> serde::Deserialize<'de> for WaveformColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "accent" => Self::Accent,
            "blue" => Self::Blue,
            "green" => Self::Green,
            "teal" => Self::Teal,
            "orange" => Self::Orange,
            "red" => Self::Red,
            "pink" => Self::Pink,
            "purple" => Self::Purple,
            _ => Self::default(),
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ModelPreset {
    Tiny,
    Light,
    #[default]
    Standard,
    LargeV3TurboQ5_0,
    Quality,
    LargeV3Turbo,
    LargeV3,
}

impl ModelPreset {
    pub const ALL: [Self; 7] = [
        Self::Tiny,
        Self::Light,
        Self::Standard,
        Self::Quality,
        Self::LargeV3TurboQ5_0,
        Self::LargeV3Turbo,
        Self::LargeV3,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Tiny => "Mini",
            Self::Light => "Klein",
            Self::Standard => "Mittel",
            Self::LargeV3TurboQ5_0 => "Turbo",
            Self::Quality => "Gross",
            Self::LargeV3Turbo => "Turbo+",
            Self::LargeV3 => "Maximal",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Tiny => "Whisper Tiny (78 MB)",
            Self::Light => "Whisper Base (148 MB)",
            Self::Standard => "Whisper Small (488 MB)",
            Self::Quality => "Whisper Medium (1,5 GB)",
            Self::LargeV3TurboQ5_0 => "Whisper Large v3 Turbo Q5_0 (574 MB)",
            Self::LargeV3Turbo => "Whisper Large v3 Turbo (1,6 GB)",
            Self::LargeV3 => "Whisper Large v3 (3,1 GB)",
        }
    }

    pub fn whisper_model(self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Light => "base",
            Self::Standard => "small",
            Self::LargeV3TurboQ5_0 => "large-v3-turbo-q5_0",
            Self::Quality => "medium",
            Self::LargeV3Turbo => "large-v3-turbo",
            Self::LargeV3 => "large-v3",
        }
    }

    pub fn default_filename(self) -> &'static str {
        match self {
            Self::Tiny => "ggml-tiny.bin",
            Self::Light => "ggml-base.bin",
            Self::Standard => "ggml-small.bin",
            Self::LargeV3TurboQ5_0 => "ggml-large-v3-turbo-q5_0.bin",
            Self::Quality => "ggml-medium.bin",
            Self::LargeV3Turbo => "ggml-large-v3-turbo.bin",
            Self::LargeV3 => "ggml-large-v3.bin",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Tiny => {
                "Winziges Modell fuer extrem schwache Rechner und sehr kurze Reaktionszeit."
            }
            Self::Light => "Kleines lokales Modell fuer schwache Rechner und schnelle Reaktion.",
            Self::Standard => {
                "Mittleres lokales Modell als guter Standard fuer Alltag und Genauigkeit."
            }
            Self::LargeV3TurboQ5_0 => {
                "Quantisierte Turbo-Variante: Large-v3-Qualitaet bei kompakter Groesse."
            }
            Self::Quality => {
                "Grosses lokales Modell mit hoeherer Genauigkeit, aber mehr CPU/RAM-Bedarf."
            }
            Self::LargeV3Turbo => {
                "Schnelles Large-v3-Turbo mit hoher Genauigkeit, gute Balance fuer aktuelle Macs."
            }
            Self::LargeV3 => "Maximale Genauigkeit. Grosser Download und hoher RAM-Bedarf.",
        }
    }

    pub fn download_url(self) -> &'static str {
        match self {
            Self::Tiny => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
            }
            Self::Light => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
            }
            Self::Standard => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
            }
            Self::LargeV3TurboQ5_0 => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin"
            }
            Self::Quality => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
            }
            Self::LargeV3Turbo => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"
            }
            Self::LargeV3 => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
            }
        }
    }

    pub fn download_size_bytes(self) -> u64 {
        match self {
            Self::Tiny => 77_691_713,
            Self::Light => 147_951_465,
            Self::Standard => 487_601_967,
            Self::LargeV3TurboQ5_0 => 574_041_195,
            Self::Quality => 1_533_763_059,
            Self::LargeV3Turbo => 1_624_555_275,
            Self::LargeV3 => 3_095_033_483,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum LlmPreset {
    Small,
    #[default]
    Medium,
    Large,
}

impl LlmPreset {
    pub const ALL: [Self; 3] = [Self::Small, Self::Medium, Self::Large];

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Klein",
            Self::Medium => "Mittel",
            Self::Large => "Gross",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Small => "Gemma 4 E2B (3,5 GB)",
            Self::Medium => "Gemma 4 E4B (5,4 GB)",
            Self::Large => "Gemma 4 26B (17 GB)",
        }
    }

    pub fn default_filename(self) -> &'static str {
        match self {
            Self::Small => "google_gemma-4-E2B-it-Q4_K_M.gguf",
            Self::Medium => "google_gemma-4-E4B-it-Q4_K_M.gguf",
            Self::Large => "google_gemma-4-26B-A4B-it-Q4_K_M.gguf",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Small => {
                "Kleines Sprachmodell (Gemma 4 E2B). Schnell und sparsam, laeuft auch auf 8 GB RAM."
            }
            Self::Medium => {
                "Mittleres Sprachmodell (Gemma 4 E4B) als guter Standard fuer 16 GB RAM und mehr."
            }
            Self::Large => {
                "Grosses Sprachmodell (Gemma 4 26B A4B, Mixture-of-Experts) mit bester Qualitaet, braucht 32 GB RAM oder mehr."
            }
        }
    }

    pub fn approx_size_label(self) -> &'static str {
        match self {
            Self::Small => "ca. 3.5 GB",
            Self::Medium => "ca. 5.4 GB",
            Self::Large => "ca. 17 GB",
        }
    }

    pub fn approx_ram_mb(self) -> u64 {
        match self {
            Self::Small => 4_096,
            Self::Medium => 8_192,
            Self::Large => 20_480,
        }
    }

    pub fn context_size(self) -> u32 {
        match self {
            Self::Small | Self::Medium => 2_048,
            Self::Large => 4_096,
        }
    }

    pub fn download_url(self) -> &'static str {
        match self {
            Self::Small => {
                "https://huggingface.co/bartowski/google_gemma-4-E2B-it-GGUF/resolve/main/google_gemma-4-E2B-it-Q4_K_M.gguf"
            }
            Self::Medium => {
                "https://huggingface.co/bartowski/google_gemma-4-E4B-it-GGUF/resolve/main/google_gemma-4-E4B-it-Q4_K_M.gguf"
            }
            Self::Large => {
                "https://huggingface.co/bartowski/google_gemma-4-26B-A4B-it-GGUF/resolve/main/google_gemma-4-26B-A4B-it-Q4_K_M.gguf"
            }
        }
    }

    pub fn download_size_bytes(self) -> u64 {
        match self {
            Self::Small => 3_462_677_760,
            Self::Medium => 5_405_167_904,
            Self::Large => 17_035_037_632,
        }
    }
}

pub const LEGACY_LLM_FILENAMES: &[&str] = &[
    "Qwen2.5-1.5B-Instruct-Q4_K_M.gguf",
    "Qwen2.5-3B-Instruct-Q4_K_M.gguf",
    "Qwen2.5-7B-Instruct-Q4_K_M.gguf",
    "google_gemma-3-1b-it-Q4_K_M.gguf",
    "google_gemma-3-4b-it-Q4_K_M.gguf",
    "google_gemma-3-12b-it-Q4_K_M.gguf",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ProviderKind {
    #[default]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum PostProcessingBackend {
    #[default]
    Local,
    Ollama,
    LmStudio,
}

impl PostProcessingBackend {
    pub const ALL: [Self; 3] = [Self::Local, Self::Ollama, Self::LmStudio];

    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Lokales Modell",
            Self::Ollama => "Ollama",
            Self::LmStudio => "LM Studio",
        }
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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CustomLlmSource {
    LocalPath { path: String },
    DownloadUrl { url: String, filename: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomLlmModel {
    pub id: String,
    pub name: String,
    pub source: CustomLlmSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ProcessingMode {
    pub id: String,
    pub name: String,
    pub prompt: String,
}

impl ProcessingMode {
    pub fn cleanup() -> Self {
        Self {
            id: "cleanup".to_owned(),
            name: "Aufraeumen".to_owned(),
            prompt: "Korrigiere Satzzeichen, Grossschreibung und offensichtliche Erkennungsfehler im diktierten Text, ohne den Inhalt zu veraendern. Gib nur den bereinigten Text zurueck.".to_owned(),
        }
    }
}

impl Default for ProcessingMode {
    fn default() -> Self {
        Self::cleanup()
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
    pub show_recording_indicator: bool,
    pub waveform_style: WaveformStyle,
    pub waveform_color: WaveformColor,
    pub local_model: ModelPreset,
    pub local_model_path: String,
    pub local_llm: LlmPreset,
    pub local_llm_path: String,
    pub local_llm_auto_unload_secs: u32,
    pub active_provider: ProviderKind,
    pub active_post_processing_backend: PostProcessingBackend,
    pub active_custom_llm_id: String,
    pub custom_llm_models: Vec<CustomLlmModel>,
    pub ollama: ExternalProviderSettings,
    pub lm_studio: ExternalProviderSettings,
    pub post_processing_enabled: bool,
    pub modes: Vec<ProcessingMode>,
    pub active_mode_id: String,
}

impl AppSettings {
    pub fn normalize(&mut self) {
        let had_standard = self.modes.iter().any(|mode| mode.id == "standard");
        if had_standard {
            self.post_processing_enabled = self.active_mode_id != "standard";
            self.modes.retain(|mode| mode.id != "standard");
            if self.active_mode_id == "standard" {
                self.active_mode_id.clear();
            }
        }

        if self.modes.is_empty() {
            self.modes.push(ProcessingMode::cleanup());
        }

        if self.active_mode_id.trim().is_empty()
            || !self.modes.iter().any(|mode| mode.id == self.active_mode_id)
        {
            self.active_mode_id = self
                .modes
                .first()
                .map(|mode| mode.id.clone())
                .unwrap_or_default();
        }

        for mode in &mut self.modes {
            if mode.name.trim().is_empty() {
                mode.name = "Neue Nachbearbeitung".to_owned();
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

    pub fn active_mode_post_processing_enabled(&self) -> bool {
        self.post_processing_enabled
    }

    pub fn active_custom_llm(&self) -> Option<&CustomLlmModel> {
        if self.active_custom_llm_id.trim().is_empty() {
            return None;
        }
        self.custom_llm_models
            .iter()
            .find(|entry| entry.id == self.active_custom_llm_id)
    }

    pub fn active_provider_summary(&self) -> String {
        if !self.post_processing_enabled {
            return format!("Lokales Whisper mit {}", self.local_model.display_label());
        }
        let mode = self.active_mode();
        match self.active_post_processing_backend {
            PostProcessingBackend::Local => {
                let label = self
                    .active_custom_llm()
                    .map(|entry| entry.name.clone())
                    .unwrap_or_else(|| self.local_llm.display_label().to_owned());
                format!("Lokales Whisper + {} ({})", label, mode.name)
            }
            PostProcessingBackend::Ollama => {
                format!("Lokales Whisper + Ollama ({})", mode.name)
            }
            PostProcessingBackend::LmStudio => {
                format!("Lokales Whisper + LM Studio ({})", mode.name)
            }
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
            show_recording_indicator: true,
            waveform_style: WaveformStyle::default(),
            waveform_color: WaveformColor::default(),
            local_model: ModelPreset::default(),
            local_model_path: String::new(),
            local_llm: LlmPreset::default(),
            local_llm_path: String::new(),
            local_llm_auto_unload_secs: 180,
            active_provider: ProviderKind::default(),
            active_post_processing_backend: PostProcessingBackend::default(),
            active_custom_llm_id: String::new(),
            custom_llm_models: Vec::new(),
            ollama: ExternalProviderSettings::ollama_defaults(),
            lm_studio: ExternalProviderSettings::lm_studio_defaults(),
            post_processing_enabled: false,
            modes: vec![ProcessingMode::cleanup()],
            active_mode_id: "cleanup".to_owned(),
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
    pub expected_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomLlmStatusDto {
    pub id: String,
    pub name: String,
    pub source_label: String,
    pub path: String,
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub is_loaded: bool,
    pub needs_download: bool,
    pub progress_basis_points: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmModelStatusDto {
    pub preset_label: String,
    pub display_label: String,
    pub path: String,
    pub summary: String,
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub is_loaded: bool,
    pub progress_basis_points: Option<u16>,
    pub expected_size_bytes: u64,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemoteModelBackend {
    Ollama,
    LmStudio,
}

impl RemoteModelBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ollama => "Ollama",
            Self::LmStudio => "LM Studio",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteModelDto {
    pub backend: RemoteModelBackend,
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordingLevelsDto {
    pub levels: Vec<f32>,
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
    pub dictation_blocked_by_missing_model: bool,
    pub blocked_model_label: String,
    pub blocked_model_is_downloading: bool,
    pub blocked_model_progress_basis_points: Option<u16>,
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
        assert!(!settings.post_processing_enabled);
        assert_eq!(settings.active_mode_name(), "Aufraeumen");
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
    fn remote_provider_summary_uses_backend_and_mode() {
        let mut settings = AppSettings::default();
        settings.active_post_processing_backend = PostProcessingBackend::Ollama;
        settings.post_processing_enabled = true;
        settings.modes.push(ProcessingMode {
            id: "dev".to_owned(),
            name: "Entwickler".to_owned(),
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
        assert_eq!(settings.modes[0].id, "cleanup");
        assert_eq!(settings.active_mode_id, "cleanup");
    }

    #[test]
    fn normalize_migrates_legacy_standard_active() {
        let mut settings = AppSettings {
            modes: vec![
                ProcessingMode {
                    id: "standard".to_owned(),
                    name: "Standard".to_owned(),
                    prompt: String::new(),
                },
                ProcessingMode::cleanup(),
            ],
            active_mode_id: "standard".to_owned(),
            post_processing_enabled: false,
            ..AppSettings::default()
        };

        settings.normalize();

        assert!(!settings.modes.iter().any(|mode| mode.id == "standard"));
        assert_eq!(settings.active_mode_id, "cleanup");
        assert!(!settings.post_processing_enabled);
    }

    #[test]
    fn normalize_migrates_legacy_custom_active() {
        let mut settings = AppSettings {
            modes: vec![
                ProcessingMode {
                    id: "standard".to_owned(),
                    name: "Standard".to_owned(),
                    prompt: String::new(),
                },
                ProcessingMode::cleanup(),
            ],
            active_mode_id: "cleanup".to_owned(),
            post_processing_enabled: false,
            ..AppSettings::default()
        };

        settings.normalize();

        assert!(!settings.modes.iter().any(|mode| mode.id == "standard"));
        assert_eq!(settings.active_mode_id, "cleanup");
        assert!(settings.post_processing_enabled);
    }

    #[test]
    fn cleanup_mode_is_default_processing_mode() {
        let cleanup = ProcessingMode::cleanup();
        assert_eq!(cleanup.id, "cleanup");
        assert!(!cleanup.prompt.is_empty());
    }

    #[test]
    fn llm_preset_medium_is_default() {
        assert_eq!(LlmPreset::default(), LlmPreset::Medium);
    }

    #[test]
    fn llm_preset_small_download_url_contains_gemma_e2b() {
        assert!(LlmPreset::Small.download_url().contains("gemma-4-E2B"));
    }

    #[test]
    fn llm_preset_default_filename_is_gemma() {
        assert_eq!(
            LlmPreset::Medium.default_filename(),
            "google_gemma-4-E4B-it-Q4_K_M.gguf"
        );
    }

    #[test]
    fn legacy_llm_filenames_cover_previous_releases() {
        assert!(
            LEGACY_LLM_FILENAMES
                .iter()
                .any(|f| f.contains("Qwen2.5-3B"))
        );
        assert!(
            LEGACY_LLM_FILENAMES
                .iter()
                .any(|f| f.contains("gemma-3-4b"))
        );
    }

    #[test]
    fn local_llm_summary_uses_global_preset_when_mode_enabled() {
        let mut settings = AppSettings::default();
        settings.local_llm = LlmPreset::Large;
        settings.active_post_processing_backend = PostProcessingBackend::Local;
        settings.post_processing_enabled = true;
        settings.modes.push(ProcessingMode {
            id: "email".to_owned(),
            name: "Email".to_owned(),
            prompt: "Formatiere als Email.".to_owned(),
        });
        settings.active_mode_id = "email".to_owned();

        let summary = settings.active_provider_summary();
        assert!(summary.contains("Email"));
        assert!(summary.contains("Gemma 4 26B"));
    }

    #[test]
    fn default_post_processing_backend_is_local() {
        let settings = AppSettings::default();
        assert_eq!(
            settings.active_post_processing_backend,
            PostProcessingBackend::Local
        );
    }
}
