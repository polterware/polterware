# w0-native-notify status

- Result: **done** (the workstream's runtime acceptance passes; the external repository gate remains failing).
- Branch / last commit: `sparky/w0-native-notify`; final commit subject `sparky(w0-native-notify): complete native runtime acceptance`. Resolve its hash with `git log -1 --format='%h %s'`.
- Host: macOS **26.6.2**, build **25G83**, arm64. Electron **44.4.5**, `NODE_MODULE_VERSION=149`, runtime N-API **10**; addon targets **NAPI_VERSION=8**.
- Resume decision: on 2026-10-04 the integrator explicitly identified the SQL baseline failure as unrelated, assigned it to W0-CONTRACTS-FIX and instructed this spike to continue. No contract, baseline, shared configuration or integration-branch changes were made.

## Acceptance criteria

| Criterion | Result | Evidence |
|---|---|---|
| One-shot delivery after kill | **Pass, both engines** | `node spikes/native-notify/acceptance.cjs`; `evidence/runtime.json`: `killed`, `afterDeadlineProcesses=[]`, `delivered` includes the synthetic one-shot, with `repeats=false` |
| Native repeating delivery after kill | **Pass, both engines** | Same run: delivered `UNCalendarNotificationTrigger`, `repeats=true`; that identifier remains in `pending` after delivery |
| Done with Electron closed | **Pass, both engines** | Real Notification Center Accessibility action; OS cold relaunch, native delegate, core spawn, authenticated `/complete`, persisted empty plan, re-plan and idle exit |
| Snooze with Electron closed | **Pass, both engines** | Real Notification Center Accessibility action; authenticated `/snooze`, persisted one-shot ten minutes later, native re-arm and idle exit |
| Hidden relaunch | **Pass in the accessory spike, both engines** | Exact `open -g -j -a <app> --args --background-replan`; the existing other app remained frontmost in every sample; zero windows, `dockVisible=false`, successful re-plan, Electron exit |
| Wake/timezone hooks | **Pass for injected hook wiring, both engines** | `testHooks` posts the actual observed NSWorkspace wake and Foundation timezone notification names. Native events cross the module boundary and cause a debounced re-plan. No physical sleep/wake or system timezone change was performed |
| Authorization prompt identity | **Observed pass** | `evidence/authorization.json`: the OS alert reads **“Sparky” Notifications**, followed by “Notifications may include alerts, sounds, and icon badges.” A fresh spike-only bundle identity was used without resetting existing permissions |
| Module recommendation and pinned ABI build | **Pass; choose Objective-C++ N-API** | Both compiled, loaded, scheduled and handled cold responses; build instructions and comparison below |
| Signing / entitlement requirement | **Pass for local ad-hoc execution** | Both bundles pass deep/strict signature verification. `evidence/signature.txt`: `Signature=adhoc`, no Team ID and no main executable entitlement blob; delivery and cold actions pass |
| Fallback decision | **Keep native delivery; do not adopt the timer fallback** | Native scheduling, OS-owned delivery and cold action application work with all spike processes killed |

The final `evidence/runtime.json` contains **16 passing checks per engine**, `allPassed=true`, and no runtime error events. The shared adapter's repeated identical-plan application returns `unchanged=2`. The core persists plans and changes across SIGKILL; it excludes expired one-shots from its returned plan. This fixes the synthetic core's initial cold-start re-plan error; the complete acceptance run was repeated after that fix.

### Cold action latency and exit

| Engine | Done click-to-applied | Snooze click-to-applied | Electron idle exit after applied action |
|---|---:|---:|---:|
| Objective-C++ N-API | **341 ms** | **318 ms** | 10.015 / 10.017 s |
| Swift + koffi | **319 ms** | **413 ms** | 10.049 / 10.047 s |

These are individual host observations, not percentiles or a latency guarantee. `clickAtMs` is recorded immediately before performing the OS notification's **Name:Done / Name:Snooze Accessibility action**. The applied timestamp is logged after the core's successful HTTP response and persisted state change. This invokes Notification Center's real action path; it does not inject a fabricated delegate response or directly call the core as the test action. Both Electron and the core were absent immediately before each click. The delegate is installed before Electron readiness, and notification completion is acknowledged after applying the action and re-planning.

The cold action and hidden re-plan launches opened no window. The hidden launch exited **10.001 s after re-plan** for each engine. The core remains a detached daemon until the driver explicitly kills it; “idle exit” above refers to Electron, not a claim that the core exits on idle.

### Size and background RSS

| Measurement | Objective-C++ N-API | Swift + koffi |
|---|---:|---:|
| Bundle logical size, uncompressed | **295.63 MiB** | **308.80 MiB** |
| Allocated size (`du -sk`) | 296.49 MiB | 310.73 MiB |
| Native module alone | 164.76 KiB | 141.14 KiB |
| Background Electron Main RSS | **118.19–120.89 MiB** | **120.00–122.63 MiB** |
| Electron Main + Electron child RSS, core excluded | 214.97–221.53 MiB | 216.72–223.41 MiB |

RSS comes from `ps -axo pid=,rss=,comm=` sampled during the actual hidden re-plan, with no window. Summing RSS can double-count shared pages; these are not private-memory measurements. Logical size sums bundle file sizes; allocated size is filesystem-dependent. Both contain the same pinned Electron runtime and bundled stub core. Swift's extra **13.17 MiB** is predominantly the bundled koffi package, not the Swift dylib. Only its Darwin arm64 prebuild is retained, but the spike still includes package support files. No compressed production artifact was measured.

## O1-final decision for the integrator

**Use Objective-C++ N-API in Electron Main.** Keep the Swift+koffi implementation as a proven alternative, not the production default. N-API delivers events directly through a thread-safe function, avoids a 20 ms polling queue and an FFI dependency, and has the smaller bundle. The small latency sample does not establish a statistically significant performance winner.

Keep the app's own `UNUserNotificationCenter` identity, install its delegate before `app.whenReady()`, buffer early responses, bootstrap/attach the core on a cold Done/Snooze, apply the operation, re-plan, finish the OS response and exit background Electron after ten seconds of inactivity. Simple recurrence uses native repeating triggers; complex finite plans are refreshed when Sparky or the core runs. Retain the honest wording **“Reminders are refreshed when Sparky runs”**. No launchd helper is registered, and the fallback **“Reminders need Sparky running”** is not warranted by these results.

**No Developer ID or notification-specific entitlement is needed for the local path demonstrated here.** All runtime acceptance was under ad-hoc signing without added entitlements. This is not a distribution/notarization or hardened-runtime acceptance test. W3-PACKAGING still owns distribution signing. Apple documents [local notification authorization](https://developer.apple.com/documentation/usernotifications/asking-permission-to-use-notifications); Electron documents [native modules](https://www.electronjs.org/docs/latest/tutorial/using-native-node-modules/) and [distribution signing](https://www.electronjs.org/docs/latest/tutorial/code-signing/).

The shipping deployment target remains **macOS 15+**; this run tested macOS 26.6.2 only. Both builds target macOS 15; no physical macOS 15 host was tested.

`00-decisions.md` is outside Owns and was not edited. Suggested resolution to append:

> 2026-10-04 O1-final (W0-NATIVE-NOTIFY, done): use an Objective-C++ N-API module in Electron Main (node-addon-api 8.5.0, N-API 8; Electron 44.4.5 / ABI 149). Both N-API and Swift+koffi delivered one-shot and native repeating reminders after every spike process was killed. Cold Done/Snooze applied in 341/318 ms for N-API and 319/413 ms for Swift+koffi. Hidden LaunchServices re-plan preserved the other frontmost app, opened no windows, had no Dock tile and exited after ten seconds in the accessory-mode spike. Local execution needs ad-hoc signing, no Developer ID and no additional notification entitlement. Keep native delivery and run-time refresh; do not adopt the timer fallback. W1 must preserve the background accessory behavior while implementing the normal foreground Dock/window lifecycle.

## Reproduction and verification

```bash
pnpm --dir spikes/native-notify install --ignore-workspace --ignore-scripts
bash spikes/native-notify/build.sh
node spikes/native-notify/acceptance.cjs
node spikes/native-notify/authorization.cjs
```

The build script verifies the pinned Electron version, obtains its headers, compiles Objective-C++ with Xcode clang++, `node-addon-api@8.5.0` and `NAPI_VERSION=8`, and builds a macOS 15 arm64 Swift C ABI dylib with `swiftc`. The latter loads via `koffi@2.15.1`. Both are copied into independent Electron bundles and signed/verified ad-hoc. The adapter and dylib must remain outside ASAR in production. Swift's earlier async-lock warnings were fixed using `NSLock.withLock`; the final build has no warnings.

- `bash spikes/native-notify/build.sh` → **pass**, both engines, strict/deep ad-hoc verification; `evidence/build-resume.txt`.
- `node spikes/native-notify/acceptance.cjs` → **pass**, both engines, all 32 checks; `evidence/runtime.json`, `runtime-output.txt`.
- Authorization observer → **prompt text observed**. The 20-second authorization-result wait expired because Accessibility exposed no Allow/Don't Allow button in that OS alert. The subsequent native apply returned `permission denied; existing requests preserved` while authorization was unavailable, with pending empty. This proves the unavailable-authorization guard, **not** an observed user-denial transition. The original `spike2` identity was already authorized and both engines used it successfully. `authorization.cjs` uses a fresh throwaway identity on repeat runs.
- `bash tooling/verify-sparky.sh` → **known external fail**: `packages/sparky-contracts/src/baseline.test.mjs`, `SQL baseline > matches the specification fence`; SQL still includes removed `embeddings`. Contract tests: **25 pass / 1 fail**. Earlier buf, Go tests/vet and Sparky TS checks pass; later gate steps are not claimed to pass. See `evidence/verify-sparky-resume.txt`. Integrator waiver permits completion without changing this baseline.
- `(cd spikes/native-notify/stubcore && GOWORK=off go vet ./...)` → **pass** after final Go edits.
- Node syntax checks for all CJS and both JXA scripts, `bash -n spikes/native-notify/build.sh`, `git diff --check` → **pass**.
- `pnpm design:lint` → **pass**, separately from the interrupted repository gate; `evidence/design-lint-resume.txt`.
- Final cleanup → **pending=[], delivered=[], spike processes=[]**; `evidence/cleanup.json`. Every SIGKILL was preceded by executable-path verification under this spike's `dist/`. No global Electron kill, screenshot, reference checkout edit, branch creation, tag or push.

## Deviations and follow-ups

- The external gate exception is the integrator's explicit resume decision, not an inferred waiver. Request 1 is resolved for this workstream; W0-CONTRACTS-FIX still owns the repair.
- The spike deliberately has `LSUIElement=true`. No Dock tile means no tile to bounce; this is not a full-screen visual observation. W1-ELECTRON must implement and verify the normal foreground Dock/reopen lifecycle without losing background behavior. No foreground production shell behavior is claimed here.
- Wake/timezone hooks were injected through the real registered notification centers. Actual sleep/wake, timezone transitions and multi-cycle repeating endurance remain hardening checks; no system clock, timezone or sleep setting was changed.
- The authorization prompt's identity was observed on a fresh bundle with the same Sparky display name. Its click completion remains unobserved; metadata is not substituted for the observed prompt text.
- N-API shutdown/lifetime cleanup, release-mode hardened signing, cold-response error/retry/idempotency behavior against the real core, single-instance launch handling and regular foreground shell behavior remain production work for W1-NATIVE-NOTIFY / W1-ELECTRON, not validated production behavior of this throwaway stub.
- `polterware` is absent from PATH, but the main checkout has `dist/polterware`. Its `format` succeeded; `status` and `query cards` returned `no project for this directory`. No new project was initialized and no unrelated project's Work cards were mutated. The final `handoff` also returned `no project for this directory`.
- Owned changes only: `spikes/native-notify/**` plus protocol-authorized status/request files. The existing unrelated gate remains outside this workstream's changes.
