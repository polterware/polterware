import AppKit
import Foundation
import UserNotifications

// C ABI: async commands enqueue JSON replies; JS owns/frees each polled string.
// Polling avoids retaining a JS callback across Swift/FFI shutdown.
private let lock = NSLock()
private var messages = [[String: Any]]()
private var delegate: NotifyDelegate?
private func emit(_ value: [String: Any]) {
    lock.lock(); defer { lock.unlock() }
    messages.append(value)
}
private func owned(_ id: String) -> Bool { id.hasPrefix("rem:") || id.hasPrefix("one:") }
private func rows(_ requests: [UNNotificationRequest]) -> [[String: Any]] {
    requests.map { ["id": $0.identifier, "title": $0.content.title, "body": $0.content.body,
                   "hash": $0.content.userInfo["hash"] as? String ?? "", "repeats": $0.trigger?.repeats ?? false,
                   "trigger": $0.trigger.map { String(describing: type(of: $0)) } ?? ""] }
}
private func auth(_ state: UNAuthorizationStatus) -> String {
    switch state {
    case .authorized: return "authorized"
    case .denied: return "denied"
    case .notDetermined: return "notDetermined"
    case .provisional: return "provisional"
    default: return "unknown"
    }
}
private final class NotifyDelegate: NSObject, UNUserNotificationCenterDelegate {
    var completions = [String: () -> Void]()
    var observers = [NSObjectProtocol]()
    override init() {
        super.init()
        observers.append(NSWorkspace.shared.notificationCenter.addObserver(forName: NSWorkspace.didWakeNotification, object: nil, queue: nil) { _ in
            emit(["event": "refresh", "reason": "wake"])
        })
        observers.append(NotificationCenter.default.addObserver(forName: .NSSystemTimeZoneDidChange, object: nil, queue: nil) { _ in
            emit(["event": "refresh", "reason": "timezone"])
        })
    }
    func userNotificationCenter(_ center: UNUserNotificationCenter, willPresent notification: UNNotification, withCompletionHandler completion: @escaping (UNNotificationPresentationOptions) -> Void) {
        emit(["event": "refresh", "reason": "delivered", "id": notification.request.identifier])
        completion([.banner, .list, .sound])
    }
    func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse, withCompletionHandler completion: @escaping () -> Void) {
        let token = UUID().uuidString
        lock.lock(); completions[token] = completion; lock.unlock()
        let info = response.notification.request.content.userInfo
        emit(["event": "action", "token": token, "action": response.actionIdentifier,
              "id": response.notification.request.identifier, "memoryId": info["memoryId"] as? String ?? "",
              "occurrenceAt": info["occurrenceAt"] as? String ?? "", "receivedAtMs": Date().timeIntervalSince1970 * 1000])
    }
}
@_cdecl("notify_start")
public func start() {
    delegate = NotifyDelegate()
    let center = UNUserNotificationCenter.current()
    center.delegate = delegate
    let done = UNNotificationAction(identifier: "DONE", title: "Done", options: [])
    let snooze = UNNotificationAction(identifier: "SNOOZE", title: "Snooze", options: [])
    let open = UNNotificationAction(identifier: "OPEN", title: "Open", options: [.foreground])
    let focus = UNNotificationAction(identifier: "START_FOCUS", title: "Start focus", options: [.foreground])
    center.setNotificationCategories([
        UNNotificationCategory(identifier: "reminder", actions: [done, snooze, open], intentIdentifiers: []),
        UNNotificationCategory(identifier: "reminder-focus", actions: [done, snooze, focus, open], intentIdentifiers: []),
        UNNotificationCategory(identifier: "focus-phase", actions: [], intentIdentifiers: [])
    ])
}
@_cdecl("notify_poll")
public func poll() -> UnsafeMutablePointer<CChar>? {
    lock.lock()
    let batch = messages; messages.removeAll()
    lock.unlock()
    guard !batch.isEmpty, let data = try? JSONSerialization.data(withJSONObject: batch), let text = String(data: data, encoding: .utf8) else { return nil }
    return strdup(text)
}
@_cdecl("notify_free")
public func release(_ ptr: UnsafeMutableRawPointer?) { free(ptr) }
@_cdecl("notify_send")
public func send(_ ridPtr: UnsafePointer<CChar>, _ methodPtr: UnsafePointer<CChar>, _ inputPtr: UnsafePointer<CChar>) {
    let rid = String(cString: ridPtr), method = String(cString: methodPtr)
    let raw = Data(String(cString: inputPtr).utf8)
    let input = try? JSONSerialization.jsonObject(with: raw, options: [.fragmentsAllowed])
    Task {
        do { emit(["rid": rid, "value": try await command(method, input)]) }
        catch { emit(["rid": rid, "error": error.localizedDescription]) }
    }
}
private func request(_ item: [String: Any]) -> UNNotificationRequest {
    let content = UNMutableNotificationContent()
    content.title = item["title"] as! String; content.body = item["body"] as! String
    content.categoryIdentifier = item["category"] as! String; content.sound = .default
    content.userInfo = ["memoryId": item["memoryId"]!, "hash": item["hash"]!, "occurrenceAt": item["occurrenceAt"] ?? item["fireAt"]!]
    let trigger: UNNotificationTrigger
    if let rule = item["repeat"] as? [String: Any] {
        var c = DateComponents()
        c.calendar = Calendar(identifier: .gregorian); c.timeZone = TimeZone(identifier: rule["tz"] as! String)
        c.month = rule["month"] as? Int; c.day = rule["day"] as? Int; c.weekday = rule["weekday"] as? Int
        c.hour = rule["hour"] as? Int; c.minute = rule["minute"] as? Int; c.second = rule["second"] as? Int
        trigger = UNCalendarNotificationTrigger(dateMatching: c, repeats: true)
    } else {
        let f = ISO8601DateFormatter(); f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        let date = f.date(from: item["fireAt"] as! String)!
        trigger = UNTimeIntervalNotificationTrigger(timeInterval: max(0.1, date.timeIntervalSinceNow), repeats: false)
    }
    return UNNotificationRequest(identifier: item["id"] as! String, content: content, trigger: trigger)
}
private func command(_ method: String, _ input: Any?) async throws -> Any {
    let center = UNUserNotificationCenter.current()
    switch method {
    case "requestAuthorization": return try await center.requestAuthorization(options: [.alert, .sound, .badge])
    case "status":
        let settings = await center.notificationSettings(), pending = await center.pendingNotificationRequests()
        return ["authorization": auth(settings.authorizationStatus), "pendingCount": pending.count,
                "bundleId": Bundle.main.bundleIdentifier ?? "", "bundleName": Bundle.main.object(forInfoDictionaryKey: "CFBundleDisplayName") as? String ?? ""] as [String: Any]
    case "pending": return rows(await center.pendingNotificationRequests())
    case "delivered": return rows(await center.deliveredNotifications().map(\.request))
    case "removeAll":
        let prefix = input as! String
        let ids = await center.pendingNotificationRequests().map(\.identifier).filter { owned($0) && $0.hasPrefix(prefix) }
        center.removePendingNotificationRequests(withIdentifiers: ids)
        let delivered = await center.deliveredNotifications().map(\.request.identifier).filter { owned($0) && $0.hasPrefix(prefix) }
        center.removeDeliveredNotifications(withIdentifiers: delivered)
        return ids
    case "apply":
        let auth = await center.notificationSettings().authorizationStatus
        guard auth == .authorized || auth == .provisional else { throw NSError(domain: "notify", code: 1, userInfo: [NSLocalizedDescriptionKey: "permission denied; existing requests preserved"]) }
        let items = input as! [[String: Any]], pending = await center.pendingNotificationRequests()
        let hashes = Dictionary(uniqueKeysWithValues: pending.map { ($0.identifier, $0.content.userInfo["hash"] as? String ?? "") })
        let desired = Set(items.map { $0["id"] as! String })
        var added = [String](), unchanged = 0
        for item in items {
            let id = item["id"] as! String
            if hashes[id] == item["hash"] as? String { unchanged += 1; continue }
            try await center.add(request(item)); added.append(id)
        }
        let stale = pending.map(\.identifier).filter { owned($0) && !desired.contains($0) }
        center.removePendingNotificationRequests(withIdentifiers: stale)
        return ["added": added, "removed": stale, "unchanged": unchanged] as [String: Any]
    case "finishAction":
        lock.lock(); let completion = delegate?.completions.removeValue(forKey: input as! String); lock.unlock()
        completion?(); return true
    case "testHooks":
        NSWorkspace.shared.notificationCenter.post(name: NSWorkspace.didWakeNotification, object: nil)
        NotificationCenter.default.post(name: .NSSystemTimeZoneDidChange, object: nil)
        return true
    default: throw NSError(domain: "notify", code: 2, userInfo: [NSLocalizedDescriptionKey: "unknown command"])
    }
}
