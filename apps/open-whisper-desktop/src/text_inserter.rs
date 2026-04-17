use std::{thread, time::Duration};

use arboard::Clipboard;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use open_whisper_core::AppSettings;

pub fn insert_text_into_active_app(text: &str, settings: &AppSettings) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Kein Text zum Einfuegen vorhanden.".to_owned());
    }

    let mut clipboard = Clipboard::new()
        .map_err(|err| format!("Clipboard konnte nicht geoeffnet werden: {err}"))?;
    let previous_text = clipboard.get_text().ok();

    clipboard
        .set_text(text.to_owned())
        .map_err(|err| format!("Clipboard konnte nicht beschrieben werden: {err}"))?;

    let delay = Duration::from_millis(settings.insert_delay_ms as u64);
    if !delay.is_zero() {
        thread::sleep(delay);
    }

    let mut enigo_settings = Settings::default();
    #[cfg(target_os = "macos")]
    {
        enigo_settings.open_prompt_to_get_permissions = true;
    }

    let mut enigo = Enigo::new(&enigo_settings)
        .map_err(|err| format!("Input-Simulation konnte nicht initialisiert werden: {err}"))?;

    let modifier = paste_modifier_key();
    enigo
        .key(modifier, Press)
        .map_err(|err| format!("Paste-Hotkey konnte nicht gedrueckt werden: {err}"))?;
    enigo
        .key(Key::Unicode('v'), Click)
        .map_err(|err| format!("Paste-Hotkey konnte nicht gesendet werden: {err}"))?;
    enigo
        .key(modifier, Release)
        .map_err(|err| format!("Paste-Hotkey konnte nicht losgelassen werden: {err}"))?;

    if settings.restore_clipboard_after_insert
        && let Some(previous_text) = previous_text
    {
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(300));
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(previous_text);
            }
        });
    }

    Ok("Transkript in die aktive App eingefuegt.".to_owned())
}

#[cfg(target_os = "macos")]
fn paste_modifier_key() -> Key {
    Key::Meta
}

#[cfg(not(target_os = "macos"))]
fn paste_modifier_key() -> Key {
    Key::Control
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_uses_command_for_paste() {
        assert_eq!(paste_modifier_key(), Key::Meta);
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_uses_control_for_paste() {
        assert_eq!(paste_modifier_key(), Key::Control);
    }
}
