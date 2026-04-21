// Phase 1 only wires a handful of these into the UI. The rest are part of
// the stable surface the later phases (settings, HUD, onboarding) will call.
#![allow(dead_code)]

//! Thin adapter over `open_whisper_bridge::bridge_api`.
//!
//! The bridge's `BridgeRuntime` is stored in a `thread_local!`, so every
//! call must originate from the same thread that first constructed it —
//! in this crate that is the GTK main thread. We re-export only the
//! functions the UI actually needs, so callers don't depend on the
//! bridge crate directly.

use open_whisper_bridge::bridge_api;
use open_whisper_core::{
    AppSettings, DiagnosticsDto, LlmPreset, ModelPreset, ModelStatusDto, RecordingLevelsDto,
    RuntimeStatusDto,
};

pub fn load_settings() -> AppSettings {
    bridge_api::load_settings()
}

pub fn save_settings(settings: AppSettings) -> Result<String, String> {
    bridge_api::save_settings(settings)
}

pub fn runtime_status() -> RuntimeStatusDto {
    bridge_api::runtime_status()
}

pub fn model_status() -> ModelStatusDto {
    bridge_api::model_status()
}

pub fn diagnostics() -> DiagnosticsDto {
    bridge_api::run_permission_diagnostics()
}

pub fn recording_levels() -> RecordingLevelsDto {
    bridge_api::recording_levels()
}

pub fn start_dictation() -> Result<String, String> {
    bridge_api::start_dictation()
}

pub fn stop_dictation() -> Result<String, String> {
    bridge_api::stop_dictation()
}

pub fn cancel_dictation() -> Result<String, String> {
    bridge_api::cancel_dictation()
}

pub fn start_model_download(preset: Option<ModelPreset>) -> Result<String, String> {
    bridge_api::start_model_download(preset)
}

pub fn delete_model(preset: Option<ModelPreset>) -> Result<String, String> {
    bridge_api::delete_model(preset)
}

pub fn start_llm_download(preset: LlmPreset) -> Result<String, String> {
    bridge_api::start_llm_download(preset)
}

pub fn validate_hotkey(text: &str) -> Result<String, String> {
    bridge_api::validate_hotkey(text)
}
