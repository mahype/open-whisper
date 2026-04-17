use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
    time::{Duration, Instant},
};

use crate::model_manager::{default_model_path, resolve_model_path};
use cpal::{
    Device, SampleFormat, Stream, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use open_whisper_core::{AppSettings, ProviderKind, TriggerMode};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub enum DictationOutcome {
    Status(String),
    TranscriptReady(String),
}

pub struct DictationController {
    available_input_devices: Vec<String>,
    recording: Option<ActiveRecording>,
    transcription_rx: Option<Receiver<Result<String, String>>>,
    model_cache: Option<ModelCache>,
}

impl DictationController {
    pub fn new() -> Self {
        Self {
            available_input_devices: Vec::new(),
            recording: None,
            transcription_rx: None,
            model_cache: None,
        }
    }

    pub fn refresh_input_devices(&mut self, settings: &mut AppSettings) -> Vec<String> {
        let mut messages = Vec::new();

        match discover_input_devices() {
            Ok(devices) => {
                self.available_input_devices = devices;
                if self.available_input_devices.is_empty() {
                    messages.push("Kein Eingabegeraet gefunden.".to_owned());
                    return messages;
                }

                let should_reset = settings.input_device_name.trim().is_empty()
                    || (settings.input_device_name != system_default_label()
                        && !self
                            .available_input_devices
                            .iter()
                            .any(|device| device == &settings.input_device_name));

                if should_reset {
                    settings.input_device_name = default_input_device_name()
                        .or_else(|| self.available_input_devices.first().cloned())
                        .unwrap_or_else(|| system_default_label().to_owned());
                    messages.push(format!(
                        "Eingabegeraet auf '{}' gesetzt.",
                        settings.input_device_name
                    ));
                }
            }
            Err(err) => messages.push(format!(
                "Eingabegeraete konnten nicht geladen werden: {err}"
            )),
        }

        messages
    }

    pub fn available_input_devices(&self) -> &[String] {
        &self.available_input_devices
    }

    pub fn suggested_model_path(&self, settings: &AppSettings) -> Result<PathBuf, String> {
        default_model_path(settings.local_model)
    }

    pub fn summary(&self) -> String {
        let recording = if self.recording.is_some() {
            "Aufnahme aktiv"
        } else {
            "Aufnahme inaktiv"
        };
        let transcription = if self.transcription_rx.is_some() {
            "Transkription laeuft"
        } else {
            "keine laufende Transkription"
        };
        format!("{recording}, {transcription}")
    }

    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    pub fn is_transcribing(&self) -> bool {
        self.transcription_rx.is_some()
    }

    pub fn invalidate_model_cache(&mut self) {
        self.model_cache = None;
    }

    pub fn handle_hotkey(
        &mut self,
        settings: &AppSettings,
        pressed: bool,
    ) -> Vec<DictationOutcome> {
        match settings.trigger_mode {
            TriggerMode::PushToTalk => {
                if pressed {
                    match self.start_recording(settings) {
                        Ok(message) => vec![DictationOutcome::Status(message)],
                        Err(err) => vec![DictationOutcome::Status(err)],
                    }
                } else {
                    match self.stop_recording_and_transcribe(settings, "Taste losgelassen") {
                        Ok(outcomes) => outcomes,
                        Err(err) => vec![DictationOutcome::Status(err)],
                    }
                }
            }
            TriggerMode::Toggle => {
                if !pressed {
                    return Vec::new();
                }

                if self.is_recording() {
                    match self.stop_recording_and_transcribe(settings, "Toggle gestoppt") {
                        Ok(outcomes) => outcomes,
                        Err(err) => vec![DictationOutcome::Status(err)],
                    }
                } else {
                    match self.start_recording(settings) {
                        Ok(message) => vec![DictationOutcome::Status(message)],
                        Err(err) => vec![DictationOutcome::Status(err)],
                    }
                }
            }
        }
    }

    pub fn start_recording(&mut self, settings: &AppSettings) -> Result<String, String> {
        if self.recording.is_some() {
            return Ok("Aufnahme laeuft bereits.".to_owned());
        }

        let recording = ActiveRecording::start(settings)?;
        self.recording = Some(recording);

        Ok(format!(
            "Aufnahme gestartet ueber '{}'{}.",
            settings.input_device_name,
            if settings.vad_enabled {
                ", Silence-Stop aktiv"
            } else {
                ""
            }
        ))
    }

    pub fn stop_recording_and_transcribe(
        &mut self,
        settings: &AppSettings,
        reason: &str,
    ) -> Result<Vec<DictationOutcome>, String> {
        let Some(recording) = self.recording.take() else {
            return Ok(Vec::new());
        };

        let audio = recording.finish()?;
        if audio.samples.is_empty() || audio.duration < Duration::from_millis(200) {
            return Ok(vec![DictationOutcome::Status(
                "Aufnahme war zu kurz oder leer.".to_owned(),
            )]);
        }

        if settings.active_provider != ProviderKind::LocalWhisper {
            return Ok(vec![DictationOutcome::Status(format!(
                "Provider '{}' ist fuer Live-Diktat noch nicht umgesetzt. Nutze vorerst 'Local Whisper'.",
                settings.active_provider.label()
            ))]);
        }

        let context = self.ensure_whisper_context(settings)?;
        let language = normalized_language(&settings.transcription_language);
        let app_settings = settings.clone();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result =
                transcribe_with_whisper(context, &app_settings, audio, language.as_deref());
            let _ = tx.send(result);
        });

        self.transcription_rx = Some(rx);

        Ok(vec![DictationOutcome::Status(format!(
            "Aufnahme beendet ({reason}). Lokale Transkription laeuft."
        ))])
    }

    pub fn poll(&mut self, settings: &AppSettings) -> Vec<DictationOutcome> {
        let mut outcomes = Vec::new();

        let pending_recording_event = self
            .recording
            .as_mut()
            .and_then(ActiveRecording::poll_event);
        match pending_recording_event {
            Some(RecordingEvent::SilenceDetected) => {
                match self.stop_recording_and_transcribe(settings, "Stille erkannt") {
                    Ok(new_outcomes) => outcomes.extend(new_outcomes),
                    Err(err) => outcomes.push(DictationOutcome::Status(err)),
                }
            }
            Some(RecordingEvent::StreamError(err)) => {
                self.recording.take();
                outcomes.push(DictationOutcome::Status(err));
            }
            None => {}
        }

        if let Some(rx) = &self.transcription_rx {
            match rx.try_recv() {
                Ok(Ok(transcript)) => {
                    self.transcription_rx = None;
                    outcomes.push(DictationOutcome::Status(
                        "Lokale Transkription abgeschlossen.".to_owned(),
                    ));
                    outcomes.push(DictationOutcome::TranscriptReady(transcript));
                }
                Ok(Err(err)) => {
                    self.transcription_rx = None;
                    outcomes.push(DictationOutcome::Status(err));
                }
                Err(TryRecvError::Disconnected) => {
                    self.transcription_rx = None;
                    outcomes.push(DictationOutcome::Status(
                        "Transkriptions-Worker wurde unerwartet beendet.".to_owned(),
                    ));
                }
                Err(TryRecvError::Empty) => {}
            }
        }

        outcomes
    }

    fn ensure_whisper_context(
        &mut self,
        settings: &AppSettings,
    ) -> Result<Arc<WhisperContext>, String> {
        let model_path = resolve_model_path(settings)?;

        if let Some(cache) = &self.model_cache
            && cache.path == model_path
        {
            return Ok(cache.context.clone());
        }

        if !model_path.exists() {
            return Err(format!(
                "Whisper-Modell fehlt: {}. Lade dort '{}' ab oder setze in den Einstellungen einen anderen Pfad.",
                model_path.display(),
                settings.local_model.default_filename()
            ));
        }

        let model_path_string = model_path.to_string_lossy().to_string();
        let context = WhisperContext::new_with_params(
            &model_path_string,
            WhisperContextParameters::default(),
        )
        .map_err(|err| format!("Whisper-Modell konnte nicht geladen werden: {err}"))?;

        let context = Arc::new(context);
        self.model_cache = Some(ModelCache {
            path: model_path,
            context: context.clone(),
        });

        Ok(context)
    }
}

struct ModelCache {
    path: PathBuf,
    context: Arc<WhisperContext>,
}

struct ActiveRecording {
    _stream: Stream,
    event_rx: Receiver<RecordingEvent>,
    shared: Arc<Mutex<RecordingBuffer>>,
    started_at: Instant,
}

impl ActiveRecording {
    fn start(settings: &AppSettings) -> Result<Self, String> {
        let device = select_input_device(&settings.input_device_name)?;
        let config = device
            .default_input_config()
            .map_err(|err| format!("Input-Konfiguration konnte nicht geladen werden: {err}"))?;

        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let (event_tx, event_rx) = mpsc::channel();
        let shared = Arc::new(Mutex::new(RecordingBuffer::new(
            sample_rate,
            settings.vad_enabled,
            settings.vad_threshold,
            settings.vad_silence_ms,
        )));

        let stream = build_input_stream(&device, &config, channels, shared.clone(), event_tx)?;
        stream
            .play()
            .map_err(|err| format!("Audioaufnahme konnte nicht gestartet werden: {err}"))?;

        Ok(Self {
            _stream: stream,
            event_rx,
            shared,
            started_at: Instant::now(),
        })
    }

    fn poll_event(&mut self) -> Option<RecordingEvent> {
        self.event_rx.try_recv().ok()
    }

    fn finish(self) -> Result<RecordedAudio, String> {
        let duration = self.started_at.elapsed();
        let mut guard = self
            .shared
            .lock()
            .map_err(|_| "Aufnahmebuffer konnte nicht gelesen werden.".to_owned())?;
        Ok(guard.finish(duration))
    }
}

enum RecordingEvent {
    SilenceDetected,
    StreamError(String),
}

struct RecordingBuffer {
    samples: Vec<f32>,
    sample_rate: u32,
    vad_enabled: bool,
    vad_threshold: f32,
    silence_limit_samples: usize,
    silence_run_samples: usize,
    voice_detected: bool,
    last_voice_sample_index: usize,
    silence_notification_sent: bool,
}

impl RecordingBuffer {
    fn new(sample_rate: u32, vad_enabled: bool, vad_threshold: f32, vad_silence_ms: u32) -> Self {
        let silence_limit_samples =
            ((sample_rate as u64 * vad_silence_ms as u64) / 1000).max(1) as usize;

        Self {
            samples: Vec::new(),
            sample_rate,
            vad_enabled,
            vad_threshold,
            silence_limit_samples,
            silence_run_samples: 0,
            voice_detected: false,
            last_voice_sample_index: 0,
            silence_notification_sent: false,
        }
    }

    fn push_chunk(&mut self, chunk: &[f32], event_tx: &Sender<RecordingEvent>) {
        if chunk.is_empty() {
            return;
        }

        self.samples.extend_from_slice(chunk);

        let rms = root_mean_square(chunk);
        if rms >= self.vad_threshold {
            self.voice_detected = true;
            self.last_voice_sample_index = self.samples.len();
            self.silence_run_samples = 0;
            self.silence_notification_sent = false;
            return;
        }

        if self.vad_enabled && self.voice_detected {
            self.silence_run_samples += chunk.len();
            if self.silence_run_samples >= self.silence_limit_samples
                && !self.silence_notification_sent
            {
                self.silence_notification_sent = true;
                let _ = event_tx.send(RecordingEvent::SilenceDetected);
            }
        }
    }

    fn finish(&mut self, duration: Duration) -> RecordedAudio {
        let trim_index = if self.voice_detected && self.last_voice_sample_index > 0 {
            self.last_voice_sample_index
        } else {
            self.samples.len()
        };

        RecordedAudio {
            samples: self.samples[..trim_index].to_vec(),
            sample_rate: self.sample_rate,
            duration,
        }
    }
}

struct RecordedAudio {
    samples: Vec<f32>,
    sample_rate: u32,
    duration: Duration,
}

fn build_input_stream(
    device: &Device,
    config: &SupportedStreamConfig,
    channels: usize,
    shared: Arc<Mutex<RecordingBuffer>>,
    event_tx: Sender<RecordingEvent>,
) -> Result<Stream, String> {
    let stream_config = config.config();
    let error_sender = event_tx.clone();
    let error_callback = move |err| {
        let _ = error_sender.send(RecordingEvent::StreamError(format!(
            "Audiofehler im Stream: {err}"
        )));
    };

    match config.sample_format() {
        SampleFormat::F32 => device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _| handle_input_data_f32(data, channels, &shared, &event_tx),
                error_callback,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::F64 => device
            .build_input_stream(
                &stream_config,
                move |data: &[f64], _| {
                    handle_input_data_iter(
                        data.iter().copied().map(|sample| sample as f32),
                        channels,
                        &shared,
                        &event_tx,
                    )
                },
                error_callback,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::I8 => device
            .build_input_stream(
                &stream_config,
                move |data: &[i8], _| {
                    handle_input_data_iter(
                        data.iter()
                            .copied()
                            .map(|sample| sample as f32 / i8::MAX as f32),
                        channels,
                        &shared,
                        &event_tx,
                    )
                },
                error_callback,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::I16 => device
            .build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    handle_input_data_iter(
                        data.iter()
                            .copied()
                            .map(|sample| sample as f32 / i16::MAX as f32),
                        channels,
                        &shared,
                        &event_tx,
                    )
                },
                error_callback,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::I32 => device
            .build_input_stream(
                &stream_config,
                move |data: &[i32], _| {
                    handle_input_data_iter(
                        data.iter()
                            .copied()
                            .map(|sample| sample as f32 / i32::MAX as f32),
                        channels,
                        &shared,
                        &event_tx,
                    )
                },
                error_callback,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::U8 => device
            .build_input_stream(
                &stream_config,
                move |data: &[u8], _| {
                    handle_input_data_iter(
                        data.iter()
                            .copied()
                            .map(|sample| (sample as f32 / u8::MAX as f32) * 2.0 - 1.0),
                        channels,
                        &shared,
                        &event_tx,
                    )
                },
                error_callback,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::U16 => device
            .build_input_stream(
                &stream_config,
                move |data: &[u16], _| {
                    handle_input_data_iter(
                        data.iter()
                            .copied()
                            .map(|sample| (sample as f32 / u16::MAX as f32) * 2.0 - 1.0),
                        channels,
                        &shared,
                        &event_tx,
                    )
                },
                error_callback,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::U32 => device
            .build_input_stream(
                &stream_config,
                move |data: &[u32], _| {
                    handle_input_data_iter(
                        data.iter()
                            .copied()
                            .map(|sample| (sample as f32 / u32::MAX as f32) * 2.0 - 1.0),
                        channels,
                        &shared,
                        &event_tx,
                    )
                },
                error_callback,
                None,
            )
            .map_err(|err| err.to_string()),
        other => Err(format!(
            "Sampleformat '{other}' wird aktuell nicht unterstuetzt."
        )),
    }
}

fn handle_input_data_f32(
    data: &[f32],
    channels: usize,
    shared: &Arc<Mutex<RecordingBuffer>>,
    event_tx: &Sender<RecordingEvent>,
) {
    let mono_chunk = interleaved_to_mono(data, channels);
    if let Ok(mut guard) = shared.lock() {
        guard.push_chunk(&mono_chunk, event_tx);
    }
}

fn handle_input_data_iter(
    data: impl Iterator<Item = f32>,
    channels: usize,
    shared: &Arc<Mutex<RecordingBuffer>>,
    event_tx: &Sender<RecordingEvent>,
) {
    let collected: Vec<f32> = data.collect();
    handle_input_data_f32(&collected, channels, shared, event_tx);
}

fn interleaved_to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }

    let mut mono = Vec::with_capacity(data.len() / channels);
    for frame in data.chunks(channels) {
        let sum: f32 = frame.iter().copied().sum();
        mono.push(sum / channels as f32);
    }
    mono
}

fn root_mean_square(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let power = samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32;
    power.sqrt()
}

fn transcribe_with_whisper(
    context: Arc<WhisperContext>,
    settings: &AppSettings,
    audio: RecordedAudio,
    language: Option<&str>,
) -> Result<String, String> {
    let mono_16khz = resample_to_16khz(&audio.samples, audio.sample_rate);
    if mono_16khz.is_empty() {
        return Err("Keine Audiodaten fuer Whisper vorhanden.".to_owned());
    }

    let mut state = context
        .create_state()
        .map_err(|err| format!("Whisper-State konnte nicht erzeugt werden: {err}"))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
    params.set_n_threads(thread_count());
    params.set_translate(false);
    params.set_language(language);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_no_timestamps(true);
    params.set_single_segment(false);

    state
        .full(params, &mono_16khz)
        .map_err(|err| format!("Whisper-Transkription fehlgeschlagen: {err}"))?;

    let transcript = state
        .as_iter()
        .map(|segment| segment.to_string().trim().to_owned())
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if transcript.is_empty() {
        return Err(format!(
            "Whisper hat keinen Text erkannt. Modell: {}, Sprache: {}.",
            settings.local_model.default_filename(),
            language.unwrap_or("auto")
        ));
    }

    Ok(transcript)
}

fn resample_to_16khz(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    if sample_rate == 16_000 {
        return samples.to_vec();
    }

    let ratio = 16_000.0 / sample_rate as f64;
    let target_len = (samples.len() as f64 * ratio).round() as usize;
    let mut output = Vec::with_capacity(target_len);

    for index in 0..target_len {
        let source_position = index as f64 / ratio;
        let source_index = source_position.floor() as usize;
        let frac = (source_position - source_index as f64) as f32;
        let current = *samples.get(source_index).unwrap_or(&0.0);
        let next = *samples.get(source_index + 1).unwrap_or(&current);
        output.push(current + (next - current) * frac);
    }

    output
}

fn thread_count() -> i32 {
    std::thread::available_parallelism()
        .map(|value| value.get().min(6) as i32)
        .unwrap_or(4)
}

fn normalized_language(language: &str) -> Option<String> {
    let trimmed = language.trim().to_lowercase();
    if trimmed.is_empty() || trimmed == "auto" {
        None
    } else {
        Some(trimmed)
    }
}

fn discover_input_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let mut devices = host
        .input_devices()
        .map_err(|err| err.to_string())?
        .filter_map(|device| {
            device
                .description()
                .ok()
                .map(|description| description.name().to_owned())
        })
        .collect::<Vec<_>>();

    devices.sort();
    devices.dedup();

    if let Some(default_name) = default_input_device_name() {
        devices.retain(|name| name != &default_name);
        devices.insert(0, default_name);
    }

    Ok(devices)
}

fn select_input_device(selected_name: &str) -> Result<Device, String> {
    let host = cpal::default_host();

    if selected_name == system_default_label() {
        return host
            .default_input_device()
            .ok_or_else(|| "Kein Standard-Eingabegeraet verfuegbar.".to_owned());
    }

    if let Some(default_device) = host.default_input_device()
        && default_device
            .description()
            .map(|description| description.name() == selected_name)
            .unwrap_or(false)
    {
        return Ok(default_device);
    }

    let mut matching = host
        .input_devices()
        .map_err(|err| err.to_string())?
        .find(|device| {
            device
                .description()
                .map(|description| description.name() == selected_name)
                .unwrap_or(false)
        });

    matching
        .take()
        .ok_or_else(|| format!("Eingabegeraet '{}' wurde nicht gefunden.", selected_name))
}

fn default_input_device_name() -> Option<String> {
    cpal::default_host()
        .default_input_device()
        .and_then(|device| {
            device
                .description()
                .ok()
                .map(|description| description.name().to_owned())
        })
}

fn system_default_label() -> &'static str {
    "System Default"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_conversion_averages_channels() {
        let stereo = [1.0, -1.0, 0.5, 0.5];
        let mono = interleaved_to_mono(&stereo, 2);
        assert_eq!(mono, vec![0.0, 0.5]);
    }

    #[test]
    fn resample_identity_keeps_length() {
        let audio = vec![0.0, 0.5, -0.5, 1.0];
        let resampled = resample_to_16khz(&audio, 16_000);
        assert_eq!(audio, resampled);
    }

    #[test]
    fn auto_language_maps_to_none() {
        assert_eq!(normalized_language("auto"), None);
        assert_eq!(normalized_language("de"), Some("de".to_owned()));
    }
}
