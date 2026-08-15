# Data Model: Shared UI Package

**Feature**: `001-shared-ui-package`  
**Date**: 2026-07-17

This feature has no runtime business database. The “model” is the ownership and composition model of the shared design system.

## Entities

### Shared UI Package

| Field | Description |
| --- | --- |
| Name | `@polterware/ui` |
| Location | `packages/ui` |
| Safety class | Browser-safe |
| Consumers | Desktop renderer (now), Web client (future) |
| Non-consumers | Electron main/preload, Node services, Rust cores |

**Validation**:
- MUST NOT import Electron, Node privileged modules, FS, exec, PTY, secrets stores, or `window.polterware`.
- MUST export only intentional public surface (primitives, helpers, styles).

### Design Primitive

Reusable low-level control or surface.

| Attribute | Description |
| --- | --- |
| Identity | Component name (e.g. Button, Dialog, Tabs) |
| Category | Action, Input, Overlay, Navigation, Feedback, Layout, Display, Material |
| Dependencies | Other primitives / hooks / CSS only |
| Privilege | None |
| Public | true/false (default true for current extracted kit) |

**Rules**:
- Pure presentation + interaction within browser APIs only.
- May compose other primitives.
- Must use shared tokens/materials (radius, surfaces, overlay patterns) per DESIGN.md.

### Design Token / Material Contract

| Attribute | Description |
| --- | --- |
| Tokens | Color, radius, typography, signal colors, sidebar tokens, glass/frosted bases |
| Materials | `polterware-surface`, `polterware-surface-muted`, `polterware-glass-surface`, `polterware-frosted-surface`, texture-button materials |
| Owner | Shared UI package CSS entry |
| Consumers | Package primitives + app shells that opt into product look |

**Rules**:
- Single source of truth after extraction.
- Desktop shell may add chrome-only CSS, not fork token meanings.
- Token changes update DESIGN.md when product-visible values change.

### App Shell

| Attribute | Description |
| --- | --- |
| Surface | Desktop (live), Web (future) |
| Owns | Navigation IA, window/chrome, dock, layout frames |
| Consumes | Shared primitives + tokens |
| Privilege | Surface-specific adapters only outside package |

**Rules**:
- Shells specialize; they do not reimplement the design kit.
- Desktop keeps `shell.css` and workbench frame ownership.

### Authority Adapter

| Attribute | Description |
| --- | --- |
| Examples | `window.polterware` bridge, IPC handlers, future Web API client |
| Location | App-owned, never in `@polterware/ui` |
| Relation | Features call adapters; primitives stay dumb |

### Domain Feature View

| Attribute | Description |
| --- | --- |
| Examples | Process boards, commander dialog composition, repository sidebar, system views |
| MVP location | `apps/desktop/src/renderer/features/*` |
| Share trigger | Second real consumer exists |
| Relation | Composes Design Primitives; may later move to `feature-*` packages |

### Consumer Surface

| Attribute | Description |
| --- | --- |
| Desktop renderer | Required MVP consumer; full parity |
| Web client | Contract consumer; readiness without full product build |
| Presentation | Out of scope |

## Relationships

```text
Consumer Surface
  ├── owns App Shell
  ├── owns Authority Adapters
  ├── owns Domain Feature Views (until multi-consumer)
  └── depends on Shared UI Package
        ├── Design Primitives
        ├── helpers (cn, primitive hooks)
        └── Design Token / Material Contract
```

## State / Lifecycle (package extraction)

Not user entity lifecycle; extraction progress states for implementation tracking:

| State | Meaning |
| --- | --- |
| `local-only` | Primitive exists only under Desktop |
| `moved` | Implementation lives in `@polterware/ui` |
| `shimmed` | Desktop path re-exports package (temporary) |
| `consumed` | Desktop imports package path; no local copy |
| `verified` | Typecheck/tests/parity checks pass for consumers |

**Allowed transitions**:
`local-only → moved → (shimmed)? → consumed → verified`

**Terminal MVP goal**: all extracted primitives in `consumed` or `verified`; zero `local-only` duplicates for extracted set.

## Validation rules (cross-entity)

1. If a module needs privilege → cannot be Design Primitive in package; split or keep in App Shell / Feature / Adapter.
2. If a module is shell chrome → App Shell, not package.
3. If a module is domain workflow UI with one consumer → Domain Feature View stays Desktop for MVP.
4. If a module is generic control used across features → Design Primitive candidate for package.
5. Token/material class names used by primitives must resolve from package style entry after migration.

## Non-entities (explicit)

- Systems, Targets, Operations, Approvals, Audit Events — unchanged domain model, not part of this feature’s data.
- npm registry publication metadata — not required for MVP.
- Web route/page model — out of scope.
