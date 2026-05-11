import SwiftUI

struct PermissionRow: View {
    let symbol: String
    let label: String

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: symbol)
                .font(.system(size: 15, weight: .medium))
                .foregroundColor(.accentColor)
                .frame(width: 22, height: 22)

            Text(label)
                .font(.system(size: 13.5))
                .foregroundColor(.primary)

            Spacer()
        }
    }
}
