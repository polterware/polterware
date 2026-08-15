# Feature Specification: Organization → Project Navigation Hierarchy

**Feature Branch**: `008-org-project-nav-hierarchy`

**Created**: 2026-08-03

**Status**: Draft

**Input**: User description: "Breadcrumb context names open their dropdowns without separate chevron buttons. Each dropdown includes explicit navigation to that context's home or overview. Environment is represented only by its semantic kind badge. 'All organizations' / org-root entry opens a root screen of all organizations (not project Settings). Navigation hierarchy is Organization → Project → project sidebar tabs. Do not nest organization list/management pages under a project's Settings."

## Problem Statement

Today the product treats workspace switching as menus only, and routes “manage / all organizations” into **Settings → Organization** while the operator is still inside a **project** shell (sidebar, project tabs, project-scoped chrome). That inverts the real hierarchy:

```text
Organizations (root)
  └── Organization (projects of that org)
        └── Project (sidebar sections / tabs)
```

Operators expect organization and project discovery to sit **above** project work, not inside project configuration. The title-bar breadcrumb provides one compact switcher trigger per context, while each menu exposes an explicit path to that context's home or overview.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Breadcrumb uses one trigger per context (Priority: P1)

As an operator working inside a project, I see each breadcrumb context as one compact trigger. Organization and project names open their switcher menus, while environment is represented only by its semantic kind badge. Each menu contains an explicit action for the current context's home or overview.

**Why this priority**: This is the primary way operators move up the hierarchy; without the split, organization home is unreachable from everyday project work.

**Independent Test**: From any project surface, activate each context trigger → its menu opens without a route change; activate Organization home or Project overview → navigate to the documented destination; select an environment → update only the active environment context.

**Acceptance Scenarios**:

1. **Given** the operator is on a project surface (e.g. Architecture), **When** they click the organization name, project name, or environment badge, **Then** the corresponding switcher opens and the underlying route remains unchanged until they choose a menu action.
2. **Given** the organization switcher is open, **When** they choose **Organization home**, **Then** they navigate to the current organization’s project list rather than project Settings.
3. **Given** the organization switcher menu is open, **When** they click outside or dismiss it, **Then** they remain on the same project surface with no route change.
4. **Given** the breadcrumb, **When** it is inspected, **Then** no context has a separate dropdown-icon button and the environment trigger visually contains only its semantic kind badge.

---

### User Story 2 - Organization home lists projects (Priority: P1)

As an operator who clicked an organization name (or entered an organization from the orgs root), I see a **simple full-page organization home** for that organization: search/filter for projects, the list of projects belonging to that organization, and a clear action to create a new project. Selecting a project enters that project’s workspace (sidebar + default project section).

**Why this priority**: Organization → Project is the core hierarchical step; without a dedicated org home, multi-project orgs cannot be browsed correctly.

**Independent Test**: Open org home for an org with multiple projects; confirm only that org’s projects appear; open one project and confirm project shell/sidebar appears.

**Acceptance Scenarios**:

1. **Given** organization home for Org A, **When** the page loads, **Then** it lists only projects that belong to Org A, with enough identity (name and optional plan/meta) to choose among them.
2. **Given** organization home, **When** the operator clicks a project, **Then** they enter that project’s workspace at a sensible default surface (e.g. Overview or Architecture) with the project sidebar sections available.
3. **Given** organization home, **When** the operator uses search, **Then** the project list filters to matching projects of that organization only.
4. **Given** organization home, **When** the operator chooses “New project” (or equivalent), **Then** project creation starts in the context of the current organization.
5. **Given** organization home, **When** chrome/sidebar is shown, **Then** project-scoped sidebar destinations (Architecture, Resources, Ops, …) are **not** presented as the primary nav of this level—org home is above project work.

---

### User Story 3 - Organizations root is above any organization (Priority: P1)

As an operator, choosing **All organizations** (from the org switcher, org home chrome, or equivalent) opens a **root full-page index of all organizations** I can access—search, list cards/rows (name, plan, project count), open an organization, or create a new organization. This screen is **not** Settings inside a project.

**Why this priority**: Without a true root, multi-org operators cannot leave project context to manage org membership and discovery.

**Independent Test**: From a project, open the organization trigger → All organizations → see root list; open an org → see that org’s projects; no Settings chrome required.

**Acceptance Scenarios**:

1. **Given** the organization switcher menu, **When** the operator chooses “All organizations”, **Then** they navigate to the organizations root full page (list of orgs), not Settings → Organization under the current project.
2. **Given** the organizations root, **When** the operator clicks an organization, **Then** they open that organization’s home (projects list).
3. **Given** the organizations root, **When** the operator chooses “New organization”, **Then** organization creation starts at root scope.
4. **Given** the organizations root, **When** search is used, **Then** the list filters to matching organizations.
5. **Given** the organizations root, **When** the page is shown, **Then** it does not present project sidebar sections as if the operator were inside a project workspace.

---

### User Story 4 - Project workspace remains project-scoped (Priority: P2)

As an operator who has entered a project, I use the **project sidebar** (Architecture, Resources, Operations, Observability, Knowledge, Access, Integrations, Settings) and project/environment breadcrumb segments for day-to-day work. Project Settings may still hold **project-level** configuration and a link or short summary of the parent organization identity, but **must not** be the place where “all organizations” or the multi-org catalog lives.

**Why this priority**: Clarifies the boundary so Settings refactors do not re-nest org discovery under the project.

**Independent Test**: Open Settings inside a project; confirm no “all organizations” catalog; org admin identity settings (if any) are clearly project/org *configuration*, not the org/project discovery hierarchy.

**Acceptance Scenarios**:

1. **Given** a project workspace, **When** the operator uses sidebar sections, **Then** they remain in project scope with the selected project and environment context.
2. **Given** Settings inside a project, **When** the operator looks for a multi-organization catalog or “all organizations” index, **Then** that catalog is **not** hosted as a Settings tab; hierarchy screens live outside project Settings.
3. **Given** project Settings, **When** organization-related *configuration* remains (name, billing link, security policy for the org that owns the project), **Then** it is clearly settings/admin content and provides navigation out to organization home or organizations root when the operator needs discovery—not a substitute for those pages.
4. **Given** the project name segment in the breadcrumb, **When** the operator activates it, **Then** it opens a project switcher limited to the **current organization** with an explicit **Project overview** action—not the global org catalog.

---

### User Story 5 - Hierarchy-aware chrome and empty states (Priority: P3)

As an operator at organizations root or organization home, chrome (title bar, breadcrumbs, empty states) reflects the current level. Empty org (no projects) and empty account (no organizations) show guided create actions without dumping the user into a fake project shell.

**Why this priority**: Completes the model for greenfield and empty accounts without blocking the main hierarchy fix.

**Independent Test**: Fixture with zero projects in an org; fixture with zero orgs; verify empty states and create CTAs.

**Acceptance Scenarios**:

1. **Given** an organization with zero projects, **When** organization home opens, **Then** an empty state explains there are no projects and offers create project.
2. **Given** an account with zero organizations, **When** organizations root opens (or app entry requires it), **Then** an empty state offers create organization (and join, if product already supports join).
3. **Given** the operator is on organizations root, **When** breadcrumb/chrome is shown, **Then** it does not pretend a project is selected as the primary workspace context.
4. **Given** the operator is on organization home, **When** breadcrumb/chrome is shown, **Then** the current organization is identified and project is not required until a project is chosen.

---

### Edge Cases

- Operator has access to multiple organizations and many projects: lists remain searchable and scannable; selecting an organization from its switcher lands on **organization home** unless product later adds “resume last project”.
- Current project belongs to Org A; operator opens All organizations and selects Org B: leave project shell; show Org B home; do not keep Org A project sidebar active.
- Deep link / back-forward: history should move between root → org home → project section without trapping the user inside Settings.
- Single-org / single-project accounts: hierarchy pages still exist and remain correct; switcher menus may be short but must not collapse discovery into Settings.
- Permission-denied organization or project: show an accessible error or filtered absence; do not open Settings as a fallback.
- “Manage organizations” wording from older UI maps to organizations **root**, not Settings.
- Onboarding (create first org / first project) remains the entry for brand-new accounts; after first project exists, hierarchy screens above remain available via breadcrumb.

## Requirements *(mandatory)*

### Functional Requirements

#### Hierarchy model

- **FR-001**: Product navigation MUST follow the hierarchy **Organizations root → Organization home → Project workspace (sidebar sections/tabs)**.
- **FR-002**: Organization list/discovery and organization project-list discovery MUST NOT be implemented as destinations under a project’s Settings section.
- **FR-003**: Project workspace sidebar sections (Architecture, Resources, Operations, Observability, Knowledge, Access, Integrations, Settings) MUST only appear as primary navigation when a **project** is the active workspace level.
- **FR-004**: Environment selection remains a **project-scoped** context control (breadcrumb or equivalent) and MUST NOT appear as a substitute for organization or project discovery pages.

#### Title-bar breadcrumb

- **FR-005**: Each visible workspace context MUST expose one ghost switcher trigger; separate dropdown-icon buttons are forbidden.
- **FR-006**: Organization and project triggers MUST display their current names. The environment trigger MUST display only a filled semantic badge for `PRODUCTION`, `STAGING`, `DEVELOPMENT`, or `LOCAL`, while retaining the environment name in its accessible label and tooltip.
- **FR-007**: Activating a breadcrumb context trigger MUST open its switcher without navigating until an item or explicit destination action is chosen.
- **FR-008**: The organization switcher MUST include **Organization home**, **All organizations**, and **New organization**. The project switcher MUST include **Project overview**. The environment switcher contains only environment choices.
- **FR-009**: Choosing **All organizations** from the switcher MUST navigate to the organizations **root** full page.

#### Organizations root

- **FR-010**: Organizations root MUST be a dedicated full-page surface listing organizations the operator can access (name, plan or tier label when available, project count or equivalent summary).
- **FR-011**: Organizations root MUST support search/filter over organizations and a primary action to create a new organization.
- **FR-012**: Selecting an organization on the root MUST open that organization’s home (projects list), not a project Settings tab.
- **FR-013**: Organizations root MUST NOT render the project sidebar as if a project workspace were active.

#### Organization home

- **FR-014**: Organization home MUST be a dedicated full-page surface listing projects that belong to the current organization.
- **FR-015**: Organization home MUST support search/filter over that organization’s projects and a primary action to create a project in that organization.
- **FR-016**: Selecting a project on organization home MUST enter the project workspace with project sidebar available.
- **FR-017**: Organization home MUST NOT host the multi-organization catalog as its only content; multi-org catalog lives at organizations root (org home may link “All organizations” in chrome/breadcrumb).

#### Project workspace & Settings boundary

- **FR-018**: Existing project-level Settings tabs (Project, Environments, Notifications, Core, Developer, etc.) MAY remain for configuration of the **active project** and related admin concerns.
- **FR-019**: Any Settings tab named “Organization” that remains MUST be limited to **organization configuration/admin** for the org that owns the current project (identity, policies, billing entry points)—and MUST NOT be the multi-org index or the org’s project picker.
- **FR-020**: Entry points previously labeled “Manage organizations” / “All organizations” that currently route into Settings MUST be retargeted to organizations root (or organization home when context is a single org action).
- **FR-021**: Project name in the breadcrumb MUST open a project switcher limited to projects of the **current organization**; **Project overview** MUST navigate to the project Overview surface.

#### Consistency & regression

- **FR-022**: Navigation among root, org home, and project MUST be representable in client history (back/forward behave predictably across the three levels).
- **FR-023**: Fixture/demo mode MUST demonstrate at least two organizations and multiple projects so hierarchy is reviewable without live backend.
- **FR-024**: UI copy and structure for these surfaces MUST remain product English; visual language MUST follow the shared operator design system (lists, search, primary create actions—simple index pages, not scenic empty shells).

### Key Entities

- **Organization**: Collaboration and billing/membership boundary above projects. Listed at organizations root; owns many projects.
- **Project (System)**: Domain root being operated; entered from organization home; hosts sidebar product sections. UI may say Project as alias for System.
- **Environment**: Named runtime context under a project (local, staging, production, …); not a hierarchy peer of Organization.
- **Workspace context**: The active triple (organization, project, environment) when inside project work; partial context at higher levels (org-only on org home; none/account on orgs root).
- **Organizations root surface**: Account-level index of organizations.
- **Organization home surface**: Organization-level index of projects.
- **Project workspace surface**: Project-level shell with sidebar sections and tabs.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Strengthens **discover-organize** and navigable operational context by placing multi-system (project) discovery under the correct organization boundary instead of burying it in project Settings.
- **Runtime owner (III)**: Desktop **Renderer** shell/chrome and feature surfaces; no privileged Main/Core behavior required for navigation model itself. Data may remain fixture until contracts exist.
- **Domain vs adapter (II, IV)**: Navigation hierarchy is product/shell domain language (Organization, Project/System, Environment). No new concrete provider coupling.
- **Contracts (V)**: Prefer extending existing client route / workspace context concepts; if persistence of last org/project is added later, version shared contracts. No requirement for new network APIs in this feature’s UX definition.
- **Product loop impact**: Entry and context selection before Model → Observe → …; does not replace System Map or operations loop.
- **Evidence expectations**: n/a for pure navigation chrome (no linked/observed resource claims).
- **Mutation safety (VII)**: Creating org/project from these screens remains plan-or-fixture level until real mutation path exists; list/navigate are read-only. Any real create MUST later follow plan → review → approve → apply → logs → audit when wired to Core.
- **Operation audit fields**: n/a for navigation; create actions when live need actor + scope.
- **Security boundary (VI)**: Unprivileged renderer; lists must respect what the actor can access; no secrets on these pages.
- **Local vs cloud authority (VIII)**: Local-first listing from Core/fixtures is fine; offline still shows last-known or fixture hierarchy without claiming live multi-tenant cloud state if offline.
- **Performance (IX)**: Index pages MUST stay scrollable/searchable; large lists SHOULD virtualize when volumes grow—no Main-thread blocking work for static lists.
- **Resilience (X)**: Missing org/project selection recovers to root or org home rather than a blank project shell.
- **Observability (XI)**: Route/level changes SHOULD be correlatable in client diagnostics if already present; no new telemetry mandate.
- **Test plan (XII)**: Unit/component checks for single-trigger behavior and route targets; shell navigation checks for root → org → project; regression that All organizations does not open Settings.
- **Structure created now (XIII)**: Prefer extending `packages/features/shell` (and thin App wiring); new scenic packages forbidden. Org/project index UI may live in shell or a small feature module only if it has a real consumer.
- **Single source of truth (XIV)**: One hierarchy model; remove dual meaning of “Organization” as both Settings tab catalog and hierarchy root. Temporary dual paths only if needed during migration and must be removed before done.
- **Big-bang / refactor context (XV)**: Corrective product-navigation fix on the active refactor branch; supersedes the incorrect “manage organizations → Settings” routing from prior operator-surface work.
- **Non-goals (I, XIII)**:
  - Not a full IAM/admin console clone.
  - Not billing checkout or domain verification deep product (may link later).
  - Not replacing project Settings configuration surfaces wholesale.
  - Not implementing live multi-tenant backend if fixtures already power shell demos—honest fixture labeling remains required.
  - Not changing the eight project sidebar destinations’ product meaning.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From any project surface, operators reach the current organization’s project list through the organization switcher's **Organization home** action with no Settings detour.
- **SC-002**: From any project surface, operators reach the organizations root in **at most two activations** (organization trigger → All organizations), and that path **never** lands on Settings.
- **SC-003**: 100% of “All organizations” / “Manage organizations” entry points in shell chrome open organizations root (or are removed if duplicate)—**zero** remaining chrome entries that open Settings for that purpose.
- **SC-004**: In a review fixture with ≥2 organizations and ≥2 projects in the active org, a new operator can open org home, identify the correct project list, and enter a project without using Settings—in under **30 seconds**.
- **SC-005**: At organizations root and organization home, project sidebar primary nav is **not** shown; at project workspace, project sidebar **is** shown—verifiable by inspection of the three levels.
- **SC-006**: Organization/project names and the environment badge each open exactly one switcher, with no separate dropdown-icon targets.
- **SC-007**: Regression: Settings → Organization (if retained) does not become the only way to list organizations or projects; hierarchy pages exist independently.

## Assumptions

- Prior specs (especially `007-remaining-operator-surfaces`) that routed “manage organizations” into Settings are **partially superseded** by this feature for **discovery/hierarchy** navigation; Settings may keep org **admin configuration** only.
- Visual reference for organizations root: simple full-page index (search + org cards/rows + New organization), consistent with the “Your Organizations” style empty/list page already intended in product demos.
- Visual reference for organization home: equally simple full-page index of **projects** for one organization (search + list/cards + New project), not a dense settings form and not a project-section workspace.
- Domain language: **System** remains canonical for the operated unit; UI continues to say **Project** where the product already does.
- Default after selecting an organization from the switcher list: navigate to **organization home** (not auto-enter a project), unless a later clarify chooses “resume last project.”
- Organization and project names are single menu triggers. Environment is also a single trigger, represented visually only by its semantic kind badge.
- Fixture data is sufficient for desktop demo until Core org/project APIs are authoritative.
- Onboarding still creates the first organization/project; this feature ensures post-onboarding navigation respects the hierarchy.
- No change to Architecture canvas semantics, resource model, or mutation pipeline beyond navigation targets.

## Out of Scope

- Full organization billing, SSO, domain capture, and member administration product depth (may remain stubs or Settings admin later).
- Redesign of all eight project sidebar destinations.
- Mobile-responsive web shell (Desktop operator UI is the target).
- Real multi-tenant backend enforcement beyond existing Core capabilities (honest fixtures allowed).
- Replacing Commander/search as a parallel jump tool (may deep-link to these surfaces later).
