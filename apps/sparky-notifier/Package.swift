// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "SparkyNotifier",
    platforms: [.macOS(.v15)],
    products: [
        .executable(name: "SparkyNotifier", targets: ["SparkyNotifier"]),
    ],
    targets: [
        .executableTarget(
            name: "SparkyNotifier",
            path: "Sources/SparkyNotifier",
            swiftSettings: [.swiftLanguageMode(.v5)]
        ),
    ]
)
