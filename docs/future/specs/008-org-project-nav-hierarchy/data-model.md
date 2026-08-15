# Data Model: Organization → Project Navigation Hierarchy

**Feature**: `008-org-project-nav-hierarchy`  
**Date**: 2026-08-03  
**Layer**: Presentation / client route + fixture view models (not a new Core schema)

---

## Hierarchy levels

```text
Account (implicit)
  └── Organization (1..n)
        └── Project / System (0..n per org)
              └── Environment (1..n per project)
```

| Level | UI surface | Primary nav chrome |
| --- | --- | --- |
| Account | Organizations root (`organizations`) | No project sidebar |
| Organization | Organization home (`organization`) | No project sidebar |
| Project | Any `NavSectionId` + project chrome sections (`overview`, …) | Project sidebar |

---

## Entities (client view models)

### OrganizationItem

Already in shell `types.ts` — extend only if needed for list cards:

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Stable org id |
| `name` | string | Display name |
| `planLabel` | string? | e.g. Free, Local |
| `projectCount` | number? | Derived from projects for root cards (computed OK) |

**Validation / rules**:

- Root list shows only organizations the actor can access (fixture = all fixtures).
- Empty list → empty state + create organization.

### ProjectSwitcherItem (project list row)

Existing fields:

| Field | Type | Notes |
| --- | --- | --- |
| `projectId` | string | |
| `name` | string | |
| `organizationId` | string | FK to org |
| `organizationName` | string | Denormalized label |
| `environmentSummary` | string | Short env hint |
| `isCurrent` | boolean | Relative to active project |

**Rules**:

- Organization home filters `organizationId === activeOrgId`.
- Project switcher in project workspace filters the same way.
- Entering a project sets workspace project + default environment for that project.

### EnvironmentItem

Unchanged. Scoped under project. Not shown as hierarchy peer of Organization.

### WorkspaceContext

Existing:

```ts
type WorkspaceContext = {
  organizationId: string;
  projectId: string;
  environmentId: string;
};
```

**Hierarchy-aware usage**:

| Surface | organizationId | projectId | environmentId |
| --- | --- | --- | --- |
| Organizations root | last-known or empty sentinel | not required | not required |
| Organization home | **required** (viewed org) | not required | not required |
| Project workspace | required | **required** | required (defaulted) |

Implementation may keep string fields always populated for simplicity but **must not** use a phantom project to justify showing project sidebar on root/home.

---

## ClientRoute extensions

Existing `ClientRoute` fields remain. Add:

| Field | Type | When set |
| --- | --- | --- |
| `organizationId` | string? | Hierarchy + project routes; required for `section === "organization"` |
| `projectId` | string? | Project workspace routes; optional on hierarchy chrome |

`historySnapshot` MUST include these fields so back/forward restores org home correctly.

### Section / tab defaults

| Section | Default tab | page |
| --- | --- | --- |
| `organizations` | `index` | `index` |
| `organization` | `index` | `index` |

No entity detail pages required for v1 hierarchy indexes.

---

## State transitions

```text
[Project workspace]
    | click org name
    v
[Organization home] --click project--> [Project workspace]
    | All organizations / breadcrumb root
    v
[Organizations root] --click org--> [Organization home]

[Project workspace]
    | chevron org → All organizations
    v
[Organizations root]

[Project workspace]
    | chevron org → select other org
    v
[Organization home of selected org]

[Any]
    | Settings → Organization (admin)
    v
[Project workspace settings tab]  // not hierarchy catalog
```

### Invalid states

| State | Recovery |
| --- | --- |
| `organization` without `organizationId` | Fall back to `organizations` root |
| Project id not in org | Clear to org home |
| Empty projects for org | Org home empty state |
| Empty orgs | Root empty state |

---

## Relationships

```text
Organization 1 ──* Project
Project 1 ──* Environment
ClientRoute.section ── interprets chrome vs nav
ClientRoute.organizationId ──> Organization.id
ClientRoute.projectId ──> Project.projectId
ClientRoute.environmentId ──> Environment.id
```

---

## Persistence

| Concern | v1 |
| --- | --- |
| List data | Fixtures (honest banner) |
| Last selected org/project | In-memory + history stack; optional later local preference |
| Mutations create org/project | Stub → onboarding or future Core; not required for hierarchy navigation acceptance |

---

## Settings Organization (admin) — non-hierarchy entity

Settings tab may edit **configuration** of the org that owns the current project (name, labels). That is **not** the Organization home list and **not** the Organizations root catalog. No separate entity required beyond existing settings form state.
