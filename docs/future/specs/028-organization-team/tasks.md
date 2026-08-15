# Tasks: Organization Team

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md, spec.md

**Tests**: Risk-proportional — pure helpers for MFA/status tones; composition source contracts for page + detail; shell vitest. Agents.md: no `pnpm dev` unless asked.

**Constitution / guardrails**:

- DetailPanel LAW + DataTable LAW
- Org members ≠ project Access
- Mock invite honesty
- FORBIDDEN: feature-access member/team, architecture, operations, integrations, project settings, terminal

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Create feature directory `specs/028-organization-team/` with plan artifacts
- [x] T002 Point `.specify/feature.json` to `specs/028-organization-team`

## Phase 2: Foundational

- [x] T003 Add `WorkspaceOrgMember` type in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T004 Add `ORG_TEAM_MOCKS` (≥5 members, both MFA, multi-role, org-scoped) in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T005 Export type + mocks from `packages/features/shell/src/workspace-mocks/index.ts` and `packages/features/shell/src/index.tsx`
- [x] T006 [P] Implement `org-member-detail-helpers.ts` (mfa/status/role labels+tones, displayOrDash)
- [x] T007 [P] Helper unit tests in `org-member-detail.test.ts`

## Phase 3: US1 Browse members (P1)

- [x] T008 [US1] Rewrite `organization-team.tsx` DataTable with PrimaryCell, role, MFA StatusChip, optional status/lastActive, org≠Access copy
- [x] T009 [US1] Page composition tests in `organization-team.test.ts` (StatusChip MFA both states, ORG_TEAM_MOCKS usage, no local-only 3-array)

## Phase 4: US2 DetailPanel (P1)

- [x] T010 [US2] Implement `org-member-detail.tsx` sections layout + overview + mock actions
- [x] T011 [US2] Wire registerCollection + activate + render OrgMemberDetail in `organization-team.tsx`
- [x] T012 [US2] Detail composition contract tests in `org-member-detail.test.ts`

## Phase 5: US3 Invite honesty (P2)

- [x] T013 [US3] Invite members mock toast/dialog with non-delivery copy; detail Invite action same honesty

## Phase 6: US4 Empty/loading + wire live path

- [x] T014 [US4] Empty table + loading/not-found detail states
- [x] T015 Wire `OrganizationSettingsPage` people tab to `OrganizationTeamPage` (remove duplicate local MEMBERS table)
- [x] T016 Export `OrgMemberDetail` helpers from shell `index.tsx` as needed

## Phase 7: Polish

- [x] T017 Run shell package tests; confirm rubric green
- [x] T018 Confirm git diff does not touch project Access detail modules

## Parallel opportunities

- T006/T007 parallel with T003–T005 after types land
- T010 can start after helpers; T008 can start after fixtures
