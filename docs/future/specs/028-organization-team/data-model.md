# Data Model: Organization Team

## WorkspaceOrgMember

Organization-scoped membership fixture (not project Access person).

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| id | string | yes | Stable fixture id (`om-1`, …) |
| organizationId | string | yes | Scopes list per org |
| name | string | yes | Display name / PrimaryCell title |
| email | string | yes | Overview |
| role | string | yes | Owner · Admin · Member (fixture vocabulary) |
| mfaEnabled | boolean | yes | StatusChip Enabled/Disabled |
| status | `"active" \| "invited" \| "suspended"` | yes | Optional table column |
| joinedAt | string | yes | Humanized join date |
| lastActive | string | no | Humanized activity; optional column |
| invitedBy | string | no | Overview when invited |
| title | string | no | Job title |
| teamCount | number | no | Fixture metric |
| projectCount | number | no | Fixture metric |
| notes | string | no | Optional notes section |

## Collections

- **`ORG_TEAM_MOCKS`**: exported `WorkspaceOrgMember[]` in shell workspace-mocks.
- Coverage: ≥5 members; ≥2 roles; both MFA states; ≥1 non-active status; members for `org-polterware` (primary); optional second org members for filter proof.

## Detail target

| Field | Value |
| --- | --- |
| scopeId | `org-team:{organizationId}` |
| entityKind | `org-member` |
| entityId | member id |

## Non-entities (out of scope)

- Project `WorkspacePerson` / `PEOPLE_MOCKS`
- Project `WorkspaceTeam` / `TEAM_MOCKS`
- Live invitation delivery records
