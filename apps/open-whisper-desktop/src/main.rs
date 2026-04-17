mod autostart;
mod desktop_integration;
mod dictation;
mod model_manager;
mod permission_diagnostics;
mod settings_store;
mod text_inserter;
mod ui_theme;

use autostart::AutostartManager;
use desktop_integration::{DesktopAction, DesktopIntegration, POLL_INTERVAL};
use dictation::{DictationController, DictationOutcome};
use eframe::egui::{self, Align, Layout, RichText, Vec2};
use model_manager::ModelDownloadManager;
use open_whisper_core::{AppSettings, ModelPreset, ProviderKind, StartupBehavior, TriggerMode};
use permission_diagnostics::{PermissionReport, PermissionStatus};
use text_inserter::insert_text_into_active_app;

fn main() -> eframe::Result<()> {
    let initial_settings = settings_store::load().unwrap_or_else(|err| {
        eprintln!("failed to load settings: {err}");
        AppSettings::default()
    });
    let start_hidden = AutostartManager::start_hidden_requested();
    let start_visible = !start_hidden || !initial_settings.onboarding_completed;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 860.0])
            .with_visible(start_visible),
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
    onboarding_step: usize,
    permission_report: PermissionReport,
    status: String,
    theme_applied: bool,
    window_visible: bool,
}

impl OpenWhisperDesktopApp {
    fn new(mut settings: AppSettings, start_hidden: bool) -> Self {
        if !settings.onboarding_completed
            && settings.startup_behavior == StartupBehavior::AskOnFirstLaunch
        {
            settings.startup_behavior = StartupBehavior::ManualLaunch;
        }

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

        let desktop = DesktopIntegration::new();
        let permission_report = PermissionReport::collect(&settings, &dictation, &desktop);
        let should_show_window = !start_hidden || !settings.onboarding_completed;

        Self {
            autostart,
            settings,
            desktop,
            dictation,
            model_downloads,
            dirty: false,
            dictation_trigger_count: 0,
            exit_requested: false,
            last_transcript: String::new(),
            onboarding_step: 0,
            permission_report,
            status,
            theme_applied: false,
            window_visible: should_show_window,
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

    fn refresh_permission_report(&mut self) {
        self.permission_report =
            PermissionReport::collect(&self.settings, &self.dictation, &self.desktop);
    }

    fn finish_onboarding(&mut self) {
        self.settings.onboarding_completed = true;
        self.dirty = true;
        self.save();
        self.set_status("Ersteinrichtung abgeschlossen.");
    }

    fn render_permission_report(&self, ui: &mut egui::Ui) {
        ui.label(ui_theme::muted(&self.permission_report.summary()));
        for item in self.permission_report.items() {
            egui::Frame::new()
                .fill(ui_theme::status_fill(item.status()))
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::same(12))
                .corner_radius(14)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(item.status().badge())
                                .strong()
                                .color(ui_theme::status_text(item.status())),
                        );
                        ui.label(RichText::new(item.title()).strong().color(ui_theme::TEXT));
                        ui.label(ui_theme::muted(item.detail()));
                    });
                });
        }
    }

    fn render_audio_setup(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(ui_theme::field_label("Eingabegeraet"));
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

        ui.label(ui_theme::field_label("Globaler Shortcut"));
        self.dirty |= ui.text_edit_singleline(&mut self.settings.hotkey).changed();

        let old = self.settings.trigger_mode;
        ui.label(ui_theme::field_label("Aufnahmemodus"));
        egui::ComboBox::from_label("Aufnahmemodus")
            .selected_text(self.settings.trigger_mode.label())
            .show_ui(ui, |ui| {
                for option in TriggerMode::ALL {
                    ui.selectable_value(&mut self.settings.trigger_mode, option, option.label());
                }
            });
        self.dirty |= old != self.settings.trigger_mode;

        ui.label(ui_theme::field_label("Sprache (`auto`, `de`, `en`, ...)"));
        self.dirty |= ui
            .text_edit_singleline(&mut self.settings.transcription_language)
            .changed();
    }

    fn render_model_setup(&mut self, ui: &mut egui::Ui) {
        self.settings.active_provider = ProviderKind::LocalWhisper;

        let old_preset = self.settings.local_model;
        egui::ComboBox::from_label("Lokales Standardmodell")
            .selected_text(self.settings.local_model.label())
            .show_ui(ui, |ui| {
                for preset in ModelPreset::ALL {
                    ui.selectable_value(
                        &mut self.settings.local_model,
                        preset,
                        format!("{} ({})", preset.label(), preset.whisper_model()),
                    );
                }
            });
        if old_preset != self.settings.local_model {
            self.sync_model_path_from_preset();
        }

        ui.label(ui_theme::muted(self.settings.local_model.description()));
        ui.label(ui_theme::muted(
            "Die drei lokalen Standardmodelle Klein, Mittel und Gross bleiben als integrierte Presets in der App verfuegbar.",
        ));
        ui.label(format!(
            "Aktueller Backend-Name: {}",
            self.settings.local_model.whisper_model()
        ));
        ui.label(format!(
            "Erwarteter Dateiname: {}",
            self.settings.local_model.default_filename()
        ));

        if let Ok(path) = self.dictation.suggested_model_path(&self.settings) {
            ui.label(format!("Standardpfad: {}", path.display()));
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

            if ui.button("Standardpfad uebernehmen").clicked() {
                self.sync_model_path_from_preset();
            }
        });

        ui.label(format!(
            "Downloadquelle: {}",
            self.settings.local_model.download_url()
        ));
        ui.label(ui_theme::muted(
            "Ollama und LM Studio bleiben optional und kommen spaeter nur als Zusatzpfad dazu.",
        ));
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui_theme::hero_card().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(ui_theme::eyebrow("Desktop Dictation"));
                    ui.label(ui_theme::title("Open Whisper"));
                    ui.label(ui_theme::muted(
                        "Lokale Sprache-zu-Text-App mit Tray, Hotkey, Whisper-Modellen und sauberem Setup.",
                    ));
                });

                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if ui
                        .add_enabled(self.dirty, ui_theme::primary_button("Speichern"))
                        .clicked()
                    {
                        self.save();
                    }

                    let status_kind = if self.dirty {
                        PermissionStatus::Warning
                    } else {
                        PermissionStatus::Ok
                    };
                    egui::Frame::new()
                        .fill(ui_theme::status_fill(status_kind))
                        .stroke(egui::Stroke::NONE)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .corner_radius(255)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(if self.dirty {
                                    "Ungespeicherte Aenderungen"
                                } else {
                                    "Konfiguration aktuell"
                                })
                                .strong()
                                .color(ui_theme::status_text(status_kind)),
                            );
                        });
                });
            });

            ui.add_space(16.0);
            ui.horizontal_wrapped(|ui| {
                self.metric_pill(ui, "Provider", "Local Whisper");
                self.metric_pill(ui, "Modell", self.settings.local_model.label());
                self.metric_pill(ui, "Hotkey", &self.settings.hotkey);
                self.metric_pill(ui, "Status", &self.status);
            });
        });
    }

    fn metric_pill(&self, ui: &mut egui::Ui, label: &str, value: &str) {
        egui::Frame::new()
            .fill(ui_theme::SURFACE_ELEVATED)
            .stroke(egui::Stroke::new(1.0, ui_theme::BORDER))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .corner_radius(255)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(ui_theme::eyebrow(label));
                    ui.label(RichText::new(value).strong().color(ui_theme::TEXT));
                });
            });
    }

    fn render_overview_cards(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;

            ui_theme::metric_card().show(ui, |ui| {
                ui.label(ui_theme::eyebrow("Modell"));
                ui.label(ui_theme::metric_value(self.settings.local_model.label()));
                ui.label(ui_theme::muted(self.settings.local_model.description()));
            });

            ui_theme::metric_card().show(ui, |ui| {
                ui.label(ui_theme::eyebrow("Aufnahmemodus"));
                ui.label(ui_theme::metric_value(self.settings.trigger_mode.label()));
                ui.label(ui_theme::muted(&self.settings.input_device_name));
            });

            ui_theme::metric_card().show(ui, |ui| {
                ui.label(ui_theme::eyebrow("Autostart"));
                ui.label(ui_theme::metric_value(
                    self.settings.startup_behavior.label(),
                ));
                ui.label(ui_theme::muted(self.autostart.summary()));
            });
        });
    }

    fn render_onboarding_step_rail(&mut self, ui: &mut egui::Ui) {
        const STEPS: [&str; 4] = [
            "Willkommen",
            "Audio und Hotkey",
            "Lokales Modell",
            "Autostart und Diagnose",
        ];

        ui_theme::card().show(ui, |ui| {
            ui.label(ui_theme::eyebrow("Setup Flow"));
            ui.label(ui_theme::section_title("Ersteinrichtung"));
            ui.label(ui_theme::muted(
                "Die App fuehrt dich in vier klaren Schritten zu einem produktiven lokalen Setup.",
            ));
            ui.add_space(12.0);

            for (index, title) in STEPS.iter().enumerate() {
                let active = self.onboarding_step == index;
                let complete = self.onboarding_step > index;
                let fill = if active {
                    ui_theme::ACCENT_SOFT
                } else if complete {
                    egui::Color32::from_rgb(223, 239, 231)
                } else {
                    ui_theme::SURFACE_ELEVATED
                };

                egui::Frame::new()
                    .fill(fill)
                    .stroke(egui::Stroke::new(1.0, ui_theme::BORDER))
                    .inner_margin(egui::Margin::same(14))
                    .corner_radius(18)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{:02}", index + 1))
                                    .strong()
                                    .color(if active || complete {
                                        ui_theme::ACCENT_STRONG
                                    } else {
                                        ui_theme::TEXT_MUTED
                                    }),
                            );
                            ui.label(
                                RichText::new(*title)
                                    .strong()
                                    .color(ui_theme::TEXT),
                            );
                        });
                    });
            }

            ui.add_space(10.0);
            ui.label(ui_theme::muted(
                "Ollama und LM Studio bleiben optional. Der Standardpfad bleibt bei lokalen Whisper-Modellen.",
            ));
        });
    }

    fn render_onboarding(&mut self, ui: &mut egui::Ui) {
        const STEP_COUNT: usize = 4;
        let step_title = match self.onboarding_step {
            0 => "Willkommen",
            1 => "Audio und Hotkey",
            2 => "Lokales Modell",
            _ => "Autostart und Diagnose",
        };

        ui.vertical(|ui| {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(270.0, ui.available_height()),
                    Layout::top_down(Align::Min),
                    |ui| self.render_onboarding_step_rail(ui),
                );

                ui.add_space(18.0);

                ui.vertical(|ui| {
                    ui_theme::card_emphasis().show(ui, |ui| {
                        ui.label(ui_theme::eyebrow("Schritt"));
                        ui.label(ui_theme::section_title(step_title));
                        ui.label(ui_theme::muted(
                            "Dieser Assistent richtet Open Whisper fuer den ersten produktiven Start ein.",
                        ));
                        ui.add(
                            egui::ProgressBar::new((self.onboarding_step + 1) as f32 / STEP_COUNT as f32)
                                .show_percentage()
                                .corner_radius(255)
                                .fill(ui_theme::ACCENT)
                                .text(format!(
                                    "Schritt {} von {}",
                                    self.onboarding_step + 1,
                                    STEP_COUNT
                                )),
                        );

                        ui.add_space(14.0);

                        match self.onboarding_step {
                            0 => {
                                ui.label(ui_theme::muted("Open Whisper startet im Hintergrund, hoert auf einen globalen Shortcut und fuegt den Text danach direkt in die aktive App ein."));
                                ui.add_space(10.0);
                                self.render_overview_cards(ui);
                            }
                            1 => {
                                self.render_audio_setup(ui);
                            }
                            2 => {
                                self.render_model_setup(ui);
                            }
                            _ => {
                                ui.label(ui_theme::muted(
                                    "Waehle jetzt, ob die App beim Systemstart automatisch im Hintergrund geladen werden soll.",
                                ));
                                ui.horizontal(|ui| {
                                    ui.radio_value(
                                        &mut self.settings.startup_behavior,
                                        StartupBehavior::LaunchAtLogin,
                                        "Mit dem System starten",
                                    );
                                    ui.radio_value(
                                        &mut self.settings.startup_behavior,
                                        StartupBehavior::ManualLaunch,
                                        "Nur manuell starten",
                                    );
                                });
                                ui.add_space(10.0);
                                self.render_permission_report(ui);
                            }
                        }
                    });

                    ui.add_space(14.0);

                    ui_theme::card().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(
                                    self.onboarding_step > 0,
                                    ui_theme::ghost_button("Zurueck"),
                                )
                                .clicked()
                            {
                                self.onboarding_step -= 1;
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if self.onboarding_step + 1 < STEP_COUNT {
                                    if ui.add(ui_theme::primary_button("Weiter")).clicked() {
                                        self.onboarding_step += 1;
                                    }
                                } else if ui
                                    .add_enabled(
                                        !self.model_downloads.is_downloading(),
                                        ui_theme::primary_button("Einrichtung abschliessen"),
                                    )
                                    .clicked()
                                {
                                    self.finish_onboarding();
                                }
                            });
                        });

                        if self.model_downloads.is_downloading() {
                            ui.label(ui_theme::muted(
                                "Einrichtung kann abgeschlossen werden, sobald der Modelldownload fertig ist.",
                            ));
                        } else if self.permission_report.has_errors() {
                            ui.label(
                                RichText::new("Es gibt noch Diagnosefehler. Du kannst die Einrichtung trotzdem abschliessen und spaeter in den Einstellungen nacharbeiten.")
                                    .color(ui_theme::ERROR),
                            );
                        } else if self.permission_report.has_warnings() {
                            ui.label(
                                RichText::new("Es gibt noch Hinweise oder Warnungen. Die App ist trotzdem bereits konfigurierbar.")
                                    .color(ui_theme::WARNING),
                            );
                        }
                    });
                });
            });
        });
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

        self.refresh_permission_report();

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
        if !self.theme_applied {
            ui_theme::apply(ui.ctx());
            self.theme_applied = true;
        }

        ui_theme::app_canvas().show(ui, |ui| {
            self.render_header(ui);
            ui.add_space(18.0);

            if !self.settings.onboarding_completed {
                self.render_onboarding(ui);
                return;
            }

            self.render_overview_cards(ui);
            ui.add_space(18.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.columns(2, |columns| {
                        columns[0].vertical(|ui| {
                            ui_theme::card().show(ui, |ui| {
                                ui.label(ui_theme::eyebrow("Startup"));
                                ui.label(ui_theme::section_title("Systemstart"));

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

                                ui.label(ui_theme::muted(self.autostart.summary()));
                            });

                            ui.add_space(14.0);

                            ui_theme::card().show(ui, |ui| {
                                ui.label(ui_theme::eyebrow("Input"));
                                ui.label(ui_theme::section_title("Aufnahme"));
                                self.render_audio_setup(ui);

                                self.dirty |= ui
                                    .checkbox(
                                        &mut self.settings.insert_text_automatically,
                                        "Transkript automatisch in die aktive App einfuegen",
                                    )
                                    .changed();

                                ui.label(ui_theme::field_label("Einfuege-Verzoegerung (ms)"));
                                self.dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut self.settings.insert_delay_ms)
                                            .speed(10)
                                            .range(0..=2_000),
                                    )
                                    .changed();

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

                                ui.label(ui_theme::field_label("VAD-Schwelle"));
                                self.dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut self.settings.vad_threshold)
                                            .speed(0.001)
                                            .range(0.001..=0.2),
                                    )
                                    .changed();

                                ui.label(ui_theme::field_label("Stille bis Stop (ms)"));
                                self.dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut self.settings.vad_silence_ms)
                                            .speed(25)
                                            .range(200..=5_000),
                                    )
                                    .changed();

                                ui.horizontal(|ui| {
                                    if ui
                                        .add_enabled(
                                            !self.dictation.is_recording()
                                                && !self.dictation.is_transcribing(),
                                            ui_theme::primary_button("Aufnahme starten"),
                                        )
                                        .clicked()
                                    {
                                        let outcomes = self
                                            .dictation
                                            .start_recording(&self.settings)
                                            .map_or_else(
                                                |err| vec![DictationOutcome::Status(err)],
                                                |message| vec![DictationOutcome::Status(message)],
                                            );
                                        self.apply_dictation_outcomes(outcomes);
                                    }

                                    if ui
                                        .add_enabled(
                                            self.dictation.is_recording(),
                                            ui_theme::secondary_button("Aufnahme stoppen"),
                                        )
                                        .clicked()
                                    {
                                        let outcomes = self
                                            .dictation
                                            .stop_recording_and_transcribe(
                                                &self.settings,
                                                "Manuell gestoppt",
                                            )
                                            .unwrap_or_else(|err| {
                                                vec![DictationOutcome::Status(err)]
                                            });
                                        self.apply_dictation_outcomes(outcomes);
                                    }
                                });
                            });

                            ui.add_space(14.0);

                            ui_theme::card().show(ui, |ui| {
                                ui.label(ui_theme::eyebrow("Checks"));
                                ui.label(ui_theme::section_title("Permission-Diagnose"));
                                self.render_permission_report(ui);
                            });
                        });

                        columns[1].vertical(|ui| {
                            ui_theme::card().show(ui, |ui| {
                                ui.label(ui_theme::eyebrow("Model"));
                                ui.label(ui_theme::section_title("Lokales Sprachmodell"));

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

                                ui.label(ui_theme::muted(self.settings.local_model.description()));
                                ui.label(ui_theme::field_label("Lokaler Modellpfad"));
                                if ui
                                    .text_edit_singleline(&mut self.settings.local_model_path)
                                    .changed()
                                {
                                    self.model_downloads.refresh_local_state(&self.settings);
                                    self.dirty = true;
                                }

                                if let Ok(path) =
                                    self.dictation.suggested_model_path(&self.settings)
                                {
                                    ui.label(ui_theme::muted(&format!(
                                        "Standardpfad: {}",
                                        path.display()
                                    )));
                                }

                                ui.label(ui_theme::muted(&self.model_downloads.summary(&self.settings)));
                                if let Some(progress) = self.model_downloads.progress_fraction() {
                                    ui.add(
                                        egui::ProgressBar::new(progress)
                                            .show_percentage()
                                            .corner_radius(255)
                                            .fill(ui_theme::ACCENT),
                                    );
                                }

                                ui.horizontal(|ui| {
                                    if ui
                                        .add_enabled(
                                            !self.model_downloads.is_downloading(),
                                            ui_theme::primary_button(
                                                "Ausgewaehltes Modell herunterladen",
                                            ),
                                        )
                                        .clicked()
                                    {
                                        match self.model_downloads.start_download(&self.settings) {
                                            Ok(message) => self.set_status(message),
                                            Err(err) => self.set_status(err),
                                        }
                                    }

                                    if ui
                                        .add(ui_theme::ghost_button("Standardpfad uebernehmen"))
                                        .clicked()
                                    {
                                        self.sync_model_path_from_preset();
                                    }

                                    if ui
                                        .add_enabled(
                                            !self.model_downloads.is_downloading(),
                                            ui_theme::secondary_button("Lokales Modell loeschen"),
                                        )
                                        .clicked()
                                    {
                                        match self
                                            .model_downloads
                                            .delete_downloaded_model(&self.settings)
                                        {
                                            Ok(message) => {
                                                self.dictation.invalidate_model_cache();
                                                self.set_status(message);
                                            }
                                            Err(err) => self.set_status(err),
                                        }
                                    }
                                });

                                ui.label(ui_theme::muted(&format!(
                                    "Downloadquelle: {}",
                                    self.settings.local_model.download_url()
                                )));
                            });

                            ui.add_space(14.0);

                            ui_theme::card().show(ui, |ui| {
                                ui.label(ui_theme::eyebrow("Optional"));
                                ui.label(ui_theme::section_title("Externe Provider"));
                                ui.label(ui_theme::muted(
                                    "Ollama und LM Studio sind nur Zusatzpfade. Standardmaessig bleibt Local Whisper aktiv.",
                                ));

                                ui.label(ui_theme::field_label("Ollama Endpoint"));
                                self.dirty |= ui
                                    .text_edit_singleline(&mut self.settings.ollama.endpoint)
                                    .changed();
                                ui.label(ui_theme::field_label("Ollama Modellname"));
                                self.dirty |= ui
                                    .text_edit_singleline(&mut self.settings.ollama.model_name)
                                    .changed();

                                ui.label(ui_theme::field_label("LM Studio Endpoint"));
                                self.dirty |= ui
                                    .text_edit_singleline(&mut self.settings.lm_studio.endpoint)
                                    .changed();
                                ui.label(ui_theme::field_label("LM Studio Modellname"));
                                self.dirty |= ui
                                    .text_edit_singleline(&mut self.settings.lm_studio.model_name)
                                    .changed();
                            });

                            ui.add_space(14.0);

                            ui_theme::card().show(ui, |ui| {
                                ui.label(ui_theme::eyebrow("Desktop"));
                                ui.label(ui_theme::section_title("Status"));
                                ui.label(ui_theme::muted(&self.desktop.summary()));
                                ui.label(ui_theme::muted(&self.dictation.summary()));
                                ui.label(ui_theme::muted(self.autostart.summary()));
                                ui.label(ui_theme::muted(&format!(
                                    "Hotkey-Ausloesungen in dieser Sitzung: {}",
                                    self.dictation_trigger_count
                                )));
                                ui.label(ui_theme::muted(
                                    "Fenster-Schliessen blendet die App in den Tray aus. Ueber den Tray kannst du das Fenster wieder anzeigen oder die App komplett beenden.",
                                ));
                            });
                        });
                    });

                    ui.add_space(16.0);

                    ui_theme::card_emphasis().show(ui, |ui| {
                        ui.label(ui_theme::eyebrow("Transcript"));
                        ui.label(ui_theme::section_title("Transkript"));
                        if self.last_transcript.is_empty() {
                            ui.label(ui_theme::muted(
                                "Noch kein Transkript vorhanden. Starte eine Aufnahme oder nutze den globalen Hotkey.",
                            ));
                        } else {
                            ui.horizontal(|ui| {
                                if ui
                                    .add(ui_theme::primary_button("In aktive App einfuegen"))
                                    .clicked()
                                {
                                    match insert_text_into_active_app(
                                        &self.last_transcript,
                                        &self.settings,
                                    ) {
                                        Ok(message) => self.set_status(message),
                                        Err(err) => self.set_status(err),
                                    }
                                }
                            });
                            ui.add_space(8.0);
                            egui::Frame::new()
                                .fill(ui_theme::SURFACE_ELEVATED)
                                .stroke(egui::Stroke::new(1.0, ui_theme::BORDER))
                                .inner_margin(egui::Margin::same(16))
                                .corner_radius(18)
                                .show(ui, |ui| {
                                    ui.label(&self.last_transcript);
                                });
                        }
                    });
                });
        });
    }
}
