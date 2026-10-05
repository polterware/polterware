# w3-packaging status

- Result: **partial**. Local ad-hoc arm64 DMG/ZIP, bundle verification, smoke and standard E2E against the ZIP app are done. CI draft-from-tag and the clean-account reminder gates remain for Erick (this task forbids push, tag, publish, `/Applications` install, account creation and system-setting changes).
- Branch: `sparky/w3-packaging-b` from `sparky-next` at `56d75c45bf17ee8012999f47d7ae3aa01355aa37`. First packaging scripts/workflow already merged; this continuation adds this status only. No push, tag or publish.
- Host: macOS 26.6.2 (25G83), arm64, Node v26.8.2, pnpm 12.6.0, Electron 44.4.5, go1.27.1. Real `~/Library/Application Support/Sparky` mtime stayed `2026-10-04 03:06:48` (not opened). `/Applications/Sparky.app` was already running and was left alone.

## Acceptance

| Criterion | Result / evidence |
| --- | --- |
| Reproducible `pnpm dist:sparky:mac` | PASS. No script bugs. Output `apps/sparky-desktop/release/0.1.0/` (gitignored). |
| Ad-hoc sign, no Developer ID / hardened runtime / notarize | PASS. `codesign --display` `Signature=adhoc`, `flags=0x2(adhoc)`, `TeamIdentifier=not set`. |
| `codesign --verify --deep --strict` | PASS. `valid on disk`; `satisfies its Designated Requirement`. |
| `spctl --assess --type execute` | Expected FAIL. Rejected, exit 3. Not Developer ID, not notarized. |
| Bundle contents | PASS. `verify-bundle.cjs`: arm64 core + `notify.node`, `sparky://`, microphone usage description, unpacked renderer, no source maps / dev-only deps / `app-update.yml`. |
| Size + SHA256 | Recorded below. |
| Startup median of 5 (launch → visible content) | **878.8 ms** (samples 4332.3, 859.8, 867.0, 878.8, 1012.7). First sample is cold. |
| Install/launch smoke on the ZIP | PASS (`scripts/smoke-mac.cjs`). Isolated temp profiles only. `sparky doctor --json` `ok: true`. Updates `disabled`. Known product gaps recorded, not fixed. |
| Standard E2E vs ZIP app | **25/25 PASS** on the final run (`SPARKY_E2E_RUN=w3-packaging-b-final`). First run was 22/25 for environmental reasons (below). |
| Workflow review | PASS. YAML loads. Not triggered. |
| Artifacts from a CI tag | UNVERIFIED here (no tag/push). Procedure below. |
| Clean-account live reminders / login item / update | Checklist below for Erick. Login item stays unavailable in v1. |

## Reproduce

```bash
pnpm dist:sparky:mac
# refuses to overwrite apps/sparky-desktop/release/<version>/

node apps/sparky-desktop/scripts/smoke-mac.cjs \
  "$PWD/apps/sparky-desktop/release/0.1.0/Sparky-0.1.0-arm64.zip"

# E2E against the ZIP app: extract off the /tmp symlink (macOS /tmp → /private/tmp
# breaks renderer-package's file:// equality). Build the SDK driver once.
(cd sparky/backend && go build -o ../../apps/sparky-desktop/tests/build/e2edriver ./tests/e2edriver)
EXTRACT="$PWD/apps/sparky-desktop/test-results/w3-packaging-b/extracted"
ditto -x -k apps/sparky-desktop/release/0.1.0/Sparky-0.1.0-arm64.zip "$EXTRACT"
SPARKY_E2E_EXECUTABLE="$EXTRACT/Sparky.app/Contents/MacOS/Sparky" \
  SPARKY_E2E_RUN=w3-packaging-b-final \
  pnpm --filter @polterware/sparky-desktop test:e2e
```

Dist report: `release/0.1.0/build-report.json` (commit `56d75c45`, `SOURCE_DATE_EPOCH` from that commit, lock SHA `d8a13430…`). Smoke: `release/0.1.0/smoke-report.json`. E2E: `test-results/w3-packaging-b-final/results.json`. None of these are committed.

## Artifacts (v0.1.0, arm64)

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `Sparky-0.1.0-arm64.dmg` | 157 175 149 (149.9 MiB) | `6bce2eebe59342b803e0bcd961b5b3f0b8b378af1e97e51317fe0870bf25892b` |
| `Sparky-0.1.0-arm64.zip` | 155 821 449 (148.6 MiB) | `ffe3fe6bd28360bf4eb53a00e87321de934ad3521a2e744d3d968148427b0eee` |
| `SHA256SUMS` | 178 | (the two lines above) |

`spctl` text: `Sparky.app: rejected` / `spctl exit: 3 (ad-hoc rejection expected)`.

## Smoke

Overall **PASS**. Visible main, bundled core, doctor clean, second-instance `sparky://memory/<id>` opens the created memory, quit leaves the isolated core alive, relaunch keeps the row, cores were stopped. Profiles lived under `/var/folders/…/T/sparky-w3-packaging-4wwenG` only.

Known open product issues (not fixed here):

- Search deep link: route becomes Search, query box stays empty (`searchQueryResult: FAIL: route opens but query is not applied`). [w3-packaging-3.md](../requests/w3-packaging-3.md).
- Native reminders: `permission: "denied"` before and after; `requestNotificationAuthorization()` → `PublicIpcError: Native notification operation failed`. Logs: `delegate-installed` then `refresh-failed`. Nothing armed. [w3-packaging-2.md](../requests/w3-packaging-2.md). This session is denied (earlier packaging note was `not_determined` + timeout). Still not proof that a clean account cannot grant permission.

## E2E (standard functional suite, ZIP app)

Final: **25/25 PASS**, 0 fail, 0 skip, 58.8 s. Same 17 spec files as the default `e2e/**` suite.

First run (`w3-packaging-b`, extract under `/tmp`) was **22/25**. Failures were not product regressions:

- `renderer-package.spec.ts`: expected `file:///tmp/…/app.asar/out/renderer/index.html`, got `file:///private/tmp/…` (symlink). Extracting to the worktree path passes.
- `calendar.spec.ts` (both cases): onboarding overlay with `data-state="closed"` intercepted the Calendar click, then the page closed. Isolated rerun on the worktree extract passed both.

Live native and perf suites were not part of this standard run.

## Workflow (reviewed, not triggered)

`.github/workflows/sparky-release.yml` — `ruby -ryaml -e 'YAML.load_file(…)'` → `YAML_OK`.

- Triggers: push tag `sparky-v*`, `workflow_dispatch` with an existing tag.
- Permissions: top-level `contents: read`; `draft` job only `contents: write`.
- Actions pinned by major: `checkout@v4`, `pnpm/action-setup@v4`, `setup-node@v4`, `setup-go@v5`, `upload-artifact@v4`, `download-artifact@v4`.
- Build: `macos-15`, `polterware/polterware` only, `pnpm install --frozen-lockfile`, `SPARKY_RELEASE_VERSION` from the tag, `dist-mac.sh`, `--publish never`.
- Uploads CI artifacts (dmg, zip, SHA256SUMS, verification files) then attaches **only** dmg + zip + SHA256SUMS to a **draft**. Refuses to modify a published release.
- Auth: `secrets.GITHUB_TOKEN` only. No extra secrets.

## Ad-hoc signing limits (honest)

Gatekeeper will refuse a first open of a downloaded/quarantined ad-hoc app. After the user trusts the build:

1. Right-click the extracted `Sparky.app` → Open, or
2. `xattr -dr com.apple.quarantine /path/to/Sparky.app` on that trusted copy only.

`spctl` rejection is expected until Developer ID + notarization exist. There is no auto-update in v1 (`electron-updater` stays disabled; no `app-update.yml`). Native reminders under this ad-hoc identity are unproven on this account (see request 2). Login items stay unavailable (O1).

## Manual update

Quit Sparky. Replace the installed `Sparky.app` with the app from the new ZIP (do not merge the bundles). Reopen. `~/Library/Application Support/Sparky` is kept. No Sparkle/Squirrel feed.

## Uninstall notes

- App: delete the `.app` the user installed (never required to be `/Applications` in v1).
- Data: `~/Library/Application Support/Sparky` (DB, logs, MCP token). Remove only if the user wants data gone.
- MCP clients: remove any copied stdio/HTTP snippets by hand.
- Login item: none to remove in v1.

## Release-tag procedure (Erick)

1. Version in `apps/sparky-desktop/package.json` is `X.Y.Z`.
2. `git tag sparky-vX.Y.Z` on the intended commit (Erick).
3. Erick pushes the tag (`git push origin sparky-vX.Y.Z`). Agents do not.
4. Workflow builds arm64 ad-hoc artifacts and opens a **draft** on `polterware/polterware`.
5. Erick inspects the draft (notes, dmg, zip, SHA256SUMS) and publishes when ready.

## Clean-account checklist (Erick, by hand)

Use a fresh macOS user. Do not reuse this development account’s TCC state.

1. Extract the ZIP (not the DMG if you want a side-by-side copy). Clear quarantine or right-click Open.
2. First launch: Welcome, then the **notification permission** prompt. Grant it. Record whether the prompt says “Sparky” and whether Settings → Reminders then shows authorized / armed counts after a real schedule.
3. Arm a near-term reminder. Quit Sparky (last window close; core may stay). Confirm the banner still fires.
4. Reboot. **Do not open Sparky.** Confirm the reminder still fires.
5. From the banner: **Done** and **Snooze** with Sparky closed. Confirm the memory updates after a later open (or hidden replan).
6. Update: quit, replace the `.app` with a newer ZIP’s app, reopen; data still present.
7. Optional: `sparky doctor` against that account’s data dir.

Login-item / start-at-login is not a v1 gate.

## Deviations

- No `native/scripts/package.sh` exists; `dist-mac.sh` + `native/scripts/build.sh` is the path.
- Root `dist:sparky:mac` and `.gitignore` `release/` already applied (`w3-packaging-1`).
- Product code, Main, search, lockfiles and shared root files were not edited. `w3-fix-startup` owns supervisor timing and the search query in parallel.
- First E2E extract used `/tmp`; final accepted run used a worktree extract. No harness assertion changes.

## Requests filed

- None new (`w3-packaging-b-*.md`).
- Still open: [w3-packaging-2.md](../requests/w3-packaging-2.md), [w3-packaging-3.md](../requests/w3-packaging-3.md).

## Follow-ups

- Erick: tag + draft publish; clean-account checklist.
- Native/Main: authorization + reboot delivery (request 2).
- Search: `q` on `sparky://search` (request 3; `w3-fix-startup`).
