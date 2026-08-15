# Quickstart Validation: Organization → Project Navigation Hierarchy

**Feature**: `008-org-project-nav-hierarchy`  
**Date**: 2026-08-03  
**Purpose**: Prove hierarchy navigation end-to-end after implement (manual + pure tests). Full `pnpm dev` only when Erick asks.

---

## Prerequisites

- Spec + plan + contracts in `specs/008-org-project-nav-hierarchy/`
- Desktop renderer composes Shell + hierarchy pages
- Fixtures: ≥2 organizations, ≥2 projects under primary org (see `packages/features/shell/src/fixtures.ts`)

---

## Automated (prefer first)

From repo root (adjust package filter if scripts differ):

```bash
# Pure route / hierarchy helpers
pnpm exec vitest run packages/features/shell --reporter=dot
```

**Expect**:

- `goOrganizationsRoot` / `goOrganizationHome` / `enterProject` helpers set section + ids correctly
- `shellLayoutMode` returns `hierarchy` for `organizations`, `organization` for org workspace, `neutral` for account/create
- `historySnapshot` includes `organizationId` / `projectId`
- No test asserts that “All organizations” opens Settings

If shell package has no vitest config yet, add focused tests next to `route.ts` in implement phase.

Optional typecheck (when Erick wants broader verify):

```bash
pnpm exec tsc -p packages/features/shell --noEmit
```

---

## Manual scenarios (Desktop)

Use fixture mode. Start in a **project** surface (e.g. Architecture).

### S1 — Org name → org home

1. Note current organization name in title-bar breadcrumb.
2. Click the organization **name** (not the chevron).
3. **Expect**: Full-page organization home listing **only that org’s** projects; project sidebar **not** primary.
4. **Expect**: Not Settings; not a popover-only switcher.

### S2 — Org chevron → menu only

1. Return to a project surface (enter a project from org home if needed).
2. Click only the organization **chevron**.
3. **Expect**: Switcher menu opens; page stays on project surface.
4. Dismiss menu.
5. **Expect**: Still on same project surface.

### S3 — All organizations → root

1. Open org chevron → **All organizations**.
2. **Expect**: “Your Organizations” (or equivalent) full-page index; ≥2 orgs in fixtures; **not** Settings → Organization.
3. Search filters orgs.
4. Click an organization.
5. **Expect**: That org’s home (projects list).

### S4 — Enter project

1. From organization home, click a project.
2. **Expect**: Project workspace with sidebar (Architecture, Resources, …).
3. Breadcrumb shows org / project / environment.

### S5 — Select other org from menu

1. From project workspace, open org chevron, select a **different** organization.
2. **Expect**: Organization home of the selected org (not auto-entered foreign project with old sidebar).

### S6 — Settings boundary

1. From project workspace, open Settings → Organization (if tab exists).
2. **Expect**: Admin/config form for owning org — **not** the multi-org catalog.
3. If a link “All organizations” exists, it goes to hierarchy root.

### S7 — Empty states (optional fixture toggle)

1. With zero projects for an org: org home shows empty + create project.
2. With zero orgs: root shows empty + create organization.

### S8 — History

1. Project → org name → All organizations → org → project.
2. Use back/forward.
3. **Expect**: Levels restore with correct org id and layout mode (sidebar hide/show).

---

## Acceptance mapping

| Spec criterion | Scenario |
| --- | --- |
| SC-001 one-click org home | S1 |
| SC-002 root in ≤2 activations, never Settings | S3 |
| SC-003 chrome retarget | S3 + code grep `onManageOrganizations` |
| SC-004 multi-org multi-project path | S3–S4 |
| SC-005 sidebar presence | S1 vs S4 |
| SC-006 name vs chevron differ | S1 vs S2 |
| SC-007 Settings not sole catalog | S6 |

---

## Grep regression (static)

```bash
rg -n "onManageOrganizations|Manage organizations|All organizations" apps/desktop packages/features/shell
```

**Expect**: Handlers navigate to `organizations` (or helper), not `settings` + tab `organization` for discovery.

---

## Out of scope for this quickstart

- Live Core multi-tenant org APIs
- Full create-org billing flows
- `pnpm build` / packaging
