# Polterware Desktop Design

This document defines the active visual and interaction contract for the first version.

## Product frame

Polterware is a dense local operator workspace, not a generic dashboard. Prefer calm hierarchy, compact controls, opaque surfaces, restrained borders, and minimal decorative effects. Use the shared design tokens and primitives in `@polterware/frontend-design-system`.

The application opens on Architecture / Canvas. The project sidebar is fixed:

```text
SYSTEM
  Architecture
  Resources
  Knowledge

EXECUTION
  Operations
  Terminal

MANAGEMENT
  Agents
  Integrations

FOOTER
  Settings
```

The title bar contains history controls, one ghost project switcher, one environment kind switcher, Commander, and quick Terminal access. There are no unrelated title-bar actions.

## Shared primitives

| Need | Required primitive |
| --- | --- |
| Text, password, search | `Input` and `.control-field` |
| Multiline input | `Textarea` |
| Selection | `Select` |
| Labeled control | `Field` |
| Actions | `Button` |
| Nested section | `Card` |
| Elevated host | `Surface` |
| Dialog | `Dialog` or `A11yDialog` |
| Anchored surface | Ark-backed `Popover` |
| Compact menu | `MenuPanel` and `MenuItem` |
| Tabs | Ark-backed `Tabs` |
| Section toolbar | compound `PageToolbar` |
| Filters | `FilterChip`, `FilterPopover`, or `ActiveFilterBar` |
| Operational collection | compound `DataTable` |
| Status | filled `StatusChip` |
| Entity details | global compound `DetailPanel` |
| Domain identity | `DitherIconTile` |

Product features must not reproduce focus management, Escape behavior, outside dismissal, portals, positioning, or presence when the shared Ark-backed primitive owns it.

## Collections and details

Operational collections use `DataTable`, sortable data columns, no actions column, and one explicit Hugeicons domain icon in the primary cell. Long descriptions and secondary metadata belong in the detail panel.

Entity detail uses the single bottom `DetailPanel` hosted by `MainSurface`. It has exactly three modes: collapsed strip, half-height open, and maximized. Features do not create a second inspector, detail route, drawer, or modal for entity content. Open and collapsed layout persist across collection scope changes in the session; maximized does not (a scope change exits to the prior open or collapsed preference). Selection and target activation are independent of layout mode, except that activating a target opens a collapsed panel (maximized stays maximized). On DataTable collections, clicking empty scroll background collapses the panel; clicking a row opens it; double-clicking a row maximizes it.

Knowledge documents use document layout and the shared Markdown editor. Operational details use section layout and the shared grid. Feature-owned rounded section cards are not structural detail chrome.

## Surface and color rules

Structural fills are opaque shared neutral tokens. Text and labeled icons use opaque semantic colors, with icons inheriting `currentColor`. Do not add ad-hoc `rgba`, alpha hex, Tailwind opacity modifiers, `color-mix(... transparent)`, glass, blur, or translucent product chrome.

Exception — macOS Desktop window chrome only: the main window title bar and sidebar may use shared `--color-shell-glass` / `--color-sidebar-glass` washes so frosted Electron vibrancy shows through while keeping the quiet dark cast. Commander (in-app palette and global overlay) uses the same opaque `surface-panel-float` material as other product dialogs — no glass, blur, or vibrancy. All other product surfaces, text, dialogs, and cards stay opaque. Settings Advanced → Reduce transparency disables main-window vibrancy / frost and restores opaque title bar and sidebar. Do not extend this exception to Commander, other in-app dialogs, cards, or other platforms.

Use only these radius roles:

- controls: `--radius-control`
- menu rows: `--radius-item`
- nested cards: `--radius-card`
- host surfaces: `--radius-surface`
- pills, avatars, and dots: `rounded-full`

Use `control-field`, `.surface-panel`, `.surface-panel-float`, `.menu-panel`, and `Card` for shared edge-light chrome. Do not hand-roll border, background, and radius recipes.

## Execution honesty

Every screen must communicate whether an action is live, queued, completed, failed, unavailable, preview-only, or based on seeded demonstration data. Terminal and integration CLI sessions use the same terminal visual primitive but report their actual core-owned state. A bounded inventory sync must not imply a complete provider graph, and no local action may imply a remote deployment or Cloud operation.

Run `pnpm design:lint` after UI changes. `tooling/check-ui-opacity.mjs` is the enforcement source; update this document and the Constitution before broadening its allowlist.
