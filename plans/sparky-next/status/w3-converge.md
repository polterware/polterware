# w3-converge status
- Result: done
- Branch / last commit: `sparky/w3-converge` (from `sparky-next`); `795329b` extract, `c0cb777` consume, then this status commit
- Acceptance criteria:
  - Shared presentational chrome extracted to `@polterware/frontend-shell` → pass (`packages/frontend/shell`)
  - Both product shells consume it; duplicated CSS/components deleted → pass
  - D7 unchanged (`@polterware/*` scope kept; no rename) → pass
  - Zero visual change in Polterware at 800/1280/1920 → pass (pixel-identical)
  - Sparky shell at the same widths → pass (pixel-identical); Commander overlay 1.8% AA mismatch only
  - Both apps' package tests, typecheck, `design:lint` green; Polterware `pnpm test` only the 5 known architecture failures → pass
- Deviations from the plan (and why): D7 rename was optional and was not done. Unsigned Electron E2E was authorized; chrome comparison used a Vite renderer harness with Shell fixtures (task-allowed gallery/renderer path) because the chrome lives in the renderer packages. `polterware` CLI was not on PATH; no Work card applied.
- Requests filed: none (`tooling/check-sparky-boundaries.sh` already allows `feature-sparky-shell` → `@polterware/frontend-*`)
- Follow-ups for other WS: integrator must regenerate `pnpm-lock.yaml` (`pnpm install`) so the new workspace package `@polterware/frontend-shell` is linked. Do not commit lockfile from this WS.

## Extracted pieces (`@polterware/frontend-shell`)

Presentational, product-agnostic only:

- `ShellRoot` — column chrome root with optional glass wash (`--color-shell-glass`) and `html[data-shell-chrome="solid"]`
- `TitleBarFrame` — 44px drag region, 78px traffic-light inset, vibrancy wash
- `SidebarFrame` / `SidebarNavFrame` / `SidebarSection` / `SidebarItem` — frame + item/section slots (no routes, labels, or counts)
- `MainFrame` / `MainSurfaceFrame` — main column and rounded content host
- `CommanderFrame` / `CommanderBody` / `CommanderSearchField` / `PopoverSearchField` / `PopoverSearchHeader` — overlay Dialog wrapper and borderless search field

Product data, routes, labels, counts, bridges, and keyboard shortcuts stay in `@polterware/feature-shell` and `@polterware/feature-sparky-shell`. Polterware keeps `window-title-bar` / `pw-shell-*` / `pw-sidebar-*` class names. Sparky keeps `.sparky-titlebar` / `.sparky-sidebar` / `.sparky-main` overrides (`glass={false}`, opaque `--color-app-background`).

Deleted from Polterware after extraction: `title-bar.css`, `sidebar.css`, `main-surface.css`, `popover-search.tsx`, `popover-search.css`. Sparky `shell.css` no longer duplicates glass/titlebar chrome.

## LOC

| Area | Added | Removed | Net |
| --- | --- | --- | --- |
| Product shells (`features/shell` + `features/sparky-shell`) | 78 | 370 | −292 |
| New package `packages/frontend/shell` (src + tests + package files) | 729 | 0 | +729 |

Duplicated chrome removed from product packages: **370 lines** (CSS + popover search + layout wrappers). Shared package includes 5 chrome tests.

## Screenshots (`plans/sparky-next/status/w3-converge/`)

Harness: `harness/` (Vite + Shell fixtures). Compare: `pixel-diff.json`.

| Before | After | Result |
| --- | --- | --- |
| `before-polterware-800.png` | `after-polterware-800.png` | identical (0) |
| `before-polterware-1280.png` | `after-polterware-1280.png` | identical (0) |
| `before-polterware-1920.png` | `after-polterware-1920.png` | identical (0) |
| `before-sparky-800.png` | `after-sparky-800.png` | identical (0) |
| `before-sparky-1280.png` | `after-sparky-1280.png` | identical (0) |
| `before-sparky-1920.png` | `after-sparky-1920.png` | identical (0) |
| `before-sparky-commander-1280.png` | `after-sparky-commander-1280.png` | 1.82% pixels (overlay AA/shadow/caret; layout unchanged) |

Same Chromium (MCP) captured before and after so font AA matches. First Playwright-vs-MCP pair was discarded as a capture mismatch.

## Verification before the last commit

| Gate | Result |
| --- | --- |
| `@polterware/frontend-shell` test / typecheck | PASS: 5 tests |
| `@polterware/feature-shell` test / typecheck | PASS: 7 tests |
| `@polterware/feature-sparky-shell` test / typecheck | PASS: 101 tests |
| `pnpm design:lint` | PASS |
| `pnpm test:ui-opacity` | PASS |
| Whole-repo `pnpm typecheck` | PASS: 28 packages including `@polterware/frontend-shell` |
| `pnpm test` (Polterware) | 5 failed / 152 passed in `@polterware/feature-architecture` only (known: node-detail ×3, view-detail ×1, canvas host ×1) |
| `git diff --check` | PASS |
| `tooling/check-sparky-boundaries.sh` | PASS (no edit) |
| `tooling/check-ts-boundaries.sh` | PASS |

`pnpm-lock.yaml` was not committed. AGENTS.md was not changed (no Turbo inject to revert). Nothing pushed or tagged.
