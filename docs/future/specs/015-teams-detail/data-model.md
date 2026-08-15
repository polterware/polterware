# Data Model: Access Teams Detail

**Feature**: `015-teams-detail`  
**Stage**: Mock fixtures (shell workspace mocks)  
**Live persistence**: out of scope

---

## Overview

```text
Team
  ├── Status (active | archived)
  ├── Lead (id + name)
  ├── Environments[]
  ├── On-call / Escalation / Schedule
  ├── MemberRoster[] → Person (optional deep link)
  ├── OwnedResource[]
  ├── PolicyLink[]
  └── TeamActivityEvent[]
```

Domain roots (`docs/spec.md`): **organizations** (teams, memberships) + **Access** human governance. UI entity is the project **Team** under Access → People (Teams table).

---

## Entities

### Team — `WorkspaceTeam`

Collaboration/ownership group shown in Access Teams list and detail.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable fixture id (`t1`…) |
| `name` | string | yes | Header title; table PrimaryCell title |
| `members` | number | yes | List column; MUST equal `memberRoster.length` for primary fixtures |
| `owned` | number | yes | List column; MUST equal `ownedResources.length` for primary fixtures |
| `onCall` | string | yes | Primary on-call display name |
| `updated` | string | yes | Humanized relative update |
| `description` | string | yes | Overview |
| `purpose` | string | no | Optional purpose line |
| `status` | `"active" \| "archived"` | yes | StatusChip |
| `leadId` | string | no | Person id when known |
| `leadName` | string | yes | Lead display |
| `environments` | string[] | yes | Environment scope labels |
| `escalation` | string | yes | Escalation path summary |
| `schedule` | string | no | On-call schedule label |
| `createdAt` | string | yes | Humanized create date |
| `memberRoster` | TeamMember[] | yes | May be empty → empty table |
| `ownedResources` | TeamOwnedResource[] | yes | May be empty |
| `policies` | TeamPolicyLink[] | yes | May be empty |
| `activity` | TeamActivityEvent[] | yes | May be empty |
| `notes` | string | no | Optional notes section |

**Validation (mock stage)**:
- `id`, `name` non-empty for list rows
- Platform primary fixture MUST have non-empty `memberRoster`, `ownedResources`, `policies`, `activity`
- Roster person ids SHOULD exist in `PEOPLE_MOCKS` when cross-navigation is expected

**State (team status)**:

| status | UI label | StatusChip tone |
| --- | --- | --- |
| `active` | Active | healthy |
| `archived` | Archived | muted |

---

### TeamMember (roster entry)

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Prefer Person id for deep link |
| `name` | string | yes | PrimaryCell title |
| `role` | string | yes | Project role label |
| `status` | `"online" \| "offline" \| "invited"` | yes | Presence → StatusChip |
| `title` | string | no | Optional job title |

---

### TeamOwnedResource

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Resource link id (fixture) |
| `name` | string | yes | Display name |
| `kind` | string | yes | e.g. Database, Service, Queue |
| `environment` | string | yes | Environment label |

---

### TeamPolicyLink

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Prefer Policy id when known |
| `name` | string | yes | Policy display name |
| `enforcement` | string | yes | e.g. Enforce, Advise |
| `status` | `"Active" \| "Disabled"` | yes | StatusChip |

---

### TeamActivityEvent

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Event id |
| `title` | string | yes | Event summary |
| `at` | string | yes | When (humanized) |
| `result` | string | no | Success / Pending / Failed / Denied → StatusChip |

Full audit remains Access → Audit.

---

## Activation target (UI navigation model)

```text
scopeId: "access:people"
entityKind: "team"
entityId: Team.id
```

`TeamDetail` mounts only when this triple matches. Member / invitation use the same scope with different `entityKind`.

---

## Fixture set (canonical)

| id | Name | Purpose |
| --- | --- | --- |
| `t1` | Platform Team | Full composition: multi-member, resources, policies, activity |
| `t2` | Payments Team | Secondary rich team (ops-focused) |
| `t3` | Identity Team | Smaller roster; may exercise thinner collections |

---

## Future live mapping (non-binding)

| UI field | Likely domain |
| --- | --- |
| Team | organizations.teams |
| memberRoster | team_memberships + identity |
| ownedResources | ownership links / inventory projections |
| policies | policy actors/scope |
| activity | audit / activity projections filtered by team |

No protobuf contract in this feature plan.
