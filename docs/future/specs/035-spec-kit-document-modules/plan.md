# Implementation Plan: Spec Kit Document Modules

**Branch**: `035-spec-kit-document-modules` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/035-spec-kit-document-modules/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` Spec Kit UI contract, DESIGN.md DetailPanel document layout LAW, shell `WorkspaceSpec` + `SPEC_MOCKS` + `logicalPathForSpec` / `specKindForTab`

**Risk control**: Touch only `packages/features/spec-kit/**` (and package test wiring). Do not edit Knowledge, overview, activity, or audit. Do not reimplement shell path/kind helpers. Do not claim Speckit CLI. Reject “already done” if detail remains only inline in the monolith.

---

## Summary

Extract and professionalize **Spec Kit artifact document details** for Constitution · Specs · Plans · Tasks · Research · Checklists:

1. Dedicated **`SpecArtifactDetail`** composition (`layout="document"`, MarkdownEditor, properties including kind / StatusChip status / featureKey / mono logicalPath / owner / author / updatedAt / source).
2. Pure **helpers** for status tones, labels, empty/create copy, property row building.
3. **Index** reduced to kind tabs + filters + DataTable collection + `registerCollection` + create dialog wiring.
4. **Tests**: kind mapping, detail composition contract, logicalPath/featureKey display, index wiring.
5. Copy honesty: never claims Speckit CLI execution.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9

**Primary Dependencies**:
- `@polterware/feature-spec-kit` (owner)
- `@polterware/feature-shell` (`SPEC_MOCKS`, `WorkspaceSpec`, `SpecKind`, `specKindForTab`, `logicalPathForSpec`)
- `@polterware/frontend-design-system` (`DetailPanel`, `MarkdownEditor`, `DataTable`, `StatusChip`, tabs/toolbar/dialog)
- `@polterware/frontend-state` (detail panel store + mode for format dock)

**Storage**: Fixture-only shell mocks. No FS / CLI / core persistence in this slice.

**Testing**:
- Vitest node in `packages/features/spec-kit` (helpers + source composition contracts)
- Existing desktop `detail-fixture-coverage` Spec Kit assertions remain green (no change required if fixtures already multi-feature)

**Target Platform**: Desktop renderer Spec Kit destination

**Project Type**: Solid feature package modularization

**Performance Goals**: Fixture detail open feels immediate; document body is the writing scroller; no IPC

**Constraints**:
- DESIGN LAW: document layout for Spec Kit artifacts; StatusChip for status; global DetailPanel only
- Reuse shell `logicalPathForSpec` / `specKindForTab` — no duplication
- Do not merge with Knowledge
- English UI strings; mock-labeled toasts

---

## Constitution Check

| Principle | Application |
| --- | --- |
| Domain before framework | Spec Kit kinds/paths are domain; Solid is composition only |
| DetailPanel LAW | Single global panel; document layout; format dock only maximized |
| SSoT | One detail module; shell owns path/kind helpers |
| Honesty | Mock create/copy; no CLI claim |
| Boundaries | Only `feature-spec-kit`; no Knowledge edits |
| Testing XII | Risk-proportional helpers + composition contracts |

---

## Project Structure

### Documentation (this feature)

```text
specs/035-spec-kit-document-modules/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/spec-kit-document-ui.md
├── checklists/requirements.md
└── tasks.md
```

### Source (implementation)

```text
packages/features/spec-kit/
├── package.json                 # vitest test script
├── vitest.config.ts             # node env
├── src/
│   ├── index.tsx                # tabs + collection + wiring only
│   ├── spec-kit-document.css    # document properties chrome
│   ├── spec-artifact-detail.tsx
│   ├── spec-artifact-detail-helpers.ts
│   └── spec-artifact-detail.test.ts
```

**Structure decision**: Single artifact detail module covers all kinds (properties vary by data, not by forking six nearly identical components). Helpers hold pure mapping and property construction.

---

## Implementation Approach

1. **Helpers first** — status tone, kind label, empty/create labels, `buildSpecArtifactProperties(spec)`.
2. **Detail module** — header (Copy markdown), `DetailPanel.Body layout="document"`, properties, MarkdownEditor with `showToolbar` only when mode is `maximized`.
3. **Slim index** — import detail; keep tabs/filters/table/create; registerCollection unchanged.
4. **Tests + package script** — green package vitest.

## Complexity Tracking

No unjustified new packages or dual implementations. CSS file stays for document property chrome already present.
