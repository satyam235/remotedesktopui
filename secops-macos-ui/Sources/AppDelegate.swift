import AppKit
import SwiftUI

final class AppDelegate: NSObject, NSApplicationDelegate {
    let args: Args
    private var window: NSWindow?
    private var outputWritten = false

    init(args: Args) {
        self.args = args
    }

    // MARK: – Launch

    func applicationDidFinishLaunching(_ notification: Notification) {
        let view = ConsentView(args: args) { [weak self] result in
            self?.writeOutput(result: result)
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) {
                NSApp.terminate(nil)
            }
        }

        let hosting = NSHostingView(rootView: view)

        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 440, height: 100),
            styleMask:   [.titled, .closable],
            backing:     .buffered,
            defer:       false
        )
        guard let window else { return }

        window.title               = "Remote Access Request"
        window.contentView         = hosting
        window.isReleasedWhenClosed = false
        window.level               = .floating      // always on top
        window.isMovable           = true
        window.center()
        window.makeKeyAndOrderFront(nil)

        if #available(macOS 14, *) {
            NSApp.activate()
        } else {
            NSApp.activate(ignoringOtherApps: true)
        }
    }

    // Close via red-X — treat as declined
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    // Last-resort fallback: if the process is killed or the window closed
    // without the view's onDecide firing, still emit a valid JSON line.
    func applicationWillTerminate(_ notification: Notification) {
        writeOutput(result: "declined")
    }

    // MARK: – Output

    func writeOutput(result: String) {
        guard !outputWritten else { return }
        outputWritten = true
        let sid = args.escapedSessionId()
        print("{\"result\":\"\(result)\",\"session_id\":\"\(sid)\"}")
        fflush(stdout)
    }
}
