# Data Model: Spec Kit Document Modules

**Feature**: [spec.md](./spec.md) · **Owner package**: `@polterware/feature-spec-kit`  
**Fixture SSOT**: `@polterware/feature-shell` workspace mocks

## Entities

### WorkspaceSpec (existing)

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Stable fixture id |
| `title` | string | Table primary cell + detail title |
| `bodyMarkdown` | string | Document body SSOT for editor |
| `specKind` | SpecKind | Filters by active tab via `specKindForTab` |
| `status` | `draft` \| `active` \| `deprecated` | StatusChip tones |
| `author` | string | Properties |
| `owner` | string | Properties |
| `updatedAt` | string | Display relative time in fixtures |
| `featureKey?` | string | Required UX for non-constitution create; column + filter |
| `logicalPath` | string | Mono property; future import/export path |
| `source?` | `project` \| `repo` | Optional provenance |
| `tags` | string[] | Search only |

### SpecKind (existing)

`constitution` | `spec` | `plan` | `tasks` | `checklist` | `research` | `other`  
(`other` is model-only; no tab.)

### SpecKitTabId (existing, shell)

`constitution` | `specs` | `plans` | `tasks` | `research` | `checklists`

## Relationships

```text
SpecKitTabId --specKindForTab--> SpecKind
(SpecKind, featureKey?) --logicalPathForSpec--> logicalPath string
WorkspaceSpec.specKind --filters--> tab collection
WorkspaceSpec.featureKey --optional filter--> multi-feature projects
```

## Property presentation model (feature-local)

```ts
type SpecArtifactPropItem = {
  label: string;
  value?: string;
  chip?: { tone: StatusTone; label: string };
  mono?: boolean;
};
```

Built by pure helper from `WorkspaceSpec` for the detail module.

## Validation / honesty rules

- No invented live FS contents.
- Create remains session mock.
- Missing artifact id → no fabricated body; honest empty/not-found handling at wiring layer.
- Do not duplicate path templates outside shell helpers.

## Fixture coverage expectations

- At least one constitution.
- At least two distinct `featureKey` values among non-constitution artifacts.
- Kinds covering tabs in use (spec, plan, tasks, research, checklist present for multi-kind smoke).
