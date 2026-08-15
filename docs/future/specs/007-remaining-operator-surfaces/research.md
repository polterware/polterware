# Research: Remaining Operator Surfaces

**Feature**: `007-remaining-operator-surfaces`  
**Date**: 2026-08-03  
**Status**: Complete — planning unknowns resolved against repo evidence + `spec.md` + `004` artifacts

---

## R1. Navigation: seven → eight + INSIGHTS

**Decision**: Replace authoritative product nav with:

| Group | Destinations |
| --- | --- |
| CORE | Architecture, Resources, Operations |
| INSIGHTS | Observability, Knowledge, Access |
| CONFIGURATION | Integrations, Settings |

Exactly **eight** primary sidebar destinations. Only **Observability** is newly added.

Relocations / chrome (not sidebar):

| Capability | Home |
| --- | --- |
| Project Overview | Logo / project name click → non-nav section `overview` |
| Project switcher | Popover from chevron beside project name |
| Notifications | Temporary drawer from title-bar bell |
| Full search results | Non-nav section `search` via palette “Show all results” |
| Synchronization center | Non-nav section `sync` from footer core status |
| Provisioning | Resources → tab `provisioning` |
| Webhooks | Integrations → tab `webhooks` |
| Teams / Invitations | Access → tabs |
| Billing / Security / Data / Org / Developer | Settings tabs |

**Rationale**: Spec FR-001–FR-007; prevents sidebar inflation; supersedes 004 FR-001 seven-destination rule within this feature’s scope.

**Alternatives considered**: Nested sidebar trees (rejected); Observability under Operations (rejected — Observe is first-class); billing as destination (rejected — settings only).

---

## R2. Non-nav sections in ClientRoute

**Decision**: Extend route model:

```ts
type NavSectionId =
  | "architecture"
  | "resources"
  | "ops"
  | "observability"
  | "knowledge"
  | "access"
  | "integrations"
  | "settings";

type ChromeSectionId = "overview" | "search" | "sync" | "onboarding";

type SectionId = NavSectionId | ChromeSectionId;
```

- `DEFAULT_NAV` lists **only** `NavSectionId` (eight items).
- Chrome sections are valid `route.section` values for history/deep links but **never** appear as permanent sidebar items.
- `isSectionId` / `resolveSection` accept both; `isNavSectionId` used for sidebar active state (overview may show no nav highlight or subtle none).

**Rationale**: Matches wireframe routes (`/overview`, `/search`, `/sync`, `/onboarding`) without inventing a second router; keeps history first-class.

**Alternatives considered**: Overview as Settings subpage (rejected — logo entry); overlays-only for search/sync (rejected — full-page wireframes for search/sync).

---

## R3. Drawers: process + notifications

**Decision**:

```ts
type DrawerState =
  | { type: "process"; processId: string }
  | { type: "notifications" }
  | null;
```

- Process drawer remains the only **domain entity** drawer (ephemeral process detail).
- Notifications drawer is a **global inbox overlay** (FR-062), not entity detail.
- All other durable entities stay full pages. Project switcher is a **popover**, not a drawer.

**Rationale**: Spec allows notification drawer; 004 process exclusivity preserved for domain entities.

**Alternatives considered**: Notifications as full page (rejected — wireframe drawer); reusing process drawer for entities (forbidden).

---

## R4. Data strategy: core adapters + fixtures

**Decision**:

| Surface | Prefer core when healthy | Fixture when offline/empty |
| --- | --- | --- |
| Overview snapshot | Health + operation/approval lists if available | Fixture status/attention/activity |
| Notifications | Future notification RPC if present | Fixture inbox |
| Search | Local indexes over loaded entities | Fixture hits from demo catalog |
| Sync center | Integration sync + health | Fixture endpoints/queue/conflicts |
| Observability | `LogService` + integration health; metrics/traces fixture-first | Labeled fixtures; never fake “live APM” |
| Provisioning | Operation/approval path for apply | Fixture catalog/plans/executions/drift |
| Teams/invites/roles | Identity/policy when ready | Fixtures |
| Webhooks | Integration adapters | Fixtures |
| Billing | None required for local-only | Unavailable state **or** fixture plan when demo mode |

Always expose `dataSource: "core" | "fixture" | "empty" | "cached" | "unavailable"` for honesty banners.

**Rationale**: Constitution honesty + VIII local-first; 004 R4 pattern; avoids blocking UI on incomplete backend depth.

**Alternatives considered**: Block UI until full observability backend (rejected); invent parallel IndexedDB domain authority (rejected — dual store).

---

## R5. Package ownership map

**Decision**:

| Surface | Package |
| --- | --- |
| Shell, route, overview page, switcher, notifications drawer, onboarding host | `packages/features/shell` |
| Observability (primary destination) | **`packages/features/observability` (new)** |
| Search results page | `packages/features/commander` (+ App Match) |
| Sync center page + shared conflict resolver | `packages/features/integrations` (sync domain home) |
| Provisioning + node wizard | `packages/features/resources` |
| Architecture import/compare/view editor | `packages/features/architecture` |
| Workflow builder + schedules | `packages/features/operations` (+ `feature-workflows` if present) |
| Teams, invitations, roles, audit detail, agent session | `packages/features/access` |
| Integration setup wizard + webhooks | `packages/features/integrations` |
| Settings expansions | `packages/features/settings` |
| Composition | `apps/desktop/src/renderer/App.tsx` |
| Log stream helpers | Reuse `packages/features/logs` / `frontend-logs` inside Observability when useful |

**Rationale**: One justified new package (Observability primary destination). Overview stays shell-owned to avoid a scenic second package. Sync lives with Integrations domain while remaining a chrome route.

**Alternatives considered**: `features/overview` package (unnecessary indirection); `features/sync` package (scenic if only one page); stuffing Observability into Operations (IA conflict).

---

## R6. Observability = hub + hand-off

**Decision**: Ship control-plane hub (health summary, filtered logs, key metrics sparklines, trace list/waterfall, monitors list/detail). Advanced analysis → **Open Grafana / Open Datadog / Open provider** when integration connected. Do not embed full third-party consoles.

**Rationale**: Spec FR-025; Constitution I non-goals; XI “not a product replacement for external APM”.

**Alternatives considered**: Embed Grafana iframe (rejected — coupling, security, incomplete clone risk).

---

## R7. Provisioning governance

**Decision**: Provisioning UI always builds an explicit **plan**, runs **policy checks**, and:

- If production approval required → **Request approval and provision** (create approval + link execution)
- Else if permitted → apply path still records operation/audit
- Never “silent create” without plan review step for multi-resource templates

Drift actions (Accept / Restore / Ignore) each write audit outcomes.

**Rationale**: Spec FR-028–FR-032; Constitution VII.

**Alternatives considered**: Instant provision button without plan (rejected).

---

## R8. Shared conflict resolution

**Decision**: One conflict detail component/view-model used by:

1. Synchronization center → Conflicts  
2. Integrations → Sync → conflict row  
3. Notification deep link “sync conflict”

Resolution choices: keep Polter / accept provider / ignore until next sync. Always audit.

**Rationale**: Spec FR-018, FR-046; single source of truth for conflict UX.

**Alternatives considered**: Separate conflict UIs per entry (rejected — dual behavior risk).

---

## R9. EntityKind and dialog expansions

**Decision**: Extend `EntityKind` with at least:

`trace` | `monitor` | `provision-execution` | `provision-plan` | `team` | `invitation` | `agent-session` | `webhook` | `audit-event` | `sync-conflict` | `schedule` | `notification` (if detail ever needed)

Extend `DialogState` / wizard modes for:

- provisioning configure (multi-step may be full page per wireframe — prefer **page wizard** for provision/node/integration setup; dialogs for small confirms)
- node registration steps
- integration connect steps
- invite people
- create team / create monitor / create webhook / create schedule

**Rationale**: Wireframes use multi-step full pages for long wizards; 004 used dialogs for short forms. Align by length/risk.

**Alternatives considered**: All wizards as dialogs (rejected — cramped, poor for token/install UX).

---

## R10. Settings tab model

**Decision**: Settings tabs become:

`project` | `organization` | `environments` | `notifications` | `core` | `developer` | `billing` | `security` | `data`

- `account` from 004 may fold into Security (sessions) + profile fields, or remain; prefer **merge profile into Security/Project** only if implement tasks prove redundancy—default **keep Account** if already shipped to avoid churn, else map Account → Security for sessions and Project for display name.
- **Recommended cutover**: tabs listed above; if Account already in code, retain until migrate profile fields then remove dual tab.

Billing shows **unavailable** when edition/account has no commercial plan (local-only).

**Rationale**: Spec FR-047–FR-052; honesty for local-first.

**Alternatives considered**: Nested settings sidebar (rejected — in-page tabs only).

---

## R11. Access tab model

**Decision**: Access tabs:

`members` | `teams` | `agents` | `policies` | `audit` | `invitations`

Roles matrix reachable from Policies via secondary route (`entityKind` role-matrix or tab `roles`).

**Rationale**: Spec FR-038–FR-041.

---

## R12. Resources / Integrations / Observability tab catalogs

**Decision**:

| Section | Tabs |
| --- | --- |
| resources | `inventory`, `nodes`, `processes`, **`provisioning`** |
| observability | `overview`, `logs`, `metrics`, `traces`, `monitors` |
| integrations | `connected`, `catalog`, `credentials`, `sync`, **`webhooks`** |
| ops workflows | primary `workflows`; secondary schedules via tab `schedules` under ops **or** filter on workflows — **choose `ops` tabs: `runs`, `workflows`, `schedules`, `approvals`, `terminal`** if schedules deserve equal weight; if not, schedules as sub-tab inside Workflows page. Spec shows secondary tab under Workflows → **implement as in-page secondary strip on Workflows** first to avoid ops tab explosion; promote to top-level ops tab only if tasks prove density issues. |

Provisioning internal strip: `catalog` | `executions` | `drift` (sub-tabs / filter, not sidebar).

**Rationale**: Spec navigation consolidated; minimize top-level tab count while matching wireframes.

---

## R13. Charts without new heavy deps

**Decision**: Use lightweight SVG sparklines / simple path charts for metrics and monitor trends. Full dashboards via external hand-off. No Recharts/Chart.js unless a later task proves insufficient—and only with concrete leverage approval.

**Rationale**: Agents.md no new deps without leverage; Observability is hub not APM.

**Alternatives considered**: Full chart library now (deferred).

---

## R14. Protobuf / IPC churn policy

**Decision**: Default **no** new proto packages in this feature’s critical path. Wire UI contracts first; use fixtures; bind to existing `log`, `health`, `integration`, `identity`, `audit`, `operation`, `workflow`, `project`, `secret` when adapters already exist. Introduce new RPCs only when implement tasks hit a hard missing authority boundary (e.g. durable notifications store)—then contracts-first in `schemas/` with generated Go + Connect-ES.

IPC (`apps/desktop/src/shared/ipc.ts`): prefer zero new channels; reuse core HTTP/Connect via preload bridge already used by App.

**Rationale**: Constitution V + XIII; 004 same policy worked.

**Alternatives considered**: Speculative notification/billing protos before UI consumers (rejected — scenic contracts).

---

## R15. Onboarding entry

**Decision**: When no organization/project exists (or first-run flag), `route.section = "onboarding"` replaces main destinations until project created. Local-first storage is default radio. Join-with-invite is alternate path. After create → `overview` or `architecture` with empty guided state.

**Rationale**: Spec FR-053–FR-054; User Story 10.

**Alternatives considered**: Force cloud signup (rejected — local-first constitution).

---

## Resolved unknowns

| Topic | Resolution |
| --- | --- |
| Where does Overview live? | Non-nav section + shell package page |
| New packages? | Only `features/observability` |
| Second drawer? | Notifications only (+ process) |
| Metrics charts? | Lightweight SVG; hand-off for full |
| Billing without cloud? | Unavailable state |
| Schedules placement? | Secondary strip under Workflows first |
| Proto expansion? | Not default |

No remaining NEEDS CLARIFICATION for planning gates.
