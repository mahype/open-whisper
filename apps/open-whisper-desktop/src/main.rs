mod autostart;
mod desktop_integration;
mod dictation;
mod model_manager;
mod settings_store;
mod text_inserter;

use autostart::AutostartManager;
use desktop_integration::{DesktopAction, DesktopIntegration, POLL_INTERVAL};
use dictation::{DictationController, DictationOutcome};
use eframe::egui::{self, RichText};
use model_manager::ModelDownloadManager;
use open_whisper_core::{AppSettings, ModelPreset, ProviderKind, StartupBehavior, TriggerMode};
use text_inserter::insert_text_into_active_app;

fn main() -> eframe::Result<()> {
    let initial_settings = settings_store::load().unwrap_or_else(|err| {
        eprintln!("failed to load settings: {err}");
        AppSettings::default()
    });
    let start_hidden = AutostartManager::start_hidden_requested();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 780.0])
            .with_visible(!start_hidden),
        ..Default::default()
    };

    eframe::run_native(
        "Open Whisper",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(OpenWhisperDesktopApp::new(
                initial_settings,
                start_hidden,
            )))
        }),
    )
}

struct OpenWhisperDesktopApp {
    autostart: AutostartManager,
    settings: AppSettings,
    desktop: DesktopIntegration,
    dictation: DictationController,
    model_downloads: ModelDownloadManager,
    dirty: bool,
    dictation_trigger_count: u64,
    exit_requested: bool,
    last_transcript: String,
    status: String,
    window_visible: bool,
}

impl OpenWhisperDesktopApp {
    fn new(mut settings: AppSettings, start_hidden: bool) -> Self {
        let mut autostart = AutostartManager::new();
        let mut dictation = DictationController::new();
        let mut model_downloads = ModelDownloadManager::new();
        let mut status = "Bereit".to_owned();

        for outcome in dictation.refresh_input_devices(&mut settings) {
            status = outcome;
        }

        if settings.local_model_path.is_empty()
            && let Ok(path) = dictation.suggested_model_path(&settings)
        {
            settings.local_model_path = path.display().to_string();
        }

        model_downloads.refresh_local_state(&settings);

        match autostart.sync_saved_settings(&settings) {
            Ok(Some(message)) => status = message,
            Ok(None) => {}
            Err(err) => status = err,
        }

        Self {
            autostart,
            settings,
            desktop: DesktopIntegration::new(),
            dictation,
            model_downloads,
            dirty: false,
            dictation_trigger_count: 0,
            exit_requested: false,
            last_transcript: String::new(),
            status,
            window_visible: !start_hidden,
        }
    }

    fn save(&mut self) {
        match settings_store::save(&self.settings) {
            Ok(path) => {
                self.dirty = false;
                self.status = format!("Gespeichert unter {}", path.display());
                match self.autostart.sync_saved_settings(&self.settings) {
                    Ok(Some(message)) => self.status = message,
                    Ok(None) => {}
                    Err(err) => self.status = err,
                }
            }
            Err(err) => {
                self.status = format!("Speichern fehlgeschlagen: {err}");
            }
        }
    }

    fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    fn apply_dictation_outcomes(&mut self, outcomes: Vec<DictationOutcome>) {
        for outcome in outcomes {
            match outcome {
                DictationOutcome::Status(message) => self.set_status(message),
                DictationOutcome::TranscriptReady(transcript) => {
                    self.last_transcript = transcript.clone();
                    if self.settings.insert_text_automatically {
                        match insert_text_into_active_app(&transcript, &self.settings) {
                            Ok(message) => self.set_status(message),
                            Err(err) => self.set_status(err),
                        }
                    } else {
                        self.set_status("Transkript bereit.");
                    }
                }
            }
        }
    }

    fn show_window(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        self.window_visible = true;
    }

    fn hide_window(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        self.window_visible = false;
    }

    fn refresh_devices(&mut self) {
        let messages = self.dictation.refresh_input_devices(&mut self.settings);
        self.apply_status_messages(messages);
        self.dirty = true;
    }

    fn apply_status_messages(&mut self, messages: Vec<String>) {
        for message in messages {
            self.set_status(message);
        }
    }

    fn sync_model_path_from_preset(&mut self) {
        if let Ok(path) = self.dictation.suggested_model_path(&self.settings) {
            self.settings.local_model_path = path.display().to_string();
            self.model_downloads.refresh_local_state(&self.settings);
            self.dirty = true;
        }
    }

    fn handle_desktop_action(&mut self, ctx: &egui::Context, action: DesktopAction) {
        match action {
            DesktopAction::ShowSettings => {
                self.show_window(ctx);
                self.set_status("Fenster aus dem Tray geoeffnet.");
            }
            DesktopAction::HideWindow => {
                self.hide_window(ctx);
                self.set_status("Fenster im Tray versteckt.");
            }
            DesktopAction::Quit => {
                self.exit_requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            DesktopAction::HotkeyPressed => {
                self.dictation_trigger_count += 1;
                let outcomes = self.dictation.handle_hotkey(&self.settings, true);
                self.apply_dictation_outcomes(outcomes);
            }
            DesktopAction::HotkeyReleased => {
                let outcomes = self.dictation.handle_hotkey(&self.settings, false);
                self.apply_dictation_outcomes(outcomes);
            }
        }
    }
}

impl eframe::App for OpenWhisperDesktopApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dictation_outcomes = self.dictation.poll(&self.settings);
        self.apply_dictation_outcomes(dictation_outcomes);

        for message in self.model_downloads.poll() {
            self.set_status(message);
        }

        for message in self.desktop.sync(&self.settings, self.window_visible) {
            self.set_status(message);
        }

        for action in self.desktop.poll_actions() {
            self.handle_desktop_action(ctx, action);
        }

        if ctx.input(|input| input.viewport().close_requested()) {
            if self.exit_requested {
                return;
            }

            if self.desktop.can_hide_to_tray() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.hide_window(ctx);
                self.set_status("Fenster geschlossen, App laeuft weiter im Tray.");
            }
        }

        ctx.request_repaint_after(POLL_INTERVAL);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Open Whisper");
                ui.label("Native Rust-Basis fuer Tray, Hotkey und lokale STT-Provider.");

                if ui
                    .add_enabled(self.dirty, egui::Button::new("Speichern"))
                    .clicked()
                {
                    self.save();
                }

                if self.dirty {
                    ui.label(RichText::new("Ungespeicherte Aenderungen").strong());
                }
            });

            ui.add_space(8.0);
            ui.label(&self.status);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Ersteinrichtung");
                ui.label(
                    "Die App kann jetzt Eingabegeraete erkennen, Audio aufnehmen, \
                     bei Stille stoppen und lokal ueber Whisper transkribieren.",
                );
                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.heading("Systemstart");

                    let old = self.settings.startup_behavior;
                    egui::ComboBox::from_label("Startverhalten")
                        .selected_text(self.settings.startup_behavior.label())
                        .show_ui(ui, |ui| {
                            for option in StartupBehavior::ALL {
                                ui.selectable_value(
                                    &mut self.settings.startup_behavior,
                                    option,
                                    option.label(),
                                );
                            }
                        });
                    self.dirty |= old != self.settings.startup_behavior;

                    ui.label(self.autostart.summary());
                    if self.settings.startup_behavior == StartupBehavior::AskOnFirstLaunch {
                        ui.label(
                            "Die eigentliche Erststart-Abfrage folgt noch. Beim Speichern bleibt der aktuelle OS-Status bis dahin unveraendert.",
                        );
                    } else {
                        ui.label(
                            "Die Auswahl wird beim Speichern direkt auf den Systemstart angewendet.",
                        );
                    }
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.heading("Aufnahme");

                    ui.horizontal(|ui| {
                        ui.label("Eingabegeraet");
                        if ui.button("Aktualisieren").clicked() {
                            self.refresh_devices();
                        }
                    });

                    egui::ComboBox::from_id_salt("input-device")
                        .selected_text(self.settings.input_device_name.as_str())
                        .show_ui(ui, |ui| {
                            for device in self.dictation.available_input_devices() {
                                if ui
                                    .selectable_value(
                                        &mut self.settings.input_device_name,
                                        device.clone(),
                                        device,
                                    )
                                    .changed()
                                {
                                    self.dirty = true;
                                }
                            }
                        });

                    ui.label("Globaler Shortcut");
                    self.dirty |= ui.text_edit_singleline(&mut self.settings.hotkey).changed();

                    let old = self.settings.trigger_mode;
                    egui::ComboBox::from_label("Aufnahmemodus")
                        .selected_text(self.settings.trigger_mode.label())
                        .show_ui(ui, |ui| {
                            for option in TriggerMode::ALL {
                                ui.selectable_value(
                                    &mut self.settings.trigger_mode,
                                    option,
                                    option.label(),
                                );
                            }
                        });
                    self.dirty |= old != self.settings.trigger_mode;

                    ui.label("Sprache (`auto`, `de`, `en`, ...)");
                    self.dirty |= ui
                        .text_edit_singleline(&mut self.settings.transcription_language)
                        .changed();

                    self.dirty |= ui
                        .checkbox(
                            &mut self.settings.insert_text_automatically,
                            "Transkript automatisch in die aktive App einfuegen",
                        )
                        .changed();

                    ui.horizontal(|ui| {
                        ui.label("Einfuege-Verzoegerung (ms)");
                        self.dirty |= ui
                            .add(
                                egui::DragValue::new(&mut self.settings.insert_delay_ms)
                                    .speed(10)
                                    .range(0..=2_000),
                            )
                            .changed();
                    });

                    self.dirty |= ui
                        .checkbox(
                            &mut self.settings.restore_clipboard_after_insert,
                            "Vorheriges Clipboard nach dem Einfuegen wiederherstellen",
                        )
                        .changed();

                    self.dirty |= ui
                        .checkbox(
                            &mut self.settings.vad_enabled,
                            "Silence-Stop fuer Toggle-Aufnahmen aktivieren",
                        )
                        .changed();

                    ui.horizontal(|ui| {
                        ui.label("VAD-Schwelle");
                        self.dirty |= ui
                            .add(
                                egui::DragValue::new(&mut self.settings.vad_threshold)
                                    .speed(0.001)
                                    .range(0.001..=0.2),
                            )
                            .changed();
                    });

                    ui.horizontal(|ui| {
                        ui.label("Stille bis Stop (ms)");
                        self.dirty |= ui
                            .add(
                                egui::DragValue::new(&mut self.settings.vad_silence_ms)
                                    .speed(25)
                                    .range(200..=5_000),
                            )
                            .changed();
                    });

                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !self.dictation.is_recording() && !self.dictation.is_transcribing(),
                                egui::Button::new("Aufnahme starten"),
                            )
                            .clicked()
                        {
                            let outcomes = self.dictation.start_recording(&self.settings).map_or_else(
                                |err| vec![DictationOutcome::Status(err)],
                                |message| vec![DictationOutcome::Status(message)],
                            );
                            self.apply_dictation_outcomes(outcomes);
                        }

                        if ui
                            .add_enabled(
                                self.dictation.is_recording(),
                                egui::Button::new("Aufnahme stoppen"),
                            )
                            .clicked()
                        {
                            let outcomes = self
                                .dictation
                                .stop_recording_and_transcribe(&self.settings, "Manuell gestoppt")
                                .unwrap_or_else(|err| vec![DictationOutcome::Status(err)]);
                            self.apply_dictation_outcomes(outcomes);
                        }
                    });
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.heading("Lokales Sprachmodell");

                    let old_provider = self.settings.active_provider;
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.settings.active_provider,
                            ProviderKind::LocalWhisper,
                            ProviderKind::LocalWhisper.label(),
                        );
                        ui.radio_value(
                            &mut self.settings.active_provider,
                            ProviderKind::Ollama,
                            ProviderKind::Ollama.label(),
                        );
                        ui.radio_value(
                            &mut self.settings.active_provider,
                            ProviderKind::LmStudio,
                            ProviderKind::LmStudio.label(),
                        );
                    });
                    self.dirty |= old_provider != self.settings.active_provider;

                    let old_preset = self.settings.local_model;
                    egui::ComboBox::from_label("Standardmodell")
                        .selected_text(self.settings.local_model.label())
                        .show_ui(ui, |ui| {
                            for preset in ModelPreset::ALL {
                                ui.selectable_value(
                                    &mut self.settings.local_model,
                                    preset,
                                    preset.label(),
                                );
                            }
                        });
                    if old_preset != self.settings.local_model {
                        self.sync_model_path_from_preset();
                    }

                    ui.label(self.settings.local_model.description());
                    ui.label(format!(
                        "Erwarteter Dateiname: {}",
                        self.settings.local_model.default_filename()
                    ));

                    ui.label("Lokaler Modellpfad");
                    if ui
                        .text_edit_singleline(&mut self.settings.local_model_path)
                        .changed()
                    {
                        self.model_downloads.refresh_local_state(&self.settings);
                        self.dirty = true;
                    }

                    if let Ok(path) = self.dictation.suggested_model_path(&self.settings) {
                        ui.label(format!("Standardpfad: {}", path.display()));
                    }

                    if ui.button("Standardpfad uebernehmen").clicked() {
                        self.sync_model_path_from_preset();
                    }

                    ui.label(self.model_downloads.summary(&self.settings));
                    if let Some(progress) = self.model_downloads.progress_fraction() {
                        ui.add(egui::ProgressBar::new(progress).show_percentage());
                    }

                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !self.model_downloads.is_downloading(),
                                egui::Button::new("Ausgewaehltes Modell herunterladen"),
                            )
                            .clicked()
                        {
                            match self.model_downloads.start_download(&self.settings) {
                                Ok(message) => self.set_status(message),
                                Err(err) => self.set_status(err),
                            }
                        }

                        if ui
                            .add_enabled(
                                !self.model_downloads.is_downloading(),
                                egui::Button::new("Lokales Modell loeschen"),
                            )
                            .clicked()
                        {
                            match self.model_downloads.delete_downloaded_model(&self.settings) {
                                Ok(message) => {
                                    self.dictation.invalidate_model_cache();
                                    self.set_status(message);
                                }
                                Err(err) => self.set_status(err),
                            }
                        }
                    });

                    ui.label(format!(
                        "Downloadquelle: {}",
                        self.settings.local_model.download_url()
                    ));
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.heading("Externe Provider");

                    ui.label("Ollama Endpoint");
                    self.dirty |= ui
                        .text_edit_singleline(&mut self.settings.ollama.endpoint)
                        .changed();
                    ui.label("Ollama Modellname");
                    self.dirty |= ui
                        .text_edit_singleline(&mut self.settings.ollama.model_name)
                        .changed();

                    ui.add_space(8.0);

                    ui.label("LM Studio Endpoint");
                    self.dirty |= ui
                        .text_edit_singleline(&mut self.settings.lm_studio.endpoint)
                        .changed();
                    ui.label("LM Studio Modellname");
                    self.dirty |= ui
                        .text_edit_singleline(&mut self.settings.lm_studio.model_name)
                        .changed();
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.heading("Desktop-Status");
                    ui.label(self.desktop.summary());
                    ui.label(self.dictation.summary());
                    ui.label(self.autostart.summary());
                    ui.label(format!(
                        "Hotkey-Ausloesungen in dieser Sitzung: {}",
                        self.dictation_trigger_count
                    ));
                    ui.label(
                        "Fenster-Schliessen blendet die App in den Tray aus. Ueber den Tray kannst du das Fenster wieder anzeigen oder die App komplett beenden.",
                    );
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.heading("Transkript");
                    if self.last_transcript.is_empty() {
                        ui.label("Noch kein Transkript vorhanden.");
                    } else {
                        if ui.button("In aktive App einfuegen").clicked() {
                            match insert_text_into_active_app(&self.last_transcript, &self.settings)
                            {
                                Ok(message) => self.set_status(message),
                                Err(err) => self.set_status(err),
                            }
                        }

                        ui.add_space(8.0);
                        ui.label(&self.last_transcript);
                    }
                });
            });
        });
    }
}
