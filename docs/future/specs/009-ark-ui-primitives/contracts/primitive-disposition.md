# Contract: Primitive Disposition + Kobalte Purge

**Feature**: `009-ark-ui-primitives`  
**SSOT**: this file + [data-model.md](../data-model.md) + [spec.md](../spec.md)

---

## Design-system public exports

| Export | Disposition | Target | Done when |
| --- | --- | --- | --- |
| `Dialog` / `A11yDialog` | migrate from Kobalte | Ark Dialog + Polter frame | API stable; no Kobalte; visual parity |
| `Popover` | migrate from Kobalte | Ark Popover | same |
| `Tabs` / `TabPanel` | migrate | Ark Tabs | hand-rolled keydown gone; API stable |
| `Drawer` | migrate | Ark sheet | no document Escape |
| `MenuPanel` / `MenuItem` | migrate (split) | presentational rows + Ark Menu wrapper | no false menu roles; DropdownMenu exists for pattern B |
| `ToastRegion` / `pushToast` | deferred/harden | optional Ark | live region + optional dismiss |
| `Input` / `Textarea` / `Select` / `Field` | leave | native | unchanged |
| `FilterChip*` | leave | native | not section Tabs |
| `Button` / Badge / Card / Surface / Separator / Status* / Page* / feedback / DataTable | leave | — | no forced headless |
| `ResizableSplit` | out-of-scope | Corvu policy | untouched |

---

## Feature hand-rolls

| Surface | Disposition | Target |
| --- | --- | --- |
| Command palette | migrate | local Dialog |
| Terminal side panel | migrate | local Drawer |
| Relationship type picker | migrate | local Popover + rows |
| Context switchers / target / user menu | verify | Popover + presentational rows |

---

## Kobalte purge checklist

| Item | Done when |
| --- | --- |
| No `@kobalte/core` under `packages/` product paths | `rg` empty |
| No `@kobalte/core` under `apps/desktop` | `rg` empty |
| design-system `package.json` has no Kobalte | dep removed |
| lockfile no longer requires Kobalte for product UI | clean install consistent |
| Tests do not assert Kobalte markup | retargeted to local API |
| CSS not depending on Kobalte-only data attrs unless remapped | Polter look preserved |

---

## Dual-path exit (XIV)

| Temporary dual | Exit |
| --- | --- |
| Kobalte Dialog + Ark Dialog | Single local Dialog, Ark only |
| Kobalte Popover + Ark Popover | Single local Popover, Ark only |
| Old Tabs + new Tabs | Single Tabs export |
| Feature custom dialog + local Dialog | Feature uses local only |

Creating **local styled wrappers over Ark is required**, not a violation.  
Hand-rolling interactive behavior Ark already provides **is** a violation.
