# Tasks: Spec Kit Document Modules

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories)

**Tests**: Risk-proportional — pure helpers; composition source contract; package vitest green. Agents.md: do not run `pnpm dev` / full build unless asked.

**Constitution / guardrails**:

- DetailPanel document LAW; StatusChip for status; mono logicalPath
- Reuse shell `logicalPathForSpec` / `specKindForTab`
- No Knowledge package edits
- No Speckit CLI claims
- Index = wiring only after extraction

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable
- **[Story]**: US1–US3 from spec.md

## Path Conventions

- Feature: `packages/features/spec-kit/src/`
- Detail: `packages/features/spec-kit/src/spec-artifact-detail.tsx`
- Helpers: `packages/features/spec-kit/src/spec-artifact-detail-helpers.ts`
- Tests: `packages/features/spec-kit/src/spec-artifact-detail.test.ts`
- Index: `packages/features/spec-kit/src/index.tsx`
- CSS: `packages/features/spec-kit/src/spec-kit-document.css`
- Shell helpers (read-only reuse): `packages/features/shell/src/workspace-mocks/types.ts`

---

## Phase 1: Setup

- [x] T001 Confirm ownership stays in `packages/features/spec-kit` per [plan.md](./plan.md)
- [x] T002 [P] Add `packages/features/spec-kit/vitest.config.ts` (node, `src/**/*.{test,spec}.ts`)
- [x] T003 [P] Wire `test` script + `vitest` devDependency in `packages/features/spec-kit/package.json`

---

## Phase 2: Foundational helpers

- [x] T004 Implement pure helpers (status tone, kind label, empty/create labels, property builder with Kind/Status chip/Feature/Path mono/Owner/Author/Updated/Source) in `packages/features/spec-kit/src/spec-artifact-detail-helpers.ts`
- [x] T005 [P] Keep UI contract SSOT in `specs/035-spec-kit-document-modules/contracts/spec-kit-document-ui.md`
- [x] T006 [P] Helper unit tests for tones, kind labels, and property builder including logicalPath + featureKey in `packages/features/spec-kit/src/spec-artifact-detail.test.ts`

**Checkpoint**: Helpers pure and tested

---

## Phase 3: User Story 1 - Document detail (P1) 🎯 MVP

**Goal**: Dedicated document detail with required properties and maximized-only format dock.

### Tests

- [x] T007 [P] [US1] Composition contract: `layout="document"`, MarkdownEditor, StatusChip, Copy markdown, maximized toolbar gating in `spec-artifact-detail.test.ts`
- [x] T008 [P] [US1] Assert properties labels include Kind, Path, Feature, Status in composition/helpers tests

### Implementation

- [x] T009 [US1] Implement `SpecArtifactDetail` in `packages/features/spec-kit/src/spec-artifact-detail.tsx` (header, document body, properties, editor, honest loading/not-found optional)
- [x] T010 [US1] Preserve document CSS hooks in `packages/features/spec-kit/src/spec-kit-document.css` (import from detail or index)

---

## Phase 4: User Story 2 - Kind-first collection honesty (P1)

**Goal**: Index keeps tabs/filters/empty/create; no Knowledge merge; no CLI claims.

- [x] T011 [US2] Slim `packages/features/spec-kit/src/index.tsx` to tabs + collection + registerCollection + create; wire `<SpecArtifactDetail />`
- [x] T012 [US2] Ensure empty copy and create dialog honesty; create path uses shell `logicalPathForSpec`; no Speckit CLI claims in strings
- [x] T013 [P] [US2] Index wiring assertions in `spec-artifact-detail.test.ts` (import detail, entityKind spec, no knowledge imports)

---

## Phase 5: User Story 3 - Regression suite (P1)

- [x] T014 [US3] Cover kind mapping via shell `specKindForTab` usage tests / tab normalize helpers in `spec-artifact-detail.test.ts`
- [x] T015 [US3] Run `pnpm --filter @polterware/feature-spec-kit test` and fix until green

---

## Phase 6: Polish

- [x] T016 Confirm rubric in [quickstart.md](./quickstart.md)
- [x] T017 Mark tasks complete; update feature status notes if needed

## Dependency graph

```text
T001–T003 → T004–T006 → T007–T010 → T011–T013 → T014–T015 → T016–T017
```

## Acceptance (rubric)

- [x] Dedicated detail module (not monolith-only)
- [x] logicalPath + featureKey visible in properties
- [x] StatusChip for artifact status
- [x] layout="document" preserved
- [x] Tests pass (10/10 package vitest)
- [x] Not merged with Knowledge
- [x] Not accepted as “already implemented” monolith
