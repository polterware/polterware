# Research: Spec Kit Document Modules

**Date**: 2026-08-12 · **Feature**: [spec.md](./spec.md)

## Decision 1 — Single detail module for all kinds

**Decision**: One `SpecArtifactDetail` (not six kind-specific detail files).

**Rationale**: All kinds share document layout, Markdown body, copy action, and the same property set with optional feature key. Kind-specific UX is already owned by tabs/empty copy, not by divergent detail chrome.

**Alternatives considered**:
- Per-kind detail files → higher drift risk, weak SSoT.
- Leave detail inline in index → rejected by delivery rubric.

## Decision 2 — Reuse shell path/kind helpers

**Decision**: Import `specKindForTab` and `logicalPathForSpec` from `@polterware/feature-shell`; do not copy path templates into the feature package.

**Rationale**: Shell types already document Spec Kit portability SSOT; duplication would diverge from create-dialog path previews and future import/export.

## Decision 3 — Properties always include kind + mono path

**Decision**: Property builder always emits Kind and Path (mono); Feature only when `featureKey` present; Source only when present; Status always via StatusChip tone mapping.

**Rationale**: Matches operator need for multi-feature portability and the rejection rubric (`logicalPath` + `featureKey` visible).

## Decision 4 — Format dock only when maximized

**Decision**: Read detail panel mode from shared store; pass `showToolbar={mode === "maximized"}` to MarkdownEditor.

**Rationale**: DESIGN document-layout LAW; half-open panel is inspection, maximized is writing surface. Already correct in monolith — preserve in extracted module.

## Decision 5 — Tests as source composition contracts

**Decision**: Node vitest reads detail/index sources for layout tokens and property labels; pure helper unit tests for tones/kind labels/property builder including logicalPath.

**Rationale**: Matches Access detail extraction pattern; avoids full DOM Solid setup while enforcing LAW.

## Decision 6 — Boundary isolation from Knowledge

**Decision**: No imports from `@polterware/feature-knowledge`; no shared “document entity” abstraction forced in this slice.

**Rationale**: Product IA separates Spec Kit (engineering standards) from Knowledge (operational memory). Parallel agents own Knowledge.

## Open questions

None blocking. Mock stage, fixtures multi-feature already present in `SPEC_MOCKS`.
