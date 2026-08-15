# Implementation Plan: Organization → Project Navigation Hierarchy

**Branch**: `008-org-project-nav-hierarchy` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/008-org-project-nav-hierarchy/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `007-remaining-operator-surfaces` (workspace breadcrumb, Settings tabs, onboarding chrome), `004-operator-product-surfaces` (ClientRoute shell), `003-operator-design-system` (tokens, PageHeader/Input/Button/Card).

**Supersedes within scope**: Chrome entry points from 007 that route **“All organizations” / “Manage organizations” → Settings → Organization**. Settings may retain **organization admin configuration** only; multi-org catalog and org project lists become hierarchy chrome sections above the project shell.

---

## Summary

Restore the product hierarchy operators expect:

```text
Organizations root  →  Organization home (projects)  →  Project workspace (sidebar tabs)
```

Technical approach (presentation-only, fixture-honest):

1. **Chrome sections** — add routable non-sidebar sections `organizations` (root index) and `organization` (org home / project list), same class as `overview` | `search` | `sync` | `onboarding`.
2. **Breadcrumb split** — organization segment becomes **name control** (navigate to org home) + **chevron control** (open switcher menu only). Project segment: name → project overview; chevron → project switcher limited to current org.
3. **Hierarchy pages** — simple full-page indexes in `packages/features/shell` (`OrganizationsIndexPage`, `OrganizationHomePage`) using design-system Input/Button/Card — not Settings tabs, not scenic empty packages.
4. **Shell chrome mode** — when `section` is hierarchy chrome (`organizations` | `organization`), **hide project sidebar** (and project-scoped footer affordances that imply an active project workspace). Project shell returns only after entering a project.
5. **App retargeting** — `onManageOrganizations` → `goSection(..., "organizations")`; org selection from menu → org home (not auto-first-project); enter project from org home → default project surface + restore sidebar.
6. **Route/workspace context** — extend `ClientRoute` (or parallel durable fields) so history carries `organizationId` / optional `projectId` for hierarchy levels; pure helpers unit-tested.
7. **Settings boundary** — keep Settings → Organization as **admin config** for the owning org of the *current project*; remove catalog/list-of-all-orgs behavior from that tab if present; no chrome “All orgs” path lands there.

No new feature package. No protobuf churn. No React. No full IAM/billing product.

---

## Technical Context

**Language/Version**: TypeScript (strict) for Desktop renderer + feature packages; CSS via Tailwind v4 + Polter design tokens

**Primary Dependencies**:

- Existing: SolidJS, Electron (renderer via `window.polterware` only for any core calls — not required for this UX), `@polterware/frontend-design-system`, `@polterware/features-shell` (`route`, `WorkspaceContextBar`, fixtures), @hugeicons/core-free-icons
- Reuse: `Popover`, `MenuItem`, `Input`, `Button`, `Card`, `PageHeader`/`PageContent`, `FixtureBanner`
- Not added: `@solidjs/router`, new UI libraries, scenic packages

**Storage**: No new domain store. List data from existing shell fixtures (`ORGANIZATION_FIXTURES`, `PROJECT_SWITCHER_FIXTURES`) until Core org/project list RPCs exist. Domain authority remains polterware-core when live.

**Testing**: Vitest for `route.ts` helpers (chrome section resolution, hierarchy navigation pure functions, history snapshot fields); component/unit tests for breadcrumb split handlers if extractable; static typecheck of shell + desktop composition. Full `pnpm dev` only when Erick asks (Agents.md).

**Target Platform**: Polterware Desktop operator UI (macOS primary)

**Project Type**: Desktop monorepo — shell IA / navigation correction

**Performance Goals**: Instant local navigation; searchable lists remain interactive for fixture-scale (≤ dozens of orgs/projects); virtualize only if list sizes grow later

**Constraints**: Constitution v2; DESIGN.md one visual language; hierarchy pages are simple indexes (spacing, not hairline-divider catalogs unless matching existing simple list patterns); never claim live multi-tenant org APIs when fixture; do not reintroduce dual “org catalog in Settings” path

**Scale/Scope**: Two new chrome surfaces + breadcrumb behavior + shell layout mode + App wiring; Settings tab wording/boundary cleanup; supersedes incorrect 007 routing only for discovery

**Unknowns at plan start**: Resolved in [research.md](./research.md) (R1–R8). No remaining NEEDS CLARIFICATION for planning.

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
| --- | --- | --- |
| I Operational architecture | **Pass** | Discover/organize systems (projects) under correct org boundary; entry path into Model→Observe loop |
| II Domain before framework | **Pass** | Hierarchy rules in pure route/workspace helpers; UI only renders |
| III Runtime separation | **Pass** | Renderer/shell only; no Main business rules; no live Core claim required |
| IV Directed deps | **Pass** | `apps/desktop` → `features/shell` → design-system; no cycles; no new package |
| V Contracts-first | **Pass** | UI contracts under `specs/008/.../contracts/`; no duplicate protobuf types |
| VI Security | **Pass** | Unprivileged renderer; lists show only fixture/accessible items; no secrets |
| VII Safe auditable exec | **Pass** | Navigate/list read-only; create org/project remain stubs/onboarding until live mutations |
| VIII Local-first | **Pass** | Fixture/local lists work offline; honest fixture banners |
| IX Performance | **Pass** | Lightweight pages; no Main work |
| X Resilience | **Pass** | Empty org / empty account states; recover to root/home not blank project shell |
| XI Observability | **Pass** | n/a product APM; route changes stay client-local |
| XII Testability | **Pass** | Pure route helpers + quickstart acceptance scenarios |
| XIII Real code | **Pass** | Pages live in existing `shell` (same pattern as OverviewPage); no scenic package |
| XIV SSoT | **Pass** | One hierarchy model; remove dual “All orgs = Settings” meaning before done |
| XV Big bang context | **Pass** | Corrective nav on authorized refactor line |
| XVI Documentation | **Pass** | Plan artifacts + contract supersession notes |

**Hard stops (VI, IV, XIV)**: No violations. Temporary dual path (old Settings link vs new root) only during implement cutover tasks — must end with chrome → hierarchy only.

### Post-design re-check

Aligned with research R1–R8: chrome section ids, sidebar hide mode, ClientRoute context fields, Settings admin-only boundary, no new package, fixture-first. No unjustified complexity.

---

## Project Structure

### Documentation (this feature)

```text
specs/008-org-project-nav-hierarchy/
├── plan.md              # This file
├── research.md          # Phase 0
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/           # Phase 1 UI/route contracts
│   ├── hierarchy-shell.md
│   └── breadcrumb-controls.md
└── tasks.md             # Phase 2 (/speckit-tasks) — not created here
```

### Source Code (paths this feature will modify)

```text
packages/features/shell/src/
├── route.ts                    # ChromeSectionId + DEFAULT_TABS + resolveSection + helpers
├── types.ts                    # Optional route/workspace field types if needed
├── context-switchers.tsx       # Split org (and project) name vs chevron; callbacks
├── title-bar.tsx               # Pass through new handlers
├── index.tsx                   # Hide sidebar in hierarchy mode; wire callbacks
├── organizations-index.tsx     # NEW — organizations root page
├── organization-home.tsx       # NEW — org projects page
├── fixtures.ts                 # Ensure multi-org multi-project fixtures + project counts
└── (tests) route hierarchy helpers

apps/desktop/src/renderer/App.tsx
├── onManageOrganizations → organizations root
├── Match sections for organizations | organization
├── enter project from org home
├── select organization → organization home (not auto-first-project)
└── history includes hierarchy context fields

packages/features/settings/src/index.tsx
└── Organization tab: admin config only; optional “View organizations” link out to hierarchy
   (no multi-org catalog as primary content)

# Not created
packages/features/organizations/   # scenic package forbidden
```

**Structure Decision**:

| Concern | Placement | Why |
| --- | --- | --- |
| Runtime owner | Desktop Renderer + shell feature | Presentation IA only |
| Domain rules | Pure helpers in `route.ts` / small workspace helpers | Testable without Electron |
| Hierarchy pages | `packages/features/shell` | Same ownership as OverviewPage/onboarding chrome; real consumer = Desktop App |
| Contracts | Spec `contracts/` UI contracts | No protobuf until Core lists orgs/projects as live API |
| New package? | **None** | Constitution XIII — Overview already lives in shell |

---

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Notes |
| --- | --- |
| Temporary dual path during implement | Old Settings deep-link may remain one release of code until App retarget + grep clean; must be gone before feature done (XIV) |
