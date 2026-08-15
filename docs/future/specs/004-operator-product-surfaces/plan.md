# Implementation Plan: Operator Product Surfaces

**Branch**: `004-operator-product-surfaces` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/004-operator-product-surfaces/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `003-operator-design-system` (shell anatomy, tokens, design-system primitives, Resources/Architecture foundation, no permanent inspectors)

---

## Summary

Complete the **Polter Desktop product surface map** under the established quiet-dark operator design system:

1. **Collapse primary navigation to seven destinations** (CORE / KNOWLEDGE / CONFIGURATION) with **in-page tabs** for all subdivisions—no extra sidebar depth.
2. **Finish Architecture** tabs (Canvas, Components, Dependencies, Views, Changes) + node detail + create-node dialog.
3. **Finish Resources** tabs (Inventory, Nodes, Processes) + resource/node detail + **process drawer only**.
4. **Ship Operations** tabs (Runs, Workflows, Approvals, Terminal) with run/command dialogs and approval gates.
5. **Ship Knowledge, Access, Integrations, Settings** as full tabbed destinations with list + detail patterns.
6. **Global overlays**: command palette, shared empty/loading/offline/permission states.
7. **Honest data**: core when healthy, labeled fixtures when offline/empty; never claim live sync/mutations that are not wired.

Technical approach: extend existing Solid feature packages + Desktop composition (`ClientRoute` + history). Prefer view models + fixtures over protobuf churn. Add design-system **Tabs**, **Drawer** (process-only), and **Dialog** coverage as shared primitives. No React; no new scenic apps; no specialized-platform clones.

---

## Technical Context

**Language/Version**: TypeScript (strict) for Desktop/renderer/features; CSS via Tailwind v4 + design tokens

**Primary Dependencies**:

- Existing: Electron, electron-vite, SolidJS, Tailwind CSS v4, `@tanstack/solid-virtual`, `@polterware/frontend-canvas`, `@polterware/frontend-data` (Connect), `@polterware/frontend-design-system`, `@polterware/frontend-terminal`, `@polterware/frontend-logs`, feature packages, @hugeicons/core-free-icons (selective)
- Add only if a first consumer needs them: `@kobalte/core` pieces for Tabs/Dialog/Drawer a11y if native path insufficient
- Not added by default: `@solidjs/router` (typed `ClientRoute` remains composition owner)

**Storage**: Domain authority remains polterware-core (SQLite). This feature is presentation + adapter wiring. Fixtures are module data only.

**Testing**: Vitest for pure route/filter/view-model helpers and design-system primitives; manual Desktop quickstart; typecheck on changed packages. Full `pnpm dev` only when user asks (Agents.md).

**Target Platform**: Polterware Desktop (macOS primary chrome; quiet dark operator UI)

**Project Type**: Desktop monorepo feature (presentation IA + feature surfaces)

**Performance Goals**: Shell interactive during canvas pan/zoom (prototype-scale); dense tables usable at hundreds of rows (virtualize); live process table updates must not freeze chrome; control transitions respect reduced-motion

**Constraints**: Constitution v2; single visual SSOT (`DESIGN.md` from 003); exactly one justified drawer (processes); brand green restricted; renderer only via `window.polterware`; secrets never re-displayed; mutations honor plan→approve→audit when governed; no claiming unimplemented runtimes live

**Scale/Scope**: Full seven-destination IA + all tabs/details/dialogs/states in the wireframe set; depth may be fixture-backed where core RPCs are thin; not full multi-cloud admin consoles

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
| --- | --- | --- |
| I Operational architecture | **Pass** | Full product loop surfaces: map, resources, operate, approve, audit, knowledge, integrations hand-off |
| II Domain before framework | **Pass** | View models in features/App; no domain rules inside design-system |
| III Runtime separation | **Pass** | Renderer presentation; Main/preload chrome/bridge; core owns execution |
| IV Directed deps | **Pass** | apps/desktop → features → frontend → generated; no cycles |
| V Contracts-first | **Pass** | Reuse existing Connect protos; UI contracts under `contracts/`; no hand-duplicated wire types |
| VI Security | **Pass** | Unprivileged renderer; credentials as references; permission-denied UX; no secret re-display |
| VII Safe auditable exec | **Pass** | Operations/Approvals/Run command UI routes through request/approve; no silent apply |
| VIII Local-first | **Pass** | Offline/cached honesty; Core settings surface local authority |
| IX Performance | **Pass** | Virtual lists, non-blocking process refresh, canvas workers as needed |
| X Resilience | **Pass** | Empty/loading/error/offline/permission states + terminal reconnect |
| XI Observability | **Pass** | Surface operation/sync failures; correlation when ops invoked |
| XII Testability | **Pass** | Route/tab helpers, filters, primitive tests; quickstart acceptance |
| XIII Real code | **Pass** | Expand existing feature packages; access package only if real consumers (see Structure) |
| XIV SSoT | **Pass** | One nav IA; retire multi-item sidebar; no permanent dual inspectors |
| XV Big bang context | **Pass** | Completes Solid operator product map on authorized architecture line |
| XVI Documentation | **Pass** | Plan artifacts + DESIGN.md navigation model update in implement phase |

**Hard stops (VI, IV, XIV)**: No violations. Temporary dual nav (old multi-item DEFAULT_NAV vs seven-item) only during migration cutover inside implement tasks—must end with seven destinations only.

### Post-design re-check

Aligned: research R1–R12 resolve nav collapse, `ClientRoute.tab`, package ownership, process drawer exclusivity, fixture honesty, Access composition, and no protobuf churn as default. No unjustified scenic packages or dual design languages.

---

## Project Structure

### Documentation (this feature)

```text
specs/004-operator-product-surfaces/
├── plan.md              # This file
├── research.md          # Phase 0
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/           # Phase 1 UI contracts
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 via /speckit-tasks (not this command)
```

### Source Code (paths this feature will modify)

```text
DESIGN.md                                    # navigation model / screen blueprints update (consume 003 tokens)

packages/frontend/design-system/src/
  page.tsx, feedback.tsx, toast.tsx, button.*  # existing
  tabs.tsx (+ tests)                           # page tab strip primitive
  drawer.tsx (+ tests)                         # process drawer only consumer initially
  dialog.tsx / a11y dialog evolution           # create-node, run-command, forms
  table helpers optional                       # dense data table styles if shared

packages/features/shell/src/
  route.ts                                     # tab, entityKind, drawer, dialog variants, history
  index.tsx / sidebar.tsx                      # DEFAULT_NAV → 7 destinations, groups CORE|KNOWLEDGE|CONFIGURATION
  title-bar.tsx, main-surface.tsx              # keep; status/identity wiring

packages/features/architecture/src/
  index + canvas + node-detail (existing)
  components-table, dependencies-table, views-grid, changes-feed
  create-node-dialog, fixtures expansion

packages/features/resources/src/
  inventory (existing index), resource-detail, form dialog
  nodes-index, node-detail
  processes-table + process-drawer integration
  fixtures expansion

packages/features/operations/src/
  runs list/detail, workflows list/detail, approvals list/detail, terminal sessions
  run-command dialog composition
  (may import/wrap feature-workflows, feature-approvals, frontend-terminal)

packages/features/knowledge/src/               # docs, decisions, runbooks, ownership + document detail
packages/features/access/src/                  # NEW real package: members, agents, policies, audit
  (agents package may re-export into access or be thin wrapper—see research R5)

packages/features/integrations/src/            # connected, catalog, credentials, sync + detail
packages/features/settings/src/                # project, environments, notifications, core, account

packages/features/commander/src/               # command palette overlay (not sidebar destination)
packages/features/processes/                   # absorb into Resources Processes or keep as pure process list helper
packages/features/logs/                        # no primary nav; open from quick actions / ops as needed

apps/desktop/src/renderer/App.tsx              # composition root: route, data adapters, section Switch
apps/desktop/src/shared/ipc.ts                 # only if new bridge methods are required (prefer none)

# Prefer existing Connect clients in packages/frontend/data — extend adapters, not reinvent
```

**Structure Decision**:

| Concern | Placement |
| --- | --- |
| Runtime owner | Desktop Renderer (+ thin Main/preload) |
| Domain vs adapter | No new domain package; view models + fixtures in features; core via `@polterware/frontend-data` |
| Contracts | Existing protos (`operation`, `approval`, `workflow`, `process`, `knowledge`, `integration`, `policy`, `audit`, `identity`, `secret`, `project`); UI contracts in this feature’s `contracts/` |
| New packages | **`packages/features/access` only** — Access is a primary destination with no package home today; agents stub is absorbed or re-exported |
| Packages not created | No scenic empty packages; no second design-system; no React paths |
| Nav retirements | Sidebar loses Processes, Logs, Approvals, Workflows, Commander, Agents as top-level items (relocated under tabs or overlays) |

---

## Complexity Tracking

> No constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| — | — | — |

---

## Implementation Phases (for `/speckit-tasks`)

Ordered for independent demonstrable value.

### A. Route model + seven-destination shell

- Extend `ClientRoute` with `tab`, `entityKind`, `drawer`
- Replace `DEFAULT_NAV` with seven items and groups CORE / KNOWLEDGE / CONFIGURATION
- History includes tab when it changes primary context
- App Switch maps only seven sections; redirects legacy section ids

### B. Shared design-system primitives

- Tabs, Drawer, Dialog completeness, PageEmpty/Loading/Error/Permission/Offline, dense table patterns
- Update DESIGN.md navigation + blueprints for full IA

### C. Architecture tabs completion

- Components, Dependencies, Views, Changes + create-node dialog
- Canvas remains full-width; node detail page retained/enhanced

### D. Resources tabs completion

- Nodes list/detail; Processes table + process drawer; inventory polish

### E. Operations destination

- Runs, Workflows, Approvals, Terminal; run-command dialog; permission/approval honesty

### F. Knowledge + Access

- Knowledge four tabs + document detail
- Access four tabs + member/agent/policy detail + audit export UI

### G. Integrations + Settings

- Integrations four tabs + detail; credentials never re-show secrets
- Settings five tabs including Core honesty

### H. Command palette + global states

- Commander as palette overlay; wire empty/offline/permission across destinations

### I. Validation

- Static checks + quickstart paths; fixture honesty banners; a11y smoke notes

---

## Agent context

No project agent-context update script is registered under `.specify/scripts/`. Active feature pointer remains `.specify/feature.json` → `specs/004-operator-product-surfaces`. Downstream `/speckit-tasks` should read this plan + contracts.
