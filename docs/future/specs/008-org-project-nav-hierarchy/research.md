# Research: Organization → Project Navigation Hierarchy

**Feature**: `008-org-project-nav-hierarchy`  
**Date**: 2026-08-03  
**Status**: Complete — unknowns resolved against repo evidence + `spec.md`

---

## R1 — Where do hierarchy pages live in the section model?

**Decision**: Add two **chrome** sections (not sidebar nav items):

| Section id | Role |
| --- | --- |
| `organizations` | Account-level organizations root index |
| `organization` | Organization home — projects of current/selected org |

Extend:

```ts
type ChromeSectionId =
  | "overview"
  | "search"
  | "sync"
  | "onboarding"
  | "organizations"
  | "organization";
```

**Rationale**: Existing IA already separates `NavSectionId` (eight project destinations) from `ChromeSectionId` (overview, search, sync, onboarding). Hierarchy surfaces are above project work and must not inflate the sidebar. Mirrors Overview/onboarding pattern.

**Alternatives considered**:

| Alternative | Why rejected |
| --- | --- |
| Settings tabs for catalog | Spec forbids; current bug |
| Single section `orgs` with only tabs | Harder to express “no project shell” + history of org entity |
| New top-level nav items | Violates eight-destination sidebar IA from 007 |
| Nested entity routes only under `settings` | Same inversion as today |

---

## R2 — How is the active organization identified on org home?

**Decision**: Persist `organizationId` on `ClientRoute` (new optional field). On `section === "organization"`, `organizationId` is required for a valid org-home route. Workspace context in Shell stays aligned via the same id.

Also add optional `projectId` on `ClientRoute` for durable project selection in history (today project id lives mostly in Shell local workspace + fixtures).

**Rationale**: History back/forward (already in App) cannot restore org home correctly without the org id in the history snapshot. Environment already exists on `ClientRoute`; org/project are the missing hierarchy keys.

**Alternatives considered**:

| Alternative | Why rejected |
| --- | --- |
| `entityId` + `entityKind: "organization"` | Overloaded with resource detail patterns; chrome pages are not entity details of project sections |
| Shell-only local state | Lost on history navigation / harder to test from App |
| Encode org in `tab` | Brittle; tabs mean in-section subdivisions |

---

## R3 — Breadcrumb: one control vs two

**Decision**: Organization (and project) segments use **two adjacent controls**:

1. **Label button** — navigates (org → org home; project → overview).
2. **Chevron button** — toggles switcher popover only (`aria-expanded`, no navigation).

**Rationale**: Matches product requirement and common cloud console patterns (label = “you are here / go up”, chevron = switch). Current `context-switchers.tsx` merges both into one button — that is the primary UX bug.

**Alternatives considered**:

| Alternative | Why rejected |
| --- | --- |
| Label opens menu, separate “home” icon | Spec wants name → home |
| Entire segment navigates; long-press for menu | Poor discoverability; not desktop primary |
| Dropdown only (status quo) | Blocks org home from breadcrumb |

---

## R4 — Selecting an organization from the switcher menu

**Decision**: Selecting an org in the chevron menu navigates to **`organization` home** for that org (clear project-required shell). Does **not** auto-enter the first project.

Entering a project happens only from org home (or project switcher while already in a project workspace, which switches project and stays in project shell).

**Rationale**: Spec assumption + edge case default; avoids surprise project entry and keeps hierarchy legible.

**Alternatives considered**:

| Alternative | Why rejected |
| --- | --- |
| Resume last project for that org | Nice later; needs persistence design; not in spec defaults |
| Auto first project (current Shell handler) | Collapses hierarchy; hides org home |

---

## R5 — Hide project sidebar at hierarchy levels

**Decision**: When `section` is `organizations` | `organization` (and keep existing chrome behavior for `onboarding` as already non-nav-active), Shell **does not render** the project `SidebarNav` column (or renders a minimal hierarchy chrome without the eight destinations). Title bar remains for breadcrumb ascent/descent.

**Rationale**: FR-003 / FR-013 / SC-005 — showing Architecture/Resources/… implies a project workspace. Overview/search/sync currently still show sidebar; this feature **tightens** hierarchy chrome specifically. Optionally leave overview with sidebar as today (project-scoped overview); hierarchy root/home must not.

**Alternatives considered**:

| Alternative | Why rejected |
| --- | --- |
| Dim sidebar but keep it | Still implies project context |
| Replace sidebar with org links | Extra scope; full-page indexes suffice |
| Full-screen dialogs for indexes | Spec wants full pages, not modals |

**Note**: `overview` / `search` / `sync` may keep current sidebar behavior in this feature unless a follow-up unifies all chrome; **in-scope hide** is `organizations` + `organization`.

---

## R6 — Settings → Organization tab fate

**Decision**:

- **Keep** tab for **admin configuration** of the organization that owns the current project (name, slug, plan display, danger zone stubs, etc.).
- **Remove** multi-org catalog / “all organizations list” as the purpose of this tab.
- **Optional** secondary link: “All organizations” → hierarchy root (for operators who opened Settings first).
- Chrome “All organizations” **never** routes here after this feature.

**Rationale**: Spec FR-018–FR-020; avoids deleting admin surface while fixing hierarchy.

---

## R7 — Package ownership

**Decision**: Implement pages and breadcrumb in **`packages/features/shell`**. Wire matches in **`apps/desktop/src/renderer/App.tsx`**. No `packages/features/organizations`.

**Rationale**: Constitution XIII + existing OverviewPage/onboarding in shell; App already owns section Match tree.

---

## R8 — Fixtures & honesty

**Decision**: Reuse/extend `ORGANIZATION_FIXTURES` and `PROJECT_SWITCHER_FIXTURES` (≥2 orgs, ≥2 projects in primary org already present). Derive project counts for org cards from project fixtures. Show `FixtureBanner` on hierarchy pages when data is fixture.

**Rationale**: Matches product honesty rules; enough data for SC-004 review without Core RPC.

---

## Resolved Technical Context

| Item | Resolution |
| --- | --- |
| Language | TypeScript + Solid |
| Storage | Fixtures / future Core — no new store |
| Testing | Vitest pure helpers + manual quickstart |
| Platform | Desktop renderer |
| Scope | Shell IA + two chrome pages + App wiring + Settings boundary |

No open NEEDS CLARIFICATION markers for planning.
