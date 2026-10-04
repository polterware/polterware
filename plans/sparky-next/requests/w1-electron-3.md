# W1-ELECTRON request 3: native notification integration

Owner: W1-NATIVE-NOTIFY / integrator.

W1-ELECTRON deliberately leaves `src/main/reminders/**` and `native/**` untouched. Please provide the production entry point and native output path, then integrate:
1. Register the notification delegate before `app.whenReady()` in Main.
2. After `ensureCore`, refresh via real `ReminderService.Plan`; apply actions using the authenticated Main clients.
3. Handle `second-instance --background-replan` without opening a window.
4. Reset the 10-second idle exit after action/re-plan work, await in-flight work before quitting, and allow explicit Open actions to show a window.
5. Subscribe to change events, wake/timezone/delivery and the 6-hour refresh timer.
6. Publish exact build command and addon resources path for the W1-ELECTRON-owned package/build configuration. Package the Electron ABI prebuild outside asar; no fallback timer or keep-alive Electron.

Current hidden launch creates no window or Dock icon and has a bounded exit, but cannot refresh/action notifications without this dependency. Integration is stopped; no native API or artifact is fabricated.
