// Newline-delimited JSON chat protocol over a plain TCP socket. The agent
// owns the listener; this binary connects out as a client. Reader and writer
// run on dedicated threads so the egui UI thread is never blocked on I/O.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Maps to the JSON `"type"` field. Known values: "chat", "session_end",
    /// "disconnect". Any other value is ignored by the receiver.
    #[serde(rename = "type", default)]
    pub kind: String,

    #[serde(default)]
    pub session_id: String,

    #[serde(default)]
    pub from: String,

    #[serde(default)]
    pub text: String,

    #[serde(default)]
    pub ts: u64,
}

pub struct ChatHandle {
    rx: Receiver<Message>,
    tx: Sender<Message>,
    connected: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
}

impl ChatHandle {
    /// Drain all currently-buffered incoming messages without blocking.
    /// The caller (UI thread) calls this once per frame.
    pub fn poll(&self) -> Vec<Message> {
        let mut out = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(m) => out.push(m),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        out
    }

    /// Queue a message for the writer thread. Drops silently if the
    /// connection died — the UI surfaces that via `failed()` / `connected()`.
    pub fn send(&self, m: Message) {
        let _ = self.tx.send(m);
    }

    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub fn failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }
}

/// Spawn the chat I/O threads and return a handle. If `addr` is empty, no
/// connection is attempted and `send` becomes a silent no-op — useful for
/// running the UI standalone (e.g. screenshot tests, demos).
pub fn start(addr: String) -> ChatHandle {
    let (in_tx, in_rx) = channel::<Message>();
    let (out_tx, out_rx) = channel::<Message>();
    let connected = Arc::new(AtomicBool::new(false));
    let failed = Arc::new(AtomicBool::new(false));

    if addr.is_empty() {
        return ChatHandle { rx: in_rx, tx: out_tx, connected, failed };
    }

    let connected_c = connected.clone();
    let failed_c = failed.clone();

    thread::spawn(move || {
        // Resolve the socket address. Strings like "localhost:9765" need DNS
        // lookup, which `to_socket_addrs` does for us.
        let sock_addr = match addr.to_socket_addrs() {
            Ok(mut iter) => match iter.next() {
                Some(a) => a,
                None => {
                    failed_c.store(true, Ordering::Relaxed);
                    return;
                }
            },
            Err(_) => {
                failed_c.store(true, Ordering::Relaxed);
                return;
            }
        };

        // Retry the connect a few times — the agent may not have its listener
        // up yet by the time the GUI launches.
        let mut stream_opt: Option<TcpStream> = None;
        for _ in 0..10 {
            match TcpStream::connect_timeout(&sock_addr, Duration::from_secs(2)) {
                Ok(s) => {
                    stream_opt = Some(s);
                    break;
                }
                Err(_) => thread::sleep(Duration::from_millis(500)),
            }
        }

        let stream = match stream_opt {
            Some(s) => s,
            None => {
                failed_c.store(true, Ordering::Relaxed);
                return;
            }
        };
        connected_c.store(true, Ordering::Relaxed);

        // Disable Nagle to keep typing-latency low and clone for the reader.
        let _ = stream.set_nodelay(true);
        let reader_stream = match stream.try_clone() {
            Ok(s) => s,
            Err(_) => {
                failed_c.store(true, Ordering::Relaxed);
                return;
            }
        };

        // ── Reader thread: blocks on the socket, parses one JSON object per
        // line, forwards to the UI through the in_tx channel.
        let in_tx_c = in_tx.clone();
        let connected_r = connected_c.clone();
        thread::spawn(move || {
            let reader = BufReader::new(reader_stream);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        let trimmed = l.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(m) = serde_json::from_str::<Message>(trimmed) {
                            if in_tx_c.send(m).is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            connected_r.store(false, Ordering::Relaxed);
        });

        // ── Writer (this thread): blocks on the outgoing channel and writes
        // each queued message as a newline-terminated JSON line.
        let mut writer = stream;
        while let Ok(msg) = out_rx.recv() {
            let json = match serde_json::to_string(&msg) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if writeln!(writer, "{}", json).is_err() {
                break;
            }
            if writer.flush().is_err() {
                break;
            }
        }
        connected_c.store(false, Ordering::Relaxed);
    });

    ChatHandle { rx: in_rx, tx: out_tx, connected, failed }
}
