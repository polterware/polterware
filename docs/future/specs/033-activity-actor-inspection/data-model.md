# Data Model: Activity Actor Inspection

## ActivityEvent (workspace mock)

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable fixture id (`act-*`) |
| `type` | ActivityEventType | yes | architecture, operation, person, agent, sync, settings, access, node |
| `title` | string | yes | Single-line table/detail title |
| `description` | string | yes | Detail body narrative |
| `actor` | string | yes | Display name (person, agent, or "System") |
| `actorKind` | `"person" \| "agent" \| "system"` | yes | First-class co-management distinction |
| `environment` | string | yes | Production / Staging / Development / All |
| `at` | string | yes | Relative timestamp display |
| `target` | string | no | Target entity display name |
| `targetKind` | string | no | component, process, node, integration, member, settings, … |
| `targetId` | string | no | Optional id for open-target callback |

## Filter model (UI state)

| Filter | Multi-select values | Matches |
| --- | --- | --- |
| type | ActivityEventType ids | `event.type` |
| actorKind | person, agent, system | `event.actorKind` |
| environment | Production, Staging, Development, All (if present) | `event.environment` |
| search | free text | title, description, actor, target (case-insensitive) |
| period | optional `recent` | slice first N (UX convenience only) |

## Detail target

| Field | Value |
| --- | --- |
| `scopeId` | `activity:index` |
| `entityKind` | `activity-event` |
| `entityId` | ActivityEvent.id |

## Separation from Audit

| Surface | Entity kind | Package | Purpose |
| --- | --- | --- | --- |
| Activity | `activity-event` | feature-shell | Operational feed |
| Access Audit | `audit-event` | feature-access | Compliance / governance trail |
