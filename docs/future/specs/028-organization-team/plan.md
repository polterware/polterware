# Implementation Plan: Organization Team

**Branch**: `028-organization-team` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/028-organization-team/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: shell hierarchy navigation (org sidebar Team), global DetailPanel LAW, design-system DataTable/StatusChip, shell workspace-mocks patterns (Inbox/ApprovalDetail as composition reference)

**Risk control**: Mock-only invite and membership actions. **Parallel agents own** project Access details, architecture, operations, integrations, project settings, terminal — this plan MUST NOT touch those files.

---

## Summary

Deliver a **production-quality Organization Team page** (mocked): org-scoped member collection with professional DataTable, reusable `ORG_TEAM_MOCKS`, global DetailPanel for member overview, honest invite/mutation feedback, and clear org ≠ project Access copy.

**Technical approach**:

1. **Add `WorkspaceOrgMember` + `ORG_TEAM_MOCKS`** in shell workspace-mocks (typed, exported).
2. **Implement `OrgMemberDetail`** (`layout="sections"`, StatusChip MFA, mock actions).
3. **Rewrite `OrganizationTeamPage`**: register collection, activate rows, DataTable columns, empty state, invite honesty.
4. **Wire Team surface**: render `OrganizationTeamPage` from org-settings people tab (current org sidebar Team route).
5. **Tests**: helpers + page/detail composition contracts.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9

**Primary Dependencies**:
- `@polterware/feature-shell` (owner)
- `@polterware/frontend-design-system` (DataTable, DetailPanel, StatusChip, Button, pushToast)
- `@polterware/frontend-state` (detail panel store)

**Storage**: Fixture-only workspace mocks. No SQLite / Connect membership API.

**Testing**: Vitest node in `packages/features/shell` — helpers + composition source contracts for page and detail.

**Target Platform**: Desktop renderer Organization → Team

**Constraints**:
- DESIGN / AGENTS LAW for DataTable + DetailPanel
- English UI strings
- Do not edit project Access member/team detail modules
- Never claim live invite delivery

**Scale/Scope**:
- One entity kind: **org-member**
- ≥5 fixtures with mixed roles and MFA postures
- Org-scoped only

---

## Constitution Check

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Org membership governance surface |
| **II. Domain Before Framework** | PASS | WorkspaceOrgMember domain model + pure helpers |
| **III. Runtime Separation** | PASS | Renderer shell feature; mock only |
| **IV. Monorepo deps** | PASS | shell → design-system, state |
| **V. Contracts-First** | PASS | `contracts/organization-team-ui.md` |
| **VI. Security** | PASS | Mock remove/role is not real revocation |
| **VII. Safe execution** | PASS | No destructive remote path |
| **VIII. Local-first** | PASS | Fixtures offline |
| **XII. Test plan** | PASS | helpers + composition tests |
| **XIV. SSoT** | PASS | Single page + detail composition |

---

## Project Structure

### Documentation (this feature)

```text
specs/028-organization-team/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/organization-team-ui.md
├── checklists/requirements.md
└── tasks.md
```

### Source (allowed touch)

```text
packages/features/shell/src/
├── organization-team.tsx          # page rewrite
├── organization-team.test.ts      # page composition tests
├── org-member-detail.tsx          # new
├── org-member-detail-helpers.ts   # new
├── org-member-detail.test.ts      # new
├── organization-settings.tsx      # people tab → OrganizationTeamPage only
├── index.tsx                      # exports
└── workspace-mocks/
    ├── types.ts                   # WorkspaceOrgMember
    ├── data.ts                    # ORG_TEAM_MOCKS
    └── index.ts                   # export
```

### Forbidden

- `packages/features/access/**` member/team detail
- architecture, operations, integrations, project settings, terminal
