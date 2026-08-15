# Quickstart: Access Teams Detail

**Feature**: `015-teams-detail`  
**Goal**: Validate the mock Teams DetailPanel end-to-end without claiming live membership or policy enforcement.

Related: [spec.md](./spec.md) · [contracts/teams-detail-ui.md](./contracts/teams-detail-ui.md) · [data-model.md](./data-model.md)

---

## Prerequisites

- Repo on branch with Access People Teams table + global DetailPanel host working (operator product surfaces baseline).
- Workspace mocks include enriched `TEAM_MOCKS` (`description`, `status`, `leadName`, `environments`, `escalation`, `memberRoster`, `ownedResources`, `policies`, `activity`, `createdAt`).
- Desktop can open a project workspace (fixture org/project).
- People detail (`012-people-detail`) available for bidirectional nav checks.

---

## Setup

```bash
# From monorepo root — only when the operator wants a live UI check
# (Agents.md: do not start dev servers unless asked)
pnpm --filter @polterware/desktop dev
```

Navigate: **Project → Access → People → Teams table**.

---

## Automated validation (preferred for agents)

```bash
# Access package contract + helpers
cd packages/features/access
pnpm test

# Desktop fixture completeness for Platform Team
cd apps/desktop
./node_modules/.bin/vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

**Expected**:
- access: `detail-targets` + `member-detail` + `team-detail` tests pass
- fixture coverage: `TEAM_MOCKS[0]` (Platform) has non-empty `memberRoster`, `ownedResources`, `policies`, `activity`; `members === memberRoster.length`; `owned === ownedResources.length`; roster includes person id `m1`

---

## Manual scenarios

### S1 — Inspect Platform Team (P1)

1. Open Access → People.
2. In the Teams table, activate **Platform Team**.
3. Confirm DetailPanel header shows name + AddTeam icon.
4. Confirm sections: Metrics (Status Active, members, owned, on-call), Overview (description, lead, escalation, schedule), Environments, Members, Owned resources, Policies, Activity, optional Notes.
5. Confirm no second page/modal inspector.

**Pass**: All sections visible; StatusChips for status/presence/policy/result; no crash.

### S2 — Mock governance actions (P1)

1. With Platform Team detail open, click **Edit team** → toast “Team editor opened (mock)”.
2. Click **Add member** → toast “Member picker opened (mock)”.
3. Click **Archive team** → toast “Team archived (mock)”.

**Pass**: Toasts honest; panel stays on team; no fake persistence claim beyond toast.

### S3 — Bidirectional navigation (P2)

1. On Platform Team members table, activate **Erick Barcelos** (`m1`).
2. Confirm People **MemberDetail** opens for Erick.
3. On member Teams table, activate **Platform Team**.
4. Confirm team detail returns.

**Pass**: Same DetailPanel host; activation switches entityKind without dual content clash.

### S4 — Empty / thin fixtures honesty (P2)

1. Activate **Identity Team** — Policies empty state: “No policies linked to this team.”
2. Force activation of unknown team id (dev hook or temporary target) → **Team not found**.

**Pass**: Honest empty/not-found; no invented rows.

### S5 — Entity kind isolation

1. On People tab, open a **Member** or **Invitation** detail.
2. Confirm `TeamDetail` is not mounted for those kinds.
3. Return to a team row → team detail returns.

**Pass**: No dual content clash.

---

## Definition of done (plan gate)

| Check | Evidence |
| --- | --- |
| FR-001–FR-016 covered by UI contract + composition | [teams-detail-ui.md](./contracts/teams-detail-ui.md) |
| Platform fixture complete | detail-fixture-coverage |
| Helpers + composition contract | `packages/features/access` team-detail tests |
| SSOT single TeamDetail | `index.tsx` imports `./team-detail` only |
| No frontend authz claim | contract Security honesty + mock toasts |
| Bidirectional nav with People | S3 + member `onOpenTeam` / team `onOpenMember` |

---

## Out of scope for this quickstart

- Live org directory / team membership CRUD APIs
- Real archive/edit persistence
- Roles matrix editor
- Agents detail
- Live inventory ownership sync
- ConnectRPC team APIs
