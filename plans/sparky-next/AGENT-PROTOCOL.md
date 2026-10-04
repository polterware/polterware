# Agent protocol

Applies to every agent executing a workstream (WS). Read it fully before touching anything.

## 1. Setup (once per WS)

The integration branch is `sparky-next`. W0-SCAFFOLD's owner creates it from the **current HEAD of `feat/atlas-grounded-sdd`**
(that branch is 28 commits ahead of `main` and carries the latest design system; confirm with Erick if `main` has moved).

```bash
# from the repo root: /Users/erickpatrickbarcelos/codes/polterware-total/polterware
git fetch --all
git worktree add ../polterware-wt/<ws-id> -b sparky/<ws-id> sparky-next   # e.g. sparky/w1-store
cd ../polterware-wt/<ws-id>
pnpm install            # TS workspaces
```

One worktree and one branch per WS. Never work in the main checkout. Never reuse a branch across WS.

**Plan availability.** `plans/sparky-next/` must be the **first commit** of `sparky-next` (`git add plans/sparky-next && git
commit -m "sparky: add Sparky Next migration plan"`), otherwise worktrees will not contain it. Until then read it from the
main checkout path `/Users/erickpatrickbarcelos/codes/polterware-total/polterware/plans/sparky-next/`. Each WS writes its own
`status/<ws-id>.md` and `requests/*.md` inside its worktree; the integrator merges them.

## 2. Ownership rules (prevents merge conflicts)

- Each WS lists **Owns** paths. You may create/edit/delete files only inside them.
- **Never edit**: other WS paths, `/Users/erickpatrickbarcelos/codes/sparky-workspace/**` (reference, read-only), Polterware product
  code outside what W1-DS explicitly owns.
- **Shared files** (root `package.json`, `pnpm-workspace.yaml`, `turbo.json`, `go.work`, `tooling/*`, `.github/*`, `AGENTS.md`,
  `apps/sparky-desktop/electron.vite.config.ts`, `apps/sparky-desktop/src/renderer/features.ts`) are owned by **W0-SCAFFOLD**.
  Need a change? Write `plans/sparky-next/requests/<ws-id>-<n>.md` (what, why, exact diff) and **stop that sub-task**; the
  integrator applies it. Do not edit the file yourself.
- Feature packages, Go packages and aliases for **every** planned package already exist as stubs after W0-SCAFFOLD, so no WS
  needs to touch shared config to add its own code.
- Contracts (`sparky/schemas/**`, `sparky/backend/platform/gen/**`, `packages/sparky-contracts/src/gen/**`) are frozen at
  tag `sparky-contracts-v1`. Changing them requires a request file and Erick's OK, then a `buf breaking` baseline update.

## 3. Execution freedom (overrides AGENTS.md for this plan)

Allowed: git worktrees, branches, commits, builds, tests, dev servers, the browser/Playwright, Electron launch, `buf`, `go`,
`pnpm`, `bun` (only for reference scripts), `xcodebuild`/`swift build` for `apps/sparky-notifier`.
Forbidden: force-push, `git reset --hard` or `clean -fd` outside your worktree, deleting outside your Owns, global installs,
committing secrets or `.env`, sending real personal data to any network service, editing the reference Swift/Bun repos.

## 4. Commits and merges

- Commit early, in small steps. Format: `sparky(<ws-id>): <imperative summary>` (English, no trailing period).
- Run the WS **Verify** commands before the final commit. Then write `plans/sparky-next/status/<ws-id>.md` (template below)
  and commit. **Do NOT push anything (no branches, no tags)**: the integrator pushes when Erick says so. Create a tag only when your WS says so, and only locally.
- The integrator merges with `git merge --no-ff sparky/<ws-id>` into `sparky-next` in the order given in each wave doc,
  runs `bash tooling/verify-sparky.sh`, and only then starts dependants.
- If a merge conflicts, the **later** WS rebases onto `sparky-next` and fixes its own side. Never resolve by editing another
  WS's files.

## 5. Stop-and-ask conditions

Stop and write a request or ask Erick when: a contract change is needed; a shared file must change; a gate fails for a reason
outside your Owns; a spike result contradicts a decision in [00-decisions.md](00-decisions.md); a task needs credentials or a
paid service; an acceptance criterion cannot be met as written.

## 6. Engineering rules

Go
- `gofmt`, `go vet`, table-driven tests, no globals, errors wrapped with `%w`, context on every I/O call.
- `domains/**` import no `database/sql`, no `modernc.org/sqlite`, no `connectrpc.com`, no `net/http` (enforced by
  `tooling/check-sparky-boundaries.sh`). Repositories are interfaces declared in the domain package.
- Times are UTC `time.Time` in code, RFC 3339 with milliseconds in SQLite and on the wire; wall-clock math always takes an
  explicit `*time.Location`. IDs are ULIDs.
- Never write to SQLite outside `providers/sqlite`. Every write goes through a domain service (so `rev`, `activity` and the
  change stream stay correct).

TypeScript / Solid
- Solid only (no React). UI built **only** from `@polterware/frontend-design-system` primitives; do not hand-roll chrome,
  menus, dialogs or tables ([DESIGN.md](../../DESIGN.md) is law; run `pnpm design:lint`).
- Opaque surfaces, token radii/colors only, Hugeicons Free Stroke Rounded only. No `rgba`, alpha hex, Tailwind opacity modifiers.
- The renderer reaches the core **only** through `window.sparky` (typed preload bridge). No Node or Electron APIs in
  `packages/**` (`tooling/check-ts-boundaries.sh`).
- Features never deep-import other features; they communicate through `@polterware/sparky-contracts` types, the shell
  registry and `window.sparky`.
- Tests next to the code (`vitest`); pure logic gets table tests; components get behavior tests, not snapshots.

All
- Execution honesty (DESIGN.md): every surface says whether something is live, queued, armed, failed, unavailable or
  preview-only. Never imply a reminder is armed when it is not.
- No new Markdown outside `plans/sparky-next/` except package-level `README.md` files that a WS lists in its deliverables.

## 7. Definition of done (every WS)

1. All acceptance criteria in the WS section are met and demonstrable by a command or test.
2. Verify commands pass in your worktree.
3. No file outside Owns changed (`git diff --stat sparky-next...HEAD` shows only owned paths + the status file).
4. Status file written and committed locally (nothing pushed).

### Status file template (`plans/sparky-next/status/<ws-id>.md`)

```markdown
# <ws-id> status
- Result: done | partial | blocked
- Branch / last commit:
- Acceptance criteria: (each criterion -> pass/fail + evidence command or test name)
- Deviations from the plan (and why):
- Requests filed (plans/sparky-next/requests/...):
- Follow-ups for other WS:
```
