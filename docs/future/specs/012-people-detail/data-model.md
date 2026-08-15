# Data Model: Access People Detail

**Feature**: `012-people-detail`  
**Stage**: Mock fixtures (shell workspace mocks)  
**Live persistence**: out of scope

---

## Overview

```text
Person (Member)
  ├── Membership Role (label)
  ├── Environments[] (scope labels)
  ├── TeamMembership → Team[]
  ├── CapabilityEffect[] (permissions picture)
  ├── MemberSession[]
  └── MemberActivityEvent[]
```

Domain roots (docs/spec.md): **identity** + **organizations/memberships**. UI entity is the project **Person / Member** under Access → People.

---

## Entities

### Person (Member) — `WorkspacePerson`

Human project collaborator shown in Access → People list and detail.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable fixture id (`m1`…) |
| `name` | string | yes | Header title; table PrimaryCell title |
| `email` | string | yes | Overview |
| `avatar` | string | no | Unused in current detail chrome |
| `role` | string | yes | Admin / Operator / Viewer (fixture set) |
| `teamIds` | string[] | yes | Links to `WorkspaceTeam.id` |
| `teams` | string | yes | Compact list label for table column |
| `environments` | string[] | yes | Environment scope labels |
| `status` | `"online" \| "offline" \| "invited"` | yes | Presence → StatusChip |
| `lastActivity` | string | yes | Humanized summary (list + overview) |
| `lastSeen` | string | yes | Humanized |
| `joinedAt` | string | yes | Humanized date for overview |
| `invitedBy` | string | no | Overview when known |
| `authMethod` | string | yes | e.g. `SSO (Google)`, `Password` |
| `mfaEnabled` | boolean | yes | Overview StatusChip Enabled/Disabled |
| `title` | string | no | Job/title line |
| `notes` | string | no | Optional notes section |
| `permissions` | CapabilityEffect[] | yes | May be empty array → empty table state |
| `sessions` | MemberSession[] | yes | May be empty |
| `activity` | MemberActivityEvent[] | yes | May be empty |

**Validation (mock stage)**:
- `id`, `name`, `email`, `role` non-empty for list rows
- Admin primary fixture MUST have non-empty `permissions`, `sessions`, `activity` (fixture-coverage gate)
- `teamIds` entries SHOULD exist in `TEAM_MOCKS` when teams section is non-empty

**State (presence)**:

| status | UI label | StatusChip tone |
| --- | --- | --- |
| `online` | Online | healthy |
| `offline` | Offline | muted |
| `invited` | Invited | warn |

---

### CapabilityEffect

Display-only permission row for least-privilege posture.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Row id |
| `capability` | string | yes | Capability key (e.g. `operations.apply`) |
| `effect` | `"allow" \| "deny" \| "request-only"` | yes | StatusChip |

| effect | Label | Tone |
| --- | --- | --- |
| `allow` | Allow | healthy |
| `deny` | Deny | danger |
| `request-only` | Request only | warn |

**Invariant**: Renderer MUST NOT enforce these rows as authorization.

---

### MemberSession

Client/session attributable to the person.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Session id |
| `device` | string | yes | Device/client label |
| `location` | string | yes | Location or origin |
| `lastActive` | string | yes | Humanized |
| `current` | boolean | no | True → Current chip; no revoke activate |

**Transitions (mock only)**:
- Non-current session + row activate → mock revoke toast (no fixture mutation required in v1)
- Current session → no revoke action

---

### MemberActivityEvent

Recent event involving the person as actor (presentation; not full Audit entity).

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Event id |
| `title` | string | yes | Event summary |
| `at` | string | yes | When (humanized) |
| `result` | string | no | Success / Pending / Failed / Denied → StatusChip |

Full audit remains Access → Audit (`WorkspaceAuditEvent`).

---

### Team — `WorkspaceTeam` (related)

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Joined via `Person.teamIds` |
| `name` | string | Teams table PrimaryCell |
| `onCall` | string | Secondary column |
| `members`, `owned`, `updated` | … | Used by team detail, not required in person table |

**Relationship**: Person N↔M Team via `teamIds` (mock: array of ids; no join table).

---

### Membership Role (value object)

Not a separate stored entity in mock stage — string label on Person (`Admin` | `Operator` | `Viewer` fixtures). Future custom roles remain out of scope (roles matrix).

---

## Activation target (UI navigation model)

Not a domain aggregate; session UI state via `@polterware/frontend-state`:

```text
scopeId: "access:people"
entityKind: "member"
entityId: Person.id
```

`MemberDetail` mounts only when this triple matches. Invitation / team use the same scope with different `entityKind`.

---

## Fixture set (canonical)

| id | Role | MFA | Purpose |
| --- | --- | --- | --- |
| `m1` | Admin | true | Full elevated permissions + multi-session + activity |
| `m2` | Operator | true | Mixed allow / request-only / deny |
| `m3` | Viewer | false | Read allows + write deny / request-only |

---

## Future live mapping (non-binding)

When core lands, map roughly:

| UI field | Likely domain |
| --- | --- |
| Person | identity user + organization membership |
| role / permissions | membership + policy capabilities |
| sessions | identity sessions |
| activity | audit / activity projections filtered by actor |
| teams | organization team memberships |

No protobuf contract in this feature plan.
