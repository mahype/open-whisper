import AppKit
import SwiftUI

struct HotkeyRecorderField: View {
    let title: String
    let currentHotkey: String
    let isCapturing: Bool
    let preview: String
    let errorText: String?
    let warningText: String?
    let onStartCapture: () -> Void
    let onCommit: (String) -> Void
    let onCancel: () -> Void
    let onClear: () -> Void
    let onPreview: (String) -> Void
    let onInvalid: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(title)
                .font(.subheadline.weight(.medium))

            HStack(spacing: 12) {
                ZStack {
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .fill(Color(nsColor: .textBackgroundColor))
                        .overlay(
                            RoundedRectangle(cornerRadius: 12, style: .continuous)
                                .stroke(isCapturing ? Color.accentColor : Color.secondary.opacity(0.16), lineWidth: isCapturing ? 1.5 : 1)
                        )

                    HStack(spacing: 8) {
                        Image(systemName: isCapturing ? "keyboard.badge.ellipsis" : "command")
                            .foregroundStyle(isCapturing ? Color.accentColor : Color.secondary)
                        Text(displayText)
                            .font(.system(.body, design: .rounded).weight(.medium))
                            .foregroundStyle(displayText == placeholderText ? .secondary : .primary)
                        Spacer(minLength: 0)
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)

                    if isCapturing {
                        HotkeyCaptureHost(
                            onPreview: onPreview,
                            onCommit: onCommit,
                            onCancel: onCancel,
                            onClear: onClear,
                            onInvalid: onInvalid
                        )
                        .frame(width: 0, height: 0)
                    }
                }
                .frame(maxWidth: .infinity, minHeight: 44)

                if isCapturing {
                    Button("Abbrechen", action: onCancel)
                    Button("Loeschen", action: onClear)
                } else {
                    Button("Tastenkombination aufnehmen", action: onStartCapture)
                        .buttonStyle(.borderedProminent)
                }
            }

            if let errorText, !errorText.isEmpty {
                Text(errorText)
                    .font(.caption)
                    .foregroundStyle(.red)
            } else if let warningText, !warningText.isEmpty {
                Text(warningText)
                    .font(.caption)
                    .foregroundStyle(.orange)
            } else {
                Text(isCapturing ? "Einzelne Tasten oder Kombinationen sind moeglich. Escape bricht ab." : "Der neue Hotkey wird erst nach dem Speichern aktiv.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var displayText: String {
        if isCapturing {
            if !preview.isEmpty {
                return hotkeyDisplayString(preview)
            }
            return placeholderText
        }

        if currentHotkey.isEmpty {
            return placeholderText
        }

        return hotkeyDisplayString(currentHotkey)
    }

    private var placeholderText: String {
        isCapturing ? "Jetzt Tastenkombination druecken" : "Noch kein Hotkey gesetzt"
    }
}

private struct HotkeyCaptureHost: NSViewRepresentable {
    let onPreview: (String) -> Void
    let onCommit: (String) -> Void
    let onCancel: () -> Void
    let onClear: () -> Void
    let onInvalid: (String) -> Void

    func makeNSView(context: Context) -> HotkeyCaptureView {
        let view = HotkeyCaptureView()
        view.onPreview = onPreview
        view.onCommit = onCommit
        view.onCancel = onCancel
        view.onClear = onClear
        view.onInvalid = onInvalid
        DispatchQueue.main.async {
            view.window?.makeFirstResponder(view)
        }
        return view
    }

    func updateNSView(_ nsView: HotkeyCaptureView, context: Context) {
        nsView.onPreview = onPreview
        nsView.onCommit = onCommit
        nsView.onCancel = onCancel
        nsView.onClear = onClear
        nsView.onInvalid = onInvalid
        DispatchQueue.main.async {
            if nsView.window?.firstResponder !== nsView {
                nsView.window?.makeFirstResponder(nsView)
            }
        }
    }
}

private final class HotkeyCaptureView: NSView {
    var onPreview: ((String) -> Void)?
    var onCommit: ((String) -> Void)?
    var onCancel: (() -> Void)?
    var onClear: (() -> Void)?
    var onInvalid: ((String) -> Void)?

    override var acceptsFirstResponder: Bool { true }

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        window?.makeFirstResponder(self)
    }

    override func keyDown(with event: NSEvent) {
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)

        if event.keyCode == 53 {
            onCancel?()
            return
        }

        if modifiers.intersection(hotkeyRelevantModifierMask).isEmpty
            && (event.keyCode == 51 || event.keyCode == 117)
        {
            onClear?()
            return
        }

        let modifierTokens = hotkeyModifierTokens(from: modifiers)
        guard let keyToken = hotkeyKeyToken(for: event) else {
            onInvalid?("Diese Taste kann gerade nicht als Hotkey verwendet werden.")
            return
        }

        let hotkey = (modifierTokens + [keyToken]).joined(separator: "+")
        onPreview?(hotkey)
        onCommit?(hotkey)
    }

    override func flagsChanged(with event: NSEvent) {
        let modifiers = hotkeyModifierTokens(from: event.modifierFlags.intersection(.deviceIndependentFlagsMask))
        if modifiers.isEmpty {
            onPreview?("")
        } else {
            onPreview?(modifiers.joined(separator: "+"))
        }
    }
}

private let hotkeyRelevantModifierMask: NSEvent.ModifierFlags = [.command, .control, .option, .shift]

private func hotkeyModifierTokens(from flags: NSEvent.ModifierFlags) -> [String] {
    let relevant = flags.intersection(hotkeyRelevantModifierMask)
    var tokens: [String] = []

    if relevant.contains(.command) {
        tokens.append("Cmd")
    }
    if relevant.contains(.control) {
        tokens.append("Ctrl")
    }
    if relevant.contains(.option) {
        tokens.append("Option")
    }
    if relevant.contains(.shift) {
        tokens.append("Shift")
    }

    return tokens
}

private func hotkeyKeyToken(for event: NSEvent) -> String? {
    switch Int(event.keyCode) {
    case 36, 76:
        return "Enter"
    case 48:
        return "Tab"
    case 49:
        return "Space"
    case 51:
        return "Backspace"
    case 53:
        return "Escape"
    case 117:
        return "Delete"
    case 115:
        return "Home"
    case 119:
        return "End"
    case 116:
        return "PageUp"
    case 121:
        return "PageDown"
    case 123:
        return "Left"
    case 124:
        return "Right"
    case 125:
        return "Down"
    case 126:
        return "Up"
    case 122:
        return "F1"
    case 120:
        return "F2"
    case 99:
        return "F3"
    case 118:
        return "F4"
    case 96:
        return "F5"
    case 97:
        return "F6"
    case 98:
        return "F7"
    case 100:
        return "F8"
    case 101:
        return "F9"
    case 109:
        return "F10"
    case 103:
        return "F11"
    case 111:
        return "F12"
    default:
        break
    }

    guard let character = event.charactersIgnoringModifiers?.trimmingCharacters(in: .whitespacesAndNewlines), character.count == 1 else {
        return nil
    }

    let scalar = character.unicodeScalars.first!
    if CharacterSet.letters.contains(scalar) {
        return character.uppercased()
    }
    if CharacterSet.decimalDigits.contains(scalar) {
        return character
    }

    switch character {
    case "-":
        return "-"
    case "=":
        return "="
    case "[":
        return "["
    case "]":
        return "]"
    case "\\":
        return "\\"
    case ";":
        return ";"
    case "'":
        return "'"
    case ",":
        return ","
    case ".":
        return "."
    case "/":
        return "/"
    case "`":
        return "`"
    default:
        return nil
    }
}

func hotkeyDisplayString(_ hotkey: String) -> String {
    hotkey
        .split(separator: "+")
        .map { token in
            switch token.lowercased() {
            case "cmd", "command", "super":
                return "⌘"
            case "ctrl", "control":
                return "⌃"
            case "option", "alt":
                return "⌥"
            case "shift":
                return "⇧"
            case "space":
                return "Space"
            case "enter":
                return "Enter"
            case "backspace":
                return "Backspace"
            case "escape", "esc":
                return "Esc"
            default:
                return String(token)
            }
        }
        .joined(separator: " ")
}
