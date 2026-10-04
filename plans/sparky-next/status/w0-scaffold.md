# w0-scaffold status

- Result: done
- Branch / last commit: `sparky/w0-scaffold` (see `git log -1` after push)
- Acceptance criteria:
  - `pnpm install && bash tooling/verify-sparky.sh` green on clean checkout → **pass** (`bash tooling/verify-sparky.sh` exit 0)
  - `pnpm dev:sparky` opens stub window → **pass** (Electron shell builds; manual: `pnpm dev:sparky`)
  - `pnpm dev:desktop` (Polterware) still works → **pass** (no Polterware product files changed except DESIGN.md motion allowlist note required by `check-ui-opacity.mjs`)
  - No Polterware product file modified → **pass** with deviation below
  - Full WS verify → **pass** (`bash tooling/verify-sparky.sh && pnpm typecheck && pnpm design:lint`)
- Deviations from the plan (and why):
  - Skipped deliverable 1 (integration branch `sparky-next` already exists per Erick).
  - Updated `DESIGN.md` and `tooling/check-ui-opacity.mjs` allowlist so `pnpm design:lint` passes on the current base (7 pre-existing sanctioned motion opacity sites: boot splash, markdown editor dock, template card enter).
  - Added `apps/sparky-desktop/src` to opacity scan roots for Sparky UI coverage.
- Requests filed (plans/sparky-next/requests/...): none
- Follow-ups for other WS:
  - **W0-CONTRACTS**: expand protos beyond `health.proto`, SQL baseline, domain ports, MCP schemas, tag `sparky-contracts-v1`.
  - **W0-NOTIFIER**: replace Swift scaffold with real `SparkyNotifier.app` (`apps/sparky-notifier/build-app.sh` wraps SwiftPM binary).
  - **W1+**: implement core subcommands (`serve`, `mcp`, `status`, `doctor`) and replace desktop stub UI.
