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
//!   • Nachbearbeitung → Submenu (Off + modes, dynamic, checkmark)
//!   • Transkriptionsmodell → Submenu (downloaded presets, checkmark)
//!   • Status-Zeile (disabled, reflects runtime.last_status)
//!   • Feedback senden… / Auf Aktualisierungen prüfen…
//!   • Beenden

#![allow(dead_code)]

#[cfg(target_os = "linux")]
pub use linux::{spawn, status_notifier_watcher_available};

#[cfg(not(target_os = "linux"))]
pub fn spawn(_app: adw::Application, _state: crate::state::AppState) {
    // no-op on non-Linux dev builds
}

#[cfg(not(target_os = "linux"))]
pub fn status_notifier_watcher_available() -> bool {
    false
}

#[cfg(target_os = "linux")]
mod linux {
    use std::sync::{Arc, Mutex};

    use adw::prelude::*;
    use ksni::{
        Handle, ToolTip, Tray,
        menu::{MenuItem, StandardItem},
    };
    use open_whisper_core::UiLanguage;

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

        let initial = state.with(|snap| TrayState {
            is_recording: snap.runtime.is_recording,
            active_mode: snap.runtime.active_mode_name.clone(),
            lang: snap.settings.ui_language,
        });
        let tray_state = Arc::new(Mutex::new(initial));

        let service = ksni::TrayService::new(OpenWhisperTray {
            tray_state: Arc::clone(&tray_state),
        });
        let handle = service.handle();
        service.spawn();

        // Re-sync tray metadata once per second. Lock hygiene: NEVER
        // call `self.method()` that re-locks `tray_state` from within a
        // lock scope — `std::sync::Mutex` is not reentrant and that
        // would deadlock the ksni thread. Every `menu()` /
        // `icon_name()` / `tool_tip()` takes the lock exactly once and
        // drops it before any side effect.
        //
        // We only trigger `handle.update` on a *real* change so
        // ubuntu-appindicators doesn't re-render the menu every second.
        // The extension tolerates a few updates/sec but not a sustained
        // one-per-second stream (it seems to drop the layout response
        // under that load). Comparing only the fields we actually
        // render keeps update rate near zero while idle.
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
        lang: UiLanguage,
    }

    struct OpenWhisperTray {
        tray_state: Arc<Mutex<TrayState>>,
    }

    /// Returns true if some process owns `org.kde.StatusNotifierWatcher` on
    /// the current session bus. Uses the standard `NameHasOwner` D-Bus method
    /// on the bus daemon itself, so it doesn't depend on ksni's internals.
    pub fn status_notifier_watcher_available() -> bool {
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
            if let Some(app) =
                gio::Application::default().and_then(|a| a.downcast::<adw::Application>().ok())
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
            // `self.icon_name()` must not be called here — that would re-lock
            // `tray_state` and `std::sync::Mutex` isn't reentrant. Compute the
            // icon name inline inside the same lock scope instead.
            let (icon_name, status) = {
                let state = self.tray_state.lock().expect("tray state lock poisoned");
                let icon = if state.is_recording {
                    "audio-input-microphone-high-symbolic".to_owned()
                } else {
                    "audio-input-microphone-symbolic".to_owned()
                };
                let status = if state.is_recording {
                    tr("tray.tooltip.recording", state.lang)
                } else {
                    tr("tray.tooltip.idle", state.lang)
                };
                (icon, status)
            };
            ToolTip {
                icon_name,
                icon_pixmap: Vec::new(),
                title: "Open Whisper".to_owned(),
                description: status,
            }
        }

        // Intentionally no `activate()` override. libappindicator (and
        // therefore the GNOME `ubuntu-appindicators` extension on
        // AnduinOS/Ubuntu) hardcodes left-click to "open context menu";
        // if we override `activate`, the compat shim silently drops the
        // menu popup and nothing at all is shown. Leaving the default
        // empty impl in place lets ubuntu-appindicators do its usual
        // thing — click = menu, pick "Einstellungen…" to surface the
        // window. On KDE/Xfce/Cinnamon, which honour SNI `activate`
        // properly, losing this shortcut is a minor UX regression; the
        // user still reaches the window via the menu entry.

        fn menu(&self) -> Vec<MenuItem<Self>> {
            tracing::debug!("tray::menu: begin");
            // Minimal menu — bisecting the ubuntu-appindicators rendering
            // issue. With 12 / 19 items (SubMenu or flat-with-section-
            // headers) the extension silently dropped the whole popup.
            // The historical working version had 5-6 items and rendered
            // fine. Starting here and expanding only if this works.
            let (is_recording, lang, active_mode) = {
                let state = self.tray_state.lock().expect("tray state lock poisoned");
                (state.is_recording, state.lang, state.active_mode.clone())
            };

            let dictate_label = if is_recording {
                tr("tray.stop_dictation", lang)
            } else {
                tr("tray.start_dictation", lang)
            };

            let mut items: Vec<MenuItem<Self>> = Vec::new();

            items.push(
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
            );
            items.push(MenuItem::Separator);
            items.push(
                StandardItem {
                    label: format!("{}: {}", tr("tray.mode", lang), active_mode),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            items.push(MenuItem::Separator);
            items.push(
                StandardItem {
                    label: tr("tray.open_settings", lang),
                    icon_name: "emblem-system-symbolic".into(),
                    activate: Box::new(|_: &mut Self| {
                        with_default_app(|app| app.activate());
                    }),
                    ..Default::default()
                }
                .into(),
            );
            items.push(
                StandardItem {
                    label: tr("tray.quit", lang),
                    icon_name: "application-exit-symbolic".into(),
                    activate: Box::new(|_: &mut Self| {
                        with_default_app(|app| app.quit());
                    }),
                    ..Default::default()
                }
                .into(),
            );

            tracing::debug!(item_count = items.len(), "tray::menu: done");
            items
        }
    }

    // `Handle<OpenWhisperTray>` is the ksni service handle kept inside the
    // 1 s refresh closure above. No external caller needs it, so we do
    // not re-export it — keeping `OpenWhisperTray` module-private.
    #[allow(dead_code, private_interfaces)]
    type _TrayHandle = Handle<OpenWhisperTray>;
}
