# Contract: `@polterware/ui` Public API

**Feature**: `001-shared-ui-package`  
**Status**: Draft for implementation  
**Audience**: Desktop/Web consumers and package maintainers

## Package identity

| Key | Value |
| --- | --- |
| Package name | `@polterware/ui` |
| Workspace path | `packages/ui` |
| Safety class | Browser-safe React design system |
| Module type | ESM / TypeScript source-first (monorepo) |

## Required exports

### JS/TS module: `@polterware/ui`

MUST export the stable design primitives and helpers needed by Desktop after extraction, including at least:

- Utility: `cn`
- Core actions/inputs: `Button`, `Input`, `Textarea`, `Label`, `Checkbox`, `Switch`, `Slider`, `Select` family, `Tabs` family, `Toggle` / `ToggleGroup`
- Overlays: `Dialog` family, `AlertDialog` family, `Sheet` family, `Drawer` family, `Popover`, `Tooltip`, `HoverCard`, `DropdownMenu`, `ContextMenu`, `Menubar` as currently used
- Surfaces/feedback: `Card` family, `Badge`, `Alert`, `Empty` family, `Skeleton`, `Spinner`, `Progress`, `Separator`
- Product materials already treated as primitives: `TextureButton`, `TextureCard` (and related named exports)
- Other primitives currently under Desktop `components/ui` that remain pure presentation after boundary review

Exact named export list MUST match moved modules so Desktop call sites need import-path changes only, not API renames, unless a rename is required to fix a privilege split.

### CSS entry

MUST provide a stylesheet entry consumable by apps, for example:

- `@polterware/ui/styles.css` (or package `exports` equivalent)

This entry MUST include:
- design tokens / CSS variables required by primitives
- shared surface utilities (`polterware-surface`, `polterware-surface-muted`, `polterware-glass-surface`, `polterware-frosted-surface` and related design-system utilities currently shared)
- co-located primitive material CSS required for public primitives (e.g. texture button materials)

Apps remain responsible for Tailwind pipeline bootstrap (`@import "tailwindcss"`, app fonts, app shell CSS).

## Non-exports (forbidden)

Package MUST NOT export or contain:
- Electron APIs
- Node privileged APIs (fs, child_process, net servers, etc.)
- PTY/terminal session controllers
- secret stores / credential vaults
- `window.polterware` or IPC catalogs
- Desktop shell layout as the only supported app frame
- Main/preload modules

## Peer / runtime expectations

| Concern | Contract |
| --- | --- |
| React | Peer dependency; single React instance per app |
| DOM | Browser or Electron renderer DOM |
| Styling | Consumer app loads package CSS entry + Tailwind as today |
| Icons | Primitives may depend on `@hugeicons/react` (or current icon approach) without pulling Electron |

## Compatibility guarantees (MVP)

1. **Semantic stability**: Component prop contracts stay compatible with pre-extraction Desktop usage unless a documented break is required for privilege split.
2. **Visual contract**: Materials/tokens continue to honor DESIGN.md (radius, glass overlays, frosted surfaces, density).
3. **No dual source**: After migration completes, Desktop must not keep a second full implementation tree of the same primitives.

## Versioning

Internal workspace package for MVP. Semantic versioning optional until external publish. Breaking public export removals require updating all in-repo consumers in the same change set.

## Verification

- Typecheck package and Desktop consumer.
- Static scan: zero forbidden imports under `packages/ui`.
- Desktop features importing public exports compile and existing tests pass.
- CSS entry resolves from Desktop styles entrypoint.

## Implemented export inventory (post-implement)

Package: `packages/ui` (`@polterware/ui`)

- Helper: `cn`, `useIsMobile`
- CSS: `@polterware/ui/styles.css`
- Component modules: accordion, alert, alert-dialog, aspect-ratio, avatar, badge, breadcrumb, button, button-group, calendar, card, carousel, chart, checkbox, collapsible, combobox, command, context-menu, dialog, direction, drawer, dropdown-menu, empty, field, file-tree, hover-card, input, input-group, input-otp, item, kbd, label, menubar, native-select, navigation-menu, pagination, popover, progress, radio-group, resizable, scroll-area, select, separator, sheet, sidebar, skeleton, slider, sonner, spinner, switch, table, tabs, textarea, texture-button, texture-card, toggle, toggle-group, tooltip
- Public surface: `export *` from each module via `packages/ui/src/index.ts`
