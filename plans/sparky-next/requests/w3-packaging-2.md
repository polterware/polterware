# Resolve native notification authorization in the real ad-hoc bundle

Owner: Native Notify / Main / Settings owners, with Erick's approval for fallback behavior. No product fallback is implemented by W3-PACKAGING.

The real arm64 release ZIP is ad-hoc signed (`codesign --verify --deep --strict` passes), hardened runtime is off, bundle id is `com.polterware.sparky`, and no entitlements/certificates are used. The real `Contents/Resources/native/notify.node` loads. This differs from w3-e2e-7's unsigned package.

On the current macOS session, `reminders.status` before authorization reports `planned:0`, `armed:0`, `permission:"not_determined"`. `requestNotificationAuthorization()` times out after the native bridge's 30-second limit with `PublicIpcError: Native notifications timed out`; status afterward stays `not_determined`, with zero armed reminders. Logs contain `delegate-installed` and `refresh-failed` for start/change. The smoke did not visually verify or interact with a system prompt. No OS preference or TCC record was reset. There is no observed native delivery. This is a failure to authorize in this session, not proof that every ad-hoc install is unsupported.

Evidence: `apps/sparky-desktop/release/0.1.0/smoke-report.json` and `test-results/w3-packaging-live/`; final results are recorded in `status/w3-packaging.md`. Use the exact signed ZIP and a genuinely clean account for the remaining permission/delivery gate. Do not substitute a stub or a different bundle id.

Requested resolution:

1. Diagnose the app's real UNUserNotificationCenter authorization path without exposing native/private error text to the renderer; distinguish unanswered OS permission UI, existing account/TCC state, and actual signing incompatibility. Confirm granted permission and delivery after quit/reboot before promising OS-armed reminders.
2. If the supported ad-hoc package still cannot use UNUserNotificationCenter, explicitly approve a fallback **before implementing it**: Main may use Electron `Notification` for timed delivery while Sparky runs. Keep it in Main (renderer uses the typed bridge), cancel/update timers on schedule changes, enforce the core plan budget, and report degraded delivery honestly. A headless Go core alone cannot call Electron Notification; do not claim that keeping only the core alive provides this fallback.
3. Settings copy for that fallback should say: `Native reminders are unavailable. Notifications can be delivered while Sparky is open. Reminders will not fire after quitting Sparky or before opening it after a restart.` Never say a reminder is armed in the OS. Do not silently reintroduce an always-running Electron/login item: current v1 quits on last-window close and login items remain unavailable.

Reboot delivery, Done/Snooze and permission on a clean account remain manual gates. No Developer ID, certificate request, system-setting change or account creation is proposed for this task.
