# Data Model: Project Settings Environments

## ProjectGeneralSettings (session mock)

| Field | Type | Notes |
| --- | --- | --- |
| name | string | Editable |
| description | string | Editable |
| defaultEnvironmentId | string | References ProjectEnvironment.id |
| projectId | string | Display + mock copy |
| repository | string | Optional display field |
| dataSource | `"fixture"` | Honesty |

## ProjectEnvironment

| Field | Type | Notes |
| --- | --- | --- |
| id | string | Stable fixture id |
| name | string | Display title |
| kind | `production` \| `staging` \| `development` \| `local` | StatusChip |
| protection | `strict` \| `standard` \| `relaxed` | StatusChip |
| resourceCount | number | Table + metric |
| nodeCount | number | Table + metric (local may use 0 or 1) |
| isDefault | boolean | Default column / metadata |
| protectionRules | string[] | Detail summary lines |
| description | string | Optional detail |
| dataSource | `"fixture"` | Honesty |

## ProtectionRule (summary line)

Plain string in fixture array — not a live policy evaluation result.

## Relationships

- One project has many environments.
- Exactly one environment SHOULD be `isDefault: true` in fixtures.
- General.defaultEnvironmentId SHOULD match that environment's id.

## Fixture set (minimum)

1. Production — kind production, Strict, default true, ≥2 rules  
2. Staging — kind staging, Standard  
3. Development — kind development, Relaxed  
