import AppKit
import SwiftUI

// MARK: - App delegate

final class ConsentDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    let config: AppConfig
    private var window: NSWindow?
    private var model: ConsentModel?

    init(config: AppConfig) { self.config = config }

    func applicationDidFinishLaunching(_ notification: Notification) {
        let m = ConsentModel(
            adminName: config.adminName,
            sessionId: config.sessionId,
            timeout:   config.timeout
        )
        self.model = m

        // Defer window creation one run-loop cycle. CLI-launched apps sometimes
        // miss makeKeyAndOrderFront if called during the launch burst before the
        // process has fully transitioned to a foreground app.
        DispatchQueue.main.async { [self] in
            let hv  = NSHostingView(rootView: ConsentView(model: m))
            let win = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: 520, height: 420),
                styleMask:   [.titled, .closable],
                backing:     .buffered,
                defer:       false
            )
            win.title                = "Remote Access Request"
            win.contentView          = hv          // NSHostingView directly; no controller
            win.center()
            win.level                = .floating
            win.isReleasedWhenClosed = false
            win.delegate             = self
            NSApp.activate(ignoringOtherApps: true)
            win.makeKeyAndOrderFront(nil)
            win.orderFrontRegardless()             // force-front for CLI-launched processes
            self.window = win
        }
    }

    func windowShouldClose(_ sender: NSWindow) -> Bool {
        model?.decide(accepted: false)
        return false
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
}

// MARK: - Model

final class ConsentModel: ObservableObject {
    let adminName: String
    let sessionId: String
    let total: Double
    @Published var remaining: Double

    private var outputWritten = false
    private var timer: Timer?

    init(adminName: String, sessionId: String, timeout: UInt64) {
        self.adminName = adminName
        self.sessionId = sessionId
        let t = Double(max(timeout, 1))
        self.total     = t
        self.remaining = t

        let t2 = Timer.scheduledTimer(withTimeInterval: 0.15, repeats: true) { [weak self] _ in
            guard let self, !self.outputWritten else { return }
            self.remaining = max(0, self.remaining - 0.15)
            if self.remaining <= 0 { self.decide(accepted: false) }
        }
        RunLoop.main.add(t2, forMode: .common)
        self.timer = t2
    }

    func decide(accepted: Bool) {
        guard !outputWritten else { return }
        outputWritten = true
        timer?.invalidate()

        let result = accepted ? "accepted" : "declined"
        let sid    = sessionId
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        let line = "{\"result\":\"\(result)\",\"session_id\":\"\(sid)\"}\n"
        FileHandle.standardOutput.write(Data(line.utf8))

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { NSApp.terminate(nil) }
    }

    deinit { timer?.invalidate() }
}

// MARK: - View

struct ConsentView: View {
    @ObservedObject var model: ConsentModel

    private var countdownColor: Color {
        model.remaining <= 5 ? .red : model.remaining <= 15 ? .orange : .green
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {

            // ── Hero row ──────────────────────────────────────────────────
            HStack(alignment: .top, spacing: 14) {
                ZStack {
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .fill(Color.accentColor.opacity(0.1))
                        .frame(width: 48, height: 48)
                    Image(systemName: "shield.fill")
                        .font(.system(size: 22))
                        .foregroundColor(.accentColor)
                }
                VStack(alignment: .leading, spacing: 4) {
                    Text("Remote Access Request")
                        .font(.system(size: 18, weight: .semibold))
                    Text("A technician is requesting permission to view and control your screen.")
                        .font(.system(size: 12.5))
                        .foregroundColor(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            .padding(.horizontal, 28)
            .padding(.top, 24)
            .padding(.bottom, 20)

            // ── Detail card ───────────────────────────────────────────────
            VStack(spacing: 0) {
                kvRow("REQUESTED BY") {
                    HStack(spacing: 10) {
                        AvatarView(name: model.adminName, size: 24)
                        Text(model.adminName).font(.system(size: 13, weight: .semibold))
                    }
                }
                Divider()
                kvRow("EXPIRES IN") {
                    VStack(alignment: .leading, spacing: 6) {
                        HStack {
                            Text("\(max(0, Int(ceil(model.remaining))))s")
                                .font(.system(size: 14, weight: .bold).monospacedDigit())
                                .foregroundColor(countdownColor)
                            Spacer()
                            Text("auto-decline at 0s")
                                .font(.system(size: 10.5))
                                .foregroundColor(.secondary.opacity(0.7))
                        }
                        GeometryReader { geo in
                            ZStack(alignment: .leading) {
                                Capsule().fill(Color.secondary.opacity(0.15)).frame(height: 4)
                                Capsule()
                                    .fill(countdownColor)
                                    .frame(
                                        width: max(0, geo.size.width * CGFloat(model.remaining / model.total)),
                                        height: 4
                                    )
                            }
                        }
                        .frame(height: 4)
                    }
                }
                Divider()
                kvRow("PRIVACY") {
                    HStack(spacing: 10) {
                        Image(systemName: "eye")
                            .font(.system(size: 12))
                            .foregroundColor(.secondary)
                        Text("Your screen activity will be visible to the technician")
                            .font(.system(size: 12.5))
                            .foregroundColor(.secondary)
                    }
                }
            }
            .background(Color(NSColor.controlBackgroundColor))
            .cornerRadius(10)
            .overlay(RoundedRectangle(cornerRadius: 10).stroke(Color.secondary.opacity(0.2), lineWidth: 1))
            .padding(.horizontal, 28)

            // ── Action buttons ────────────────────────────────────────────
            HStack {
                Spacer()
                Button("Decline") { model.decide(accepted: false) }
                    .keyboardShortcut(.escape, modifiers: [])
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                Button("Allow Access") { model.decide(accepted: true) }
                    .keyboardShortcut(.return, modifiers: [])
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
            }
            .padding(.horizontal, 28)
            .padding(.top, 20)

            Spacer()

            // ── Footer ────────────────────────────────────────────────────
            HStack(spacing: 5) {
                Image(systemName: "shield")
                    .font(.system(size: 9))
                    .foregroundColor(.secondary.opacity(0.4))
                Text("SecOps Solution")
                    .font(.system(size: 10.5, weight: .semibold))
                    .foregroundColor(.secondary.opacity(0.4))
                Text("·").foregroundColor(.secondary.opacity(0.4))
                Text("Managed by your IT department")
                    .font(.system(size: 10.5))
                    .foregroundColor(.secondary.opacity(0.4))
                Spacer()
            }
            .padding(.horizontal, 28)
            .padding(.bottom, 16)
        }
        .frame(width: 520, height: 420)
        .background(Color(NSColor.windowBackgroundColor))
    }
}

// MARK: - Shared helpers (used in SessionWindow too)

@ViewBuilder
func kvRow<Content: View>(_ label: String, @ViewBuilder content: () -> Content) -> some View {
    HStack(alignment: .center, spacing: 0) {
        Text(label)
            .font(.system(size: 10.5, weight: .semibold))
            .foregroundColor(.secondary)
            .frame(width: 110, alignment: .leading)
        content()
        Spacer()
    }
    .padding(.horizontal, 16)
    .padding(.vertical, 10)
}

struct AvatarView: View {
    let name: String
    let size: CGFloat
    var body: some View {
        ZStack {
            Circle().fill(Color.accentColor).frame(width: size, height: size)
            Text(String(name.prefix(1)).uppercased())
                .font(.system(size: size * 0.45, weight: .bold))
                .foregroundColor(.white)
        }
    }
}
