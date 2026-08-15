# Quickstart: Access Agents Detail

**Feature**: `017-agents-detail`  
**Goal**: Validate the mock Agents DetailPanel end-to-end without claiming live IdentityService or agent runtime control.

Related: [spec.md](./spec.md) · [contracts/agents-detail-ui.md](./contracts/agents-detail-ui.md) · [data-model.md](./data-model.md)

---

## Prerequisites

- Repo on branch with Access Agents table + global DetailPanel host working (operator product surfaces baseline).
- Workspace mocks include enriched `AGENT_MOCKS` (structured `capabilities`, `tools`, `environments`, `recentOperationIds`, optional `description` / `activity` / `notes`).
- Desktop can open a project workspace (fixture org/project).
- Sibling session/policy/audit details may be incomplete in parallel — agent detail only requires session activation target wiring already present in Access.

---

## Setup

```bash
# From monorepo root — only when the operator wants a live UI check
# (Agents.md: do not start dev servers unless asked)
pnpm --filter @polterware/desktop dev
```

Navigate: **Project → Access → Agents**.

---

## Automated validation (preferred for agents)

```bash
# Access package contract + helpers
cd packages/features/access
pnpm test

# Desktop fixture completeness for Deployment Agent
cd apps/desktop
./node_modules/.bin/vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

**Expected**:
- access: `agent-detail` tests pass (helpers + composition + Access wiring)
- fixture coverage: `AGENT_MOCKS[0]` (Deployment) has structured capabilities/tools, environments, recentOperationIds; related session exists for `agentId === "a1"`

---

## Manual scenarios

### S1 — Inspect Deployment Agent (P1)

1. Open Access → Agents.
2. Activate **Deployment Agent**.
3. Confirm DetailPanel header shows name + Bot icon.
4. Confirm sections: Metrics (Status Active, Autonomy, Scope, Environments), Overview, Capabilities, Tools, Environments, Linked operations, Recent sessions, optional Activity/Notes.
5. Confirm no second page/modal inspector.
6. Confirm status uses StatusChip (filled, no hand-rolled pill).

**Pass**: All sections visible; structured capability/tool rows; no crash.

### S2 — Mock governance actions (P1)

1. With Deployment Agent detail open, click **Pause** → toast mentions mock pause.
2. (If testing Paused agent Linear Bridge) click **Resume** → mock resume toast.
3. Click **Edit permissions** → toast “Permissions editor (mock)” (or equivalent).
4. Click **Revoke** → toast “Agent revoked (mock)” (or equivalent).

**Pass**: Toasts honest; panel stays on agent; no fake persistence claim beyond toast.

### S3 — Session and operation hand-off (P2)

1. On Deployment Agent, use **Open recent session** or activate a Recent sessions row.
2. Confirm Access activates the session target (`agent-session`) — session detail may be sibling-owned UI.
3. Activate linked operation **run-1** when Operations hand-off is wired.

**Pass**: Callbacks fire; agent detail does not embed full session steps UI as a second design.

### S4 — Empty / thin fixtures honesty (P2)

1. Activate an agent with empty `recentOperationIds` (Architecture Agent) — Linked operations empty copy.
2. Force activation of unknown agent id → **Agent not found**.

**Pass**: Honest empty/not-found; no invented rows.

### S5 — Entity kind isolation

1. On Agents tab, activate a **Recent sessions** row (session entity).
2. Confirm `AgentDetail` is not mounted for `agent-session`.
3. Return to an agent row → agent detail returns.

**Pass**: No dual content clash with session detail.

---

## Definition of done (plan gate)

| Check | Evidence |
| --- | --- |
| FR-001–FR-018 covered by UI contract + composition | [agents-detail-ui.md](./contracts/agents-detail-ui.md) |
| Deployment fixture complete | detail-fixture-coverage |
| Helpers + composition contract | `packages/features/access` agent-detail tests |
| SSOT single AgentDetail | `index.tsx` imports `./agent-detail` only for agent ready path |
| No frontend authz / live runtime claim | contract Security honesty + mock toasts |
| Session hand-off only | S3 + `onOpenSession` |

---

## Out of scope for this quickstart

- Live IdentityService / agent control APIs
- Real pause/resume/revoke persistence
- Full agent session detail product QA (sibling)
- Policy / audit detail QA (siblings)
- ConnectRPC agent APIs
