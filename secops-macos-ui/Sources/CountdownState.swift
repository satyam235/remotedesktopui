import Foundation
import Combine

final class CountdownState: ObservableObject {
    @Published private(set) var remaining: Int
    let total: Int
    private var cancellable: AnyCancellable?

    init(seconds: Int) {
        let s = max(1, seconds)
        remaining = s
        total     = s
        cancellable = Timer
            .publish(every: 1, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                guard let self, self.remaining > 0 else { return }
                self.remaining -= 1
            }
    }

    var fraction: Double { Double(remaining) / Double(total) }

    func stop() {
        cancellable?.cancel()
        cancellable = nil
    }

    deinit { stop() }
}
