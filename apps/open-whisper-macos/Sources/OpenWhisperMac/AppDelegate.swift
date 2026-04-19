import AppKit
import SwiftUI

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate, NSWindowDelegate {
    let model = AppModel()

    private var statusItem: NSStatusItem!
    private let statusMenu = NSMenu()
    private var dictationItem: NSMenuItem!
    private var settingsItem: NSMenuItem!
    private var onboardingItem: NSMenuItem!
    private var modeSummaryItem: NSMenuItem!
    private var modeSwitchItem: NSMenuItem!
    private var modelItem: NSMenuItem!
    private var statusItemLine: NSMenuItem!
    private var quitItem: NSMenuItem!
    private var settingsWindow: NSWindow?
    private var onboardingWindow: NSWindow?
    private var recordingIndicatorWindow: NSWindow?
    private let modeMenu = NSMenu()

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem.button?.imagePosition = .imageOnly
        statusItem.button?.toolTip = "Open Whisper"

        dictationItem = NSMenuItem(title: "Diktat starten", action: #selector(toggleDictation), keyEquivalent: "")
        settingsItem = NSMenuItem(title: "Einstellungen...", action: #selector(showSettings), keyEquivalent: ",")
        onboardingItem = NSMenuItem(title: "Onboarding erneut oeffnen", action: #selector(showOnboarding), keyEquivalent: "")
        modeSummaryItem = NSMenuItem(title: "Modus wird geladen...", action: nil, keyEquivalent: "")
        modeSummaryItem.isEnabled = false
        modeSwitchItem = NSMenuItem(title: "Modus wechseln", action: nil, keyEquivalent: "")
        modeSwitchItem.submenu = modeMenu
        modelItem = NSMenuItem(title: "Modellstatus wird geladen...", action: nil, keyEquivalent: "")
        modelItem.isEnabled = false
        statusItemLine = NSMenuItem(title: "Status wird geladen...", action: nil, keyEquivalent: "")
        statusItemLine.isEnabled = false
        quitItem = NSMenuItem(title: "Beenden", action: #selector(quitApp), keyEquivalent: "q")

        statusMenu.delegate = self
        statusMenu.items = [
            dictationItem,
            .separator(),
            settingsItem,
            onboardingItem,
            .separator(),
            modeSummaryItem,
            modeSwitchItem,
            .separator(),
            modelItem,
            statusItemLine,
            .separator(),
            quitItem,
        ]
        statusItem.menu = statusMenu

        model.onStateChanged = { [weak self] in
            self?.refreshMenuState()
        }
        refreshMenuState()

        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.model.refreshDiagnostics()
            if !self.model.runtime.onboardingCompleted {
                self.showOnboarding(nil)
            }
        }
    }

    func menuWillOpen(_ menu: NSMenu) {
        refreshMenuState()
    }

    @objc private func toggleDictation() {
        model.toggleDictation()
    }

    @objc private func showSettings(_ sender: Any?) {
        let window = settingsWindow ?? makeWindow(
            title: "Open Whisper Einstellungen",
            size: NSSize(width: 820, height: 720),
            rootView: SettingsView(model: model)
        )
        if settingsWindow == nil {
            window.delegate = self
        }
        settingsWindow = window
        show(window)
    }

    func windowWillClose(_ notification: Notification) {
        guard let window = notification.object as? NSWindow, window === settingsWindow else {
            return
        }
        model.flushAutoSave()
    }

    @objc private func showOnboarding(_ sender: Any?) {
        model.reopenOnboarding()
        let window = onboardingWindow ?? makeWindow(
            title: "Open Whisper Setup",
            size: NSSize(width: 760, height: 520),
            rootView: OnboardingView(model: model) { [weak self] in
                self?.onboardingWindow?.orderOut(nil)
            }
        )
        onboardingWindow = window
        show(window)
    }

    @objc private func quitApp() {
        NSApp.terminate(nil)
    }

    @objc private func selectMode(_ sender: NSMenuItem) {
        guard let modeID = sender.representedObject as? String else {
            return
        }
        model.persistActiveModeImmediately(modeID)
    }

    private func refreshMenuState() {
        let runtime = model.runtime
        dictationItem.title = runtime.isRecording ? "Diktat stoppen" : "Diktat starten"
        modeSummaryItem.title = model.trayModeLabel
        rebuildModeMenu()
        modelItem.title = model.trayModelLabel
        statusItemLine.title = model.bridgeError ?? runtime.lastStatus
        statusItem.button?.image = statusImage(recording: runtime.isRecording)
        statusItem.button?.toolTip = model.bridgeError ?? runtime.lastStatus
        updateRecordingIndicatorVisibility()
    }

    private func updateRecordingIndicatorVisibility() {
        let runtime = model.runtime
        let phase: IndicatorPhase? = {
            if runtime.dictationBlockedByMissingModel {
                let progress = runtime.blockedModelProgressBasisPoints.map { Double($0) / 10_000.0 }
                return .modelNotReady(
                    label: runtime.blockedModelLabel,
                    progress: progress,
                    isDownloading: runtime.blockedModelIsDownloading
                )
            }
            if runtime.isRecording { return .recording }
            if runtime.isTranscribing { return .transcribing }
            if runtime.isPostProcessing { return .postProcessing }
            return nil
        }()

        guard model.settings.showRecordingIndicator, let phase else {
            recordingIndicatorWindow?.orderOut(nil)
            return
        }

        let style = model.settings.waveformStyle
        let color = model.settings.waveformColor
        let modeName = model.activeModeName
        let window = recordingIndicatorWindow ?? makeRecordingIndicatorWindow(phase: phase, style: style, color: color, modeName: modeName)
        recordingIndicatorWindow = window
        updateIndicatorPhase(window: window, phase: phase, style: style, color: color, modeName: modeName)
        positionRecordingIndicatorWindow(window)
        window.orderFrontRegardless()
    }

    private func updateIndicatorPhase(window: NSWindow, phase: IndicatorPhase, style: WaveformStyle, color: WaveformColor, modeName: String) {
        guard let hosting = window.contentViewController as? NSHostingController<RecordingIndicatorView> else {
            return
        }
        if hosting.rootView.phase != phase
            || hosting.rootView.style != style
            || hosting.rootView.color != color
            || hosting.rootView.modeName != modeName {
            hosting.rootView = RecordingIndicatorView(phase: phase, style: style, color: color, modeName: modeName)
        }
    }

    private func makeRecordingIndicatorWindow(phase: IndicatorPhase, style: WaveformStyle, color: WaveformColor, modeName: String) -> NSWindow {
        let size = NSSize(width: 260, height: 86)
        let panel = NSPanel(
            contentRect: NSRect(origin: .zero, size: size),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        panel.isFloatingPanel = true
        panel.becomesKeyOnlyIfNeeded = true
        panel.level = .floating
        panel.backgroundColor = .clear
        panel.isOpaque = false
        panel.hasShadow = true
        panel.ignoresMouseEvents = true
        panel.hidesOnDeactivate = false
        panel.collectionBehavior = [.canJoinAllSpaces, .stationary, .fullScreenAuxiliary]
        panel.isReleasedWhenClosed = false

        let hosting = NSHostingController(rootView: RecordingIndicatorView(phase: phase, style: style, color: color, modeName: modeName))
        hosting.view.frame = NSRect(origin: .zero, size: size)
        panel.contentViewController = hosting
        return panel
    }

    private func positionRecordingIndicatorWindow(_ window: NSWindow) {
        guard let screenFrame = NSScreen.main?.visibleFrame else { return }
        let margin: CGFloat = 16
        let size = window.frame.size
        let origin = NSPoint(
            x: screenFrame.midX - size.width / 2,
            y: screenFrame.maxY - size.height - margin
        )
        window.setFrameOrigin(origin)
    }

    private func rebuildModeMenu() {
        modeMenu.removeAllItems()

        for mode in model.persistedModes {
            let item = NSMenuItem(
                title: mode.name,
                action: #selector(selectMode(_:)),
                keyEquivalent: ""
            )
            item.target = self
            item.representedObject = mode.id
            item.state = model.persistedActiveModeID == mode.id ? .on : .off
            modeMenu.addItem(item)
        }
    }

    private func show(_ window: NSWindow) {
        NSApp.activate(ignoringOtherApps: true)
        window.makeKeyAndOrderFront(nil)
    }

    private func makeWindow<Content: View>(title: String, size: NSSize, rootView: Content) -> NSWindow {
        let window = NSWindow(
            contentRect: NSRect(origin: .zero, size: size),
            styleMask: [.titled, .closable, .miniaturizable],
            backing: .buffered,
            defer: false
        )
        window.title = title
        window.center()
        window.isReleasedWhenClosed = false
        window.contentViewController = NSHostingController(rootView: rootView)
        return window
    }

    private func statusImage(recording: Bool) -> NSImage? {
        let symbolName = recording ? "mic.circle.fill" : "waveform.circle"
        let image = NSImage(systemSymbolName: symbolName, accessibilityDescription: "Open Whisper")
        image?.isTemplate = true
        return image
    }
}
