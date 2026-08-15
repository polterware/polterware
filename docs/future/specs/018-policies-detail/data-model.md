# Data Model: Access Policies Detail

**Feature**: `018-policies-detail`  
**Stage**: Mock fixtures (shell workspace mocks)  
**Live persistence**: out of scope

---

## Overview

```text
Policy
  ├── Status (Active | Disabled)
  ├── Scope / Enforcement / RequiresApproval
  ├── RiskPosture / ApprovalRequirement
  ├── Environments[]
  ├── Actors (string[] list + structured actorEntries[])
  ├── AllowedActions[] / BlockedActions[]
  ├── Activity[] (optional trail)
  ├── Update history (updated, updatedBy)
  └── RelatedAudit[] (derived from Audit fixtures by policyId/policyName)
```

Domain roots (`docs/spec.md`): **Access / governance** — policies, approvals, audit. UI entity is the project **Policy** under Access → Policies.

---

## Entities

### Policy — `WorkspacePolicy`

Governance rule shown in Access Policies list and detail.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable fixture id (`p1`…) |
| `name` | string | yes | Header title; table PrimaryCell title |
| `description` | string | yes | Description section |
| `scope` | string | yes | List + metric |
| `environments` | string[] | yes | Environment scope labels |
| `actors` | string[] | yes | List-friendly actor labels |
| `allowedActions` | string[] | yes | Capability identifiers allowed |
| `blockedActions` | string[] | yes | Capability identifiers blocked (may be empty) |
| `requiresApproval` | boolean | yes | Approval metric |
| `enforcement` | string | yes | e.g. Required / Optional |
| `status` | `"Active" \| "Disabled"` | yes | StatusChip |
| `updated` | string | yes | Humanized relative update |
| `updatedBy` | string | yes | Last editor display name |
| `actorEntries` | PolicyActor[] | no | Structured actors for detail; preferred when present |
| `riskPosture` | `"low" \| "medium" \| "high" \| "critical"` | no | Risk StatusChip |
| `approvalRequirement` | string | no | Human-readable approval path |
| `dualApproval` | boolean | no | Dual-control flag |
| `activity` | PolicyActivityEvent[] | no | Optional activity trail |
| `notes` | string | no | Optional notes section |

**Validation (mock stage)**:
- `id`, `name` non-empty for list rows
- Primary Production operations approval fixture (`p1`) MUST have non-empty `allowedActions`, `blockedActions`, `actorEntries` (or `actors`), `environments`, and enough related audit linkage via audit fixtures
- At least one fixture SHOULD demonstrate empty `blockedActions` (empty-state demo)
- At least one fixture SHOULD use `status: "Disabled"` for Enable action demo

**State (policy status)**:

| status | UI label | StatusChip tone |
| --- | --- | --- |
| `Active` | Active | healthy |
| `Disabled` | Disabled | muted |

**Risk posture**:

| risk | StatusChip tone |
| --- | --- |
| `low` | healthy / muted (display: Low) |
| `medium` | warn |
| `high` | danger |
| `critical` | danger |

---

### PolicyActor

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable row id |
| `kind` | `"person" \| "agent" \| "role"` | yes | Icon + column |
| `name` | string | yes | PrimaryCell title |
| `label` | string | no | Secondary role/title label |

---

### PolicyActivityEvent

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Row id |
| `title` | string | yes | Event title |
| `at` | string | yes | Humanized when |
| `actor` | string | yes | Who performed the change |
| `result` | `"Success" \| "Pending" \| "Failed" \| "Denied"` | no | Result StatusChip |

---

### Related Audit Event (derived, not owned)

Uses existing `WorkspaceAuditEvent` fields relevant to listing:

| Field | Use |
| --- | --- |
| `id` | Row id + open callback |
| `actor` / `actorKind` | Display |
| `action` | Mono action |
| `entity` | Target entity |
| `result` | StatusChip |
| `at` | When |
| `policyId` / `policyName` | Filter key |

**Filter rule**: include event when `event.policyId === policy.id` OR `event.policyName === policy.name`.

---

## Relationships

```text
POLICY_MOCKS[p*] ──filter──► AUDIT_EVENT_MOCKS (read-only)
Policy.actorEntries may reference person/agent ids for display only
```

No write path from policy detail into audit fixtures in this slice.
