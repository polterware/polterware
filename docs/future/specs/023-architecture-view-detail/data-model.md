# Data Model: Architecture View Detail

**Feature**: `023-architecture-view-detail`  
**Stage**: Fixture / mock

## ArchViewMember

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| id | string | yes | Architecture node id (e.g. `n-pay`) |
| name | string | yes | Display name from known node/component |
| type | string | no | Component type label |
| layer | string | no | Optional layer label |

**Rule**: Members exist only when fixture authors list them. Never generated from `nodeCount`.

## ArchViewCard (enriched)

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| id | string | yes | Stable view id |
| name | string | yes | Single-line title |
| description | string | yes | May be empty string → empty copy in UI |
| nodeCount | number | yes | Scalar metric; not a substitute for members |
| updated | string | yes | Relative or absolute update label |
| owner | string | no | Human owner label |
| environments | string[] | no | Environment scope labels |
| tags | string[] | no | Purpose/tags |
| purpose | string | no | Short purpose phrase |
| members | ArchViewMember[] | no | When present (even empty), membership is known |
| notes | string | no | Operator notes |
| filterSummary | string | no | Human-readable filter/lens summary |
| dataSource | `"fixture"` \| string | no | Honesty label; default fixture |

## Membership resolution (helpers)

| Condition | UI |
| --- | --- |
| `members` is array with length > 0 | DataTable of members |
| `members` is empty array | Honest empty: no members in this view |
| `members` is undefined | Unavailable: membership list not available in mock; still show nodeCount metric |

## Relationships

- **Architecture View** → optional **Members** (0..n architecture nodes)
- **Architecture View** → optional environments/tags (presentation only in mock)
- Open → canvas activation (session state), not a stored relation rewrite

## Validation

- `id`, `name` non-empty for ready fixtures
- `nodeCount` ≥ 0
- If `members` provided, each member has `id` + `name`
- Enriched primary fixtures (≥2) must include at least one of: owner, environments, tags/purpose, members, notes, filterSummary beyond the baseline four fields
