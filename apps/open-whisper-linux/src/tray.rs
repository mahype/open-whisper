//! StatusNotifierItem tray integration (Linux-only).
//!
//! Registers the app with the desktop's `org.kde.StatusNotifierWatcher`
//! D-Bus service so KDE, Xfce, Cinnamon, Budgie and MATE show the icon
//! natively. Vanilla GNOME needs the "AppIndicator and KStatusNotifierItem
//! Support" extension; we surface that hint in onboarding.
//!
//! Menu parity with the macOS AppDelegate:
//!   • Diktat starten/stoppen
//!   • Einstellungen…
//!   • (Active mode indicator — text only, read-only)
//!   • Beenden
//!
//! Mode and model submenus (editable) live in the settings window. Keeping
//! the tray menu short avoids desktop-specific rendering quirks and matches
//! GNOME HIG recommendations when an indicator is shown.

#![allow(dead_code)]

#[cfg(target_os = "linux")]
pub use linux::spawn;

#[cfg(not(target_os = "linux"))]
pub fn spawn(_app: adw::Application, _state: crate::state::AppState) {
    // no-op on non-Linux dev builds
}

#[cfg(target_os = "linux")]
mod linux {
    use std::sync::{Arc, Mutex};

    use adw::prelude::*;
    use ksni::{
        Handle, ToolTip, Tray,
        menu::{MenuItem, StandardItem},
    };

    use crate::bridge;
    use crate::i18n::tr;
    use crate::state::AppState;

    pub fn spawn(app: adw::Application, state: AppState) {
        let tray_state = Arc::new(Mutex::new(TrayState {
            is_recording: false,
            active_mode: String::new(),
            lang: state.with(|snap| snap.settings.ui_language),
        }));

        let service = ksni::TrayService::new(OpenWhisperTray {
            app: app.clone(),
            tray_state: Arc::clone(&tray_state),
        });
        let handle = service.handle();
        service.spawn();

        // Re-sync tray metadata on the same 350 ms cadence as the main
        // window. Updating in place (via the Handle) keeps the D-Bus
        // service alive across the app's lifetime.
        glib::timeout_add_local(std::time::Duration::from_millis(350), move || {
            let snap = state.snapshot();
            let mut guard = tray_state.lock().expect("tray state lock poisoned");
            let changed = guard.is_recording != snap.runtime.is_recording
                || guard.active_mode != snap.runtime.active_mode_name
                || guard.lang != snap.settings.ui_language;
            if changed {
                guard.is_recording = snap.runtime.is_recording;
                guard.active_mode = snap.runtime.active_mode_name.clone();
                guard.lang = snap.settings.ui_language;
                drop(guard);
                handle.update(|_| {});
            }
            glib::ControlFlow::Continue
        });
    }

    struct TrayState {
        is_recording: bool,
        active_mode: String,
        lang: open_whisper_core::UiLanguage,
    }

    struct OpenWhisperTray {
        app: adw::Application,
        tray_state: Arc<Mutex<TrayState>>,
    }

    impl Tray for OpenWhisperTray {
        fn icon_name(&self) -> String {
            let state = self.tray_state.lock().expect("tray state lock poisoned");
            if state.is_recording {
                "audio-input-microphone-high-symbolic".to_owned()
            } else {
                "audio-input-microphone-symbolic".to_owned()
            }
        }

        fn title(&self) -> String {
            "Open Whisper".to_owned()
        }

        fn tool_tip(&self) -> ToolTip {
            let state = self.tray_state.lock().expect("tray state lock poisoned");
            let status = if state.is_recording {
                tr("tray.tooltip.recording", state.lang)
            } else {
                tr("tray.tooltip.idle", state.lang)
            };
            ToolTip {
                icon_name: self.icon_name(),
                icon_pixmap: Vec::new(),
                title: "Open Whisper".to_owned(),
                description: status,
            }
        }

        fn activate(&mut self, _x: i32, _y: i32) {
            // Left-click opens (or focuses) the main window.
            self.app.activate();
        }

        fn menu(&self) -> Vec<MenuItem<Self>> {
            let (is_recording, active_mode, lang) = {
                let state = self.tray_state.lock().expect("tray state lock poisoned");
                (
                    state.is_recording,
                    state.active_mode.clone(),
                    state.lang,
                )
            };

            let dictate_label = if is_recording {
                tr("tray.stop_dictation", lang)
            } else {
                tr("tray.start_dictation", lang)
            };

            vec![
                StandardItem {
                    label: dictate_label,
                    icon_name: "audio-input-microphone-symbolic".into(),
                    activate: Box::new(move |_: &mut Self| {
                        let outcome = if is_recording {
                            bridge::stop_dictation()
                        } else {
                            bridge::start_dictation()
                        };
                        if let Err(err) = outcome {
                            tracing::warn!(%err, "tray dictation toggle failed");
                        }
                    }),
                    ..Default::default()
                }
                .into(),
                MenuItem::Separator,
                StandardItem {
                    label: format!("{}: {}", tr("tray.mode", lang), active_mode),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
                MenuItem::Separator,
                StandardItem {
                    label: tr("tray.open_settings", lang),
                    icon_name: "emblem-system-symbolic".into(),
                    activate: Box::new(|tray: &mut Self| {
                        tray.app.activate();
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: tr("tray.quit", lang),
                    icon_name: "application-exit-symbolic".into(),
                    activate: Box::new(|tray: &mut Self| {
                        tray.app.quit();
                    }),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    // Convenience: Handle is actually from ksni, re-exported so the state
    // module doesn't need to pull in ksni types directly.
    #[allow(dead_code)]
    pub type TrayHandle = Handle<OpenWhisperTray>;
}
