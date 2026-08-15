# Contract: Share vs Shell Ownership Boundary

**Feature**: `001-shared-ui-package`  
**Status**: Draft for implementation  
**Audience**: Contributors deciding where new UI code lives

## Rule (constitution)

```text
Share design system + domain feature views.
Specialize app shells and authority adapters.
```

This feature implements the **design system** half. Domain feature-view extraction is deferred until a second consumer exists.

## Decision table

| If the code is… | Put it in… | Examples |
| --- | --- | --- |
| Generic control/material with no product privilege | `@polterware/ui` | Button, Dialog, Tabs, Card, TextureButton, Empty, Field |
| Design token or shared surface utility | `@polterware/ui` styles | CSS variables, `polterware-glass-surface`, radius tokens |
| Hook only needed by shared primitives | `@polterware/ui` | `use-mobile` for Sidebar primitive |
| Desktop window chrome / dock / app-region | `apps/desktop` shell | `shell.css`, window controls, dock mechanics |
| Feature workflow composition | `apps/desktop/.../features/<feature>` | process boards, commander dialog wiring, repo sidebar logic |
| Calls `window.polterware`, IPC, FS, PTY, secrets | App adapter / main / future Rust | never package |
| Pure presentational piece embedded in privileged feature | Split: presentation → package candidate; privilege stays app | presentational row + run action adapter |
| Domain view with only one consumer | Keep in Desktop feature for now | system map screens, pipeline builder |
| Marketing/presentation deck UI | `apps/presentation` (out of scope) | Remotion slides |

## Split pattern for mixed modules

When a module mixes pure UI and privilege:

1. Extract pure presentational component into `@polterware/ui` **only if** reuse is real or it is already a design primitive.
2. Keep side effects, persistence to privileged stores, and control-plane calls in the feature/adapter.
3. Pass data and callbacks in via props.

Do **not** move mixed modules wholesale “for cleanliness.”

## Temporary shims

Allowed:
- `apps/desktop/src/renderer/components/ui/*` re-exporting `@polterware/ui`
- `apps/desktop/src/renderer/lib/utils.ts` re-exporting `cn`

Not allowed as end state:
- Full duplicate component implementations in Desktop and package

## Review checklist for new PRs

- [ ] New primitive? Prefer `@polterware/ui` if browser-safe and generic.
- [ ] New shell chrome? Desktop/Web app only.
- [ ] New feature screen? Feature folder; compose package primitives.
- [ ] Any Node/Electron import in `packages/ui`? Reject.
- [ ] Token change? Update package styles + DESIGN.md when product-visible.

## Success signal (SC-006)

A contributor can answer “shared package or app?” in under five minutes using this table plus `docs/ARCHITECTURE.md` / `CONVENTIONS.md` pointers updated during implementation.
