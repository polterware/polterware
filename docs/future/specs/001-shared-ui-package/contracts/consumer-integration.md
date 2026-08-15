# Contract: Consumer Integration (Desktop now, Web later)

**Feature**: `001-shared-ui-package`  
**Status**: Draft for implementation

## Desktop (required MVP consumer)

### Dependency

```json
{
  "dependencies": {
    "@polterware/ui": "workspace:*"
  }
}
```

Implemented in `apps/desktop/package.json`.

### Styles

Desktop renderer global stylesheet MUST load package design-system styles after/with Tailwind bootstrap, conceptually:

1. Tailwind / animation / font imports (app-owned toolchain)
2. `@polterware/ui` design-system CSS entry
3. Desktop-only leftovers if any
4. `shell.css` remains separate Desktop chrome ownership

Implemented in `apps/desktop/src/renderer/styles.css` via `@import "@polterware/ui/styles.css";`.

### Component imports

Preferred end state:

```ts
import { Button, Dialog, Tabs, cn } from "@polterware/ui";
```

Implemented: Desktop features import `@polterware/ui` directly. Local `apps/desktop/src/renderer/components/ui` no longer holds primitive implementations.

### Bundler

Electron Vite renderer MUST resolve the workspace package (source export) consistently with other workspace packages. If bundling issues appear, apply the same class of fix already used for `@polterware/core` (resolve real paths / inline workspace package as needed) without moving UI into main process.

### shadcn / codegen

New design primitives SHOULD be added under `packages/ui`, not recreated under `apps/desktop/src/renderer/components/ui`. Update `components.json` paths/aliases accordingly during implementation.

### Parity surfaces

After integration, these Desktop surfaces MUST remain usable without intentional UX regression:
- Commander overlays/dialogs
- Settings (including frosted appearance controls composition)
- Process/command boards and builders
- Repository side panels
- System views that use shared primitives
- Shell composition that hosts shared primitives

## Web (readiness consumer, not full product)

### Minimum readiness bar

A future Web app MUST be able to:
1. Depend on `@polterware/ui`
2. Import primitives and `cn`
3. Load package CSS entry in a browser bundler
4. Render core controls without Electron/Node packages

### Explicit non-requirements for this feature

- No full Web app scaffold required to close MVP if Desktop consumption + boundary checks pass and package has no desktop-only imports
- No Web routing, auth, or governance features
- No claim that Web edition is live runtime

### Proof options (any one is enough for SC-003)

- Minimal documented import smoke in quickstart (type-level / isolated consumer notes)
- Or a small future `apps/web` dependency stub when Web work starts
- Or package-level browser-safe compile check proving no Electron types required

## Anti-patterns

| Anti-pattern | Why forbidden |
| --- | --- |
| Package imports `electron` | Breaks Web + security boundary |
| Desktop keeps full primitive copies “just in case” | Dual source of truth |
| Feature imports deep into package internals outside public exports | Unstable contract |
| Putting IPC calls inside shared Button/Dialog wrappers | Privilege leak |
| Moving shell dock CSS into package | Force-sharing shell |

## Acceptance mapping

| Spec item | Integration proof |
| --- | --- |
| FR-002 / SC-001 | Desktop uses package; no parallel primitive tree |
| FR-003 / SC-003 | Package browser-safe and documented for Web |
| FR-004 / SC-004 | Static forbidden-import scan clean |
| FR-008 / SC-002 / SC-005 | Workbench parity + DESIGN materials intact |
| FR-009 | Workspace package dependency wired |
