import AppKit
import Foundation
import ServiceManagement

/// Spike parent app. Registers the nested SparkyNotifier as an SMAppService agent
/// and as a login item. The helper bundle must already be inside this app.
@main
enum SparkyHostMain {
    static func main() {
        let app = NSApplication.shared
        let delegate = HostDelegate()
        app.delegate = delegate
        app.setActivationPolicy(.accessory)
        app.run()
    }
}

final class HostDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        Task.detached {
            let code = run()
            exit(code)
        }
    }
}

private func run() -> Int32 {
    let args = Array(CommandLine.arguments.dropFirst())
    let command = args.first ?? "--status"
    switch command {
    case "--status":
        emit(snapshot())
        return 0
    case "--register-agent":
        return register(SMAppService.agent(plistName: "com.polterware.sparky.notifier.plist"), label: "agent")
    case "--unregister-agent":
        return unregister(SMAppService.agent(plistName: "com.polterware.sparky.notifier.plist"), label: "agent")
    case "--register-login-item":
        return register(SMAppService.loginItem(identifier: "com.polterware.sparky.notifier"), label: "login-item")
    case "--unregister-login-item":
        return unregister(SMAppService.loginItem(identifier: "com.polterware.sparky.notifier"), label: "login-item")
    default:
        fputs("unknown command \(command)\n", stderr)
        return 2
    }
}

private func snapshot() -> String {
    let payload: [String: String] = [
        "agent": statusName(SMAppService.agent(plistName: "com.polterware.sparky.notifier.plist").status),
        "loginItem": statusName(SMAppService.loginItem(identifier: "com.polterware.sparky.notifier").status),
        "bundleId": Bundle.main.bundleIdentifier ?? "",
    ]
    guard let data = try? JSONSerialization.data(withJSONObject: payload),
          let line = String(data: data, encoding: .utf8) else {
        return "{}"
    }
    return line
}

private func statusName(_ status: SMAppService.Status) -> String {
    switch status {
    case .notRegistered:
        return "not-registered"
    case .enabled:
        return "enabled"
    case .requiresApproval:
        return "requires-approval"
    case .notFound:
        return "not-found"
    @unknown default:
        return "unknown"
    }
}

private func register(_ service: SMAppService, label: String) -> Int32 {
    do {
        try service.register()
        emit("\(label)-register ok \(snapshot())")
        return 0
    } catch {
        let nsError = error as NSError
        emit("\(label)-register error domain=\(nsError.domain) code=\(nsError.code) \(nsError.localizedDescription) \(snapshot())")
        return 5
    }
}

private func unregister(_ service: SMAppService, label: String) -> Int32 {
    do {
        try service.unregister()
        emit("\(label)-unregister ok \(snapshot())")
        return 0
    } catch {
        let nsError = error as NSError
        emit("\(label)-unregister error domain=\(nsError.domain) code=\(nsError.code) \(nsError.localizedDescription) \(snapshot())")
        return 5
    }
}

private func emit(_ line: String) {
    print(line)
    fflush(stdout)
}
