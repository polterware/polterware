# Polterware Desktop Design

This document defines the active visual and interaction contract for the first version.

## Product frame

Polterware is a dense local operator workspace, not a generic dashboard. Prefer calm hierarchy, compact controls, opaque surfaces, restrained borders, and minimal decorative effects. Use the shared design tokens and primitives in `@polterware/frontend-design-system`.

The application opens on Inbox. The project sidebar is fixed:

```text
Inbox
Work
Atlas
Decisions
Reality

FOOTER
  Settings
```

Settings is a dialog with a grouped rail: Project (General, Environments), Agents (Access, Sessions, Context), Integrations (Connected, Catalog), and App (Shortcuts, Advanced).

The title bar contains history controls, one ghost project switcher, one environment kind switcher, Commander, and a Help menu. There are no unrelated title-bar actions.

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
| Loading placeholder | `Skeleton` / `PageSkeleton` (via `PageLoading`) |

Product features must not reproduce focus management, Escape behavior, outside dismissal, portals, positioning, or presence when the shared Ark-backed primitive owns it.

## Collections and details

Operational collections use `DataTable`, sortable data columns, a narrow unsortable final actions column when row commands are needed, and one explicit Hugeicons domain icon in the primary cell. Long descriptions and secondary metadata belong in the detail panel.

Static reference tables (for example the CLI reference options, commands, and keys) also use `DataTable`, with `sortable={false}` columns, no row activation, no primary-cell icon, and wrapping description cells. The operational-collection rules above do not apply to them.

Entity detail uses the single lateral `DetailPanel` hosted by `MainSurface`, inside the central surface on its right side. `DetailPanel.CollectionHeader` mounts collection controls in a full-width upper band: `PageToolbar.Root` and `ActiveFilterBar` join it automatically inside the panel and render normally outside it. Toolbar padding is `8px 8px 4px`; wrapping controls and active filters determine the header's natural height. A separate workspace below contains the collection, sidebar and divider. Opening, closing, resizing or expanding details never changes the header width or disables its controls. The opaque, rounded panel reserves a column beside the collection; closing returns that width. The default lateral width is 400px, with a 320px minimum and 720px maximum, fitted to the available workspace. Its 8px divider supports direct pointer resize, cancellation, keyboard arrows (16px; Shift 64px), Home/End limits and double-click reset. A separate numeric sidebar-width key stores the preference; temporary host changes never rewrite it. Only old numeric widths migrate from the original key; floating geometry stays intact. Collection toolbars order search, filters and auxiliary actions before creation actions, with the primary creation action last; visual and keyboard focus order match.

Details start closed. Only explicit entity activation, including keyboard activation and entity navigation, opens them; changing collections closes them while preserving a valid selection. There is no toolbar opener or toggle slot. Cmd/Ctrl+J closes open details, and the expansion shortcut also requires an open panel. The detail header contains only close and expand/restore controls. Titles, metadata and entity actions remain in the scrollable content with 20px lateral padding. Expanded details occupy only the workspace below the collection header and restore to the saved lateral width. Escape cancels a resize, restores expansion, or returns focus to the selected item. Outside dismissal starts only on a completed click, including keyboard-generated clicks, preserving the clicked action and focus. Press-and-hold, movement of at least 3px, scrolling and cancelled gestures do not dismiss; tracking never takes pointer capture from the collection or Atlas. Closing disables panel input immediately and animates a 16px rightward translation and fade with the 160ms `spring.slow.exit`. The departing frame retains its lateral or expanded geometry; its lateral reservation releases together with hiding the frame at the end of the exit, never on pointer release. Reduced motion removes travel and keeps the 60ms `spring.fast.exit` fade. Reopening interrupts the exit from the current pose, and stale completion callbacks cannot hide reopened details. Panel content, registered owned portals including callback confirmations, and common `data-detail-opener` activators are exempt. Empty, loading and error states retain controls; unavailable collections hide them. Features do not create another entity inspector, detail route, drawer or modal.

`FloatingWindow` is a reusable design-system compound component, independent of entity selection and shell shortcuts: `Root` defines the boundary and controlled `open`, `minimized`, `expanded` state with `onOpenChange`, `onMinimizedChange`, and `onExpandedChange`; `Workspace` holds the background; `Content` mounts arbitrary window content; optional `Header` customizes chrome styling. Shared controls exist without content, with unique per-instance IDs. No product surface currently mounts a floating window. The existing geometry in `polterware.shell.detail.width` remains available for a future caller's optional `storageKey`. Without a key, geometry remains session-local.

The floating window preserves its opaque raised shell, shared radius, subtle rim and soft lift; default 520 × 420px at bottom right with a 16px margin, minimum 320 × 280px fitted to smaller hosts. Four corners and four edge midpoints are the eight docks; ties retain the current dock. Card and 48px minimized circle share the dock. Dragging follows freely and settles to the nearest possible component position with `spring.moderate`; four edges and four corners resize via 8px targets. Versioned geometry persists dock and dimensions, accepting old numeric widths and normalized anchors; host changes fit without rewriting preferences. Red closes, yellow minimizes, green expands/restores. The same yellow button becomes the draggable reopen circle, preserving focus. Closing uses the fast exit and leaves no circle; outside activation closes without consuming the action, excluding owned portals and `data-floating-window-opener` activators. Expansion disables gestures and restore recovers geometry. Arrow keys (16px; Shift 64px) move or resize, docking on release; Escape cancels gestures or restores expansion. Pointer capture, 3px dead zone, release-click suppression, rollback, reduced motion and cleanup remain shared behavior. `createDragMotion` and `motion` supply interruptible tracking and `spring.slow` morphs with unscaled content; the first paint does not animate.

Operational records use the record layout: `DetailPanel.Main` holds overview text, tables, and notes with 20px lateral padding; an optional `DetailPanel.Rail` holds `DetailPanel.Properties`, a label/value list where empty values are hidden (or offer an explicit add affordance). The rail stacks below the main column when the panel is narrower than 720px. Knowledge documents use document layout: title, inline properties and `DetailPanel.LinkedEntities` chips under it, then the shared Markdown editor. There is no shared span grid, and feature-owned rounded section cards are not structural detail chrome.

## Spacing and grouping

Separate sibling content groups with spacing, not divider lines. Do not add `border-t` / `border-b` rules or `<hr>` between entries in a list, section, or document; use a consistent gap scale instead (tighter inside a group, wider between groups). Borders stay on shared chrome — page headers, controls, surfaces, and tables — where they define an object, not a gap.

Reference and long-form reading pages (for example the CLI reference) use a single centered column (`mx-auto w-full max-w-3xl`) inside `PageContent`, with generous vertical padding. Long reference pages may add an "On this page" summary pinned to the left edge of `PageContent` (the reading column stays centered in the remaining space): sticky, hidden below `lg`, with the active item following the section in view. It uses the hover and selected surface tokens, not a rail line or divider.

## Surface and color rules

Structural fills are opaque shared neutral tokens. Text and labeled icons use opaque semantic colors, with icons inheriting `currentColor`. Do not add ad-hoc `rgba`, alpha hex, Tailwind opacity modifiers, `color-mix(... transparent)`, or translucent product chrome.

Exception — macOS Desktop window chrome only: the main window title bar and sidebar may use shared `--color-shell-glass` / `--color-sidebar-glass` washes so frosted Electron vibrancy shows through while keeping the quiet dark cast. Commander (in-app palette and global overlay) uses the same opaque `surface-panel-float` material as other product dialogs. All other product surfaces, text, dialogs, and cards stay opaque. Settings Advanced → Reduce transparency disables main-window vibrancy / frost and restores opaque title bar and sidebar. Do not extend this exception to Commander, other in-app dialogs, cards, or other platforms.

Use only these radius roles:

- controls: `--radius-control`
- menu rows: `--radius-item`
- nested cards: `--radius-card`
- host surfaces: `--radius-surface`
- pills, avatars, and dots: `rounded-full`

Use `control-field`, `.surface-panel`, `.surface-panel-float`, `.menu-panel`, and `Card` for shared edge-light chrome. Do not hand-roll border, background, and radius recipes.

## Execution honesty

Every screen must communicate whether an action is live, queued, completed, failed, unavailable, preview-only, or based on seeded demonstration data. Integration connect sessions report their actual core-owned state. A bounded inventory sync must not imply a complete provider graph, and no local action may imply a remote deployment or Cloud operation.

Motion may fade opacity only in transitions: presence keyframes in the shared popup, dialog, drawer, and toast CSS, and Motion values in shared design-system motion primitives (the fluid hover highlight and gliding indicators). Resting surfaces stay opaque.

Run `pnpm design:lint` after UI changes. `tooling/check-ui-opacity.mjs` is the enforcement source; update this document before broadening its allowlist.
