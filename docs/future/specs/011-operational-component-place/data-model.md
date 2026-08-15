# Data Model: Operational Component Placement

**Feature**: `011-operational-component-place`  
**Date**: 2026-08-11  
**Scope**: Authoring-time identities and links in the architecture document (+ ports for env/inventory mocks)

---

## 1. Entities

### 1.1 ProjectEnvironment (port DTO)

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Stable; prefer lowercase (`production`, `staging`, `development`, `local`) |
| `label` | yes | Display (`Production`) |
| `kind` | no | production-like / preview / local for future policy |
| `protected` | no | Hint only in v1 |

**Source**: Project settings / fixtures. Not stored inside architecture document as definitions — only **references** via component `environmentIds`.

---

### 1.2 PlaceIdentityDraft (transient UI)

Not persisted until commit.

| Field | Required | Notes |
| --- | --- | --- |
| `typeId` | yes | Catalog definition id |
| `name` | yes | Instance name |
| `environmentId` | yes | Must exist in ProjectEnvironment list |
| `description` | no | Instance purpose |
| `role` | no | Enum id (primary, replica, …) |
| `owner` | no | Free text team/person |
| `resourceId` | no | Link candidate |

**Validation** (pure):

- name trim non-empty, max 200 (align canvas sanitize)
- environmentId ∈ project envs
- role empty or ∈ allowlist
- resourceId if set must pass link validation

---

### 1.3 Architecture Component (ArchNode — persisted)

Existing document entity; fields this feature **writes**:

| Field | Unlinked place | Linked place |
| --- | --- | --- |
| `name` | instance name | instance name (may default from resource) |
| `type` / `subtype` / `provider` | from catalog | from catalog (+ resource type hints) |
| `environmentIds` | `[environmentId]` | same |
| `description` | purpose optional | optional |
| `owner` | optional | optional |
| `metadata.role` | optional | optional |
| `origin` | `manual` | `manual` (or `discovered` only if future import sets it) |
| `state` | `planned` | `linked` |
| `linkStatus` | `unlinked` | `linked` |
| `resourceId` | `null` | resource id |
| geometry / ports | defaults | defaults |

**Does not write**: observed health, provider-owned metrics, secrets, target ids.

---

### 1.4 LinkableResource (port DTO)

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Inventory id |
| `name` | yes | Display |
| `type` | yes | Inventory type string |
| `environmentId` | yes | Normalized |
| `environmentLabel` | no | Display |
| `provider` | no | e.g. AWS |
| `dataSource` | yes | `fixture` \| `core` |

---

### 1.5 Component–Resource Link (v1 embedding)

v1 embeds link on ArchNode (`resourceId` + `linkStatus` + `state`).  
Future: first-class `component_resource_links` with provenance columns (spec domain) — out of mock scope but model must not prevent migration.

**Provenance mock**:

| Field | v1 |
| --- | --- |
| linkedAt | optional metadata or omit |
| linkedBy | omit (local mock) |
| method | `manual` implied |

---

## 2. Relationships

```text
Project
  └── ProjectEnvironment 1..*
        └── referenced by ArchNode.environmentIds[]

ArchNode (component)
  └── optional resourceId → LinkableResource

Catalog ComponentTypeDefinition
  └── selected → seeds ArchNode.type/subtype/provider
```

---

## 3. State transitions

```text
                    place (no resource)
[none] ──────────────────────────────► planned + unlinked
         place (with resource)
      ──────────────────────────────► linked + linkStatus linked

planned + unlinked ── link resource ──► linked
linked ── unlink (later) ──► planned + unlinked   (MAY; detail action optional v1)

linked ── observe evidence ──► observed          (OUT OF SCOPE this feature)
```

**Forbidden transitions at place**:

- → `observed` without evidence pipeline  
- → healthy UI from form fields alone  

---

## 4. Validation rules

| Rule | Error |
| --- | --- |
| Empty name | Name is required |
| Unknown environmentId | Environment is required |
| No project environments | Cannot place — add an environment first |
| Invalid role id | Role is invalid |
| Resource env mismatch | Cannot link resource from a different environment |
| Resource not found | Resource unavailable |
| Non-placeable catalog type | Not available yet |

---

## 5. Display mapping

| Storage | Detail / Card |
| --- | --- |
| environmentId | ProjectEnvironment.label |
| metadata.role | PLACE_ROLES label |
| description | Operational notes / purpose |
| state planned | Planned |
| state linked | Linked (not Healthy) |
| resourceId | Resource name if resolvable else id |

---

## 6. Fixture alignment

Minimum fixture coverage for quickstart:

| Env ids | Resources |
| --- | --- |
| production, staging, development, local | At least one Database in production, one Service, one empty-env-or-staging row |

Normalize fixture environment strings (`Production` → id `production`) at port boundary.
