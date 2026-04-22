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
    use open_whisper_core::{AppSettings, ModelPreset, ModelStatusDto, UiLanguage};

    use crate::bridge;
    use crate::i18n::tr;
    use crate::state::AppState;

    /// GitHub destinations for the "Send feedback" and "Check for updates"
    /// menu entries. macOS uses an in-app Sparkle updater and a SwiftUI
    /// feedback form; on Linux we delegate to the default browser since we
    /// have no equivalent in-process. Keeping the URLs here means changing
    /// them is a one-line edit.
    const FEEDBACK_URL: &str = "https://github.com/mahype/open-whisper/issues/new";
    const RELEASES_URL: &str = "https://github.com/mahype/open-whisper/releases";

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
            last_status: snap.runtime.last_status.clone(),
            lang: snap.settings.ui_language,
            settings: snap.settings.clone(),
            models: Vec::new(),
        });
        let tray_state = Arc::new(Mutex::new(initial));

        let service = ksni::TrayService::new(OpenWhisperTray {
            tray_state: Arc::clone(&tray_state),
        });
        let handle = service.handle();
        service.spawn();

        // Re-sync tray metadata once per second. The model list is pulled
        // from the bridge here because it's not part of `AppSnapshot` yet
        // and the tray submenu needs fresh download state to filter. The
        // call is cheap — it only stat()'s the model files.
        //
        // Lock hygiene: NEVER call `self.method()` that re-locks
        // `tray_state` from within a lock scope — `std::sync::Mutex` is
        // not reentrant and that would deadlock the ksni thread, which in
        // turn would freeze the GTK main loop when this timer next tries
        // to `tray_state.lock()`. Every `menu()`/`icon_name()`/`tool_tip()`
        // takes the lock exactly once and drops it before any side effect.
        glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
            let snap = state.snapshot();
            let models = bridge::model_status_list();
            let mut guard = tray_state.lock().expect("tray state lock poisoned");
            let changed = guard.is_recording != snap.runtime.is_recording
                || guard.active_mode != snap.runtime.active_mode_name
                || guard.last_status != snap.runtime.last_status
                || guard.lang != snap.settings.ui_language
                || guard.settings != snap.settings
                || guard.models != models;
            if changed {
                guard.is_recording = snap.runtime.is_recording;
                guard.active_mode = snap.runtime.active_mode_name.clone();
                guard.last_status = snap.runtime.last_status.clone();
                guard.lang = snap.settings.ui_language;
                guard.settings = snap.settings.clone();
                guard.models = models;
                drop(guard);
                handle.update(|_| {});
            }
            glib::ControlFlow::Continue
        });
    }

    struct TrayState {
        is_recording: bool,
        active_mode: String,
        last_status: String,
        lang: UiLanguage,
        settings: AppSettings,
        models: Vec<ModelStatusDto>,
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
            if let Some(app) = gio::Application::default()
                .and_then(|a| a.downcast::<adw::Application>().ok())
            {
                f(&app);
            } else {
                tracing::warn!("tray dispatch: no default Application registered");
            }
        });
    }

    /// Apply a settings mutation and persist on the GTK main thread. The
    /// tray is the wrong place to block on I/O (ksni's thread owns no file
    /// descriptors our bridge expects), so we marshal the whole load /
    /// mutate / save cycle onto the main thread via `on_main_thread`.
    fn mutate_settings<F>(f: F)
    where
        F: FnOnce(&mut AppSettings) + Send + 'static,
    {
        on_main_thread(move || {
            let mut settings = bridge::load_settings();
            f(&mut settings);
            if let Err(err) = bridge::save_settings(settings) {
                tracing::warn!(%err, "tray: save_settings failed");
            }
        });
    }

    /// Open an http(s) URL in the user's default browser. Uses GIO's
    /// `AppInfo::launch_default_for_uri`, which is synchronous but does not
    /// need a parent window — handy because the tray has none.
    fn open_external_url(url: &'static str) {
        on_main_thread(move || {
            if let Err(err) =
                gio::AppInfo::launch_default_for_uri(url, gio::AppLaunchContext::NONE)
            {
                tracing::warn!(%err, url, "tray: failed to open URL");
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
            // Snapshot everything we need out of the lock in one shot, then
            // build the menu tree against locals. Keeps the lock scope short
            // and rules out any chance of re-entry.
            let (
                is_recording,
                lang,
                last_status,
                post_processing_enabled,
                active_mode_id,
                modes,
                local_model,
                downloaded_models,
            ) = {
                let state = self.tray_state.lock().expect("tray state lock poisoned");
                let downloaded: Vec<ModelStatusDto> = state
                    .models
                    .iter()
                    .filter(|m| m.is_downloaded)
                    .cloned()
                    .collect();
                (
                    state.is_recording,
                    state.lang,
                    state.last_status.clone(),
                    state.settings.post_processing_enabled,
                    state.settings.active_mode_id.clone(),
                    state.settings.modes.clone(),
                    state.settings.local_model,
                    downloaded,
                )
            };
            tracing::debug!(
                modes = modes.len(),
                downloaded = downloaded_models.len(),
                post_processing_enabled,
                "tray::menu: snapshot"
            );

            let dictate_label = if is_recording {
                tr("tray.stop_dictation", lang)
            } else {
                tr("tray.start_dictation", lang)
            };

            // Flat menu structure. ubuntu-appindicators (the GNOME
            // Shell extension that renders the tray on AnduinOS/Ubuntu
            // GNOME sessions) drops `SubMenu` and `CheckmarkItem` entries
            // silently — our `menu()` returned 12 items including submenus
            // and the Shell rendered an empty popup. The workaround that
            // every AppIndicator app uses is to flatten the hierarchy
            // into a single-level list and fake the checkmark with a
            // "✓ " prefix on the active label; that survives the
            // libappindicator compat shim intact.
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
                    label: tr("tray.open_settings", lang),
                    icon_name: "emblem-system-symbolic".into(),
                    activate: Box::new(|_: &mut Self| {
                        with_default_app(|app| app.activate());
                    }),
                    ..Default::default()
                }
                .into(),
            );

            // Post-processing section — header (disabled) + flat list of
            // modes, prefixed by "✓ " on the active one and "   " on the
            // rest so they stay visually aligned.
            items.push(MenuItem::Separator);
            items.push(section_header(tr("tray.post_processing", lang)));
            items.push(
                StandardItem {
                    label: check_label(!post_processing_enabled, &tr("tray.post_processing_off", lang)),
                    activate: Box::new(|_: &mut Self| {
                        mutate_settings(|s| s.post_processing_enabled = false);
                    }),
                    ..Default::default()
                }
                .into(),
            );
            for mode in &modes {
                let mode_id = mode.id.clone();
                let mode_id_for_activate = mode_id.clone();
                let is_active = post_processing_enabled && active_mode_id == mode_id;
                items.push(
                    StandardItem {
                        label: check_label(is_active, &mode.name),
                        activate: Box::new(move |_: &mut Self| {
                            let id = mode_id_for_activate.clone();
                            mutate_settings(move |s| {
                                s.post_processing_enabled = true;
                                s.active_mode_id = id;
                            });
                        }),
                        ..Default::default()
                    }
                    .into(),
                );
            }

            // Transcription model section — same shape.
            items.push(MenuItem::Separator);
            items.push(section_header(tr("tray.transcription_model", lang)));
            let mut any_model = false;
            for preset in ModelPreset::ALL {
                let label_key = preset.label();
                let is_available = downloaded_models
                    .iter()
                    .any(|m| m.preset_label == label_key);
                if !is_available {
                    continue;
                }
                any_model = true;
                let is_active = preset == local_model;
                items.push(
                    StandardItem {
                        label: check_label(is_active, preset.display_label()),
                        activate: Box::new(move |_: &mut Self| {
                            mutate_settings(move |s| s.local_model = preset);
                        }),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            if !any_model {
                items.push(
                    StandardItem {
                        label: format!("   {}", tr("tray.status.ready", lang)),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                );
            }

            // Status line — disabled item showing the bridge's last-status
            // string (or "Ready" as fallback). Matches the macOS menu's
            // `statusItemLine`.
            items.push(MenuItem::Separator);
            let status_line_label = if last_status.is_empty() {
                tr("tray.status.ready", lang)
            } else {
                last_status
            };
            items.push(
                StandardItem {
                    label: status_line_label,
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );

            items.push(MenuItem::Separator);
            items.push(
                StandardItem {
                    label: tr("tray.send_feedback", lang),
                    activate: Box::new(|_: &mut Self| open_external_url(FEEDBACK_URL)),
                    ..Default::default()
                }
                .into(),
            );
            items.push(
                StandardItem {
                    label: tr("tray.check_for_updates", lang),
                    activate: Box::new(|_: &mut Self| open_external_url(RELEASES_URL)),
                    ..Default::default()
                }
                .into(),
            );
            items.push(MenuItem::Separator);
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

    /// Section-header item: disabled, slightly emphasised. We avoid
    /// `SubMenu` entirely because ubuntu-appindicators drops it.
    fn section_header(label: String) -> MenuItem<OpenWhisperTray> {
        StandardItem {
            label,
            enabled: false,
            ..Default::default()
        }
        .into()
    }

    /// "✓ Label" / "   Label" — keeps columns aligned in a monospace-ish
    /// menu. Three trailing spaces after the tick match the three-space
    /// pad on the inactive entries. Used as a flat-menu substitute for
    /// `CheckmarkItem`.
    fn check_label(checked: bool, text: &str) -> String {
        if checked {
            format!("\u{2713}  {}", text)
        } else {
            format!("   {}", text)
        }
    }

    // `Handle<OpenWhisperTray>` is the ksni service handle kept inside the
    // 1 s refresh closure above. No external caller needs it, so we do
    // not re-export it — keeping `OpenWhisperTray` module-private.
    #[allow(dead_code, private_interfaces)]
    type _TrayHandle = Handle<OpenWhisperTray>;
}
