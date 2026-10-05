# w0-ds-audit status

- Result: done
- Branch / last commit: `sparky/w0-ds-audit` (see `git log -1` after this commit)
- Acceptance criteria:
  - Status file contains (a)–(f) with file paths and line references, enough for W1-DS to start without re-reading the design-system source → **pass** (this file)
  - WS **Verify** commands → **n/a** (the W0-DS-AUDIT section in `plans/sparky-next/07-wave0.md` lists no Verify line)
- Deviations from the plan (and why):
  - Did not append the O5 line to `plans/sparky-next/00-decisions.md`. That file is outside Owns (`plans/sparky-next/status/w0-ds-audit.md` only). The integrator pastes the line in [O5 resolution](#o5-resolution-for-the-integrator) at G0.
  - `06-ui.md` says four files mention `"polterware"` in strings/keys. That count is wrong. See (b). No storage key inside the design system contains `polterware`.
- Requests filed (plans/sparky-next/requests/...): none
- Follow-ups for other WS:
  - **Integrator (G0):** append the O5 line below to the resolutions log in `00-decisions.md`. Do not treat this audit as a contract change.
  - **W1-DS:** execute (f) in order. Do not rename existing `polterware.*` localStorage keys. Do not bump `@ark-ui/solid` (pinned `5.39.2` in `packages/frontend/design-system/package.json` and `pnpm-lock.yaml`); a lockfile edit is a shared-file request to W0-SCAFFOLD.
  - **W1-ELECTRON / W2-SHELL:** import the design-system entry so `installSpringCssProperties()` runs (`packages/frontend/design-system/src/index.ts` lines 4–6). Spring CSS variables are not declared in `base.css`.
  - **W2-MEMORIES:** `MarkdownEditor` has no tests (`packages/frontend/design-system/src/markdown-editor.tsx`). Do not assume editor behavior is covered.

## O5 resolution (for the integrator)

```text
2026-10-03 O5: Design-system coupling is three production imports of @polterware/frontend-state (table.tsx double-click calls detailPanelStore.expand; detail-panel.tsx hardcodes shortcut ids shell.toggle_detail and shell.maximize_detail; panels/resizable.tsx hardcodes shell.toggle_sidebar). Product default copy is PageOffline ("Polter cannot reach the local backend") plus the vendor map DITHER_BRAND_ACCENTS. No design-system file hardcodes a polterware.* storage key; those keys live at Polterware call sites. @ark-ui/solid@5.39.2 ships Checkbox, DatePicker (date only), NumberInput, Slider, ColorPicker (including SwatchGroup), TreeView, SegmentGroup, and Progress.Circle. It does not ship TimePicker. (W0-DS-AUDIT, plans/sparky-next/status/w0-ds-audit.md)
```

## Read set

Every `.ts` / `.tsx` / `.css` file under `packages/frontend/design-system/src` (111 files), plus `packages/frontend/design-system/panels/resizable.tsx` because `src/index.ts` lines 145–146 re-export it, plus `packages/configuration/tailwind/base.css` (359 lines). CSS modules were searched for `polter` / `Polter`; none match. `@ark-ui/solid@5.39.2` public types were read from the published package (`unpkg`, lockfile version), not from a local `node_modules` (this worktree had not installed it).

`@polterware/frontend-state` was read only to explain the three design-system imports (`src/index.ts` `detailPanelStore`; `src/shortcuts/catalog.ts` and `types.ts`). W1-DS owns that package; the notes below are the parts the design system actually calls.

---

## (a) Components and Polterware specificity

Legend: **no** = shared chrome, safe for Sparky unchanged. **defaults** = shared component whose default copy, tone names, or vendor map are Polterware-flavored; keep the defaults so Polterware does not change. **coupled** = behavior is wired to Polterware state or shortcut ids.

| Module | Public surface | Specific? | Evidence |
|---|---|---|---|
| `src/button.tsx` | `Button` | no | Variants `primary\|secondary\|ghost\|danger\|brand\|icon` (lines 5–6). No product nouns. |
| `src/badge.tsx` | `Badge` | no | Tones `ok\|warn\|danger\|muted\|info` (line 5). Children are the label. |
| `src/floating-count-badge.tsx` | `FloatingCountBadge` | no | Count chip only. |
| `src/card.tsx` | `Card`, `Surface` | no | `.card-panel` / `.surface-panel`. |
| `src/separator.tsx` | `Separator` | no | Layout rule. |
| `src/status.tsx` | `StatusDot`, `StatusChip` | defaults | Default words `Healthy`, `Degraded`, `Down`, `Syncing`, `Warning`, `Danger`, `Unknown` (lines 33–43). `StatusChip` uses `children`, then `label`, then that table (line 103). Sparky reminder chips must pass `children` (`armed`, `planned`, `notifier off`, `permission denied`). Tones already cover those states: `ok`/`healthy`, `info`, `muted`, `danger`. |
| `src/filter-mark.tsx` | `FilterMark` | no | Shapes `circle\|square\|diamond` (line 4). Not a checkbox. |
| `src/dither-icon.tsx` | `DitherIconTile`, `DITHER_BRAND_ACCENTS`, `resolveDitherIconAccent` | defaults | Tone union is Polterware atlas language: `architecture`, `infrastructure`, `data`, `execution`, `knowledge`, `identity`, `security`, `integration`, `observability` (lines 4–14). Accent map is vendor names (`aws`, `github`, `postgres`, …) lines 31–68. Aliases lines 83–90. |
| `src/template-card.tsx` | `TemplateCard`, `TemplateCardGrid` | no | Title, description, icon, and `onSelect` are props. |
| `src/feedback.tsx` | `PageLoading`, `PageEmpty`, `PageError`, `FixtureBanner`, `PageOffline`, `PageForbidden` | defaults | See the copy table under (b). `PageEmpty` already takes `title`, `description`, `action`, `children` (lines 9–14). |
| `src/skeleton.tsx` | `Skeleton`, `PageSkeleton` | no | Layout id `"canvas"` (line 5) is a full-bleed bone, not Atlas behavior. Default label `"Loading…"` (line 102). |
| `src/toast.tsx` | `ToastRegion`, `pushToast` | no | Module singleton (line 12). One renderer per app, so Sparky and Polterware do not share it. |
| `src/page.tsx` | `PageHeader`, `PageContent`, `PageToolbar` | no | `PageToolbar.Root` portals into `DetailPanelCollectionHeader` (lines 82–88). Comment on line 7 says "entity" as a layout word. |
| `src/toolbar-icon-button.tsx` | `ToolbarIconButton` | no | `label` is required from the caller. |
| `src/search-bar.tsx` | `SearchBar` | no | `placeholder` is required (line 6). |
| `src/tabs.tsx` | `Tabs`, `TabPanel` | no | Default `aria-label` `"Page sections"` (line 181). File comment lines 7–8: not for alternate collection views. Sparky list/table and calendar Day/Week/Month use the new `SegmentedControl`, not `Tabs`. |
| `src/drawer.tsx` | `Drawer` | no | Ark Drawer. Comments mention a terminal (lines 18–20); the API is generic. Sparky does not need it. |
| `src/popover.tsx` | `Popover` | no | Ark Popover. Comments say "Polter surface" (lines 6, 58). |
| `src/tooltip.tsx` | `Tooltip` | no | Ark Tooltip. |
| `src/kbd.tsx` | `Kbd` | no | Presentational `<kbd>`. Exists; do not rebuild. Callers pass the glyph (`⌘J`). |
| `src/springs.ts`, `src/spring-css.ts`, `src/reduced-motion.ts` | `spring`, `installSpringCssProperties`, `prefersReducedMotion` | no | Tiers `fast` 0.08s, `moderate` 0.16s, `slow` 0.24s (`springs.ts` lines 11–30). |
| `src/collapse.ts`, `src/drag-motion.ts`, `src/placement-motion.ts`, `src/popup-motion.ts`, `src/icon-swap.tsx`, `src/fluid-hover.tsx`, `src/panel-interaction.ts` | motion helpers | no | No product nouns. |
| `src/switch.tsx` | `Switch`, `SwitchRow` | no | Ark Switch. |
| `src/filter-popover.tsx`, `src/filter-options-panel.tsx` | `FilterPopover`, `FilterCategoryPopover` | no | Default placeholder `"Add filter…"` (`filter-popover.tsx` line 150). Default clear label `"Clear filters"` (line 300). |
| `src/filter-chip.tsx` | `FilterChip`, `FilterChipGroup` | no | `aria-pressed` selection. |
| `src/active-filter-bar.tsx` | `ActiveFilterBar` | no | Default `aria-label` `"Active filters"` (line 31). `clearLabel` is a prop. |
| `src/input.tsx`, `src/field.tsx` | `Input`, `Textarea`, `Field` | no | `.control-field` chrome. |
| `src/select.tsx` | `Select` | no | Built on Ark **Menu** (`dropdown-menu.tsx`), not `@ark-ui/solid/select` (lines 3–9, 17–19). New date/time pickers must not copy this pattern. |
| `src/menu.tsx`, `src/menu-highlight.tsx` | `MenuPanel`, `MenuList`, `MenuItem`, `SelectionCheck` | no | `SELECTION_ACK_MS = 300` (`menu-highlight.tsx` line 29). |
| `src/dropdown-menu.tsx`, `src/context-menu.tsx` | `DropdownMenu`, `ContextMenu` | no | Ark Menu. Comment "Polter rows" (`dropdown-menu.tsx` line 6). |
| `src/dialog.tsx` | `Dialog`, `A11yDialog` | no | Ark Dialog. Comments name Commander and Architecture (lines 15–18, 31, 42, 57). No product string is rendered. |
| `src/markdown-editor.tsx` | `MarkdownEditor` | no | TipTap. Toolbar labels are generic (`Bold`, `Bullet list`, … lines 110–160). Placeholder default `"Start writing…"` (line 204). `aria-label` default `"Document body"` (line 221). Comment lines 32–35 mention Knowledge; `showToolbar` is already a prop. |
| `src/detail-panel-properties.tsx` | `DetailPanel.Properties`, `.Property` | no | Empty values hide the row unless `onAdd` (lines 13, 36–38). |
| `src/detail-panel-primary-action.tsx` | `DetailPanel.PrimaryAction` | no | `label` is a required prop. |
| `src/detail-panel-menu.tsx` | `DetailPanel.MenuItem` (menu body is `DetailPanelMenu`) | defaults | Rendered strings `"More actions"`, `"Restore width"`, `"Expand details"` (lines 34, 43). Generic chrome, not Polterware nouns. |
| `src/detail-panel-linked-entities.tsx` | `DetailPanel.LinkedEntities` | defaults | Hardcoded `"Link entity"`, `"Search entities…"`, `"No matching entities."`, `` `Unlink ${title}` `` (lines 79, 104, 110, 113, 122). `linkLabel` overrides the button and popover title only (lines 98, 104). Sparky memory detail does not need this part. |
| `src/detail-panel.tsx` | `DetailPanel` | coupled | Imports `createShortcutLabel`, `matchesShortcut` (line 13). Close chord id `shell.toggle_detail` (lines 119, 230–232) **closes**; it does not open. Expand id `shell.maximize_detail` (lines 120, 228–229). Width defaults 400 / min 320 / max 720 (lines 16–18), matching `06-ui.md`. `storageKey` and `legacyStorageKey` are props (lines 42–44, 235–238, 427–431). Layout `"document"` comment says "knowledge writing canvas" (line 53); the mode itself is a markdown column. DOM id `pw-detail-panel-sidebar` (line 19) assumes one panel. |
| `src/table.tsx` | `DataTable` | coupled | Imports `detailPanelStore` (line 3). Row double-click calls `detailPanelStore.getState().expand()` (line 752) with no prop to turn it off. Selection toolbar defaults `"No shared actions"`, `"Clear selection"`, `"Select all visible rows"` (lines 874, 916, 926–927) are generic. |
| `src/floating-window.tsx`, `src/floating-window-geometry.ts` | `FloatingWindow` | no | `storageKey` is a prop (line 43). Labels are generic window chrome (`"Close window"`, `"Move window"`, line 521). A test forbids importing frontend-state (see (e)). Sparky does not need this (no canvas inspector). |
| `src/hugeicons.tsx` | curated Hugeicons set | defaults | `POLTER_ICON_STROKE_WIDTH = 1.5` (line 132). Type name `PolterIconName` (line 347). The set is the only product icon pack; Sparky `IconPicker` draws from it. Do not rename unless a call site requires it. |
| `panels/resizable.tsx` | `ResizableSidebar`, `ResizableSplit` | coupled | Imports `matchesShortcut` (line 5). Keydown calls `shell.toggle_sidebar` (lines 283–289). `storageKey` is a prop (line 33). Default width 232 (line 39) matches `--sidebar-width` in `base.css` line 116. |
| `src/index.ts` | package entry | no | Side effect: `installSpringCssProperties()` when `document` exists (lines 4–6). |

`createSelectionStore` in `packages/frontend/state/src/index.ts` lines 4–7 (`"architecture" | "ops" | "logs" | "settings" | "knowledge"`) is **not** imported by the design system. Sparky must not use it. Leave it alone unless a Polterware call site still needs it.

Shortcut ids the design system calls, and their default chords (`packages/frontend/state/src/shortcuts/catalog.ts`):

| Id | Catalog lines | Default chord | Design-system use |
|---|---|---|---|
| `shell.toggle_detail` | 33–37 | ⌘J | `detail-panel.tsx` 119, 230. Closes while open. |
| `shell.maximize_detail` | 39–43 | ⌘⇧J | `detail-panel.tsx` 120, 228. |
| `shell.toggle_sidebar` | 27–31 | ⌘B | `panels/resizable.tsx` 284. |

Those three chords already match Sparky (`06-ui.md`: ⌘J closes detail). The `architecture.*` catalog entries are not referenced by the design system. Overrides are persisted by id through `window.polterware.app` (`packages/features/settings/src/shortcut-persist.ts` lines 23–29), not through a `polterware.*` localStorage key. Do not rename the three shell ids; Polterware settings already store them.

---

## (b) Every `polter` / `Polter` reference

Case-insensitive search of `packages/frontend/design-system/**` and `packages/configuration/tailwind/base.css`.

| File | Line | What it is |
|---|---|---|
| `packages/frontend/design-system/package.json` | 2 | Package name `@polterware/frontend-design-system`. |
| `packages/frontend/design-system/package.json` | 21 | Dependency `@polterware/frontend-state`. |
| `packages/frontend/design-system/tsconfig.json` | 7 | Path alias `@polterware/*`. |
| `src/table.tsx` | 3 | Import `detailPanelStore` from `@polterware/frontend-state`. |
| `src/table.test.tsx` | 8 | Test import of `detailPanelDefaultState`, `detailPanelStore`, `createDetailPanelSelector`. |
| `src/detail-panel.tsx` | 13 | Import `createShortcutLabel`, `matchesShortcut`. |
| `panels/resizable.tsx` | 5 | Import `matchesShortcut`. |
| `src/floating-window.test.tsx` | 491 | Assertion that `floating-window.tsx` does **not** contain `@polterware/frontend-state`. |
| `src/feedback.tsx` | 71 | Default message `"Polter cannot reach the local backend. Cached data may be displayed."` |
| `src/hugeicons.tsx` | 132, 347 | Identifier `POLTER_ICON_STROKE_WIDTH`; type `PolterIconName`. |
| `src/dialog.tsx` | 7, 66 | Comments: "Polter tokens", "Polter surface". |
| `src/popover.tsx` | 6, 58 | Comments: "Polter surface", "Polter tokens". |
| `src/dropdown-menu.tsx` | 6 | Comment: "Polter rows". |
| `src/drawer.tsx` | 4 | Comment: "Polter surface". |
| `packages/configuration/tailwind/base.css` | 1 | Comment: "Polter quiet dark operator tokens". |

No file under `src/` or `base.css` contains a `polterware.*` storage key.

Polterware call sites that pass those keys into design-system props (do not rename; they are user preferences):

| File | Lines | Key | Prop |
|---|---|---|---|
| `packages/features/shell/src/main-surface.tsx` | 42 | `polterware.shell.detail.sidebar.width` | `DetailPanel` `storageKey` |
| `packages/features/shell/src/main-surface.tsx` | 43 | `polterware.shell.detail.width` | `legacyStorageKey` |
| `packages/features/shell/src/index.tsx` | 190 | `polterware.shell.sidebar.sizes` | `ResizableSidebar` `storageKey` |

Not design-system keys (do not touch from W1-DS except to leave them): `packages/features/architecture/src/canvas/type-prefs-local.ts` lines 4–5 (`polterware.arch.typeRecents`, `polterware.arch.typeFavorites`).

### Product default copy to parameterize (06-ui item 2)

| Component | File:lines | Current default | Change |
|---|---|---|---|
| `PageOffline` | `feedback.tsx` 57–78 | Title `"Local core offline"` (68). Message `"Polter cannot reach the local backend. Cached data may be displayed."` (71). Icon is the glyph `○` (64–66), not a prop. | Add optional `icon`. Keep both strings as defaults. |
| `PageForbidden` | `feedback.tsx` 82–108 | Title `"You do not have permission"` (94). Message is generic, or a `` `This action requires the \`${permission}\` permission.` `` template (97–100). Icon glyph `◈` (90–92). | Add optional `icon`. Keep the copy as defaults. This is not Polterware-named; it is still the component 06-ui lists. |
| `FixtureBanner` | `feedback.tsx` 49–54 | Accepts `class`, `visible`, `message` and **returns null**. | Leave it returning null. A visible banner would change Polterware. Props may stay for call-site compatibility. |
| `PageError` | `feedback.tsx` 34, 38 | `"Something went wrong"`, button `"Retry"`. | Already generic. No change required. |
| `DITHER_BRAND_ACCENTS` | `dither-icon.tsx` 31–68 | 35 vendor hexes. Tests lock five of them (`dither-icon.test.tsx` 92–96). | Registry apps can extend. Ship the current object as the default registry. Add an optional `accent` color prop so a Mind can pass a color without a vendor name. Do not change existing hexes. Do not rename tone union members; Polterware passes those tone strings. |

---

## (c) Tokens Sparky screens need

`base.css` is one dark theme (`:root { color-scheme: dark }` line 133). `--color-brand: #01dd96` already exists (line 39). D9 is an override of that variable at the Sparky app root. No new brand token. No light theme.

These Sparky roles are already covered. Do not add tokens for them:

| Screen role | Use |
|---|---|
| Reminder honesty (`armed`, `planned`, `notifier off`, `permission denied`) | `StatusChip` tones `ok`, `info`, `muted`, `danger` and the status color pairs (lines 64–74). |
| Overdue / warning | `--color-danger`, `--color-warning`. |
| Today column, selected row | `--color-surface-selected`. |
| Calendar grid lines | `--color-border-subtle` or `--color-chart-grid` (line 55). |
| Now line, focus accent, primary brand button | `--color-brand`. |
| Mind dither tiles | `--color-dither-*` (lines 58–62) plus `DitherIconTile`. |
| List complete (160 ms) | `--duration-control` (line 113). |
| Calendar view fade | `--duration-enter` (line 114) and `--ease-enter` (line 110). The 8 px shift is a length in the component, not a global token. |
| Controls, cards, overlays, focus ring | `--radius-*`, `--color-overlay`, `--color-shadow-*`, focus outline lines 354–357. |
| Reduced motion | Global `prefers-reduced-motion` block lines 343–351 (`!important` on durations). JS helpers must still call `prefersReducedMotion()`. |

Chart colors are **duplicated** (`base.css` lines 46–55 and 76–84). Do not add a third copy.

### Tokens that do not exist

1. **Heatmap ramp (required).** `--color-chart-*` (lines 47–53) is a categorical set, not a 5-step scale of one hue. Review's contribution calendar needs five **opaque** colors. Do not use Tailwind opacity modifiers or `rgba` (UI law; alpha is only the existing border, shadow, scrim, and edge-light tokens). Do not `color-mix` the brand: D9 says the app accent is only `--color-brand`, and a brand override must not repaint the heatmap. Add, inside `@theme` and again under the "always emit" `:root` block (Tailwind v4 drops unused theme vars; see the comment at lines 151–154):

   - `--color-heatmap-0` empty cell, near `--color-surface-active` (`#1e1e1e`)
   - `--color-heatmap-1` … `--color-heatmap-4` four opaque greens ending at `#01dd96` (`--color-brand`'s current value, copied as a hex, not `var(--color-brand)`)

   Polterware does not render a heatmap today, so the new variables are invisible there.

2. **Calendar layout variables (component-scoped, not global).** `CalendarGrid` should define these on its root, not in `base.css`: hour-row height, all-day lane min height, time gutter width. There is no calendar section in `base.css`.

3. **Progress ring size and stroke.** Component-local. Track = `--color-border-subtle`. Value = `--color-brand`. Ark `Progress.Circle` animates `stroke-dashoffset`, not opacity. That satisfies the opacity law. `06-ui.md` also says "transform-only"; stroke-dashoffset is the Ark mechanism and should be kept, with reduced motion jumping to the value. Do not invent a second ring.

### Not tokens

- Mind / occurrence color is data on the memory, not a token. Pill foreground on a light mind color can use `--color-dither-icon-mark` (`#121212`, line 61) or `--color-primary-foreground`.
- Priority maps to existing status tones (muted / warning / danger). No `--color-priority-*`.
- `EmptyIllustration` is a slot, not a color.
- Map tiles are W0-MAP, not the design system.
- Focus character art is W1-ASSETS.

### Tailwind scan gap (not a token; W1-DS owns this file)

`base.css` lines 5–7 scan `apps/desktop`, `packages/features`, and `packages/frontend` only. `packages/features/**` already includes `sparky-*`. Classes that exist **only** in `apps/sparky-desktop/src` or `apps/gallery` will not be generated. Add, when those apps gain markup:

```css
@source "../../../apps/sparky-desktop/src/**/*.{html,ts,tsx}";
@source "../../../apps/gallery/src/**/*.{html,ts,tsx}";
```

That is additive and does not change Polterware's generated CSS for classes it already uses.

---

## (d) Ark UI parts at `@ark-ui/solid@5.39.2`

Pinned in `packages/frontend/design-system/package.json` line 18 and `pnpm-lock.yaml` (`@ark-ui/solid@5.39.2`). Subpath export `./*` → `dist/components/*/index.js`. HTTP 200/404 checked against that version. Zag packages in that release are `1.43.3`.

| Planned primitive | Ark module | Verdict |
|---|---|---|
| Checkbox (round + square) | `@ark-ui/solid/checkbox` (200). Parts: `Root`, `Control`, `Indicator`, `Label`, `HiddenInput`, `Group`. | Use it. Round vs square is `border-radius` on `Control`/`Indicator`, one component, a `shape` prop. Do not use `FilterMark` or a native `<input type="checkbox">` as the widget. |
| DatePicker | `@ark-ui/solid/date-picker` (200). Parts include `Root`, `Input`, `Trigger`, `Content`, `Table` / `TableCell`, `MonthSelect`, `PrevTrigger`, `NextTrigger`, `View`. **No** `Hour`, `Minute`, or `Time` part in `dist/components/date-picker/index.d.ts`. | Use it for the calendar popover and the date field only. |
| TimePicker | `@ark-ui/solid/time-picker` **404**. | Does not exist. `@ark-ui/solid/timer` (200) is a countdown (`Root`, `Item`, `ActionTrigger`, `Separator`), not a time-of-day field. Build TimePicker as the existing `Popover` plus two `NumberInput`s (hour 0–23, minute 0–59) or a listbox. The popover owns focus, Escape, and outside press. |
| NumberStepper | `@ark-ui/solid/number-input` (200). Parts: `Root`, `Input`, `IncrementTrigger`, `DecrementTrigger`, `Scrubber`, `Label`. | Use it. Map the public name to `NumberStepper`. |
| Slider | `@ark-ui/solid/slider` (200). Parts: `Root`, `Control`, `Track`, `Range`, `Thumb`, `Marker`, `Label`. | Use it. |
| ColorSwatchPicker | `@ark-ui/solid/color-picker` (200). Parts include `Swatch`, `SwatchGroup`, `SwatchTrigger`, `SwatchIndicator`, plus area, channel sliders, and eyedropper. | Mind color is `SwatchGroup` only. Do not mount the HSV area for that picker. |
| TreeView (Minds) | `@ark-ui/solid/tree-view` (200). Parts: `Root`, `Tree`, `Branch`, `BranchTrigger`, `BranchContent`, `BranchIndentGuide`, `Item`, `NodeCheckbox`, `NodeRenameInput`. | Use it, including `BranchIndentGuide` and `NodeCheckbox` (compose the DS Checkbox styling). |
| SegmentedControl (not in the Ark question list; 06-ui asks for it) | `@ark-ui/solid/segment-group` (200). Parts: `Root`, `Item`, `ItemControl`, `ItemText`, `Indicator`. Built on Zag radio-group. | Use it for Day/Week/Month and list/table. Do not use `Tabs` (see (a)). |
| ProgressRing | `@ark-ui/solid/progress` (200). `Circle`, `CircleTrack`, `CircleRange`. | Use it. Not currently imported anywhere in the design system. |
| IconPicker | none | `Popover` + the curated set in `hugeicons.tsx`. `@ark-ui/solid/listbox` exists if a roving list is needed. |
| Heatmap | none | Opaque cells using the tokens in (c). |
| CalendarGrid (week/day/month schedule) | DatePicker's `Table` is a **month date grid only**. | Month field = DatePicker. Week and day time grids, all-day lane, and occurrence pill are new layout components. Drag-to-reschedule stays in W2-CALENDAR; W1-DS ships the layout, the pill, and keyboard focus. |
| Kbd | none needed | `src/kbd.tsx` already exports `Kbd`. |
| EmptyIllustration | none | Add an optional `illustration` prop to `PageEmpty` (`feedback.tsx` lines 9–14). `children` already renders under the copy (line 22). Default `undefined` so Polterware is unchanged. |

Already wrapped, do not re-wrap: Dialog, Drawer, Menu, Popover, Switch, Tabs, Tooltip (`dialog.tsx` line 10, `drawer.tsx` line 7, `dropdown-menu.tsx` line 16, `popover.tsx` line 13, `switch.tsx` line 9, `tabs.tsx` line 10, `tooltip.tsx`).

Stay on 5.39.2. A newer Ark that adds TimePicker still requires a `pnpm-lock.yaml` change, which this workstream cannot make and W1-DS should not make without a request.

---

## (e) Test coverage

Runner: `packages/frontend/design-system/package.json` script `test` → Vitest. Tests live next to the source. Depth: **behavior** (renders or asserts interaction), **export** (module shape or a source-string check only), **none**.

| Source | Test | Cases | Depth |
|---|---|---|---|
| `tooltip.tsx` | `tooltip.test.tsx` | 6 | behavior (hover, focus, Escape, reduced motion) |
| `toolbar-icon-button.tsx` | `toolbar-icon-button.test.tsx` | 2 | behavior |
| `template-card.tsx` | `template-card.test.tsx` | 2 | behavior |
| `tabs.tsx` | `tabs.test.tsx` | 4 | export / prop shape, not keyboard |
| `table.tsx` | `table.test.tsx` | 16 | behavior. Locks double-click expand: lines 400 and 459. |
| `switch.tsx` | `switch.test.tsx` | 1 | export only |
| `status.tsx` | `status.test.tsx` | 4 | filled-chip contract |
| `spring-css.ts` | `spring-css.test.ts` | 4 | behavior of the easing samples |
| `skeleton.tsx` | `skeleton.test.ts` | 3 | export plus CSS contract. Also asserts `PageLoading` delegates. |
| `popover.tsx` | `popover.test.tsx` | 4 | motion source contract |
| `kbd.tsx` | `kbd.test.tsx` | 1 | render |
| `input.tsx` | `input.test.tsx` | 2 | Input/Textarea defaults. Also checks that `Select`, `Field`, and `Menu` are exported. |
| `hugeicons.tsx` | `hugeicons.test.tsx` | 3 | behavior |
| `fluid-hover.tsx` | `fluid-hover.test.ts` | 7 | behavior (`pickNearest`) |
| `floating-window.tsx` | `floating-window.test.tsx` | 24 + 4 geometry | behavior. Line 491 forbids a frontend-state import. |
| `floating-count-badge.tsx` | `floating-count-badge.test.tsx` | 2 | behavior |
| `filter-popover.tsx` | `filter-popover.test.tsx` | 5 | behavior of pure helpers and prop contracts |
| `filter-chip.tsx` | `filter-chip.test.tsx` | 1 | behavior |
| `dropdown-menu.tsx` | `dropdown-menu.test.tsx` | 6 | behavior / motion |
| `drawer.tsx` | `drawer.test.tsx` | 3 | export and source strings |
| `dither-icon.tsx` | `dither-icon.test.tsx` | 7 | behavior. Lines 92–96 lock brand hexes. |
| `dialog.tsx` | `dialog.test.tsx` | 3 | export and prop types |
| `detail-panel.tsx` and the four `detail-panel-*.tsx` parts | `detail-panel.test.tsx` | 27 | behavior. Also reads Polterware feature source: lines 502–510 (`packages/features/knowledge/src/index.tsx`, `packages/features/architecture/src/index.tsx` toolbar order) and line 638 (`node-detail.tsx`, `view-detail.tsx`). Line 264 locks ⌘J / ⌘⇧J while closed. |
| `context-menu.tsx` | `context-menu.test.tsx` | 4 | behavior |
| `button.tsx` | `button.test.tsx` | 1 | export only |
| `toast.tsx`, `separator.tsx`, `select.tsx`, `search-bar.tsx`, `page.tsx`, `menu.tsx`, `menu-highlight.tsx`, `markdown-editor.tsx`, `field.tsx`, `feedback.tsx` (except `PageLoading` via skeleton), `filter-options-panel.tsx`, `filter-mark.tsx`, `icon-swap.tsx`, `card.tsx`, `badge.tsx`, `active-filter-bar.tsx`, `collapse.ts`, `drag-motion.ts`, `placement-motion.ts`, `reduced-motion.ts`, `springs.ts`, `panels/resizable.tsx` | — | 0 | none |

`popup-motion.ts` is only covered indirectly by popover and dropdown tests. Do not backfill tests for untouched files. Every **new** primitive needs the W1-DS set: keyboard, focus, controlled/uncontrolled, reduced motion, and a gallery entry.

Polterware tests that will fail if W1-DS changes behavior instead of defaults:

- `table.test.tsx` "delegates background dismissal…" (line 400) and "expands the detail sidebar when double-clicking…" (line 459).
- `detail-panel.test.tsx` shortcut test (line 264) and the feature-source order test (line 502).
- `dither-icon.test.tsx` pastel hexes (lines 92–96).
- `floating-window.test.tsx` line 491.

---

## (f) Ordered work for W1-DS

Risk scale is the chance of a visible Polterware change. Each step keeps today's defaults when the provider or the new props are omitted.

1. **Decouple the three imports without renaming keys.** Add `DesignSystemProvider` (`storagePrefix`, shortcut matchers, detail expand). Polterware passes `storagePrefix="polterware"` and the current functions (`detailPanelStore.expand`, `matchesShortcut` for the three shell ids). Sparky passes `storagePrefix="sparky"` and the same shell ids (the chords are already ⌘J / ⌘⇧J / ⌘B).
   - Do **not** prefix `storageKey` / `legacyStorageKey`. Those strings are already caller-owned. Prefixing them would stop reading `polterware.shell.detail.sidebar.width` and `polterware.shell.sidebar.sizes`.
   - `table.tsx` line 752: call the provider's expand, defaulting to `detailPanelStore.expand` so the two table tests stay green. Sparky sets expand to its own detail store.
   - `detail-panel.tsx` lines 119–120 and 228–232, and `panels/resizable.tsx` lines 283–289: read shortcut ids from the provider, defaulting to the three ids above.
   - **Risk: high** if the default context is not byte-for-byte the current calls. Wire Polterware at `apps/desktop` and the shell only by passing the provider. No visual edit.
   - `packages/frontend/state` shortcut catalog stays. Do not add Sparky commands to `ShortcutCommandId` in that shared union unless Sparky settings need them; extra ids show up wherever Polterware lists `SHORTCUT_CATALOG`.

2. **Generalize flavored components (defaults unchanged).**
   - `PageOffline` and `PageForbidden`: optional `icon` prop; default glyphs and strings stay (`feedback.tsx` 64–71 and 90–100).
   - `FixtureBanner`: keep `return null`.
   - `DITHER_BRAND_ACCENTS`: `registerDitherAccents` plus the existing map as the default. Optional `accent` string on `DitherIconTile` wins over tone and brand. Existing hexes stay.
   - **Risk: medium** only if a default string or hex changes. `dither-icon.test.tsx` lines 92–96 are the guard.

3. **New primitives, in this order.** Each is an Ark wrapper plus tokens, a Vitest behavior test, and a gallery entry. No hand-rolled focus or overlay.
   1. Checkbox (`@ark-ui/solid/checkbox`), `shape: "round" | "square"`.
   2. SegmentedControl (`segment-group`).
   3. NumberStepper (`number-input`).
   4. Slider (`slider`).
   5. DatePicker (`date-picker`, date only).
   6. TimePicker (Popover + NumberStepper). Not Timer. Not a new focus trap.
   7. ColorSwatchPicker (`color-picker` swatch parts only).
   8. IconPicker (Popover + `hugeicons.tsx`).
   9. TreeView (`tree-view`) for Minds.
   10. ProgressRing (`progress` Circle). Reduced motion snaps. No opacity animation.
   11. Heatmap, after the five opaque tokens in (c).
   12. CalendarGrid layout (all-day lane, hour rows, occurrence pill, overflow `+n` slot). Month **picker** stays DatePicker. No drag logic here.
   13. `PageEmpty` `illustration` prop. Kbd stays as it is.
   - **Risk: low** for Polterware if the new files are additive and Polterware does not import them.
   - **Risk: medium** on opacity lint. Heatmap cells and calendar pills are opaque fills. Presence opacity stays inside the existing popup/dialog primitives.

4. **Gallery** `apps/gallery` (Vite + Solid), one theme, reduced-motion toggle, one entry per primitive including the pre-existing ones Sparky uses (`Button`, `StatusChip`, `DataTable`, `DetailPanel`, `MarkdownEditor`, `DitherIconTile`, `Kbd`). Add the `@source` line in (c) for `apps/gallery` and `apps/sparky-desktop`. **Risk: low** if the app is a new package and does not change `apps/desktop`.

5. **Non-regression.** `pnpm typecheck`, `pnpm test`, `pnpm design:lint`, and a manual `pnpm dev:desktop` smoke (Inbox, Work board, Atlas, Settings) with before/after notes in the W1-DS status file. Update `DESIGN.md` before any opacity or material exception. `tooling/check-ui-opacity.mjs` is owned by W0-SCAFFOLD; extend it only through a request.

Call-site edits W1-DS is allowed to make are limited to mounting `DesignSystemProvider` and passing the new optional props. Do not reorder `PageToolbar` actions in `packages/features/knowledge` or `packages/features/architecture`; `detail-panel.test.tsx` lines 502–510 compare source order (`Import from project root` before `Add source`; `Auto layout` before `Add component`).
