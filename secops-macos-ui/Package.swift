// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "secops-macos-ui",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "SecOpsUI",
            path: "Sources/SecOpsUI"
        )
    ]
)
