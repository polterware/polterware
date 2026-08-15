# Implementation Plan: Remaining Operator Surfaces

**Branch**: `007-remaining-operator-surfaces` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/007-remaining-operator-surfaces/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (seven-destination shell, tabs, detail pages, process drawer, palette, shared states) and `003-operator-design-system` (tokens, shell anatomy).

**Supersedes within scope**: Hard “exactly seven destinations” + group label **KNOWLEDGE** from 004 → **eight** destinations and group **INSIGHTS**.

---

## Summary

Close remaining product surface gaps already implied by Polter architecture, without sidebar bloat:

1. **Shell IA v2** — eight primary destinations (CORE / **INSIGHTS** / CONFIGURATION); only **Observability** added to the sidebar; logo → **Project Overview**; project switcher popover; footer → **Sync center**.
2. **Global chrome** — notification drawer, full search results, synchronization center + conflict resolution.
3. **Observability hub** — Overview / Logs / Metrics / Traces / Monitors with provider hand-off (not a full APM clone).
4. **Resources provisioning** — Catalog / Executions / Drift + multi-step plan→approve→apply + node registration wizard.
5. **Architecture deep flows** — import mapping, snapshot compare, view editor.
6. **Operations** — workflow builder + schedules.
7. **Access** — Teams, Invitations, roles matrix, audit detail, agent sessions.
8. **Integrations** — setup wizard, webhooks; conflict reuse with Sync.
9. **Settings** — Organization, Developer, Billing, Security, Data (+ keep Project/Environments/Notifications/Core).
10. **Onboarding** — create/join org + first project (local-first).

Technical approach: extend Solid feature packages + Desktop `ClientRoute` composition. Prefer **view models + honest fixtures** over protobuf churn. Expand drawers only for **process** and **notifications**. One new primary package: **`packages/features/observability`**. No React; no scenic empty packages; no specialized-platform clones.

---

## Technical Context

**Language/Version**: TypeScript (strict) for Desktop/renderer/features; CSS via Tailwind v4 + design tokens

**Primary Dependencies**:

- Existing: Electron, electron-vite, SolidJS, Tailwind CSS v4, `@tanstack/solid-virtual`, `@polterware/frontend-design-system`, `@polterware/frontend-data`, feature packages (`shell`, `resources`, `operations`, `access`, `integrations`, `settings`, `architecture`, `commander`, `logs`, …), @hugeicons/core-free-icons (selective)
- Add only if first consumer needs: Kobalte pieces for drawer/popover a11y if native path insufficient
- Not added by default: `@solidjs/router`, chart libraries (prefer lightweight SVG/CSS sparklines or simple canvas charts; hand-off for full dashboards)

**Storage**: Domain authority remains polterware-core (SQLite). This feature is presentation + adapter wiring. Fixtures are module data only. No second client-side domain store of record.

**Testing**: Vitest for route helpers, filters, view-model pure functions, design-system primitives; package typecheck on changed surfaces; manual Desktop quickstart. Full `pnpm dev` only when user asks (Agents.md).

**Target Platform**: Polterware Desktop (macOS primary chrome; quiet dark operator UI)

**Project Type**: Desktop monorepo feature (presentation IA + feature surfaces)

**Performance Goals**: Shell remains interactive with live log tails paused by default or throttled; virtualize logs/search/audit/traces/notifications; sync status polling non-blocking; reduced-motion respected

**Constraints**: Constitution v2; DESIGN.md quiet operator language; renderer only via `window.polterware`; secrets write-once/rotate; mutations honor plan→review→approve→apply→logs→audit; never claim unimplemented runtimes live; Observability is hub+hand-off

**Scale/Scope**: ~39 wireframe surfaces grouped into 10 user stories; depth may be fixture-backed where core RPCs are thin; not full multi-cloud/APM/billing engines

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
| --- | --- | --- |
| I Operational architecture | **Pass** | Observe (hub, overview, notifications), organize (import, provision catalog), operate/approve (provisioning, workflows), collaborate (teams/invites/orgs), audit (audit detail, conflicts), external tools (dashboard hand-off) |
| II Domain before framework | **Pass** | View models in features/App; no domain rules inside design-system |
| III Runtime separation | **Pass** | Renderer presentation; Main/preload chrome/bridge; core owns execution/sync; MCP cannot bypass policy; Developer settings honest about MCP/CLI |
| IV Directed deps | **Pass** | apps/desktop → features → frontend → generated; no cycles |
| V Contracts-first | **Pass** | Reuse existing local protos; UI contracts under `contracts/`; extend protos only when a real core consumer ships |
| VI Security | **Pass** | Unprivileged renderer; secrets/tokens write-once; permission-denied UX; install tokens short-lived |
| VII Safe auditable exec | **Pass** | Provisioning/drift restore/workflow apply/webhook/org security changes route through governance; conflict resolution audited |
| VIII Local-first | **Pass** | Onboarding local-first; offline queue + conflict UX; billing unavailable state when edition lacks it |
| IX Performance | **Pass** | Virtual lists, pause live logs, non-blocking sync status |
| X Resilience | **Pass** | Empty/loading/error/offline/permission; webhook retry visibility; recoverable onboarding |
| XI Observability | **Pass** | Product Observability hub is aggregation+hand-off; correlation IDs on failures; not a Grafana clone |
| XII Testability | **Pass** | Route/tab helpers, pure mappers, quickstart acceptance; risk-proportional tests |
| XIII Real code | **Pass** | Expand existing packages; **only** new package with real consumer: `features/observability` (primary destination). Overview modules may live under shell to avoid scenic package |
| XIV SSoT | **Pass** | One nav IA (eight destinations); temporary dual seven/eight flags removed before done |
| XV Big bang context | **Pass** | Continues Solid operator product map on authorized architecture line |
| XVI Documentation | **Pass** | Plan artifacts + DESIGN.md IA update in implement phase |

**Hard stops (VI, IV, XIV)**: No violations. Temporary dual nav (seven vs eight) only during cutover tasks—must end with eight destinations + INSIGHTS only.

### Post-design re-check

Aligned: research R1–R14 resolve non-nav chrome sections, second drawer (notifications), package ownership, fixture honesty, provisioning governance, observability hand-off, and no protobuf churn by default. No unjustified scenic packages or dual design languages.

---

## Project Structure

### Documentation (this feature)

```text
specs/007-remaining-operator-surfaces/
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
DESIGN.md                                      # navigation model: 8 destinations, INSIGHTS, chrome surfaces

packages/features/shell/src/
  route.ts                                     # SectionId + non-nav sections; tabs; entityKinds; drawers; dialogs
  route.test.ts                                # expand coverage
  index.tsx / sidebar.tsx / title-bar.tsx      # DEFAULT_NAV 8 items; logo→overview; switcher; footer→sync
  project-switcher.tsx                         # large popover
  notification-drawer.tsx                      # temporary inbox drawer
  overview-page.tsx                            # Project Overview (shell-owned to avoid scenic package)

packages/features/observability/               # NEW real package — primary destination
  src/index.tsx                                # tabs shell
  overview, logs, metrics, traces, monitors
  trace-detail, monitor-detail, log-event-dialog
  fixtures.ts, types.ts

packages/features/resources/src/
  + provisioning-catalog, plan, execution, drift
  + node-registration-wizard
  SECTION_TABS resources: inventory|nodes|processes|provisioning

packages/features/architecture/src/
  + import-resources wizard, snapshot-compare, view-editor

packages/features/operations/src/
  + workflow-builder, schedules tab
  (compose feature-workflows as needed)

packages/features/access/src/
  + teams, invitations, roles-matrix, audit-detail, agent-session-detail
  tabs: members|teams|agents|policies|audit|invitations

packages/features/integrations/src/
  + setup wizard, webhooks tab, conflict detail (shared resolver)
  tabs: connected|catalog|credentials|sync|webhooks

packages/features/settings/src/
  + organization, developer, billing, security, data
  keep project|environments|notifications|core (+ account merge or retain)

packages/features/commander/src/
  + “Show all results” → section search; search-results page composition

packages/features/logs/src/                    # optional reuse inside Observability Logs tab
packages/frontend/design-system/src/           # popover/drawer/wizard step patterns if missing

apps/desktop/src/renderer/App.tsx              # Match blocks: overview, observability, search, sync, onboarding
apps/desktop/src/shared/ipc.ts                 # only if new bridge methods required (prefer none)

# Prefer existing Connect clients — extend adapters, not reinvent
```

**Structure Decision**:

| Concern | Placement |
| --- | --- |
| Runtime owner | Desktop Renderer (+ thin Main/preload) |
| Domain vs adapter | No new domain package; view models + fixtures in features; core via `@polterware/frontend-data` when healthy |
| Contracts | Existing protos where applicable (`log`, `health`, `integration`, `identity`, `audit`, `project`, `operation`, `workflow`, `secret`); UI contracts in this feature’s `contracts/` |
| New packages | **`packages/features/observability` only** — primary destination without a home today |
| Shell-owned chrome | Overview, project switcher, notification drawer, onboarding host views |
| Packages not created | No second design-system; no React paths; no scenic `features/sync` if Integrations+route suffice |
| Nav retirements | None beyond 004; **add** Observability; relabel KNOWLEDGE → INSIGHTS |

---

## Complexity Tracking

> No constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| — | — | — |

---

## Implementation Phases (for `/speckit-tasks`)

Ordered for independent demonstrable value.

### A. Route model + eight-destination shell + chrome entry points

- Extend `SectionId` with `observability`; non-nav sections: `overview`, `search`, `sync`, `onboarding`
- `DEFAULT_NAV` eight items; groups CORE / INSIGHTS / CONFIGURATION
- Logo → overview; chevron → project switcher; footer core status → sync; bell → notifications drawer
- History includes non-nav sections; legacy seven-destination deep links still resolve
- App Switch Match for new sections

### B. Project Overview + project switcher + notifications + search results

- Overview status / attention / activity / quick actions
- Switcher popover with org grouping + create project entry
- Notification drawer filters + deep links + mark all read
- Full search results page from palette “Show all results”

### C. Synchronization center + conflict resolution

- Status / Offline queue / Conflicts / History
- Shared conflict resolve UI used by Integrations Sync and Sync center
- Honest offline/cached banners

### D. Observability package

- Tabs Overview, Logs, Metrics, Traces, Monitors
- Trace detail, monitor detail, log event dialog
- External dashboard hand-off; fixtures + optional log/core adapters

### E. Resources provisioning + node wizard

- Provisioning tab (Catalog, Executions, Drift)
- Configure → plan review → approval gate → execution detail
- Node registration wizard with expiring token UX

### F. Architecture deep flows

- Import resources mapping wizard
- Snapshot comparison
- View editor

### G. Operations workflow builder + schedules

- Step list + step config panel
- Schedules secondary tab

### H. Access expansions

- Teams + team detail; Invitations; Roles matrix; Audit event detail; Agent session detail

### I. Integrations wizard + webhooks

- Multi-step connect wizard
- Webhooks list/detail/test/deliveries
- Wire conflicts to shared resolver

### J. Settings expansions + onboarding

- Organization, Developer, Billing, Security, Data
- First-run onboarding org/project local-first

### K. Validation

- Static IA checks (eight destinations, INSIGHTS)
- Route tests; fixture honesty; a11y smoke notes; quickstart paths
- DESIGN.md navigation update

---

## Agent context

No project agent-context update script is registered under `.specify/scripts/`. Active feature pointer remains `.specify/feature.json` → `specs/007-remaining-operator-surfaces`. Downstream `/speckit-tasks` should read this plan + contracts.
