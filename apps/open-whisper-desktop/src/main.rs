mod settings_store;

use eframe::egui::{self, RichText};
use open_whisper_core::{AppSettings, ModelPreset, ProviderKind, StartupBehavior, TriggerMode};

fn main() -> eframe::Result<()> {
    let initial_settings = settings_store::load().unwrap_or_else(|err| {
        eprintln!("failed to load settings: {err}");
        AppSettings::default()
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Open Whisper",
        options,
        Box::new(|_cc| Ok(Box::new(OpenWhisperDesktopApp::new(initial_settings)))),
    )
}

struct OpenWhisperDesktopApp {
    settings: AppSettings,
    dirty: bool,
    status: String,
}

impl OpenWhisperDesktopApp {
    fn new(settings: AppSettings) -> Self {
        Self {
            settings,
            dirty: false,
            status: "Noch nicht gespeichert".to_owned(),
        }
    }

    fn save(&mut self) {
        match settings_store::save(&self.settings) {
            Ok(path) => {
                self.dirty = false;
                self.status = format!("Gespeichert unter {}", path.display());
            }
            Err(err) => {
                self.status = format!("Speichern fehlgeschlagen: {err}");
            }
        }
    }
}

impl eframe::App for OpenWhisperDesktopApp {
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
                    "Dieses Grundgeruest bildet bereits Startup-Verhalten, Eingabegeraet, \
                     Modellwahl und externe Provider fuer die spaetere Tray-App ab.",
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
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.heading("Aufnahme");

                    ui.label("Eingabegeraet");
                    self.dirty |= ui
                        .text_edit_singleline(&mut self.settings.input_device_name)
                        .changed();

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

                    self.dirty |= ui
                        .checkbox(
                            &mut self.settings.insert_text_automatically,
                            "Transkript automatisch in die aktive App einfuegen",
                        )
                        .changed();
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
                    self.dirty |= old_preset != self.settings.local_model;

                    ui.label(self.settings.local_model.description());
                    ui.label(format!(
                        "Geplanter Download fuer lokale Nutzung: {}",
                        self.settings.local_model.whisper_model()
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
                    ui.heading("Aktiver Pfad");
                    ui.label(self.settings.active_provider_summary());
                    ui.label(
                        "Naechste Integrationsschritte: Tray, globaler Hotkey, Audioaufnahme, \
                         Text-Einfuegen ins aktive Fremdprogramm.",
                    );
                });
            });
        });
    }
}
