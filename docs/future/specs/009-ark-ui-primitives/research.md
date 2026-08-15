# Research: Ark UI–Backed Interactive Primitives

**Feature**: `009-ark-ui-primitives`  
**Date**: 2026-08-03  
**Status**: Complete — headless backend decision superseded Kobalte → **Ark UI**

---

## R1 — Headless library: Ark UI (not Kobalte)

**Decision**: Adopt **`@ark-ui/solid`** as the **only** interactive headless layer for product UI. **Remove Kobalte** (`@kobalte/core`) from maintained code and product manifests when migration completes.

**Rationale**: User mandate; single behavior SSOT; avoid partial Kobalte coverage and fragile import patterns; Ark is unstyled/headless and can be fully styled with Polter tokens.

**Alternatives considered**:

| Alternative | Why rejected |
| --- | --- |
| Keep/expand Kobalte | Explicitly replaced |
| Corvu for menus/dialogs | AGENTS: Corvu only for resizable panels |
| Solid UI wholesale / generators | Overwrites tokens; forbidden |
| Hand-roll primitives | Spec problem; fragile a11y |
| React Aria / Radix | React product UI forbidden |

**Essential formulation:**

> Ark UI controls how the component **works**. DESIGN.md, tokens, and Tailwind control how it **looks**.

---

## R2 — Visual non-goals (hard constraints)

**Decision**: Ark **must not** introduce:

- colors, borders, radius, spacing, typography, shadows  
- density, visual animation language  
- default library / docs sample appearance  
- Chakra-like or other Ark showcase look  

All local wrappers apply existing Polter materials (`control-field`, `surface-panel-float`, `menu-panel`, radius tokens, project motion CSS, DESIGN.md states).

**Do not** rewrite DESIGN.md to accommodate Ark.

---

## R3 — Local components stay the public API

**Decision**: Features import **only** `@polterware/frontend-design-system` for chrome. Design-system encapsulates Ark parts (`Dialog.Root`, `Popover.Positioner`, etc.—exact Ark Solid API as implemented) and applies project classes.

Preserve existing public APIs where practical:

| Local API (examples) | Preserve |
| --- | --- |
| `A11yDialog` / `Dialog` (`open`, `onClose`, `title`, `chrome`, …) | Yes |
| `Popover` (`open`, `onClose`, `anchorRef`, `placement`, …) | Yes where possible |
| `Tabs` (`items`, `value`, `onChange`) | Yes |
| `Drawer` (`open`, `onClose`, `title`, `width`, …) | Yes |

Domain/features **must not** depend on Ark internal structure when a local wrapper exists.

---

## R4 — MenuItem inside Popover vs real Menu

**Decision**: Same split as prior research, backend = Ark:

| Pattern | Headless | Local API |
| --- | --- | --- |
| **A. Searchable switcher** | Ark **Popover** | Presentational rows (no false `role="menu"`) |
| **B. Compact dropdown/context** | Ark **Menu** / Context Menu | Local `DropdownMenu` / `ContextMenu` wrappers |

Do not nest Menu root inside Popover root.

---

## R5 — Drawer

**Decision**: Local `Drawer` backed by Ark **Dialog** (or Ark Drawer if available and simpler) as modal edge sheet via **CSS only**. Same dismiss/focus contract as dialogs.

---

## R6 — Tabs API

**Decision**: Keep `items` / `value` / `onChange` + pill `.pw-tabs` CSS. Internals = Ark Tabs. View filters stay non-Tabs.

---

## R7 — Command palette & terminal

**Decision**:

- Commander → local `A11yDialog` (Ark-backed); product frame CSS unchanged.
- Terminal → local `Drawer` (Ark-backed).
- Remove document Escape listeners.

---

## R8 — Relationship picker

**Decision**: Local `Popover` + presentational/listbox rows; pixel anchor via Ark positioning API or zero-size anchor.

---

## R9 — Toast / Combobox

**Decision**: Toast light-harden optional. Native Select stays; Ark Combobox/Select only when a complex call site requires it in this feature.

---

## R10 — Kobalte purge procedure

For each hit (`@kobalte/core`, Kobalte-based wrappers, CSS coupled to Kobalte DOM, tests):

1. Map Ark equivalent  
2. Swap internal implementation  
3. Preserve Polter styles  
4. Preserve public API when possible  
5. Fix inevitable DOM diffs (selectors only)  
6. Validate focus/keyboard/a11y  
7. Visual compare  
8. Delete Kobalte path  

Done only when: no maintained Kobalte imports, dependency removed, visual + keyboard validated—not compile-only.

---

## R11 — Wrap-once & import policy

**Decision**:

- Design-system: may import `@ark-ui/solid`  
- Features/apps: **must not** import Ark or Kobalte when a local wrapper exists  
- Residual grep: `@kobalte/core`, `@ark-ui/solid` under `packages/features`, document Escape, custom `aria-modal`

---

## R12 — Migration order

1. Add `@ark-ui/solid`; keep Kobalte temporarily only during cutover of already-migrated files  
2. Dialog → Ark (highest leverage)  
3. Popover → Ark  
4. Tabs → Ark  
5. Drawer → Ark sheet  
6. Menu split + DropdownMenu  
7. Commander, terminal, relationship picker  
8. Residual hand-rolls  
9. Remove all Kobalte imports + dependency  
10. AGENTS/DESIGN sync; quickstart static + keyboard  
11. Optional toast  

---

## R13 — Testing

- Unit/export tests against **local** public APIs  
- Static inventory greps  
- Manual keyboard/focus/visual (Desktop when Erick allows)  
- No agent `pnpm dev` unless asked  

---

## Resolved unknowns

| Unknown | Resolution |
| --- | --- |
| Headless lib | `@ark-ui/solid` only |
| Kobalte fate | Full removal |
| Visual owner | DESIGN + tokens + Tailwind |
| Public API | Preserve local wrappers |
| Menu composition | Pattern A/B split |
| Done criteria | Behavior + visual + purge + tests—not compile-only |
)
