import AppKit
import SwiftUI

// MARK: - App delegate

final class ConsentDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    let config: AppConfig
    private var panel: NSPanel?
    private var model: ConsentModel?

    init(config: AppConfig) { self.config = config }

    func applicationDidFinishLaunching(_ notification: Notification) {
        let m = ConsentModel(
            adminName:  config.adminName,
            adminEmail: config.adminEmail,
            sessionId:  config.sessionId,
            timeout:    config.timeout
        )
        self.model = m

        DispatchQueue.main.async { [self] in
            let hv = NSHostingView(rootView: ConsentView(model: m))

            let panel = NSPanel(
                contentRect: NSRect(x: 0, y: 0, width: 460, height: 448),
                styleMask:   [.titled, .closable],
                backing:     .buffered,
                defer:       false
            )
            panel.title                = "Remote Access Request"
            panel.contentView          = hv
            panel.center()
            panel.level                = .floating
            panel.isReleasedWhenClosed = false
            panel.hidesOnDeactivate    = false
            panel.collectionBehavior   = [.canJoinAllSpaces, .fullScreenAuxiliary]
            panel.delegate             = self
            panel.makeKeyAndOrderFront(nil)
            panel.orderFrontRegardless()
            self.panel = panel
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
    let adminName:  String
    let adminEmail: String
    let sessionId:  String
    let total:      Double
    @Published var remaining: Double

    private var outputWritten = false
    private var timer: Timer?

    init(adminName: String, adminEmail: String, sessionId: String, timeout: UInt64) {
        self.adminName  = adminName
        self.adminEmail = adminEmail
        self.sessionId  = sessionId
        let t           = Double(max(timeout, 1))
        self.total      = t
        self.remaining  = t

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

// MARK: - Circular timer

private struct CircularTimerView: View {
    let fraction:  Double
    let remaining: Int
    let color:     Color

    var body: some View {
        ZStack {
            Circle()
                .stroke(Color.primary.opacity(0.10), lineWidth: 5)
            Circle()
                .trim(from: 0, to: max(0.01, fraction))
                .stroke(color, style: StrokeStyle(lineWidth: 5, lineCap: .round))
                .rotationEffect(.degrees(-90))
                .animation(.linear(duration: 0.9), value: fraction)
            Text("\(remaining)")
                .font(.system(size: 18, weight: .semibold, design: .rounded))
                .foregroundColor(color)
                .monospacedDigit()
        }
        .frame(width: 58, height: 58)
    }
}

// MARK: - Permission row

private struct PermissionRowView: View {
    let systemName: String
    let text:       String

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: systemName)
                .font(.system(size: 14))
                .foregroundColor(.accentColor)
                .frame(width: 20)
            Text(text)
                .font(.system(size: 13.5))
        }
    }
}

// MARK: - Consent view

struct ConsentView: View {
    @ObservedObject var model: ConsentModel

    private var countdownColor: Color {
        model.remaining <= 5 ? .red : model.remaining <= 15 ? .orange : .accentColor
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {

            // ── Header: circular timer + title/subtitles ──────────────────
            HStack(alignment: .top, spacing: 14) {
                CircularTimerView(
                    fraction:  model.remaining / model.total,
                    remaining: max(0, Int(ceil(model.remaining))),
                    color:     countdownColor
                )
                .padding(.top, 2)

                VStack(alignment: .leading, spacing: 5) {
                    Text("Remote Access Request")
                        .font(.system(size: 18, weight: .semibold))
                        .foregroundColor(Color(nsColor: .labelColor))
                    Text("A technician is requesting permission to view and control your screen.")
                        .font(.system(size: 12.5))
                        .foregroundColor(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                    Text("Approve only if you initiated this request.")
                        .font(.system(size: 12))
                        .foregroundColor(Color(nsColor: .tertiaryLabelColor))
                }
            }
            .padding(.bottom, 18)

            // ── Admin identity card ───────────────────────────────────────
            HStack(spacing: 12) {
                AvatarView(name: model.adminName, size: 40)

                VStack(alignment: .leading, spacing: 3) {
                    Text(model.adminName)
                        .font(.system(size: 14.5, weight: .semibold))
                    if !model.adminEmail.isEmpty {
                        Text(model.adminEmail)
                            .font(.system(size: 12))
                            .foregroundColor(.secondary)
                    }
                }
                Spacer()
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 12)
            .frame(maxWidth: .infinity)
            .background(Color(nsColor: .controlBackgroundColor))
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: 10).stroke(Color.secondary.opacity(0.2), lineWidth: 1))
            .padding(.bottom, 18)

            // ── Permissions ───────────────────────────────────────────────
            Text("PERMISSIONS REQUESTED")
                .font(.system(size: 11, weight: .semibold))
                .foregroundColor(.secondary)
                .padding(.bottom, 10)

            VStack(alignment: .leading, spacing: 9) {
                PermissionRowView(systemName: "eye",                   text: "View your screen")
                PermissionRowView(systemName: "computermouse",         text: "Control mouse & keyboard")
                PermissionRowView(systemName: "arrow.up.arrow.down",   text: "Can download/upload files from/to the system")
            }
            .padding(.bottom, 18)

            // ── Encryption note ───────────────────────────────────────────
            HStack(spacing: 8) {
                Image(systemName: "lock.fill")
                    .font(.system(size: 13))
                    .foregroundColor(.accentColor)
                Text("Session is end-to-end encrypted and audit-logged.")
                    .font(.system(size: 12.5))
                    .foregroundColor(.accentColor)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.accentColor.opacity(0.08))
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .padding(.bottom, 22)

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
            .padding(.bottom, 14)

            // ── Footer ────────────────────────────────────────────────────
            HStack(spacing: 4) {
                Image(systemName: "shield")
                    .font(.system(size: 10))
                Text("SecOps Solution").fontWeight(.semibold)
                Text("·")
                Text("Managed by your IT department")
            }
            .font(.system(size: 10.5))
            .foregroundColor(Color(nsColor: .tertiaryLabelColor))
        }
        .padding(24)
        .frame(width: 460, height: 448)
        .background(Color(nsColor: .windowBackgroundColor))
    }
}

// MARK: - Avatar (shared with SessionWindow)

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
