# Wave 3 — integration, security, packaging, polish, cleanup (+ verification)

Starts after G2. Merge order: W3-E2E → W3-SECURITY, W3-POLISH (parallel) → W3-PACKAGING → W3-CONVERGE → W3-CLEANUP. Gate **G3**
at the end.

---

## W3-E2E — end-to-end and performance

**Owns.** `apps/sparky-desktop/tests/**` (runner config, shared fixtures, helpers; feature specs from Wave 2 stay with their
authors but must pass), `sparky/backend/tests/**` (integration/perf), root script `test:e2e:sparky` (request to W0-SCAFFOLD's
owner if a shared file must change).
**Do.** Playwright `_electron` runner with an isolated temp data dir (`SPARKY_DATA_DIR` override honored by Main and core) and a
stub notifier mode; seeded fixtures (10k memories, recurring series across DST); scenarios: first run/onboarding, capture →
schedule → complete, Mind cascade delete, agent write while window open (via the real MCP stdio client), conflict banner,
search modes, attachments import, map with tiles off, focus phase, core crash/restart recovery, window close with core alive.
Performance suite against budgets in [01-architecture.md](01-architecture.md) (RSS, startup, list/FTS p95, FPS traces).
**Acceptance.** `pnpm test:e2e:sparky` green locally and in CI (headless where possible); perf report committed to the status
file; flaky tests quarantined with an issue note, not skipped silently.

## W3-SECURITY — review and hardening

**Owns.** Fixes may touch any Sparky path, **one finding per commit** (`sparky(w3-security): <finding>`), plus
`plans/sparky-next/status/w3-security.md` (findings table: severity, evidence, fix commit).
**Do.** Audit against [01-architecture.md](01-architecture.md) §Security: Electron config (contextIsolation/sandbox/CSP/navigation/IPC
allowlist/schema validation/protocol handlers/deep-link parser fuzz), core (bearer, Host/Origin, body limits, timeouts, path handling,
SSRF, SQL parameterization via grep + tests, log secrecy), MCP (token handling, DNS rebinding with a real browser fetch from a
malicious page, `confirm` semantics, file-attach gate), notifier (identifier handling, action authenticity), supply chain
(`pnpm audit`, `govulncheck`, lockfile review, no postinstall surprises), secrets at rest (0600 files, embeddings key only in
Keychain). Use the `claude-security` / `security-review` tooling if available in your environment.
**Acceptance.** No open High/Critical; every Medium has a fix or a recorded risk acceptance by Erick; DNS-rebinding and CSRF PoCs fail.

## W3-POLISH — design, motion, accessibility, copy

**Owns.** Visual/UX fixes across `packages/features/sparky-*` and `packages/frontend/**` (coordinate through small commits;
rebase often), `plans/sparky-next/status/w3-polish.md`.
**Do.** Pass over every screen against DESIGN.md and 06-ui.md: spacing scale, radii roles, no divider lines between siblings,
hover/selected tokens, empty states, skeleton loading, toasts, motion (springs, reduced motion), keyboard and VoiceOver audit
(Accessibility Inspector notes), contrast, truncation, long titles, RTL-safe layout basics, window resize extremes (min size,
fullscreen), vibrancy/"Reduce transparency" setting, app/dock icon, About panel, copy review. Run `design:lint`.
**Acceptance.** Checklist per screen in the status file with before/after screenshots; zero `design:lint` violations; every
interactive element reachable and labeled; Erick signs off on screenshots.

## W3-PACKAGING — release build

**Owns.** `apps/sparky-desktop/electron-builder.yml`, `apps/sparky-desktop/scripts/**`, `.github/workflows/sparky-release.yml`
(request to scaffold owner if needed), `apps/sparky-notifier/scripts/package.sh`.
**Do.** Reproducible `pnpm dist:sparky:mac`: builds `dist/sparky` (universal or arm64 per decision), embeds core + notifier,
signs (A12: ad-hoc/unsigned by default; **ask Erick** before buying/using a Developer ID), dmg + zip, `electron-updater`
channel via GitHub Releases for the Sparky repo (decision: same repo as Polterware or a new one — ask Erick), install/launch
test on a clean macOS user account (fresh data dir): first-run permission prompt, login item, reminders after reboot, update
flow (if signing allows; otherwise document the manual update path honestly), uninstall notes (login item + data).
**Acceptance.** Artifacts built by CI from a tag; clean-account smoke checklist passed and recorded; size and startup numbers
recorded; `sparky doctor` is clean on the installed app.

## W3-CONVERGE — remove duplication between the two apps (optional if time is short)

**Owns.** `packages/frontend/shell/**` (new) and edits to `packages/features/shell` + `packages/features/sparky-shell`.
**Do.** Extract the title bar, sidebar, main-surface and Commander frame that were copy-adapted in W2-SHELL into a shared
package consumed by both shells; decide the final package scope (D7) and rename if Erick wants a neutral scope.
**Acceptance.** Both apps' tests, typecheck and `design:lint` green; before/after screenshots show no visual change in
Polterware; duplicated CSS removed.

## W3-CLEANUP — close the plan

**Owns.** `plans/sparky-next/**` (deletion), docs/README updates.
**Do.** Add a **short** "Sparky" section to the root `README.md` (what it is, how to run `pnpm dev:sparky`, where the code lives)
and update `AGENTS.md` product boundary text to mention Sparky as a second app (**ask Erick** before editing AGENTS.md). Move any
lasting decision to code comments/ADR-style notes where the repo already records decisions (the Polterware store), delete
`plans/sparky-next/`, delete spike directories (`spikes/**`) after extracting what's worth keeping into tests, delete merged
`sparky/*` branches and worktrees.
**Acceptance.** Repo has no leftover spikes/plan files; `bash tooling/verify-sparky.sh` and Polterware `verify.sh` green.

---

## Gate G3 — release checklist

- [ ] G2 checklist still green on the release candidate.
- [ ] Security status: no open High/Critical; Mediums resolved or accepted.
- [ ] Clean-account install test passed (reminders fire after reboot with Sparky never opened).
- [ ] Performance budgets met (report attached).
- [ ] Gallery + screenshots approved by Erick; icon variants approved.
- [ ] Docs: root README section; `sparky doctor` documented via `--help`.
- [ ] `plans/sparky-next/` removed (W3-CLEANUP).

# Verification matrix (what "green" means)

| Check | Command | Runs in |
|-------|---------|---------|
| Contracts | `cd sparky/schemas && buf lint && buf generate && buf breaking --against .buf/baseline.binpb` | verify-sparky |
| Go | `cd sparky/backend && go vet ./... && go test ./... -race -count=1` | verify-sparky |
| Go boundaries | `bash tooling/check-sparky-boundaries.sh` | verify-sparky |
| TS typecheck + unit | `pnpm --filter "@polterware/*sparky*" --filter @polterware/frontend-design-system --filter @polterware/gallery typecheck test` | verify-sparky |
| Design lint | `pnpm design:lint` (and `pnpm test:ui-opacity`) | verify-sparky |
| Polterware non-regression | `pnpm typecheck && pnpm test` | verify-sparky (full) |
| Notifier | `xcodebuild test` / `swift test` in `apps/sparky-notifier` | W1-NOTIFIER, G1 |
| E2E | `pnpm test:e2e:sparky` | W3-E2E, G2/G3 |
| Supply chain | `pnpm audit --prod`, `govulncheck ./...` | W3-SECURITY |
