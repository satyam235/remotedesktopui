// Suppress the console window on Windows release builds. Debug builds keep
// the console attached so logs and panics are visible during development.
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod chat;
mod consent;
mod icons;
mod session;
mod theme;
mod timer;

#[derive(Debug, Default)]
struct Args {
    mode: String,
    admin_name: String,
    timeout: u64,
    session_id: String,
    chat_socket: String,
}

fn parse_args() -> Args {
    let mut a = Args {
        mode: String::new(),
        admin_name: "IT Support".to_string(),
        timeout: 30,
        session_id: String::new(),
        chat_socket: String::new(),
    };

    for raw in std::env::args().skip(1) {
        if let Some(v) = raw.strip_prefix("--mode=") {
            a.mode = v.to_string();
        } else if let Some(v) = raw.strip_prefix("--admin-name=") {
            a.admin_name = v.to_string();
        } else if let Some(v) = raw.strip_prefix("--timeout=") {
            a.timeout = v.parse().unwrap_or(30);
        } else if let Some(v) = raw.strip_prefix("--session-id=") {
            a.session_id = v.to_string();
        } else if let Some(v) = raw.strip_prefix("--chat-socket=") {
            a.chat_socket = v.to_string();
        }
    }

    // Environment fall-backs (allow launching without CLI args, e.g. via systemd/launchd).
    if a.mode.is_empty() {
        if let Ok(v) = std::env::var("SECOPS_MODE") {
            a.mode = v;
        }
    }
    if a.session_id.is_empty() {
        if let Ok(v) = std::env::var("SECOPS_SESSION_ID") {
            a.session_id = v;
        }
    }
    if a.chat_socket.is_empty() {
        if let Ok(v) = std::env::var("SECOPS_CHAT_SOCKET") {
            a.chat_socket = v;
        }
    }
    if std::env::var("SECOPS_ADMIN_NAME").is_ok() {
        if let Ok(v) = std::env::var("SECOPS_ADMIN_NAME") {
            if !v.is_empty() {
                a.admin_name = v;
            }
        }
    }

    if a.timeout == 0 {
        a.timeout = 30;
    }
    a
}

fn print_usage() {
    eprintln!(
        "secops-endpoint-ui — end-user GUI for SecOps remote desktop\n\
         \n\
         USAGE:\n\
            secops-endpoint-ui --mode=consent  [--admin-name=<n>] [--timeout=<s>] [--session-id=<id>]\n\
            secops-endpoint-ui --mode=session  [--admin-name=<n>]                  [--session-id=<id>] [--chat-socket=<host:port>]\n\
         \n\
         consent mode emits one JSON line on stdout: {{\"result\":\"accepted|declined\",\"session_id\":\"<id>\"}}\n\
         session mode opens a chat overlay and exchanges newline-delimited JSON with --chat-socket.\n"
    );
}

fn main() -> Result<(), eframe::Error> {
    // On Linux without DISPLAY/WAYLAND_DISPLAY (e.g. SSH-only headless box) the
    // GUI can't be shown — tell the agent and exit so it can auto-allow / skip.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
            println!(r#"{{"error":"no_display"}}"#);
            std::process::exit(2);
        }
    }

    let args = parse_args();

    match args.mode.as_str() {
        "consent" => consent::run(args.admin_name, args.session_id, args.timeout),
        "session" => session::run(args.admin_name, args.session_id, args.chat_socket),
        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}
