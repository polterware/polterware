// swift-tools-version: 5.9
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
            path: "Sources/SparkyNotifier"
        ),
    ]
)
