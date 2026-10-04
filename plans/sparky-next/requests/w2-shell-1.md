# W2-SHELL request 1 — notification authorization bridge

- Owner: W1-ELECTRON / W1-NATIVE-NOTIFY / integrator.
- Blocked sub-task: onboarding step 2 requesting macOS notification authorization.
- Evidence: `app` in `apps/sparky-desktop/src/shared/bridge.ts` has no authorization method. Main only calls `reminders.requestAuthorization()` when `SPARKY_NOTIFY_E2E=1` (`src/main/index.ts`). The controller already implements that operation.
- Requested additive change (no protobuf or SparkyFeature change):

```diff
 // SparkyBridge.app in apps/sparky-desktop/src/shared/bridge.ts
+    requestNotificationAuthorization(): Promise<boolean>;
 // APP_METHODS in the same file
+  "requestNotificationAuthorization",
```

Add the matching Main IPC handler, guarded by the existing trusted-sender policy, calling `reminders.requestAuthorization()`. Preload must expose the method through its existing APP_METHODS mapping. Extend method-validation and IPC tests. Keep permission state authoritative through `reminders.status`.

- Current behavior: onboarding explains native reminders, reports the real permission status and states that authorization is unavailable; skipping onboarding never implies reminders were armed.
- Follow-up: after the bridge is merged, add an explicit Enable notifications button to the onboarding step, call the bridge method and refresh status. Do not request permission automatically on mount.
- No file outside W2-SHELL ownership was edited.
