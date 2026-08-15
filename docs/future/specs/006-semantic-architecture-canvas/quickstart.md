# Quickstart Validation: Semantic Architecture Canvas

**Feature**: `006-semantic-architecture-canvas`  
**Purpose**: Manual + focused automated checks after implementation slices  
**Spec**: [spec.md](./spec.md) · **Plan**: [plan.md](./plan.md)

---

## Prerequisites

- Repo: `/Users/erickpatrickbarcelos/codes/polterware`
- Desktop + core: `pnpm dev:stack` (or `pnpm build:core` then stack)
- Core listening log expected (loopback session)
- Feature packages: `@polterware/feature-architecture`, `@polterware/frontend-canvas`

Do **not** claim complete if canvas still creates generic cards or silent untyped edges.

---

## Automated (domain)

From repo root (when implementing):

```bash
# Focused unit tests for this feature
pnpm exec vitest run packages/features/architecture/src/domain
pnpm exec vitest run packages/frontend/canvas/src
```

Expected:

- Registry search: postgres→database, rabbitmq→queue, next→web_application
- `suggestRelationshipTypes` returns ordered non-empty lists
- `canConnect` rejects notes / invalid self
- (When added) provisional cancel does not call save port
- Clipboard rewire still passes existing clipboard tests

Typecheck touched packages if available:

```bash
pnpm exec tsc -p packages/features/architecture --noEmit
pnpm exec tsc -p packages/frontend/canvas --noEmit
```

(Full monorepo test/build only when explicitly requested.)

---

## Manual scenarios

### M1 — Typed component create (P1)

1. Open Architecture → Canvas.
2. Click `+` (or press `N`).
3. **Expect**: dialog **Add to architecture**; **no** new card yet.
4. Search `api` → select **API**.
5. **Expect**: placement mode; preview follows cursor.
6. Click canvas → name `Payments API` → Enter.
7. **Expect**: card shows API semantics; status Saving→Saved (if core up).
8. Undo → node gone; Redo → node returns.

### M2 — Cancel create

1. Start add → choose type → place → Escape during name.
2. **Expect**: no node remains; no durable save for that create.

### M3 — Typed relationship (P1)

1. Two nodes exist.
2. Connect drag A→B.
3. **Expect**: popover with Suggested + Other; no silent edge alone without type path.
4. Confirm **Calls**.
5. **Expect**: edge labeled/typed; undo removes edge.

### M4 — Quick connect

1. Connect tool → choose type Calls.
2. Draw two edges.
3. **Expect**: both Calls without popover; Escape exits mode.

### M5 — Contextual edit

1. Select node → toolbar Open details / Rename / Duplicate / Delete.
2. Select edge → change type / reverse / delete.
3. Multi-select → duplicate preserves internal edge.

### M6 — Autosave soft-fail

1. Stop core or break endpoint.
2. Create typed node.
3. **Expect**: canvas remains; Save failed + retry; local node kept (no full-page “Something went wrong” replacing canvas).

### M7 — Detail round-trip

1. Pan/zoom to distinctive view; select node; Open details.
2. Back to Architecture.
3. **Expect**: viewport + selection (if valid) restored.

### M8 — Search & fit (P2)

1. Search known name → center + select.
2. Multi-select → fit selection.

### M9 — Environment filter (P2/P3)

1. Control labeled as environment scope.
2. Filter Production dims/hides non-matching.

### M10 — No engine regression

1. Pan, zoom, marquee, drag, minimap click still work as before semantic flows.

---

## Acceptance gate (feature complete)

Use the 30-item list in [spec.md](./spec.md) § Acceptance Criteria.  
**Dialogs-only is not complete.**

---

## Handoff

Next: `/speckit-tasks` to expand Slice A/B/C from [plan.md](./plan.md) into ordered tasks.  
Then `/speckit-analyze` before `/speckit-implement`.
