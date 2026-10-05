# W1-NATIVE-NOTIFY request 2 — shell, build and core integration

- Owners: W1-ELECTRON for Main/package/builder integration; W1-CORE for hidden relaunch on committed agent writes; integrator for shared config.
- Blocked sub-tasks: delegate activation in production Main, packaging, cold launch against real core, and real-core E2E acceptance.
- This worktree starts at `acacc8d`: `src/main/index.ts` only opens a scaffold window; `runtimes/cli/main.go serve` prints a stub and exits. No real supervisor, clients, change watcher or RPC server is present. Do not substitute the W0 synthetic core for real-core acceptance.

Required Main changes after W1-ELECTRON lands:

1. Import `installReminders` from `./reminders/index` and call it **before app.whenReady()**, immediately after obtaining the single-instance lock. Supply `ensureCore(): Promise<Transport>` from the supervisor, `openMemory(url)` through the validated deep-link/router path, `hasWindow()` and `quit()`.
2. Use `isBackgroundReminderLaunch(process.argv)` to suppress initial windows for `--background-replan` and macOS notification-response launches. A secondary `--background-replan` launch calls `runtime.requestRefresh("core-write")`; a normal/deep-link launch keeps foreground behavior. Closing the last window quits Electron per 00-decisions; the core remains detached.
3. In `app.on("ready", (_event, launchInfo))`, call `runtime.prepareLaunch(launchInfo)` before deciding whether to open a window, then `runtime.start()`. The addon initially uses accessory activation and synchronously tracks early native responses, covering callbacks buffered before V8 delivery. The return value of prepareLaunch controls initial window suppression. Restore regular activation when Open/Start focus or a normal second-instance launch opens a window; on before-quit call `runtime.close()`. Publish runtime.status()/onStatus through the typed validated reminder IPC channels and route requestAuthorization only from onboarding. Pass ChangeService.Watch messages to runtime.onChanges(); reconnection snapshot markers force a refresh.
4. Build scripts: add `"build:native": "bash native/scripts/build.sh"` and prepend `pnpm build:native &&` to desktop `build`, `dev`, and distribution build paths. The native package installs its own pinned node-addon-api dependency without changing the shared lockfile.
5. electron-builder.yml extraResources addition:

```yaml
  - from: native/build/notify.node
    to: native/notify.node
```

Use the host/target matching build (Electron 44.4.5, N-API 8, macOS 15+, no ASAR). W3-PACKAGING signs the addon with the app; no extra notification entitlement or launchd helper is required.

Required core behavior:

- After committed notification-relevant writes from MCP/CLI, coalesce bursts for 500 ms and lazily invoke `open -g -j -a <installed Sparky.app> --args --background-replan` using argument arrays. A live Main may receive a second-instance event; closed Main launches hidden. No shell interpolation; resolve the installed bundle via supervisor/runstate, not an untrusted app path. Do not relaunch for Heartbeat/Ack-only writes or changes beyond notification content.
- CompleteOccurrence must be idempotent for a repeated exact occurrence. Snooze has no idempotency key in v1.1; the controller does not blindly retry an ambiguous failed Snooze.
- Real-core E2E requires Health.Check, Memory.Create/List/Get, Schedule.Set/CompleteOccurrence, Reminder.Plan/Heartbeat/Status/Snooze, real change events, and installed production Main integration. Run `bash apps/sparky-desktop/native/scripts/e2e.sh` after these land. Until then the script must fail its real-core preflight rather than report synthetic success.
