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
| Filters | `FilterChip` or `FilterPopover` |
| Operational collection | compound `DataTable` |
| Status | filled `StatusChip` |
| Entity details | global compound `DetailPanel` |
| Domain identity | `DitherIconTile` |

Product features must not reproduce focus management, Escape behavior, outside dismissal, portals, positioning, or presence when the shared Ark-backed primitive owns it.

## Collections and details

Operational collections use `DataTable`, sortable data columns, no actions column, and one explicit Hugeicons domain icon in the primary cell. Long descriptions and secondary metadata belong in the detail panel.

Entity detail uses the single bottom `DetailPanel` hosted by `MainSurface`. It has exactly three modes: collapsed strip, half-height open, and maximized. Features do not create a second inspector, detail route, drawer, or modal for entity content.

Knowledge documents use document layout and the shared Markdown editor. Operational details use section layout and the shared grid. Feature-owned rounded section cards are not structural detail chrome.

## Surface and color rules

Structural fills are opaque shared neutral tokens. Text and labeled icons use opaque semantic colors, with icons inheriting `currentColor`. Do not add ad-hoc `rgba`, alpha hex, Tailwind opacity modifiers, `color-mix(... transparent)`, glass, blur, or translucent product chrome.

Use only these radius roles:

- controls: `--radius-control`
- menu rows: `--radius-item`
- nested cards: `--radius-card`
- host surfaces: `--radius-surface`
- pills, avatars, and dots: `rounded-full`

Use `control-field`, `.surface-panel`, `.surface-panel-float`, `.menu-panel`, and `Card` for shared edge-light chrome. Do not hand-roll border, background, and radius recipes.

## Fixture honesty

Every screen must visibly communicate fixture or preview behavior where execution could be inferred. Terminal and integration CLI transcripts use the same terminal visual primitive. Their controls may move through visual states but never claim a real command, login, connection, or synchronization.

Run `pnpm design:lint` after UI changes. `tooling/check-ui-opacity.mjs` is the enforcement source; update this document and the Constitution before broadening its allowlist.
