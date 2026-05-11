import Foundation

struct Args {
    var mode: String      = ""
    var adminName: String = "IT Support"
    var adminEmail: String = ""
    var sessionId: String = ""
    var timeout: Int      = 30

    static func parse() -> Args {
        var a = Args()
        let env = ProcessInfo.processInfo.environment

        for arg in CommandLine.arguments.dropFirst() {
            if      let v = arg.dropping(prefix: "--mode=")        { a.mode       = v }
            else if let v = arg.dropping(prefix: "--admin-name=")  { a.adminName  = v }
            else if let v = arg.dropping(prefix: "--admin-email=") { a.adminEmail = v }
            else if let v = arg.dropping(prefix: "--session-id=")  { a.sessionId  = v }
            else if let v = arg.dropping(prefix: "--timeout="), let n = Int(v), n > 0 {
                a.timeout = n
            }
        }

        // Environment fallbacks (same vars as the Rust/Go agent uses)
        if a.mode.isEmpty,      let v = env["SECOPS_MODE"],         !v.isEmpty { a.mode       = v }
        if let v = env["SECOPS_ADMIN_NAME"],  !v.isEmpty { a.adminName  = v }
        if let v = env["SECOPS_ADMIN_EMAIL"], !v.isEmpty { a.adminEmail = v }
        if let v = env["SECOPS_SESSION_ID"],  !v.isEmpty { a.sessionId  = v }
        if let v = env["SECOPS_TIMEOUT"], let n = Int(v), n > 0    { a.timeout    = n }

        return a
    }

    /// Escape special characters for embedding in a JSON string value.
    func escapedSessionId() -> String {
        sessionId
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
    }
}

private extension String {
    func dropping(prefix: String) -> String? {
        hasPrefix(prefix) ? String(dropFirst(prefix.count)) : nil
    }
}
