# Quickstart Validation: Shared UI Package

**Feature**: `001-shared-ui-package`  
**Purpose**: Prove extraction + Desktop consumption + browser-safe boundary without claiming live Web runtime.

## Prerequisites

- Repo root install already works (`pnpm install` done by human if needed)
- Feature artifacts present under `specs/001-shared-ui-package/`
- Implementation landed per [plan.md](./plan.md) and contracts in [contracts/](./contracts/)

## 1) Package exists and typechecks

```bash
# from repo root
pnpm --filter @polterware/ui typecheck
```

Expected:
- `packages/ui` exists with package name `@polterware/ui`
- Typecheck exits 0

## 2) Boundary scan (no privilege in package)

```bash
rg -n "from ['\"]electron|from ['\"]node:|from ['\"]fs['\"]|child_process|node-pty|window\\.polterware|@polterware/core" packages/ui
```

Expected:
- No matches for Electron/Node privilege, PTY, or desktop bridge usage inside `packages/ui`
- (Adjust pattern only if false positives on comments/docs; code imports must stay clean)

Also confirm package source does not rely on Desktop `@/` path aliases.

## 3) Desktop depends on package

Check:
- `apps/desktop/package.json` lists `"@polterware/ui": "workspace:*"`
- Desktop renderer styles load package CSS entry
- No full duplicate primitive implementations remain under `apps/desktop/src/renderer/components/ui` (re-export shims only if temporary)

```bash
pnpm --filter @polterware/desktop typecheck
```

Expected: exits 0 after import rewires.

## 4) Behavioral parity (Desktop tests humans/agents may run when asked)

```bash
pnpm --filter @polterware/desktop test
```

Expected:
- Existing renderer/feature tests remain green
- Failures indicate incomplete import migration or accidental API/behavior drift

> Agents: only run if Erick asks in the current request; otherwise treat as human validation step.

## 5) Manual workbench parity checklist

With Desktop running (human):

| Surface | Check |
| --- | --- |
| Commander overlay | Opens/closes with shared dialog material; actions work |
| Settings | Controls render; frosted appearance still coherent |
| Process/command board | Cards, tabs, dialogs, texture actions intact |
| Repository panel | Shared inputs/menus/scroll areas work |
| System views | Shared primitives render with product surfaces |
| Global look | Radius 8px, glass/frosted materials, no dual visual kit |

## 6) Web readiness smoke (no full Web app required)

Prove package is browser-safe for a second consumer:

1. From a non-Electron TS file or documented consumer snippet, import:
   - `import { Button, cn } from "@polterware/ui"`
2. Confirm resolution does not require Electron types
3. Confirm CSS entry path is documented in [contracts/package-public-api.md](./contracts/package-public-api.md)

Expected: SC-003 satisfied by consumability, not by shipping Web product.

## 7) Ownership docs reachable

Read in under five minutes:
- [contracts/ownership-boundary.md](./contracts/ownership-boundary.md)
- `CONVENTIONS.md` / `docs/ARCHITECTURE.md` pointers updated during implementation

Expected: clear answer to “new primitive vs shell vs feature.”

## Success mapping

| Criterion | Quickstart step |
| --- | --- |
| SC-001 no parallel primitives | Step 3 |
| SC-002 operator task parity | Steps 4–5 |
| SC-003 second consumer readiness | Step 6 |
| SC-004 zero privileged imports | Step 2 |
| SC-005 visual/token consistency | Steps 3, 5 |
| SC-006 ownership clarity | Step 7 |
| SC-007 only justified package | Step 1 + repo tree review |

## Out of scope for this quickstart

- Building/distributing installers
- Scaffolding full Polterware Web
- Extracting domain feature packages
- Redesigning the visual language
