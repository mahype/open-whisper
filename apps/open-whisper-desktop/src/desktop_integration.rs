use std::time::Duration;

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};
use open_whisper_core::AppSettings;
use tray_icon::{
    MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, SubmenuBuilder},
};

const MENU_ID_SHOW_SETTINGS: &str = "show-settings";
const MENU_ID_QUIT: &str = "quit";

pub enum DesktopAction {
    ShowSettings,
    HideWindow,
    Quit,
    HotkeyPressed,
    HotkeyReleased,
}

pub struct DesktopIntegration {
    tray: Option<TrayController>,
    hotkey: Option<HotKeyController>,
    initialization_status: Option<String>,
}

impl DesktopIntegration {
    pub fn new() -> Self {
        Self {
            tray: None,
            hotkey: None,
            initialization_status: None,
        }
    }

    pub fn ensure_ready(&mut self, settings: &AppSettings) -> Vec<String> {
        let mut messages = Vec::new();

        if self.tray.is_none() {
            match TrayController::new() {
                Ok(tray) => {
                    self.tray = Some(tray);
                    messages.push("Tray wurde initialisiert.".to_owned());
                }
                Err(err) => {
                    let message = format!("Tray konnte nicht initialisiert werden: {err}");
                    if self.initialization_status.as_deref() != Some(message.as_str()) {
                        self.initialization_status = Some(message.clone());
                        messages.push(message);
                    }
                }
            }
        }

        if self.hotkey.is_none() {
            match HotKeyController::new() {
                Ok(mut hotkey) => match hotkey.apply_settings(settings) {
                    Ok(Some(message)) => {
                        self.hotkey = Some(hotkey);
                        messages.push(message);
                    }
                    Ok(None) => {
                        self.hotkey = Some(hotkey);
                    }
                    Err(err) => {
                        messages.push(err);
                        self.hotkey = Some(hotkey);
                    }
                },
                Err(err) => {
                    let message =
                        format!("Globaler Hotkey konnte nicht initialisiert werden: {err}");
                    if self.initialization_status.as_deref() != Some(message.as_str()) {
                        self.initialization_status = Some(message.clone());
                        messages.push(message);
                    }
                }
            }
        }

        messages
    }

    pub fn sync(&mut self, settings: &AppSettings, window_visible: bool) -> Vec<String> {
        let mut messages = self.ensure_ready(settings);

        if let Some(hotkey) = &mut self.hotkey {
            match hotkey.apply_settings(settings) {
                Ok(Some(message)) => messages.push(message),
                Ok(None) => {}
                Err(err) => messages.push(err),
            }
        }

        if let Some(tray) = &mut self.tray {
            tray.sync(window_visible);
        }

        messages
    }

    pub fn poll_actions(&mut self) -> Vec<DesktopAction> {
        let mut actions = Vec::new();

        if let Some(tray) = &mut self.tray {
            actions.extend(tray.poll_actions());
        }

        if let Some(hotkey) = &mut self.hotkey {
            actions.extend(hotkey.poll_actions());
        }

        actions
    }

    pub fn can_hide_to_tray(&self) -> bool {
        self.tray.is_some()
    }

    pub fn summary(&self) -> String {
        let tray_status = if self.tray.is_some() {
            "Tray aktiv"
        } else {
            "Tray noch nicht aktiv"
        };

        let hotkey_status = self
            .hotkey
            .as_ref()
            .and_then(HotKeyController::summary)
            .unwrap_or("Hotkey noch nicht aktiv".to_owned());

        format!("{tray_status}, {hotkey_status}")
    }
}

struct HotKeyController {
    manager: GlobalHotKeyManager,
    registered_hotkey: Option<HotKey>,
    registered_text: Option<String>,
}

impl HotKeyController {
    fn new() -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new().map_err(|err| err.to_string())?;
        Ok(Self {
            manager,
            registered_hotkey: None,
            registered_text: None,
        })
    }

    fn apply_settings(&mut self, settings: &AppSettings) -> Result<Option<String>, String> {
        if self.registered_text.as_deref() == Some(settings.hotkey.as_str()) {
            return Ok(None);
        }

        if let Some(old) = self.registered_hotkey.take() {
            self.manager
                .unregister(old)
                .map_err(|err| format!("Vorherigen Hotkey konnte ich nicht abmelden: {err}"))?;
        }

        let parsed: HotKey = settings
            .hotkey
            .parse()
            .map_err(|err| format!("Hotkey '{}' ist ungueltig: {err}", settings.hotkey))?;

        self.manager.register(parsed).map_err(|err| {
            format!(
                "Hotkey '{}' konnte nicht registriert werden: {err}",
                settings.hotkey
            )
        })?;

        self.registered_hotkey = Some(parsed);
        self.registered_text = Some(settings.hotkey.clone());

        Ok(Some(format!(
            "Globaler Hotkey aktiv: {}",
            self.registered_text.as_deref().unwrap_or_default()
        )))
    }

    fn poll_actions(&mut self) -> Vec<DesktopAction> {
        let mut actions = Vec::new();

        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if self
                .registered_hotkey
                .as_ref()
                .is_some_and(|registered| registered.id() == event.id)
            {
                match event.state {
                    HotKeyState::Pressed => actions.push(DesktopAction::HotkeyPressed),
                    HotKeyState::Released => actions.push(DesktopAction::HotkeyReleased),
                }
            }
        }

        actions
    }

    fn summary(&self) -> Option<String> {
        self.registered_text
            .as_ref()
            .map(|hotkey| format!("Hotkey aktiv auf {hotkey}"))
    }
}

struct TrayController {
    _tray_icon: TrayIcon,
    show_settings_item: MenuItem,
}

impl TrayController {
    fn new() -> Result<Self, String> {
        let show_settings_item =
            MenuItem::with_id(MENU_ID_SHOW_SETTINGS, "Fenster anzeigen", true, None);
        let quit_item = MenuItem::with_id(MENU_ID_QUIT, "Beenden", true, None);
        let separator = PredefinedMenuItem::separator();

        let app_menu = SubmenuBuilder::new()
            .text("Open Whisper")
            .items(&[&show_settings_item, &separator, &quit_item])
            .build()
            .map_err(|err| err.to_string())?;

        let tray_menu = Menu::new();
        tray_menu.append(&app_menu).map_err(|err| err.to_string())?;

        let mut builder = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Open Whisper")
            .with_icon(create_tray_icon()?)
            .with_menu_on_left_click(false);

        #[cfg(target_os = "macos")]
        {
            builder = builder.with_icon_as_template(true);
        }

        let tray_icon = builder.build().map_err(|err| err.to_string())?;

        Ok(Self {
            _tray_icon: tray_icon,
            show_settings_item,
        })
    }

    fn sync(&mut self, window_visible: bool) {
        let text = if window_visible {
            "Fenster ausblenden"
        } else {
            "Fenster anzeigen"
        };
        self.show_settings_item.set_text(text);
    }

    fn poll_actions(&mut self) -> Vec<DesktopAction> {
        let mut actions = Vec::new();

        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == MENU_ID_SHOW_SETTINGS {
                if self.show_settings_item.text().contains("anzeigen") {
                    actions.push(DesktopAction::ShowSettings);
                } else {
                    actions.push(DesktopAction::HideWindow);
                }
            } else if event.id == MENU_ID_QUIT {
                actions.push(DesktopAction::Quit);
            }
        }

        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Down,
                    ..
                } => actions.push(DesktopAction::ShowSettings),
                _ => {}
            }
        }

        actions
    }
}

fn create_tray_icon() -> Result<tray_icon::Icon, String> {
    const SIZE: u32 = 16;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let alpha = if y < 3 || y > 12 || x < 2 || x > 13 {
                0
            } else if (3..=12).contains(&y) && (2..=13).contains(&x) {
                255
            } else {
                0
            };

            let cutout = (x == 5 && y == 12) || (x == 6 && y == 13) || (x == 7 && y == 14);
            let fill = if cutout { 0 } else { alpha };

            rgba.extend_from_slice(&[24, 24, 24, fill]);
        }
    }

    tray_icon::Icon::from_rgba(rgba, SIZE, SIZE).map_err(|err| err.to_string())
}

pub const POLL_INTERVAL: Duration = Duration::from_millis(150);
