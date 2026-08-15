# Implementation Plan: Operator Design System & Primary Surfaces

**Branch**: `003-operator-design-system` (feature dir; git may be `refactor/new`) | **Date**: 2026-08-02 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/003-operator-design-system/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

---

## Summary

Deliver a **quiet dark operator design system** and the **primary Desktop surfaces** Polter needs to feel like a mature control plane:

1. Rewrite root **`DESIGN.md`** as the visual single source of truth (identity, principles, tokens, components, navigation, blueprints, a11y, performance, acceptance).
2. Implement tokens in **`packages/configuration/tailwind/base.css`** and restyle **`@polterware/frontend-design-system`** primitives (quiet controls, dialogs, status, surfaces—no brand-green chrome).
3. Replace provisional **Shell** with integrated sidebar + single rounded **MainSurface** anatomy.
4. Expand **Resources** (index table + detail page + create/edit dialog) and **Architecture** (full-width canvas + node detail page; **remove permanent inspector**).
5. Wire composition in **`apps/desktop`** with typed client routes, core data when healthy, fixtures when empty, honest loading/empty/error/offline.

Stack stays: Electron + SolidJS + Tailwind v4 + existing canvas/data packages. Add **@hugeicons/core-free-icons** and **@kobalte/core** only as first real consumers need them. No React, no CSS Modules, no solidui-cli overwrite of tokens.

---

## Technical Context

**Language/Version**: TypeScript (strict) for Desktop/renderer/features; CSS via Tailwind v4

**Primary Dependencies**:

- Existing: Electron, electron-vite, SolidJS, Tailwind CSS v4, `@tanstack/solid-virtual`, `@polterware/frontend-canvas` (+ layout worker), `@polterware/frontend-data` (Connect), design-system, feature packages
- Add on demand: `@hugeicons/core-free-icons` (individual imports), `@kobalte/core` (complex a11y primitives only)
- Not added unless justified in tasks: `@solidjs/router` (see research R4)

**Storage**: N/A for design system; layout/resources via existing core SQLite through Connect. Fixtures are in-memory/module data only.

**Testing**: Vitest (design-system + pure view helpers + canvas); manual visual quickstart; typecheck/lint/test/build on changed packages for completion claims

**Target Platform**: Polterware Desktop (macOS primary chrome; dark operator UI)

**Project Type**: Desktop monorepo feature (presentation + shared design-system)

**Performance Goals**: Shell remains interactive during canvas pan/zoom on prototype-scale graphs; Resources tables usable at hundreds of rows (virtualize when needed); control transitions ≤ ~160ms; reduced-motion respected

**Constraints**: Constitution v2; no permanent dual design languages; no permanent inspector on Resources/Architecture; brand green restricted; no React; no claiming live integrations that are not wired; Agents.md forbids agent-run full `pnpm dev` unless user asks—validation via static checks + user-run dev when needed

**Scale/Scope**: Shell + design system + Resources (index/detail/dialog) + Architecture (canvas/detail) + stubs restyle for other nav items as needed; not full specialized-provider consoles

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
| --- | --- | --- |
| I Operational architecture | **Pass** | Architecture + Resources strengthen understand/discover; hand-off actions not platform clones |
| II Domain before framework | **Pass** | UI/view models only; no domain rules inside design-system |
| III Runtime separation | **Pass** | Renderer presentation; Main/preload chrome only; core remains authority |
| IV Directed deps | **Pass** | apps/desktop → features → frontend → generated/contracts; no cycles |
| V Contracts-first | **Pass** | Reuse Connect; UI contracts documented under `contracts/` without duplicate protobuf unless gap |
| VI Security | **Pass** | `window.polterware` only; no Node in renderer; secrets not listed as plain permanent fields |
| VII Safe auditable exec | **Pass** | Create/edit UI does not invent silent destructive apply; layout save explicit; governed ops unchanged |
| VIII Local-first | **Pass** | Offline/core-down empty/fixture/error honesty |
| IX Performance | **Pass** | Virtual lists, canvas non-blocking, workers for heavy layout |
| X Resilience | **Pass** | Loading/empty/error/offline states required |
| XI Observability | **Pass** | Surface errors; correlate if ops invoked |
| XII Testability | **Pass** | Primitive + pure helper tests; quickstart acceptance |
| XIII Real code | **Pass** | Extend existing packages only; no scenic new apps |
| XIV SSoT | **Pass** | One DESIGN.md + one token shell; remove provisional dual chrome on in-scope screens |
| XV Big bang context | **Pass** | Completes Solid operator chrome on authorized path |
| XVI Documentation | **Pass** | DESIGN.md + feature plan artifacts |

**Hard stops (VI, IV, XIV)**: No violations. Temporary dual token aliases during migration allowed only until in-scope screens finish—then remove dead light-theme product claims for Desktop operator chrome.

### Post-design re-check

Aligned: research R1–R14 resolve token home, shell anatomy, inspector removal, fixtures honesty, dependency minimalism. No unjustified new runtimes or scenic packages.

---

## Project Structure

### Documentation (this feature)

```text
specs/003-operator-design-system/
├── plan.md              # This file
├── research.md          # Phase 0
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/           # Phase 1 UI contracts
├── checklists/
└── tasks.md             # Phase 2 via /speckit-tasks (not this command)
```

### Source Code (paths this feature will modify)

```text
DESIGN.md                              # rewrite SSOT (design.md same path on macOS)

packages/configuration/tailwind/
  base.css                             # quiet dark @theme tokens + aliases

packages/frontend/design-system/
  src/{button,badge,card,separator,status,...}.tsx
  a11y/dialog.tsx                      # evolve dialog
  panels/resizable.tsx                 # keep; not default Architecture chrome
  (+ selective: input, tabs, tooltip, menu, toast, page primitives)

packages/frontend/canvas/
  src/                                 # selection styling helpers, layer view support as needed
  worker/layout.ts                     # use for heavy layout if expanded

packages/features/shell/src/           # integrated sidebar + MainSurface
packages/features/resources/src/       # index, detail, form dialog
packages/features/architecture/src/    # canvas page, node detail; drop permanent inspector

apps/desktop/src/renderer/
  App.tsx                              # route state, data adapter, composition
  styles.css                           # import tokens only
```

**Structure Decision**:

| Concern | Placement |
| --- | --- |
| Runtime owner | Desktop Renderer (+ thin Main for window only) |
| Domain vs adapter | No new domain package; view models in features/App; core via frontend-data |
| Contracts | Existing Connect; UI contracts in feature `contracts/` |
| New packages | **None** — extend design-system, shell, resources, architecture, canvas, configuration |
| Why no new package | Constitution XIII; consumers already exist |

---

## Complexity Tracking

> No constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| — | — | — |

---

## Implementation Phases (for `/speckit-tasks`)

Ordered for independent value; details become tasks.md later.

### A. Design SSOT + tokens

1. Analyze `/prototype` + current DESIGN.md conflicts (done for plan; reaffirm in implement).  
2. Rewrite `DESIGN.md` with all required sections from product brief.  
3. Implement token set in `base.css` with backward-compatible aliases.  
4. Align desktop styles + remove conflicting accent blue as default product accent.

### B. Design-system primitives

1. Quiet Button (sizes, variants, focus-visible, no scale gimmick default).  
2. Status, Badge, Separator, Surface/Page primitives.  
3. Dialog (a11y), form controls as needed.  
4. Toast region primitive.  
5. Icon helper (@hugeicons/core-free-icons individual imports).
6. Tests for critical primitives.

### C. Shell + navigation

1. AppSidebar groups + active neutral state + status footer.  
2. MainSurface rounded host.  
3. PageHeader / toolbar slots.  
4. ClientRoute state in App; section + detail pages.  
5. Window drag region / traffic light inset.

### D. Resources

1. View models + fixtures.  
2. Index table + filters + search + states.  
3. Detail page.  
4. Create/Edit dialog.  
5. Wire core listResources + honesty labeling.

### E. Architecture

1. Remove permanent inspector split.  
2. Canvas full-width + neutral selection + pan/zoom basics.  
3. Layered/grouped layout presentation.  
4. Node detail page.  
5. Layout save + core/fixture wiring.

### F. Polish + verify

1. Restyle remaining nav stubs to tokens (no fake dashboards).  
2. A11y pass (keyboard, focus, reduced motion).  
3. Quickstart checklist.  
4. typecheck / lint / test / build on changed surface.

---

## Dependencies to add (expected)

| Dependency | Why | When |
| --- | --- | --- |
| `@hugeicons/core-free-icons` | Single icon set, tree-shakeable | First shell/resources icons |
| `@kobalte/core` | Accessible menu/dialog/select if native insufficient | First complex overlay beyond simple dialog |

Justify each in PR/tasks; do not pre-install unused kits.

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Core returns sparse resource fields | View-model defaults + fixtures; status `unknown` |
| Texture button fights quiet system | Restyle or retire texture as default |
| DESIGN.md vs design.md case | One rewrite; lint path already DESIGN.md |
| Scope creep into all nav destinations | Stubs only; P1 is Resources + Architecture |
| Agent forbidden `pnpm dev` | Quickstart documents user-run visual check |

---

## Success mapping

| Spec SC | Plan evidence |
| --- | --- |
| SC-001 shell anatomy | Shell rewrite + checklist |
| SC-002 brand restraint | Tokens + status-only green |
| SC-003 paths < 30s | Page routes Resources/Architecture |
| SC-004 dialogs | ResourceFormDialog |
| SC-005 keyboard | focus-visible + nav buttons |
| SC-006 canvas interactive | no permanent block; worker if needed |
| SC-007 long lists | virtual when N large |
| SC-008 states | loading/empty/error components |
| SC-009 anti-patterns | inspector removed; audit |
| SC-010 DESIGN.md matches UI | rewrite + token sync |
)
