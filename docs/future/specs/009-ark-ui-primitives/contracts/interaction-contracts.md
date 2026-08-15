# Contract: Interactive Chrome Behavior (Ark-backed locals)

**Feature**: `009-ark-ui-primitives`  
**Consumers**: Desktop features via `@polterware/frontend-design-system`  
**Behavior backend**: `@ark-ui/solid` (inside design-system only)  
**Visual SSOT**: DESIGN.md + Polter tokens + project Tailwind/CSS

---

## Layering rule

| Concern | Owner |
| --- | --- |
| State, keyboard, focus, ARIA, portal, dismiss, position, selection | Ark UI via local wrappers |
| Color, border, radius, space, type, shadow, density, motion look, state paint | Project design system only |

Features MUST call **local** components, not Ark, when a wrapper exists.

---

## Shared open/close model

| Prop pattern | Meaning |
| --- | --- |
| `open: boolean` | Parent owns visibility |
| `onClose` / `onOpenChange` | Parent updates when headless requests close |

No second document-level Escape listener for the same surface.

---

## C1 — Modal dialog (`A11yDialog` / `Dialog`)

| Rule | Value |
| --- | --- |
| Backend | Ark Dialog (or equivalent) |
| Modality | Modal |
| Escape / scrim | Dismiss |
| Focus | Trap + return |
| Title | Required accessible name; may be sr-only when `chrome="flush"` |
| Size | One product frame (`.pw-dialog-content` / `.pw-product-frame`) |
| Look | `surface-panel-float` + dialog.css — not Ark samples |

**Includes**: Command palette chrome.

---

## C2 — Drawer / side sheet (`Drawer`)

| Rule | Value |
| --- | --- |
| Backend | Ark Dialog/Drawer as sheet |
| Escape / scrim | Dismiss |
| Focus | Trap + return |
| Layout | Full-height edge; width prop; **CSS placement only** |

**Includes**: Terminal side panel.

---

## C3 — Large anchored popover (`Popover`)

| Rule | Value |
| --- | --- |
| Backend | Ark Popover |
| Modality | Non-modal default for switchers |
| Escape / outside | Dismiss |
| Anchor | Prefer anchor ref + placement |
| Look | `surface-panel-float`; breadcrumb width ~300 |

**Composition**: Search + presentational rows OK. Rows must not fake `role="menu"` without Menu root.

---

## C4 — Compact dropdown / context menu

| Rule | Value |
| --- | --- |
| Backend | Ark Menu / Context Menu |
| Keyboard | Arrows, Enter/Space, Escape |
| Look | `.menu-panel`, radius-item |
| API | Local `DropdownMenu` / items — not raw Ark in features |

---

## C5 — Section tabs (`Tabs`)

| Rule | Value |
| --- | --- |
| Backend | Ark Tabs |
| Public props | `items` / `value` / `onChange` |
| Look | `.pw-tabs` / `.pw-tab` pills |
| Not for | Grid/table view filters |

---

## C6 — Feature prohibitions

For C1–C5 surfaces, features MUST NOT:

1. Document-level Escape as primary dismiss  
2. Hand-built `role="dialog"` + `aria-modal` shells  
3. Hand-built focus traps  
4. Direct `@ark-ui/solid` or `@kobalte/core` imports when a local wrapper exists  

---

## C7 — Kobalte purge signals

Before done, zero hits in maintained product paths for:

- `@kobalte/core`  
- design-system dependency on Kobalte  

And residual scan for feature Escape/dialog shells per [quickstart.md](../quickstart.md).

---

## C8 — Completion is not compile-only

Each migrated primitive MUST pass: keyboard, focus move/restore, open/closed/disabled/selected visual states under Polter styles, and typecheck/tests for the changed surface.
