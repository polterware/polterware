# Contract: Spec Kit UI

**Owner**: `packages/features/spec-kit`  
**Consumers**: `App.tsx`  
**Nav section**: `speckit` (SYSTEM · label **Spec Kit**)

## Purpose

Project engineering standards shaped like Spec Kit on disk. Kind-first tabs so constitution, feature specs, plans, tasks, research, and checklists are never mixed in one list. Supports multi-feature projects (N APIs / product slices) via `featureKey`.

**Does not** run Speckit CLI. Phase 1 is UI + logical paths only — no live bind to `.specify/` or `specs/`.

## Tabs (kind-first)

| Tab id | Label | Artifact kind | Logical path |
| --- | --- | --- | --- |
| `constitution` | Constitution | `constitution` | `.specify/memory/constitution.md` |
| `specs` | Specs | `spec` | `specs/{featureKey}/spec.md` |
| `plans` | Plans | `plan` | `specs/{featureKey}/plan.md` |
| `tasks` | Tasks | `tasks` | `specs/{featureKey}/tasks.md` |
| `research` | Research | `research` | `specs/{featureKey}/research.md` |
| `checklists` | Checklists | `checklist` | `specs/{featureKey}/checklists/{slug}.md` |

Default tab: `constitution`.

Legacy aliases: section `specs` / `spec-kit` → `speckit`; `knowledge/specs` → `speckit/specs`; kind ids `spec`/`plan`/`checklist` map to tab ids.

## List UX

- Each tab filters `WorkspaceSpec` by a single `specKind`.
- **Constitution**: Title · Status · Updated (singleton empty state when none).
- **Other tabs**: Title · **Feature** (`featureKey`) · Status · Updated.
- Toolbar: search + status filter + feature filter (feature filter omitted on Constitution).
- `DataTable` only; no Actions column.

## Detail

- Global `DetailPanel`, `Body layout="document"`.
- Properties: status, owner, author, updated, feature (when present), **Path** (`logicalPath`, mono), source.
- Body: Markdown via design-system `MarkdownEditor`.
- Actions: Copy markdown.
- No second detail page / permanent split editor.

## Create

- Dialog scoped to active tab kind.
- Constitution: title only.
- Other kinds: title + **feature key** (required).
- Mock create only; body edited in panel afterward.

## Model

```ts
type SpecKind =
  | "constitution"
  | "spec"
  | "plan"
  | "tasks"
  | "checklist"
  | "research"
  | "other"; // model-only; no dedicated tab

type WorkspaceSpec = {
  id: string;
  title: string;
  bodyMarkdown: string;
  specKind: SpecKind;
  status: "draft" | "active" | "deprecated";
  featureKey?: string; // required for non-constitution
  logicalPath: string; // Spec Kit path SSOT for future import/export
  source?: "project" | "repo";
  // author, owner, updatedAt, tags…
};
```

## Non-goals (this surface)

- Live repo FS bind / Speckit CLI
- Feature-first hub as primary navigation (kind-first is locked)
- Contracts-folder browser tab
- Backend document persistence beyond session mocks
