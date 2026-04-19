use std::{
    num::NonZeroU32,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaModel, params::LlamaModelParams},
    sampling::LlamaSampler,
};
use once_cell::sync::OnceCell;
use open_whisper_core::LlmPreset;

use crate::llm_model_manager::default_llm_model_path;

const MAX_OUTPUT_TOKENS: i32 = 512;
const STOP_SEQUENCE: &str = "<turn|>";
const PROMPT_BATCH_CAPACITY: usize = 512;

static LLAMA_BACKEND: OnceCell<Arc<LlamaBackend>> = OnceCell::new();

fn backend() -> Result<Arc<LlamaBackend>, String> {
    LLAMA_BACKEND
        .get_or_try_init(|| {
            LlamaBackend::init().map(Arc::new).map_err(|err| {
                format!("llama.cpp-Backend konnte nicht initialisiert werden: {err}")
            })
        })
        .cloned()
}

pub struct LocalLlmRuntime {
    loaded: Option<LoadedModel>,
    last_used: Instant,
}

struct LoadedModel {
    preset: LlmPreset,
    path: PathBuf,
    model: LlamaModel,
}

impl LocalLlmRuntime {
    pub fn new() -> Self {
        Self {
            loaded: None,
            last_used: Instant::now(),
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded.is_some()
    }

    pub fn loaded_preset(&self) -> Option<LlmPreset> {
        self.loaded.as_ref().map(|loaded| loaded.preset)
    }

    pub fn maybe_unload(&mut self, auto_unload_secs: u32) {
        if auto_unload_secs == 0 {
            return;
        }
        if self.loaded.is_none() {
            return;
        }
        if self.last_used.elapsed() >= Duration::from_secs(auto_unload_secs as u64) {
            self.loaded = None;
        }
    }

    pub fn unload(&mut self) {
        self.loaded = None;
    }

    pub fn generate(
        &mut self,
        preset: LlmPreset,
        system_prompt: &str,
        user_text: &str,
    ) -> Result<String, String> {
        let target_path = default_llm_model_path(preset)?;

        if !target_path.exists() {
            return Err(format!(
                "Lokales Sprachmodell ({}) ist noch nicht heruntergeladen.",
                preset.display_label()
            ));
        }

        self.ensure_loaded(preset, &target_path)?;
        self.last_used = Instant::now();

        let loaded = self
            .loaded
            .as_ref()
            .expect("ensure_loaded guarantees loaded model");

        let backend = backend()?;
        let n_ctx_value = preset.context_size();
        let n_ctx = NonZeroU32::new(n_ctx_value)
            .ok_or_else(|| "context_size must be greater than zero".to_owned())?;
        let ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));

        let mut ctx = loaded
            .model
            .new_context(&backend, ctx_params)
            .map_err(|err| format!("LLM-Kontext konnte nicht erstellt werden: {err}"))?;

        let prompt = build_gemma_chat_prompt(system_prompt, user_text);
        let tokens = loaded
            .model
            .str_to_token(&prompt, AddBos::Always)
            .map_err(|err| format!("LLM-Tokenisierung fehlgeschlagen: {err}"))?;

        if tokens.is_empty() {
            return Err("LLM-Prompt ergab keine Tokens.".to_owned());
        }

        let n_input = tokens.len() as i32;
        if n_input + MAX_OUTPUT_TOKENS >= n_ctx_value as i32 {
            return Err(format!(
                "Eingabe ist zu lang fuer das Sprachmodell-Kontextfenster ({} Tokens, max {}).",
                n_input,
                n_ctx_value as i32 - MAX_OUTPUT_TOKENS
            ));
        }

        let mut batch = LlamaBatch::new(PROMPT_BATCH_CAPACITY.max(tokens.len()), 1);
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add(*token, i as i32, &[0], is_last)
                .map_err(|err| format!("LLM-Batch konnte nicht gefuellt werden: {err}"))?;
        }

        ctx.decode(&mut batch)
            .map_err(|err| format!("LLM-Decode des Prompts fehlgeschlagen: {err}"))?;

        let mut sampler = LlamaSampler::chain_simple([LlamaSampler::greedy()]);

        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut output = String::new();
        let mut n_cur = n_input;
        let n_max = n_input + MAX_OUTPUT_TOKENS;

        while n_cur < n_max {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if loaded.model.is_eog_token(token) {
                break;
            }

            let piece = loaded
                .model
                .token_to_piece(token, &mut decoder, false, None)
                .map_err(|err| format!("LLM-Detokenisierung fehlgeschlagen: {err}"))?;

            output.push_str(&piece);

            if let Some(idx) = output.find(STOP_SEQUENCE) {
                output.truncate(idx);
                break;
            }

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|err| format!("LLM-Batch-Update fehlgeschlagen: {err}"))?;
            n_cur += 1;

            ctx.decode(&mut batch)
                .map_err(|err| format!("LLM-Decode fehlgeschlagen: {err}"))?;
        }

        let trimmed = output.trim().to_owned();
        if trimmed.is_empty() {
            return Err("Das Sprachmodell lieferte keinen Text zurueck.".to_owned());
        }

        Ok(trimmed)
    }

    fn ensure_loaded(
        &mut self,
        target_preset: LlmPreset,
        target_path: &PathBuf,
    ) -> Result<(), String> {
        let needs_load = match &self.loaded {
            Some(loaded) => loaded.preset != target_preset || loaded.path != *target_path,
            None => true,
        };

        if !needs_load {
            return Ok(());
        }

        self.loaded = None;

        let backend = backend()?;
        let params = LlamaModelParams::default().with_n_gpu_layers(1_000);
        let model = LlamaModel::load_from_file(&backend, target_path, &params)
            .map_err(|err| format!("Sprachmodell konnte nicht geladen werden: {err}"))?;

        self.loaded = Some(LoadedModel {
            preset: target_preset,
            path: target_path.clone(),
            model,
        });

        Ok(())
    }
}

impl Default for LocalLlmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

static SHARED_RUNTIME: OnceLock<Mutex<LocalLlmRuntime>> = OnceLock::new();

pub fn shared_runtime() -> &'static Mutex<LocalLlmRuntime> {
    SHARED_RUNTIME.get_or_init(|| Mutex::new(LocalLlmRuntime::new()))
}

pub fn generate_with_shared_runtime(
    preset: LlmPreset,
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    let mut runtime = shared_runtime()
        .lock()
        .map_err(|_| "Lokales Sprachmodell-Runtime-Mutex wurde vergiftet.".to_owned())?;
    runtime.generate(preset, system_prompt, user_text)
}

pub fn maybe_unload_shared_runtime(auto_unload_secs: u32) {
    if let Some(mutex) = SHARED_RUNTIME.get()
        && let Ok(mut runtime) = mutex.lock()
    {
        runtime.maybe_unload(auto_unload_secs);
    }
}

fn build_gemma_chat_prompt(mode_instruction: &str, transcript: &str) -> String {
    let instruction = mode_instruction.trim();
    let text = transcript.trim();

    let body = if instruction.is_empty() {
        format!(
            "Du bereinigst einen diktierten Text. Korrigiere Satzzeichen, Grossschreibung und offensichtliche Erkennungsfehler, ohne den Inhalt zu veraendern.\n\nText zum Bereinigen:\n{text}\n\nGib ausschliesslich den bereinigten Text zurueck, ohne Erklaerungen, Kommentare oder Anfuehrungszeichen."
        )
    } else {
        format!(
            "Du ueberarbeitest einen diktierten Text nach folgender Anweisung. Wende die Anweisung auf den Text an, ohne die Anweisung selbst zurueckzugeben.\n\nAnweisung:\n{instruction}\n\nText zum Ueberarbeiten:\n{text}\n\nGib ausschliesslich den ueberarbeiteten Text zurueck, ohne Erklaerungen, Kommentare oder Anfuehrungszeichen."
        )
    };

    format!("<bos><|turn>user\n{body}<turn|>\n<|turn>model\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gemma_prompt_labels_instruction_and_text() {
        let prompt = build_gemma_chat_prompt("Schreibe foermlicher.", "hallo welt");
        assert!(prompt.starts_with("<bos><|turn>user\n"));
        assert!(prompt.contains("Anweisung:\nSchreibe foermlicher."));
        assert!(prompt.contains("Text zum Ueberarbeiten:\nhallo welt"));
        assert!(prompt.contains("ohne Erklaerungen"));
        assert!(prompt.ends_with("<|turn>model\n"));
    }

    #[test]
    fn gemma_prompt_falls_back_to_cleanup_when_instruction_empty() {
        let prompt = build_gemma_chat_prompt("   ", "hallo welt");
        assert!(prompt.contains("bereinigst"));
        assert!(prompt.contains("Text zum Bereinigen:\nhallo welt"));
    }

    #[test]
    fn runtime_starts_unloaded() {
        let runtime = LocalLlmRuntime::new();
        assert!(!runtime.is_loaded());
        assert!(runtime.loaded_preset().is_none());
    }

    #[test]
    fn maybe_unload_noop_on_zero_secs() {
        let mut runtime = LocalLlmRuntime::new();
        runtime.last_used = Instant::now() - Duration::from_secs(3_600);
        runtime.maybe_unload(0);
        assert!(!runtime.is_loaded());
    }
}
