import SwiftUI
import AppKit

struct ConsentView: View {
    let args: Args
    let onDecide: (String) -> Void   // called with "accepted" or "declined"

    @StateObject private var countdown: CountdownState
    @State private var decided = false

    init(args: Args, onDecide: @escaping (String) -> Void) {
        self.args     = args
        self.onDecide = onDecide
        self._countdown = StateObject(
            wrappedValue: CountdownState(seconds: args.timeout)
        )
    }

    // MARK: – Timer colour

    private var timerColor: Color {
        if countdown.remaining <= 5  { return .red }
        if countdown.remaining <= 15 { return .orange }
        return .accentColor
    }

    // MARK: – Body

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {

            // ── Header ────────────────────────────────────────────────
            HStack(alignment: .top, spacing: 16) {
                CircularTimerView(
                    fraction:  countdown.fraction,
                    remaining: countdown.remaining,
                    color:     timerColor
                )
                .padding(.top, 2)

                VStack(alignment: .leading, spacing: 5) {
                    Text("Remote Access Request")
                        .font(.system(size: 18, weight: .semibold))

                    Text("A technician is requesting permission to view and control your screen.")
                        .font(.system(size: 12.5))
                        .foregroundColor(.secondary)

                    Text("Approve only if you initiated this request.")
                        .font(.system(size: 12))
                        .foregroundColor(.secondary)
                }
            }
            .padding(.bottom, 18)

            // ── Admin identity card ───────────────────────────────────
            HStack(spacing: 12) {
                // Avatar circle with initial
                ZStack {
                    Circle()
                        .fill(Color.accentColor)
                        .frame(width: 40, height: 40)
                    Text(args.adminName.prefix(1).uppercased())
                        .font(.system(size: 17, weight: .semibold))
                        .foregroundColor(.white)
                }

                VStack(alignment: .leading, spacing: 3) {
                    Text(args.adminName)
                        .font(.system(size: 14, weight: .semibold))
                    if !args.adminEmail.isEmpty {
                        Text(args.adminEmail)
                            .font(.system(size: 12))
                            .foregroundColor(.secondary)
                    }
                }

                Spacer()
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 12)
            .frame(maxWidth: .infinity)
            .background(Color.primary.opacity(0.05))
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .padding(.bottom, 18)

            // ── Permissions ───────────────────────────────────────────
            Text("PERMISSIONS REQUESTED")
                .font(.system(size: 11, weight: .semibold))
                .foregroundColor(.secondary)
                .padding(.bottom, 10)

            VStack(alignment: .leading, spacing: 9) {
                PermissionRow(symbol: "eye",
                              label:  "View your screen")
                PermissionRow(symbol: "computermouse",
                              label:  "Control mouse & keyboard")
                PermissionRow(symbol: "arrow.up.arrow.down",
                              label:  "Can download/upload files from/to the system")
            }
            .padding(.bottom, 18)

            // ── Encryption note ───────────────────────────────────────
            Label {
                Text("Session is end-to-end encrypted and audit-logged.")
                    .font(.system(size: 12.5))
            } icon: {
                Image(systemName: "lock.fill")
                    .font(.system(size: 13))
            }
            .foregroundColor(.accentColor)
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.accentColor.opacity(0.08))
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .padding(.bottom, 22)

            // ── Action buttons ────────────────────────────────────────
            HStack(spacing: 10) {
                Spacer()

                Button("Decline") { decide("declined") }
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                    .keyboardShortcut(.escape, modifiers: [])

                Button("Allow Access") { decide("accepted") }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
                    .keyboardShortcut(.return, modifiers: [])
            }
            .padding(.bottom, 14)

            // ── Footer ────────────────────────────────────────────────
            HStack(spacing: 4) {
                Image(systemName: "shield")
                    .font(.system(size: 10))
                Text("SecOps Solution")
                    .fontWeight(.semibold)
                Text("·")
                Text("Managed by your IT department")
            }
            .font(.system(size: 10))
            .foregroundColor(Color(nsColor: .tertiaryLabelColor))
        }
        .padding(24)
        .frame(width: 440)
        // Auto-decline when timer hits zero
        .onChange(of: countdown.remaining, perform: { r in
            if r == 0 { decide("declined") }
        })
    }

    // MARK: – Decision

    private func decide(_ result: String) {
        guard !decided else { return }
        decided = true
        countdown.stop()
        onDecide(result)
    }
}
