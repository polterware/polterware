# Implementation Plan: Ark UI–Backed Interactive Primitives

**Branch**: `009-ark-ui-primitives` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/009-ark-ui-primitives/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Supersedes**: Prior draft that targeted Kobalte-as-SSOT. **Ark UI is now the sole interactive headless backend; Kobalte is removed.**

**Depends on**: Existing design-system materials (003+), Desktop feature composition.

---

## Summary

Migrate all applicable **interactive** design-system components so that:

| Layer | Owner |
| --- | --- |
| State, keyboard, focus, ARIA, portal, dismiss, positioning, selection | **`@ark-ui/solid`** |
| Colors, radius, spacing, type, shadows, density, motion look, state paint | **DESIGN.md + Polter tokens + project Tailwind/CSS** |
| Official app API | **Local components** in `@polterware/frontend-design-system` |

Do **not** replace visual abstractions with raw Ark. Do **not** hand-roll behavior Ark already provides. Preserve public local APIs where practical. Eliminate feature-local modal stacks. **Purge Kobalte** from code and manifests.

Formulation:

> **Ark UI controls how the component works. The project’s design.md, tokens, and Tailwind control how it looks.**

---

## Technical Context

**Language/Version**: TypeScript strict, SolidJS 1.9+, Tailwind v4 + Polter tokens

**Primary Dependencies**:

- **Add**: `@ark-ui/solid` (design-system)
- **Remove**: `@kobalte/core` after migration (design-system + lockfile)
- Features consume only `@polterware/frontend-design-system`

**Storage**: N/A

**Testing**: Vitest export/behavior tests on local APIs; residual `rg`; typecheck; keyboard/visual acceptance. No agent `pnpm dev` unless Erick asks.

**Target Platform**: Desktop renderer

**Project Type**: Shared frontend library + feature hand-roll cleanup

**Performance Goals**: Portal content only while open; no Main work

**Constraints**:

- Behavior-only Ark — no docs themes / Chakra look  
- DESIGN.md not rewritten for Ark  
- No feature-level Ark/Kobalte imports when wrappers exist  
- One dialog product frame  
- XIV: no permanent dual headless or dual local primitive  

**Scale/Scope**: Design-system interactive set + commander/shell/architecture hand-rolls + full Kobalte purge

**Unknowns**: Resolved in research R1–R13

---

## Constitution Check

| Principle | Status | Evidence |
| --- | --- | --- |
| I Operational architecture | **Pass** | Reliable operator chrome |
| II Domain before framework | **Pass** | UI only |
| III Runtime separation | **Pass** | Renderer + design-system |
| IV Directed deps | **Pass** | features → design-system → ark-ui; no cycles |
| V Contracts-first | **Pass** | UI contracts in `specs/009-…/contracts/` |
| VI Security | **Pass** | Unprivileged; Escape always |
| VII Auditable exec | **Pass** | No system mutations |
| VIII Local-first | **Pass** | Offline UI |
| IX Performance | **Pass** | Headless presence; no Main |
| X Resilience | **Pass** | Shared dismiss |
| XI Observability | **Pass** | n/a |
| XII Testability | **Pass** | Unit + static + quickstart |
| XIII Real code | **Pass** | Existing package only |
| XIV SSoT | **Pass** | One headless (Ark); Kobalte exit required |
| XV Big bang | **Pass** | Decisive headless swap |
| XVI Docs | **Pass** | AGENTS/DESIGN LAW update |

**Hard stops**: IV/VI/XIV pass. Temporary Kobalte+Ark during file-by-file cutover only—must end with Ark-only.

### Post-design re-check

Aligned with R1–R13. Gate **Pass**.

---

## Project Structure

### Documentation

```text
specs/009-ark-ui-primitives/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── interaction-contracts.md
│   └── primitive-disposition.md
└── tasks.md
```

### Source paths

```text
packages/frontend/design-system/
├── package.json                    # +@ark-ui/solid; -@kobalte/core when done
└── src/
    ├── dialog.tsx / dialog.css     # Ark Dialog + Polter frame
    ├── popover.tsx / popover.css   # Ark Popover
    ├── tabs.tsx / tabs.css         # Ark Tabs
    ├── drawer.tsx                  # Ark dialog/drawer sheet
    ├── menu.tsx                    # Presentational rows
    ├── dropdown-menu.tsx (new)     # Ark Menu
    ├── toast.tsx                   # optional harden
    └── index.ts

packages/features/commander/src/index.tsx
packages/features/shell/src/terminal-side-panel.tsx
packages/features/shell/src/context-switchers.tsx
packages/features/architecture/src/canvas/relationship-type-popover.tsx
AGENTS.md / DESIGN.md
```

**Structure Decision**: No new packages. Runtime = Renderer. Contracts = UI only.

---

## Implementation phases (for tasks)

| # | Work |
| --- | --- |
| 0 | Baseline grep (Kobalte + hand-rolls); add Ark dep |
| 1 | Dialog → Ark (preserve API + CSS) |
| 2 | Popover → Ark |
| 3 | Tabs → Ark |
| 4 | Drawer → Ark sheet |
| 5 | Menu split + DropdownMenu |
| 6 | Commander, terminal, relationship picker |
| 7 | Residual feature hand-rolls |
| 8 | Kobalte purge (imports + package.json + lockfile) |
| 9 | AGENTS/DESIGN LAW; presentational audit; quickstart |
| 10 | Optional toast |

---

## Complexity Tracking

| Note | Detail |
| --- | --- |
| Temporary dual headless | Allowed only mid-migration; exit = zero Kobalte |
| DOM diffs Ark vs Kobalte | Adjust CSS selectors only; keep Polter look |
| Menu pattern split | Required for correct a11y with searchable switchers |

---

## Agent context

Update root **AGENTS.md** A11y LAW: Ark via design-system; behavior vs look split; Kobalte forbidden. DESIGN.md one-line stack pointer. No separate Speckit agent script in repo.
)
