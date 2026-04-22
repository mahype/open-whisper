use std::{thread, time::Duration};

use arboard::Clipboard;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use open_whisper_core::{AppSettings, InsertTextMode};

pub fn insert_text_into_active_app(text: &str, settings: &AppSettings) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("No text available to paste.".to_owned());
    }

    tracing::info!(
        chars = text.len(),
        mode = ?settings.insert_text_mode,
        "text_inserter: start"
    );

    // Clipboard-only mode: never drive a paste event, just surface the text
    // via the clipboard. Used when the compositor blocks input simulation
    // (e.g. Wayland without libei/RemoteDesktop portal available).
    if matches!(settings.insert_text_mode, InsertTextMode::ClipboardOnly) {
        copy_to_clipboard(text)?;
        tracing::info!("text_inserter: clipboard-only mode, transcript copied");
        #[cfg(target_os = "linux")]
        notify_transcript_ready(text);
        return Ok("Transcript copied to clipboard. Press Ctrl/Cmd+V to paste.".to_owned());
    }

    // On Wayland `enigo`'s libei backend silently swallows keystrokes when
    // the compositor (notably GNOME Mutter) hasn't issued a RemoteDesktop
    // portal grant — `.key(...)` returns Ok but nothing lands in the
    // focused app. Detection: `XDG_SESSION_TYPE=wayland`. In `Auto` mode we
    // therefore bypass enigo and do clipboard + notification. Users can
    // still force the enigo path by explicitly picking `InsertTextMode::Enigo`
    // (useful on Sway/Hyprland/KDE where libei delivers keys correctly).
    #[cfg(target_os = "linux")]
    if matches!(settings.insert_text_mode, InsertTextMode::Auto) && is_wayland_session() {
        tracing::info!(
            "text_inserter: Wayland session + Auto mode → clipboard fallback with notification"
        );
        copy_to_clipboard(text)?;
        notify_transcript_ready(text);
        return Ok(
            "Wayland detected — transcript copied to clipboard. Press Ctrl+V to paste.".to_owned(),
        );
    }

    // Portal mode is reserved for Linux; until it's wired to `ashpd`, we fall
    // through to enigo. The caller's clipboard-fallback in `finish_transcript`
    // still handles the failure path.

    let mut clipboard =
        Clipboard::new().map_err(|err| format!("Clipboard could not be opened: {err}"))?;
    let previous_text = clipboard.get_text().ok();

    clipboard
        .set_text(text.to_owned())
        .map_err(|err| format!("Clipboard could not be written to: {err}"))?;
    tracing::debug!("text_inserter: clipboard updated with transcript");

    let delay = Duration::from_millis(settings.insert_delay_ms as u64);
    if !delay.is_zero() {
        thread::sleep(delay);
    }

    // `mut` is only used inside the macos-cfg block below; keep the
    // declaration shared so the non-macos branch doesn't warn about
    // unused_mut nor duplicate the initialisation.
    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut enigo_settings = Settings::default();
    #[cfg(target_os = "macos")]
    {
        enigo_settings.open_prompt_to_get_permissions = true;
    }

    let mut enigo = Enigo::new(&enigo_settings).map_err(|err| {
        tracing::warn!(%err, "text_inserter: Enigo::new failed");
        format!("Input simulation could not be initialized: {err}")
    })?;
    tracing::debug!("text_inserter: enigo initialised");

    let modifier = paste_modifier_key();
    enigo.key(modifier, Press).map_err(|err| {
        tracing::warn!(%err, step = "press_modifier", "text_inserter: enigo key failed");
        format!("Paste hotkey could not be pressed: {err}")
    })?;
    enigo.key(Key::Unicode('v'), Click).map_err(|err| {
        tracing::warn!(%err, step = "click_v", "text_inserter: enigo key failed");
        format!("Paste hotkey could not be sent: {err}")
    })?;
    enigo.key(modifier, Release).map_err(|err| {
        tracing::warn!(%err, step = "release_modifier", "text_inserter: enigo key failed");
        format!("Paste hotkey could not be released: {err}")
    })?;
    tracing::info!("text_inserter: paste chord dispatched via enigo");

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

    Ok("Transcript inserted into the active app.".to_owned())
}

#[cfg(target_os = "linux")]
fn is_wayland_session() -> bool {
    matches!(std::env::var("XDG_SESSION_TYPE").as_deref(), Ok("wayland"))
}

/// Send a desktop notification with a preview of the transcript so the
/// user knows it's ready and that they need to paste manually. Fires via
/// `notify-send` (shipped with libnotify-bin on virtually every Linux
/// desktop, preinstalled on GNOME/KDE/Xfce).
#[cfg(target_os = "linux")]
fn notify_transcript_ready(text: &str) {
    const PREVIEW_CHARS: usize = 60;
    let preview: String = text.chars().take(PREVIEW_CHARS).collect();
    let suffix = if text.chars().count() > PREVIEW_CHARS {
        "\u{2026}"
    } else {
        ""
    };
    let body = format!("{preview}{suffix}\n\nPress Ctrl+V to paste.");
    let _ = std::process::Command::new("notify-send")
        .args([
            "--app-name=Open Whisper",
            "--icon=audio-input-microphone",
            "--expire-time=4000",
            "Transcript ready",
            &body,
        ])
        .spawn();
}

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("No text available to copy.".to_owned());
    }

    let mut clipboard =
        Clipboard::new().map_err(|err| format!("Clipboard could not be opened: {err}"))?;
    clipboard
        .set_text(text.to_owned())
        .map_err(|err| format!("Clipboard could not be written to: {err}"))?;
    #[cfg(target_os = "linux")]
    notify_transcript_ready(text);
    Ok(())
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
