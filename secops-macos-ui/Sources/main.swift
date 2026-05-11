import AppKit

let args = Args.parse()

// Validate mode — only "consent" is implemented; session overlay is handled
// by the in-session chat UI which the agent spawns separately.
guard args.mode == "consent" else {
    fputs(
        """
        secops-macos-ui — end-user GUI for SecOps remote desktop (macOS)

        USAGE:
            secops-macos-ui --mode=consent [--admin-name=<n>] [--admin-email=<e>] \
        [--timeout=<s>] [--session-id=<id>]

        consent mode emits one JSON line on stdout:
            {"result":"accepted|declined","session_id":"<id>"}

        ENV VARS (fallbacks):
            SECOPS_MODE, SECOPS_ADMIN_NAME, SECOPS_ADMIN_EMAIL,
            SECOPS_SESSION_ID, SECOPS_TIMEOUT

        """,
        stderr
    )
    exit(1)
}

let app      = NSApplication.shared
app.setActivationPolicy(.regular)

let delegate = AppDelegate(args: args)
app.delegate = delegate

app.run()
