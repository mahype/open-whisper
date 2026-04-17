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
        Self::PushToTalk
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
}

impl AppSettings {
    pub fn active_provider_summary(&self) -> String {
        match self.active_provider {
            ProviderKind::LocalWhisper => format!(
                "{} mit lokalem Modell '{}'",
                self.active_provider.label(),
                self.local_model.whisper_model()
            ),
            ProviderKind::Ollama => format!(
                "{} ueber {} mit Modell '{}'",
                self.active_provider.label(),
                self.ollama.endpoint,
                self.ollama.model_name
            ),
            ProviderKind::LmStudio => format!(
                "{} ueber {} mit Modell '{}'",
                self.active_provider.label(),
                self.lm_studio.endpoint,
                self.lm_studio.model_name
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
            transcription_language: "de".to_owned(),
            insert_text_automatically: true,
            insert_delay_ms: 120,
            restore_clipboard_after_insert: true,
            vad_enabled: true,
            vad_threshold: 0.014,
            vad_silence_ms: 900,
            local_model: ModelPreset::default(),
            local_model_path: String::new(),
            active_provider: ProviderKind::default(),
            ollama: ExternalProviderSettings::ollama_defaults(),
            lm_studio: ExternalProviderSettings::lm_studio_defaults(),
        }
    }
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
        let settings = AppSettings {
            active_provider: ProviderKind::Ollama,
            ..AppSettings::default()
        };

        assert!(settings.active_provider_summary().contains("11434"));
        assert!(settings.active_provider_summary().contains("whisper"));
    }
}
