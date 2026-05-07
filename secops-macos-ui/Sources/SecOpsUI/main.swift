import AppKit

struct AppConfig {
    var mode       = ""
    var adminName  = "IT Support"
    var timeout: UInt64 = 30
    var sessionId  = ""
    var chatSocket = ""
}

private func parseArgs() -> AppConfig {
    var c = AppConfig()
    for arg in CommandLine.arguments.dropFirst() {
        if      let v = arg.strippingPrefix("--mode=")        { c.mode = v }
        else if let v = arg.strippingPrefix("--admin-name=")  { c.adminName = v }
        else if let v = arg.strippingPrefix("--timeout="),
                let n = UInt64(v)                             { c.timeout = n }
        else if let v = arg.strippingPrefix("--session-id=")  { c.sessionId = v }
        else if let v = arg.strippingPrefix("--chat-socket=") { c.chatSocket = v }
    }
    let env = ProcessInfo.processInfo.environment
    if c.mode.isEmpty       { c.mode       = env["SECOPS_MODE"]        ?? "" }
    if c.sessionId.isEmpty  { c.sessionId  = env["SECOPS_SESSION_ID"]  ?? "" }
    if c.chatSocket.isEmpty { c.chatSocket = env["SECOPS_CHAT_SOCKET"] ?? "" }
    if let v = env["SECOPS_ADMIN_NAME"], !v.isEmpty { c.adminName = v }
    if c.timeout == 0 { c.timeout = 30 }
    return c
}

private extension String {
    func strippingPrefix(_ prefix: String) -> String? {
        hasPrefix(prefix) ? String(dropFirst(prefix.count)) : nil
    }
}

let config = parseArgs()
let app    = NSApplication.shared

switch config.mode {
case "consent":
    app.setActivationPolicy(.accessory)
    let d = ConsentDelegate(config: config)
    app.delegate = d
    app.run()
case "session":
    app.setActivationPolicy(.accessory)
    let d = SessionDelegate(config: config)
    app.delegate = d
    app.run()
default:
    fputs("""
    secops-macos-ui — end-user GUI for SecOps remote desktop

    USAGE:
      secops-macos-ui --mode=consent  [--admin-name=<n>] [--timeout=<s>] [--session-id=<id>]
      secops-macos-ui --mode=session  [--admin-name=<n>] [--session-id=<id>] [--chat-socket=<host:port>]

    consent  prints {\"result\":\"accepted|declined\",\"session_id\":\"<id>\"} to stdout then exits.
    session  opens an always-on-top chat overlay connected to --chat-socket.

    """, stderr)
    exit(1)
}
