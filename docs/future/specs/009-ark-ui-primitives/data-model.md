# Data Model: Ark UI–Backed Interactive Primitives

**Feature**: `009-ark-ui-primitives`  
**Date**: 2026-08-03  
**Layer**: Presentation / design-system interaction model (no Core schema)

---

## Entities

### InteractivePrimitive

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Local export name (`A11yDialog`, `Tabs`, …) |
| `kind` | enum | dialog \| popover \| menu \| tabs \| drawer \| combobox \| tooltip \| toast \| other |
| `headlessBackend` | enum | `ark-ui` \| `none` \| `native` \| `kobalte-legacy` (must be zero at done) |
| `disposition` | enum | migrate \| leave-presentational \| out-of-scope \| deferred |
| `publicApiStable` | boolean | Prefer prop-compatible migration |
| `modalPolicy` | enum? | modal \| non-modal \| n/a |

**Rules**: Overlay kinds must end as `ark-ui` (or documented native). `kobalte-legacy` forbidden at feature complete.

### LocalStyledComponent

| Field | Type | Notes |
| --- | --- | --- |
| `exportName` | string | Public design-system API |
| `arkPartsUsed` | string[] | e.g. Dialog.Root, Popover.Positioner |
| `styleSources` | string[] | tokens, CSS modules, Tailwind classes |

**Rules**: Features depend on `exportName`, not `arkPartsUsed`.

### PresentationalAtom

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Card, Badge, Input, … |
| `disposition` | leave-presentational |

### FeatureHandRoll

| Field | Type | Notes |
| --- | --- | --- |
| `surface` | string | commander.CommandPalette, … |
| `package` | path | packages/features/… |
| `antiPatterns` | string[] | document-escape, custom-aria-modal, … |
| `targetPrimitive` | string | local export |
| `status` | open \| migrated \| deferred |

### InteractionContract

| Field | Type | Notes |
| --- | --- | --- |
| `primitiveId` | string | |
| `escapeDismisses` | boolean | |
| `outsideDismisses` | boolean | |
| `focusTrap` | boolean | |
| `focusReturn` | boolean | |
| `keyboardModel` | string | |
| `portal` | boolean | |

**Open surface state:**

```text
closed ──open──► open
open ──escape|outside*|close──► closed (+ focus return when required)
```

### KobaltePurgeItem

| Field | Type | Notes |
| --- | --- | --- |
| `path` | string | file or package.json |
| `kind` | import \| dep \| css-coupled \| test |
| `status` | open \| removed |

**Rule**: All open items closed before SC-008.

---

## Baseline disposition snapshot

| id | disposition | headless target |
| --- | --- | --- |
| Dialog / A11yDialog | migrate (from Kobalte) | ark-ui |
| Popover | migrate (from Kobalte) | ark-ui |
| Tabs / TabPanel | migrate | ark-ui |
| Drawer | migrate | ark-ui sheet |
| MenuPanel / MenuItem | migrate (split) | presentational + ark menu |
| Toast | deferred/harden | optional ark |
| Input/Textarea/Field/Select native/FilterChip/Button/Badge/Card/… | leave | none/native |
| ResizableSplit | out-of-scope | corvu policy |

| hand-roll | target | status at plan |
| --- | --- | --- |
| CommandPalette | Dialog | open |
| TerminalSidePanel | Drawer | open |
| RelationshipTypePopover | Popover | open |
| switchers / pickers | Popover + rows | verify |

---

## Relationships

```text
LocalStyledComponent ──uses──► Ark UI parts (behavior only)
LocalStyledComponent ──styles──► Polter tokens / CSS / Tailwind
InteractivePrimitive 1──* InteractionContract
FeatureHandRoll *──1 LocalStyledComponent
KobaltePurgeItem tracks removal of old backend
```
