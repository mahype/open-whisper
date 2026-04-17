// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "OpenWhisperMac",
    platforms: [
        .macOS(.v14),
    ],
    products: [
        .executable(name: "OpenWhisperMac", targets: ["OpenWhisperMac"]),
    ],
    targets: [
        .systemLibrary(
            name: "OpenWhisperBridgeFFI",
            path: "Bridge"
        ),
        .executableTarget(
            name: "OpenWhisperMac",
            dependencies: ["OpenWhisperBridgeFFI"],
            path: "Sources/OpenWhisperMac",
            linkerSettings: [
                .unsafeFlags(["-L", "../../target/debug", "-lopen_whisper_bridge"]),
                .linkedLibrary("c++"),
                .linkedFramework("Accelerate"),
                .linkedFramework("AppKit"),
                .linkedFramework("ApplicationServices"),
                .linkedFramework("AudioToolbox"),
                .linkedFramework("Carbon"),
                .linkedFramework("CoreAudio"),
                .linkedFramework("SystemConfiguration"),
            ]
        ),
    ]
)
