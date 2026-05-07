import AppKit
import SwiftUI

// MARK: - App delegate

final class SessionDelegate: NSObject, NSApplicationDelegate {
    let config: AppConfig
    private var panel: NSPanel?

    init(config: AppConfig) { self.config = config }

    func applicationDidFinishLaunching(_ notification: Notification) {
        let model = SessionModel(
            adminName:  config.adminName,
            sessionId:  config.sessionId,
            chatSocket: config.chatSocket
        )
        let vc = NSHostingController(rootView: SessionView(model: model))
        vc.view.wantsLayer          = true
        vc.view.layer?.isOpaque     = false
        vc.view.layer?.backgroundColor = .clear

        let panel = NSPanel(
            contentRect: NSRect(x: 0, y: 0, width: 360, height: 580),
            styleMask:   [.borderless, .nonactivatingPanel],
            backing:     .buffered,
            defer:       false
        )
        panel.level                    = .floating
        panel.isOpaque                 = false
        panel.backgroundColor          = .clear
        panel.hasShadow                = false  // shadow rendered in SwiftUI
        panel.isMovableByWindowBackground = true
        panel.collectionBehavior       = [.canJoinAllSpaces, .fullScreenAuxiliary]
        panel.contentViewController    = vc
        panel.isReleasedWhenClosed     = false

        // Position top-right of the primary screen's visible area
        if let screen = NSScreen.main {
            let vf = screen.visibleFrame
            let x  = vf.origin.x + vf.width  - 360 - 24
            let y  = vf.origin.y + vf.height  - 580 - 24
            panel.setFrameOrigin(NSPoint(x: x, y: y))
        }

        panel.orderFrontRegardless()
        self.panel = panel
    }
}

// MARK: - Model

struct ChatLine: Identifiable {
    let id   = UUID()
    let from: String
    let text: String
    let ts:   UInt64
}

enum ConnectionStatus { case connecting, connected, disconnected }

final class SessionModel: ObservableObject {
    let adminName: String
    let sessionId: String

    @Published var history:          [ChatLine]        = []
    @Published var inputText:        String            = ""
    @Published var connectionStatus: ConnectionStatus  = .connecting
    @Published var sessionSeconds:   Int               = 0

    private let client:     ChatClient
    private var ticker:     Timer?
    private let sessionStart = Date()
    private var closing      = false

    init(adminName: String, sessionId: String, chatSocket: String) {
        self.adminName = adminName
        self.sessionId = sessionId
        self.client    = ChatClient()

        client.onMessage = { [weak self] msg in
            self?.handle(msg)
        }
        client.onConnectionChange = { [weak self] in
            self?.refreshStatus()
        }

        if !chatSocket.isEmpty { client.connect(to: chatSocket) }

        let t = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            guard let self else { return }
            self.sessionSeconds = Int(Date().timeIntervalSince(self.sessionStart))
            self.refreshStatus()
        }
        RunLoop.main.add(t, forMode: .common)
        self.ticker = t
    }

    private func handle(_ msg: ChatClient.Message) {
        switch msg.kind {
        case "chat":
            let from = msg.from.isEmpty ? "admin" : msg.from
            history.append(ChatLine(from: from, text: msg.text,
                                    ts: msg.ts > 0 ? msg.ts : currentUnixMs()))
        case "session_end":
            DispatchQueue.main.asyncAfter(deadline: .now() + 3) { NSApp.terminate(nil) }
        default: break
        }
    }

    private func refreshStatus() {
        if client.hasFailed        { connectionStatus = .disconnected }
        else if client.isConnected { connectionStatus = .connected    }
        else                       { connectionStatus = .connecting   }
    }

    func send() {
        let text = inputText.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }
        let ts = currentUnixMs()
        client.send(ChatClient.Message(kind: "chat", session_id: sessionId,
                                       from: "user", text: text, ts: ts))
        history.append(ChatLine(from: "user", text: text, ts: ts))
        inputText = ""
    }

    func disconnect() {
        guard !closing else { return }
        closing = true
        client.send(ChatClient.Message(kind: "disconnect", session_id: sessionId,
                                       from: "user", ts: currentUnixMs()))
        DispatchQueue.main.asyncAfter(deadline: .now() + 1) { NSApp.terminate(nil) }
    }

    var durationString: String {
        let s = sessionSeconds
        if s >= 3600 { return String(format: "%d:%02d:%02d", s/3600, (s%3600)/60, s%60) }
        return String(format: "%02d:%02d", s/60, s%60)
    }

    deinit { ticker?.invalidate() }
}

// MARK: - Root view

struct SessionView: View {
    @ObservedObject var model: SessionModel
    @FocusState private var inputFocused: Bool
    @State private var pulse = false

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            chatArea
            Divider()
            composer
            statusStrip
        }
        .frame(width: 360)
        .background(Color(NSColor.windowBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .shadow(color: .black.opacity(0.18), radius: 24, x: 0, y: 8)
        .background(Color.clear)
        .onAppear { inputFocused = true; pulse = true }
    }

    // ── Header ──────────────────────────────────────────────────────────────

    private var header: some View {
        HStack(spacing: 8) {
            // Pulsing live dot
            ZStack {
                Circle()
                    .fill(Color.red.opacity(0.25))
                    .frame(width: 14, height: 14)
                    .scaleEffect(pulse ? 1.4 : 1.0)
                    .opacity(pulse ? 0.0 : 1.0)
                    .animation(.easeOut(duration: 1.2).repeatForever(autoreverses: false), value: pulse)
                Circle().fill(Color.red).frame(width: 8, height: 8)
            }

            VStack(alignment: .leading, spacing: 0) {
                Text("LIVE SESSION")
                    .font(.system(size: 9, weight: .semibold))
                    .foregroundColor(.secondary)
                Text("Screen sharing in progress")
                    .font(.system(size: 11.5))
                    .foregroundColor(.secondary)
            }

            Spacer()

            Text(model.adminName)
                .font(.system(size: 12, weight: .semibold))
                .foregroundColor(.secondary)
            AvatarView(name: model.adminName, size: 22)

            Button(action: { model.disconnect() }) {
                Image(systemName: "xmark")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundColor(.secondary)
                    .frame(width: 26, height: 26)
                    .background(Color.secondary.opacity(0.1))
                    .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(Color(NSColor.windowBackgroundColor))
    }

    // ── Chat scroll area ────────────────────────────────────────────────────

    private var chatArea: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 6) {
                    if model.history.isEmpty {
                        emptyState
                    } else {
                        ForEach(Array(model.history.enumerated()), id: \.element.id) { idx, line in
                            let prevFrom = idx > 0 ? model.history[idx - 1].from : ""
                            BubbleView(line: line,
                                       showSender: line.from.lowercased() != prevFrom.lowercased(),
                                       adminName: model.adminName)
                                .id(line.id)
                        }
                    }
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 12)
            }
            .background(Color(NSColor.controlBackgroundColor).opacity(0.6))
            .onChange(of: model.history.count) { _ in
                if let last = model.history.last {
                    withAnimation(.easeOut(duration: 0.2)) {
                        proxy.scrollTo(last.id, anchor: .bottom)
                    }
                }
            }
        }
    }

    // ── Composer ─────────────────────────────────────────────────────────────

    private var composer: some View {
        HStack(spacing: 8) {
            TextField("Type a message…", text: $model.inputText)
                .textFieldStyle(.roundedBorder)
                .focused($inputFocused)
                .onSubmit { model.send() }

            let hasText = !model.inputText.trimmingCharacters(in: .whitespaces).isEmpty
            Button(action: { model.send() }) {
                Image(systemName: "paperplane.fill")
                    .font(.system(size: 14))
                    .foregroundColor(hasText ? .white : .secondary)
                    .frame(width: 36, height: 36)
                    .background(
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .fill(hasText ? Color.accentColor : Color.secondary.opacity(0.1))
                    )
            }
            .buttonStyle(.plain)
            .disabled(!hasText)
        }
        .padding(.horizontal, 14)
        .padding(.top, 10)
        .padding(.bottom, 6)
    }

    // ── Status strip ─────────────────────────────────────────────────────────

    private var statusStrip: some View {
        HStack(spacing: 6) {
            Circle().fill(statusColor).frame(width: 6, height: 6)
            Text(statusLabel)
                .font(.system(size: 11, weight: .semibold))
                .foregroundColor(.secondary)
            Spacer()
            Text("Duration").font(.system(size: 10.5)).foregroundColor(.secondary.opacity(0.6))
            Text(model.durationString)
                .font(.system(size: 11, weight: .bold).monospacedDigit())
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
    }

    private var statusColor: Color {
        switch model.connectionStatus {
        case .connecting:    return .orange
        case .connected:     return .green
        case .disconnected:  return .red
        }
    }

    private var statusLabel: String {
        switch model.connectionStatus {
        case .connecting:   return "Connecting…"
        case .connected:    return "Connected"
        case .disconnected: return "Disconnected"
        }
    }

    // ── Empty state ───────────────────────────────────────────────────────────

    private var emptyState: some View {
        VStack(spacing: 10) {
            ZStack {
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(Color.accentColor.opacity(0.1))
                    .frame(width: 48, height: 48)
                Image(systemName: "eye")
                    .font(.system(size: 22))
                    .foregroundColor(.accentColor)
            }
            Text("No messages yet")
                .font(.system(size: 13.5, weight: .semibold))
                .foregroundColor(.secondary)
            Text("Use chat to coordinate with your IT technician.")
                .font(.system(size: 12))
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding(.top, 40)
    }
}

// MARK: - Chat bubble

struct BubbleView: View {
    let line:       ChatLine
    let showSender: Bool
    let adminName:  String

    private var isUser: Bool { line.from.lowercased() == "user" }

    private var displayName: String {
        switch line.from.lowercased() {
        case "admin", "": return adminName
        default:          return line.from
        }
    }

    private var timeLabel: String {
        guard line.ts > 0 else { return "" }
        let date = Date(timeIntervalSince1970: TimeInterval(line.ts) / 1000.0)
        let fmt  = DateFormatter()
        fmt.dateFormat = "HH:mm"
        return fmt.string(from: date)
    }

    var body: some View {
        HStack {
            if isUser { Spacer(minLength: 56) }
            VStack(alignment: isUser ? .trailing : .leading, spacing: 2) {
                if showSender && !isUser {
                    Text(displayName)
                        .font(.system(size: 10.5, weight: .semibold))
                        .foregroundColor(.secondary)
                }
                Text(line.text)
                    .font(.system(size: 13))
                    .foregroundColor(isUser ? Color.accentColor : .primary)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 7)
                    .background(
                        RoundedRectangle(cornerRadius: 14, style: .continuous)
                            .fill(isUser
                                  ? Color.accentColor.opacity(0.12)
                                  : Color(NSColor.controlBackgroundColor))
                            .overlay(
                                RoundedRectangle(cornerRadius: 14)
                                    .stroke(isUser
                                            ? Color.accentColor.opacity(0.3)
                                            : Color.secondary.opacity(0.2),
                                            lineWidth: 1)
                            )
                    )
                if !timeLabel.isEmpty {
                    Text(timeLabel)
                        .font(.system(size: 9.5))
                        .foregroundColor(.secondary.opacity(0.6))
                }
            }
            if !isUser { Spacer(minLength: 56) }
        }
    }
}
