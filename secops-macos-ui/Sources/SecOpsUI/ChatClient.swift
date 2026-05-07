import Foundation
import Network

// Newline-delimited JSON over TCP. Connects as a client; the agent owns the
// listener. All NWConnection callbacks run on ioQueue; UI callbacks are
// dispatched to the main queue before delivery.

final class ChatClient {

    struct Message: Codable {
        var kind: String       = ""
        var session_id: String = ""
        var from: String       = ""
        var text: String       = ""
        var ts: UInt64         = 0

        enum CodingKeys: String, CodingKey {
            case kind = "type"
            case session_id, from, text, ts
        }

        init(kind: String, session_id: String = "", from: String = "",
             text: String = "", ts: UInt64 = 0) {
            self.kind       = kind
            self.session_id = session_id
            self.from       = from
            self.text       = text
            self.ts         = ts
        }

        init(from decoder: Decoder) throws {
            let c    = try decoder.container(keyedBy: CodingKeys.self)
            kind       = (try? c.decode(String.self, forKey: .kind))       ?? ""
            session_id = (try? c.decode(String.self, forKey: .session_id)) ?? ""
            from       = (try? c.decode(String.self, forKey: .from))       ?? ""
            text       = (try? c.decode(String.self, forKey: .text))       ?? ""
            ts         = (try? c.decode(UInt64.self, forKey: .ts))         ?? 0
        }
    }

    // State is written on ioQueue then dispatched to main before callbacks fire,
    // so callers that read these from the main thread see consistent values.
    private(set) var isConnected = false
    private(set) var hasFailed   = false

    var onMessage:          ((Message) -> Void)?
    var onConnectionChange: (() -> Void)?

    private var connection:    NWConnection?
    private var receiveBuffer: Data = Data()
    private let ioQueue = DispatchQueue(label: "secops.chat.io")

    private var retryCount = 0
    private var pendingHost: NWEndpoint.Host?
    private var pendingPort: NWEndpoint.Port?

    func connect(to address: String) {
        let parts = address.split(separator: ":", maxSplits: 1)
        guard parts.count == 2,
              let portNum = UInt16(parts[1]),
              let nwPort  = NWEndpoint.Port(rawValue: portNum) else {
            hasFailed = true
            DispatchQueue.main.async { self.onConnectionChange?() }
            return
        }
        let host = NWEndpoint.Host(String(parts[0]))
        pendingHost = host
        pendingPort = nwPort
        createConnection(host: host, port: nwPort)
    }

    private func createConnection(host: NWEndpoint.Host, port: NWEndpoint.Port) {
        let conn = NWConnection(host: host, port: port, using: .tcp)
        self.connection = conn

        conn.stateUpdateHandler = { [weak self] state in
            // Already on ioQueue (passed to conn.start below)
            guard let self else { return }
            switch state {
            case .ready:
                self.retryCount = 0
                DispatchQueue.main.async {
                    self.isConnected = true
                    self.hasFailed   = false
                    self.onConnectionChange?()
                }
                self.startReceiving()

            case .failed:
                if self.retryCount < 10 {
                    self.retryCount += 1
                    self.ioQueue.asyncAfter(deadline: .now() + 0.5) { [weak self] in
                        guard let self,
                              let h = self.pendingHost,
                              let p = self.pendingPort else { return }
                        self.createConnection(host: h, port: p)
                    }
                } else {
                    DispatchQueue.main.async {
                        self.isConnected = false
                        self.hasFailed   = true
                        self.onConnectionChange?()
                    }
                }

            case .cancelled:
                DispatchQueue.main.async {
                    self.isConnected = false
                    self.onConnectionChange?()
                }

            default: break
            }
        }

        conn.start(queue: ioQueue)
    }

    private func startReceiving() {
        connection?.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isComplete, error in
            guard let self else { return }
            if let data, !data.isEmpty {
                self.receiveBuffer.append(data)
                self.processBuffer()
            }
            if error != nil || isComplete {
                DispatchQueue.main.async {
                    self.isConnected = false
                    self.onConnectionChange?()
                }
                return
            }
            self.startReceiving()
        }
    }

    private func processBuffer() {
        while let nl = receiveBuffer.firstIndex(of: UInt8(ascii: "\n")) {
            let slice = Data(receiveBuffer[..<nl])
            receiveBuffer = Data(receiveBuffer[(nl + 1)...])
            guard !slice.isEmpty,
                  let msg = try? JSONDecoder().decode(Message.self, from: slice) else { continue }
            DispatchQueue.main.async { [weak self] in
                self?.onMessage?(msg)
            }
        }
    }

    func send(_ message: Message) {
        guard let encoded = try? JSONEncoder().encode(message),
              var line = String(data: encoded, encoding: .utf8) else { return }
        line += "\n"
        connection?.send(content: Data(line.utf8), completion: .idempotent)
    }

    func disconnect() { connection?.cancel() }

    deinit { connection?.cancel() }
}

func currentUnixMs() -> UInt64 {
    UInt64(Date().timeIntervalSince1970 * 1000)
}
