use std::{
    num::NonZeroU32,
    path::PathBuf,
    sync::Arc,
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
use open_whisper_core::{AppSettings, LlmPreset};

use crate::llm_model_manager::resolve_llm_model_path;

const MAX_OUTPUT_TOKENS: i32 = 512;
const STOP_SEQUENCE: &str = "<|im_end|>";
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
        settings: &AppSettings,
        system_prompt: &str,
        user_text: &str,
    ) -> Result<String, String> {
        let target_preset = settings.local_llm;
        let target_path = resolve_llm_model_path(settings)?;

        if !target_path.exists() {
            return Err(format!(
                "Lokales Sprachmodell ({}) ist noch nicht heruntergeladen.",
                target_preset.display_label()
            ));
        }

        self.ensure_loaded(target_preset, &target_path)?;
        self.last_used = Instant::now();

        let loaded = self
            .loaded
            .as_ref()
            .expect("ensure_loaded guarantees loaded model");

        let backend = backend()?;
        let n_ctx_value = target_preset.context_size();
        let n_ctx = NonZeroU32::new(n_ctx_value)
            .ok_or_else(|| "context_size must be greater than zero".to_owned())?;
        let ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));

        let mut ctx = loaded
            .model
            .new_context(&backend, ctx_params)
            .map_err(|err| format!("LLM-Kontext konnte nicht erstellt werden: {err}"))?;

        let prompt = build_qwen_chat_prompt(system_prompt, user_text);
        let tokens = loaded
            .model
            .str_to_token(&prompt, AddBos::Never)
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

fn build_qwen_chat_prompt(system_prompt: &str, user_text: &str) -> String {
    format!(
        "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n",
        system = system_prompt.trim(),
        user = user_text.trim(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qwen_prompt_contains_role_markers() {
        let prompt = build_qwen_chat_prompt("Du bist hilfreich.", "Hallo Welt");
        assert!(prompt.starts_with("<|im_start|>system\n"));
        assert!(prompt.contains("<|im_start|>user\nHallo Welt<|im_end|>"));
        assert!(prompt.ends_with("<|im_start|>assistant\n"));
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
