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

    pub fn spawn(_app: adw::Application, state: AppState) {
        // ksni panics (via `.unwrap()` on the D-Bus registration result) when
        // no StatusNotifierWatcher owns the well-known name on the session
        // bus. Vanilla GNOME ships without one — the user has to install the
        // "AppIndicator and KStatusNotifierItem Support" extension — and
        // sandboxed/CI sessions often lack it entirely. Probe first and skip
        // gracefully so the main window still opens.
        if !status_notifier_watcher_available() {
            tracing::warn!(
                "no org.kde.StatusNotifierWatcher on session bus — tray disabled. \
                 On GNOME install the \"AppIndicator and KStatusNotifierItem Support\" extension."
            );
            return;
        }

        let tray_state = Arc::new(Mutex::new(TrayState {
            is_recording: false,
            active_mode: String::new(),
            lang: state.with(|snap| snap.settings.ui_language),
        }));

        let service = ksni::TrayService::new(OpenWhisperTray {
            tray_state: Arc::clone(&tray_state),
        });
        let handle = service.handle();
        service.spawn();

        // Re-sync tray metadata once per second. The tray only changes
        // when recording toggles or the user switches mode/language, so a
        // tighter loop just burns CPU. Updating in place (via the Handle)
        // keeps the D-Bus service alive across the app's lifetime.
        glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
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
        tray_state: Arc<Mutex<TrayState>>,
    }

    /// Returns true if some process owns `org.kde.StatusNotifierWatcher` on
    /// the current session bus. Uses the standard `NameHasOwner` D-Bus method
    /// on the bus daemon itself, so it doesn't depend on ksni's internals.
    fn status_notifier_watcher_available() -> bool {
        use glib::Variant;

        let Ok(bus) = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) else {
            return false;
        };
        let args = Variant::tuple_from_iter([Variant::from("org.kde.StatusNotifierWatcher")]);
        match bus.call_sync(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "NameHasOwner",
            Some(&args),
            None,
            gio::DBusCallFlags::NONE,
            500,
            gio::Cancellable::NONE,
        ) {
            Ok(reply) => reply.child_value(0).get::<bool>().unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Dispatch a thunk onto the GTK main thread.
    ///
    /// ksni runs the tray service on its own thread, but GTK widgets and the
    /// Rust bridge are thread-local: every `Application`, `bridge::…` call has
    /// to run on the thread that owns them. `MainContext::default().invoke`
    /// schedules a `Send` closure onto that thread; inside we look the
    /// application up via `gio::Application::default()` rather than carrying a
    /// non-Send reference through the tray struct.
    fn on_main_thread<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        glib::MainContext::default().invoke(f);
    }

    fn with_default_app<F>(f: F)
    where
        F: FnOnce(&adw::Application) + Send + 'static,
    {
        on_main_thread(move || {
            if let Some(app) = gio::Application::default()
                .and_then(|a| a.downcast::<adw::Application>().ok())
            {
                f(&app);
            } else {
                tracing::warn!("tray dispatch: no default Application registered");
            }
        });
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
            with_default_app(|app| app.activate());
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
                        on_main_thread(move || {
                            let outcome = if is_recording {
                                bridge::stop_dictation()
                            } else {
                                bridge::start_dictation()
                            };
                            if let Err(err) = outcome {
                                tracing::warn!(%err, "tray dictation toggle failed");
                            }
                        });
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
                    activate: Box::new(|_: &mut Self| {
                        with_default_app(|app| app.activate());
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: tr("tray.quit", lang),
                    icon_name: "application-exit-symbolic".into(),
                    activate: Box::new(|_: &mut Self| {
                        with_default_app(|app| app.quit());
                    }),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    // `Handle<OpenWhisperTray>` is the ksni service handle kept inside the
    // 350 ms refresh closure above. No external caller needs it, so we do
    // not re-export it — keeping `OpenWhisperTray` module-private.
    #[allow(dead_code, private_interfaces)]
    type _TrayHandle = Handle<OpenWhisperTray>;
}
