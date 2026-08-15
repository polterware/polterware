# Quickstart Validation Guide

**Feature**: `005-architecture-canvas`  
**Scope**: Architecture canvas engine + domain host + layout persistence  
**Related**: [plan.md](./plan.md) · [spec.md](./spec.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/)

> Agent policy: prefer static checks; do not run long `pnpm dev` / full dist unless the user asks (AGENTS.md).  
> Visual QA uses `/prototype` for composition density, not pixel-perfect Excalidraw or prototype clones.

---

## Prerequisites

- Checkout with `specs/005-architecture-canvas/` artifacts  
- pnpm workspace install available  
- Optional: polterware-core healthy for live layout RPC; fixtures cover empty/offline honesty  

---

## 1. Planning artifacts present

```bash
test -f specs/005-architecture-canvas/spec.md
test -f specs/005-architecture-canvas/plan.md
test -f specs/005-architecture-canvas/research.md
test -f specs/005-architecture-canvas/data-model.md
test -f specs/005-architecture-canvas/contracts/overview.md
```

**Expect**: All files exist; research resolves package split, hybrid render, viewport-in-body, no Excalidraw embed.

---

## 2. Document format contract

```bash
rg -n "formatVersion" specs/005-architecture-canvas/data-model.md specs/005-architecture-canvas/contracts/architecture-document.md
```

**Expect**: v1 document fields and migrate-from-legacy rules documented.

After implementation:

```bash
# package tests (example names — adjust to actual test files)
pnpm exec vitest run packages/frontend/canvas --passWithNoTests
pnpm exec vitest run packages/features/architecture --passWithNoTests
```

**Expect**: migrate legacy `{nodes,edges}` → v1; invalid JSON rejected without throw-crash.

---

## 3. Engine purity / browser-safety

```bash
rg -n "electron|node:fs|node-pty|ipcRenderer" packages/frontend/canvas/src || true
rg -n "from \"electron\"|from 'electron'" packages/features/architecture/src || true
```

**Expect**: No privileged imports in canvas engine or architecture feature.

---

## 4. Persistence path still Connect layout API

```bash
rg -n "updateLayout|getLayout" packages/frontend/data/src/index.ts apps/desktop/src/renderer/App.tsx
rg -n "body_json|UpdateLayout" schemas/polterware/local/v1/architecture.proto
```

**Expect**: Autosave uses existing RPC; no parallel FS layout store for P1.

---

## 5. Manual Desktop scenarios (when user runs app)

### 5.1 Creation and connection

1. Open Architecture → Canvas.  
2. Create API + Database.  
3. Connect with type `Reads from`.  
4. Undo once → edge gone; Redo → edge back.  
5. Wait for Saved indicator; restart app → graph restored.

### 5.2 Multi-select drag

1. Marquee ≥3 nodes; drag.  
2. Relative positions stable; one undo restores move.

### 5.3 Clipboard

1. Select 3 nodes with 2 internal edges; copy/paste.  
2. New ids; internal edges rewired; external edges not duplicated.

### 5.4 Detail restore

1. Pan/zoom to a node; select; open detail.  
2. Back → same viewport, selection, environment filter.

### 5.5 Invalid import

1. Import malformed JSON.  
2. Current document unchanged; recoverable error.

### 5.6 Large document (benchmark)

1. Load or generate 500 nodes / 1000 edges.  
2. Pan, zoom, drag selection — no multi-second freezes; no full remount thrash.

---

## 6. Design / product constraints

```bash
rg -n "inspector|ResizableSplit" packages/features/architecture/src || true
rg -n "@excalidraw" package.json packages/*/package.json packages/*/*/package.json 2>/dev/null || true
```

**Expect**: No permanent inspector on Architecture canvas; no `@excalidraw` product dependency.

Visual: quiet toolbar, sparse brand green, HTML nodes look technical (not stickies), edges muted unless evidence.

---

## 7. Typecheck / lint (changed surface)

When implementation exists (user-requested full verify):

```bash
pnpm typecheck --filter=@polterware/frontend-canvas --filter=@polterware/feature-architecture --filter=@polterware/desktop
# or repo turbo scripts as available
```

**Expect**: No new errors on touched packages.

---

## 8. Definition of done (pointer)

Match [spec.md](./spec.md) Global Acceptance Criteria 1–20 and recorded benchmark note before claiming complete.
