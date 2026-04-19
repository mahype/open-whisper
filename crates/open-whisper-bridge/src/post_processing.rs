use std::{path::Path, time::Duration};

use open_whisper_core::{AppSettings, CustomLlmSource, PostProcessingBackend};
use reqwest::blocking::Client;
use serde_json::{Value, json};

use crate::{llm_model_manager, local_llm};

const USER_AGENT: &str = "open-whisper-bridge/0.1";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);

pub fn process_text(settings: &AppSettings, raw_transcript: &str) -> Result<String, String> {
    if !settings.post_processing_enabled {
        return Ok(raw_transcript.to_owned());
    }

    let mode = settings.active_mode();

    let text = match settings.active_post_processing_backend {
        PostProcessingBackend::Local => {
            if let Some(custom) = settings.active_custom_llm() {
                match &custom.source {
                    CustomLlmSource::LocalPath { path } => {
                        local_llm::generate_with_custom_path(
                            &custom.id,
                            &custom.name,
                            Path::new(path),
                            &mode.prompt,
                            raw_transcript,
                        )?
                    }
                    CustomLlmSource::DownloadUrl { .. } => {
                        let path = llm_model_manager::default_custom_llm_path(&custom.id)?;
                        if !path.exists() {
                            return Err(format!(
                                "Eigenes Sprachmodell '{}' wurde noch nicht heruntergeladen.",
                                custom.name
                            ));
                        }
                        local_llm::generate_with_custom_path(
                            &custom.id,
                            &custom.name,
                            &path,
                            &mode.prompt,
                            raw_transcript,
                        )?
                    }
                }
            } else {
                local_llm::generate_with_shared_runtime(
                    settings.local_llm,
                    &mode.prompt,
                    raw_transcript,
                )?
            }
        }
        PostProcessingBackend::Ollama => {
            let client = build_http_client()?;
            let system_prompt = build_system_prompt(&mode.prompt);
            request_ollama(
                &client,
                &settings.ollama.endpoint,
                &settings.ollama.model_name,
                &system_prompt,
                raw_transcript,
            )?
        }
        PostProcessingBackend::LmStudio => {
            let client = build_http_client()?;
            let system_prompt = build_system_prompt(&mode.prompt);
            request_lm_studio(
                &client,
                &settings.lm_studio.endpoint,
                &settings.lm_studio.model_name,
                &system_prompt,
                raw_transcript,
            )?
        }
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Die Nachverarbeitung lieferte keinen Text zurueck.".to_owned());
    }

    Ok(trimmed.to_owned())
}

fn build_http_client() -> Result<Client, String> {
    Client::builder().timeout(REQUEST_TIMEOUT).build().map_err(|err| {
        format!("HTTP-Client fuer Nachverarbeitung konnte nicht erstellt werden: {err}")
    })
}

fn build_system_prompt(mode_prompt: &str) -> String {
    let base = "Du bearbeitest diktierten Text nach einer konfigurierten Rolle. Gib ausschliesslich den finalen Text ohne Erklaerungen oder Meta-Kommentare zurueck.";
    let trimmed = mode_prompt.trim();
    if trimmed.is_empty() {
        base.to_owned()
    } else {
        format!("{base}\n\nRollen-Kontext:\n{trimmed}")
    }
}

fn request_ollama(
    client: &Client,
    endpoint: &str,
    model_name: &str,
    system_prompt: &str,
    raw_transcript: &str,
) -> Result<String, String> {
    let url = join_base_url(endpoint, "/api/chat");
    let response = client
        .post(&url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .json(&json!({
            "model": model_name,
            "stream": false,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt,
                },
                {
                    "role": "user",
                    "content": raw_transcript,
                }
            ]
        }))
        .send()
        .map_err(|err| format!("Ollama-Nachverarbeitung konnte nicht gestartet werden: {err}"))?;

    let status = response.status();
    let value: Value = response
        .json()
        .map_err(|err| format!("Ollama-Antwort konnte nicht gelesen werden: {err}"))?;
    if !status.is_success() {
        return Err(format!(
            "Ollama lieferte HTTP {} bei der Nachverarbeitung.",
            status
        ));
    }

    value
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            value
                .get("response")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .ok_or_else(|| "Ollama-Antwort enthielt keinen verarbeiteten Text.".to_owned())
}

fn request_lm_studio(
    client: &Client,
    endpoint: &str,
    model_name: &str,
    system_prompt: &str,
    raw_transcript: &str,
) -> Result<String, String> {
    let url = join_base_url(endpoint, "/v1/chat/completions");
    let response = client
        .post(&url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .json(&json!({
            "model": model_name,
            "temperature": 0.1,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt,
                },
                {
                    "role": "user",
                    "content": raw_transcript,
                }
            ]
        }))
        .send()
        .map_err(|err| {
            format!("LM-Studio-Nachverarbeitung konnte nicht gestartet werden: {err}")
        })?;

    let status = response.status();
    let value: Value = response
        .json()
        .map_err(|err| format!("LM-Studio-Antwort konnte nicht gelesen werden: {err}"))?;
    if !status.is_success() {
        return Err(format!(
            "LM Studio lieferte HTTP {} bei der Nachverarbeitung.",
            status
        ));
    }

    value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "LM-Studio-Antwort enthielt keinen verarbeiteten Text.".to_owned())
}

fn join_base_url(endpoint: &str, suffix: &str) -> String {
    let base = endpoint.trim().trim_end_matches('/');
    if suffix.starts_with("/v1/") && base.ends_with("/v1") {
        return format!("{base}{}", &suffix[3..]);
    }
    if suffix.starts_with("/api/") && base.ends_with("/api") {
        return format!("{base}{}", &suffix[4..]);
    }
    format!("{base}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use open_whisper_core::{AppSettings, PostProcessingBackend, ProcessingMode};

    #[test]
    fn empty_prompt_gets_safe_default_instruction() {
        let prompt = build_system_prompt("");
        assert!(prompt.contains("Gib ausschliesslich den finalen Text"));
    }

    #[test]
    fn disabled_mode_returns_original_text() {
        let settings = AppSettings::default();
        let result = process_text(&settings, "roher text").unwrap();
        assert_eq!(result, "roher text");
    }

    #[test]
    fn join_base_url_trims_trailing_slash() {
        assert_eq!(
            join_base_url("http://127.0.0.1:11434/", "/api/chat"),
            "http://127.0.0.1:11434/api/chat"
        );
    }

    #[test]
    fn join_base_url_avoids_duplicate_version_prefix() {
        assert_eq!(
            join_base_url("http://127.0.0.1:1234/v1", "/v1/chat/completions"),
            "http://127.0.0.1:1234/v1/chat/completions"
        );
    }

    #[test]
    fn active_backend_reflects_global_setting() {
        let mut settings = AppSettings::default();
        settings.active_post_processing_backend = PostProcessingBackend::Ollama;
        settings.post_processing_enabled = true;
        settings.modes.push(ProcessingMode {
            id: "dev".to_owned(),
            name: "Entwickler".to_owned(),
            prompt: "Nutze Entwickler-Sprache.".to_owned(),
        });
        settings.active_mode_id = "dev".to_owned();

        assert!(settings.active_mode_post_processing_enabled());
        assert_eq!(
            settings.active_post_processing_backend,
            PostProcessingBackend::Ollama
        );
    }
}
