// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "SparkyHost",
    platforms: [.macOS(.v15)],
    products: [
        .executable(name: "SparkyHost", targets: ["SparkyHost"]),
    ],
    targets: [
        .executableTarget(
            name: "SparkyHost",
            path: "Sources/SparkyHost",
            swiftSettings: [.swiftLanguageMode(.v5)]
        ),
    ]
)
