# w0-native-notify status

- Result: blocked
- Branch / last commit: `sparky/w0-native-notify`; implementation commit `b075afa`; the final status/request commit is `git log -1 --format='%h %s'` on this branch.
- Blocking condition: `bash tooling/verify-sparky.sh` exits 1 in `packages/sparky-contracts/src/baseline.test.mjs`, `SQL baseline > matches the specification fence`. The baseline still includes the removed `embeddings` table, while the specification no longer does. The failure is present in integration base `18e9281`; this workstream changes neither file. Execution stopped under AGENT-PROTOCOL section 5 (gate failure outside Owns).

## Acceptance criteria

1. Delivery after kill for one-shot and repeating triggers → **unverified / not accepted**. Both native implementations contain one-shot `UNTimeIntervalNotificationTrigger` and repeating `UNCalendarNotificationTrigger` support. No reminder was armed and no kill/delivery run was performed before the external gate failure stopped work.
2. Done action applied with Electron closed, with click-to-applied latency → **unverified / not accepted**. Both implementations register a delegate before Electron readiness and expose buffered action callbacks; Main contains a stub-core supervisor, complete/snooze calls, re-plan and a ten-second idle exit. None of these paths was observed with a real notification action. There is no latency number; no simulated action is being counted as acceptance evidence. Snooze also remains unverified.
3. Hidden relaunch works or is documented as not viable → **unverified / not accepted**. A LaunchServices background-mode command started with zero windows, but the exact `open -g -j -a <app> --args --background-replan` scenario, frontmost-app preservation, Dock behavior and automatic idle exit were not measured. `LSUIElement=true` is configured for the spike; this alone does not prove that scenario.
4. Recommended form and build steps for the pinned Electron ABI → **partial**. Objective-C++ N-API using node-addon-api and a Swift C ABI dylib loaded through koffi both compile and pass deep/strict ad-hoc signature verification. The N-API module loaded in Electron **44.4.5**, `NODE_MODULE_VERSION=149`, runtime N-API **10**; it is compiled with `NAPI_VERSION=8` using the 44.4.5 Electron headers. Swift+koffi was built but not launched. No final recommendation has been made from build-only evidence.
5. Developer ID / entitlement requirement → **partial**. The N-API Electron stub launched through LaunchServices and requested authorization under an ad-hoc signature without added entitlements or Developer ID. Scheduling, survival and action relaunch were not verified, so their signature/entitlement requirements remain unproven by this workstream. This does not change the previous spike's conclusions about distribution signing.
6. Fallback decision → **pending / not accepted**. No native delivery failure has been demonstrated. The existing fallback in `03-recurrence-and-reminders.md` remains available; this blocked run neither adopts it nor resolves O1-final. Do not treat G0's O1-final checkbox as complete.

## Completed work and evidence

- Only `spikes/native-notify/**`, this status file and the request file were changed.
- Two native engines and a shared Main-only JavaScript adapter expose `requestAuthorization`, `apply`, `pending`, `removeAll`, `onAction`, and `status`. The adapter validates ownership, categories, timestamps, duplicate IDs, repeat fields/timezones and the 60-item budget before native dispatch. Mutations are serialized. Native diffing preserves foreign identifiers and returns an error without removing existing requests if authorization is unavailable.
- N-API dispatch uses a thread-safe function. The Swift C ABI uses a locked JSON reply queue with a 20 ms JS poll and explicit allocation/free; it does not retain a JS function pointer across FFI shutdown. These are spike implementations, not production modules.
- Native delegate events include wake/timezone hooks, delivered notifications and action callbacks. Main has a debounced re-plan, six-hour timer and Electron resume hook. Synthetic hooks are available through the spike-only `testHooks` command, but were **not exercised**. No actual wake or timezone change was performed.
- Authorization identity: N-API `status` returned `bundleId=com.polterware.sparky.spike2`, `bundleName=Sparky`, `authorization=notDetermined`, `pendingCount=0`. `requestAuthorization` subsequently returned `true`. A scoped AppleScript returned `No readable Sparky authorization prompt`; the prompt's visible wording was **not recorded**. Bundle metadata is not substituted for prompt observation.
- Bundle size and background RSS were **not measured**.
- No launchd service or helper was registered. No reminder was armed. Any remaining processes whose executable path was inside this spike's `dist/` were stopped; no global Electron process kill was used.
- Chromium user data and Electron logs are now directed into the spike's ignored `run/` directory. The initial probe preceded that path-scoping change and used Electron's default spike user-data directory; nothing outside Owns was deleted. The final path-scoping edit was syntax-checked; no final bundle/runtime run was performed after the block.
- Logs: `spikes/native-notify/evidence/build.txt`, `report.json`, `verify-sparky.txt`, `design-lint.txt`. No core session token or generated runtime file is committed.

## Commands and results

- `pnpm install` → **pass**, workspace lockfile unchanged.
- `pnpm --dir spikes/native-notify install --ignore-workspace --ignore-scripts` → **pass**. Koffi ships the Darwin arm64 prebuild; `require('koffi')` reported version 2.15.1. An earlier default install returned `ERR_PNPM_IGNORED_BUILDS`; the explicit script-free install succeeded. No global install or shared dependency/config change was made.
- `bash spikes/native-notify/build.sh` → **pass**, both forms compiled, copied into separate ad-hoc Electron bundles and verified with `codesign --verify --deep --strict`. Swift emitted two scoped-locking warnings in Swift 5 mode; address those before production adoption.
- `open -n -a "$PWD/spikes/native-notify/dist/napi/Sparky.app" --args --command=status` → **observed pass**, native module loaded and returned the status described above. Result-file creation is asynchronous; a two-second initial read was too early. Use a bounded file wait rather than a fixed short sleep when continuing.
- `open -n -a "$PWD/spikes/native-notify/dist/napi/Sparky.app" --args --command=requestAuthorization` → **observed pass**, returned `true` under the Sparky bundle identity. The visible prompt text remains unverified.
- `bash tooling/verify-sparky.sh` → **fail**, external SQL specification mismatch. `buf lint`, baseline breaking check, Sparky Go tests, Sparky Go vet and Sparky TypeScript checks passed before the contract test stopped the script. Contract suite: **25 passed / 1 failed**. Checks after that failure were not run by this script.
- `node --check spikes/native-notify/electron/main.cjs`, `node --check spikes/native-notify/electron/adapter.cjs`, `bash -n spikes/native-notify/build.sh` → **pass** after the final source edits.
- `pnpm design:lint` → **pass**, `ui opacity check OK`; see `spikes/native-notify/evidence/design-lint.txt`. Run separately because the broader gate exited before its design check.
- `git diff --check`, staged diff check and ownership review → **pass** before the final status commit. Only Owns paths plus the protocol-authorized status/request files are changed.

## Deviations from the plan (and why)

- Acceptance execution and O1-final are unfinished because the protocol requires stopping on the external gate failure. No contradictory native result was inferred.
- W0-NATIVE-NOTIFY does not list a separate Verify command. The repository's `bash tooling/verify-sparky.sh` was run as the available Sparky gate and exposed the blocker.
- `00-decisions.md` is outside Owns, so it was not edited. A resolution must be proposed to the integrator after runtime acceptance is complete.
- `polterware` is absent from PATH and no built binary exists at the usual worktree `dist/polterware` / `backend/dist/polterware` paths. `polterware status` and `polterware handoff` returned `command not found`; format/query/apply were therefore unavailable. No Work card was written and no Polterware product binary was built outside this workstream.
- No branch creation, tag, remote push, reference-repository edit or unrelated source change.

## Requests filed

- `plans/sparky-next/requests/w0-native-notify-1.md`: merge the authorized W0-CONTRACTS-FIX changes to unblock the SQL baseline gate. Includes the immediate failing SQL diff and reproduction command.

## Follow-ups for other WS

- Integrator / W0-CONTRACTS-FIX: resolve request 1, then resume this workstream on the corrected integration base. Do not start dependent native-notification work on the assumption that O1-final is resolved.
- W0-NATIVE-NOTIFY continuation: rebuild final sources, launch Swift+koffi, then run both engines' API/diff/denial/foreign-ID checks; native one-shot and repeating survival; real cold Done/Snooze, click-to-applied latency and idle exit; hidden relaunch while another app is frontmost; hooks; prompt observation; bundle/RSS measurements. Emit a reproducible acceptance driver and sanitized evidence. Resolve or document runtime limitations honestly.
- W1-NATIVE-NOTIFY / W1-ELECTRON: the spike is unfinished. Do not copy it into production or interpret the action supervisor and accessory-mode setup as validated behavior.
