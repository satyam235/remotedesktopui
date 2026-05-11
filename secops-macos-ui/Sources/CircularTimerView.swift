import SwiftUI

struct CircularTimerView: View {
    let fraction: Double   // 1.0 = full, drains to 0
    let remaining: Int
    let color: Color

    var body: some View {
        ZStack {
            // Track
            Circle()
                .stroke(Color.primary.opacity(0.10), lineWidth: 5)

            // Draining arc — starts at 12 o'clock, clockwise
            Circle()
                .trim(from: 0, to: max(0.01, fraction))
                .stroke(color, style: StrokeStyle(lineWidth: 5, lineCap: .round))
                .rotationEffect(.degrees(-90))
                .animation(.linear(duration: 0.9), value: fraction)

            // Countdown number
            Text("\(remaining)")
                .font(.system(size: 18, weight: .semibold, design: .rounded))
                .foregroundColor(color)
                .monospacedDigit()
        }
        .frame(width: 58, height: 58)
    }
}
