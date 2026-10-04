---
summary: Main support for Settings Advanced and Data folder actions
rationale: W2-SETTINGS can persist keep-running and other SettingsService keys, but logs, revealing the data folder, a login item, and reduce-transparency are Electron Main responsibilities. They are not on window.sparky.app today. Reset has no Maintenance RPC.
---

Owner: W1-ELECTRON / integrator.

W2-SETTINGS registered these as **unavailable** (honest copy, no fake toggles). After Main/core support exists, the Settings feature can wire the controls.

Requested additive `window.sparky.app` methods (do not invent renderer-only behavior):

```ts
app.openLog(): Promise<void>
app.openDataFolder(): Promise<void>
app.setLoginItem(enabled: boolean): Promise<{ enabled: boolean }>
app.getLoginItem(): Promise<{ enabled: boolean }>
app.setReduceTransparency?(enabled: boolean): Promise<void>
```

Notes:

- `revealInFinder` currently takes a `sparky-file://` attachment URL, so it cannot open `~/Library/Application Support/Sparky/`.
- Login item / keep-alive is not observed under ad-hoc signing (00-decisions O1). If Main cannot implement `setLoginItem`, keep the Advanced row unavailable.
- Reduce transparency applies to the macOS title bar / sidebar vibrancy only.
- Reset: there is no `MaintenanceService.Reset`. Add a typed RPC (or a Main-owned wipe that stops the core, deletes the database, and relaunches) if product still wants typed confirmation to actually wipe data. Until then Settings shows the confirmation and states that no data was changed.

Also: Connect `invalid_argument` FieldViolation reasons (`must be HH:MM`, `must be mon or sun`, …) should appear in the IPC `Error.message` so Settings can show the core validation text. Today Electron likely serializes only `[invalid_argument] invalid argument`.
