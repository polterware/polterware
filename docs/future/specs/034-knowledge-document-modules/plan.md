# Implementation Plan: Knowledge Document Detail Modules

**Branch**: `034-knowledge-document-modules` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/034-knowledge-document-modules/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: Existing Knowledge monolith (`packages/features/knowledge/src/index.tsx`), shell `DOCUMENT_MOCKS` / `DECISION_MOCKS` / `RUNBOOK_MOCKS`, design-system `DetailPanel` document layout + `MarkdownEditor` + `StatusChip` + `Badge`, `detailPanelStore` mode for format dock.

**Risk control**: Extract and densify document details only. Do not touch Spec Kit, Architecture, Overview, Activity, Audit. Preserve maximize-for-format-dock and collection orchestration.

**Parallelism boundary**: MAY touch `packages/features/knowledge/**` and this specs folder. MUST NOT edit Spec Kit package or other feature packages.

---

## Summary

Extract Knowledge Docs · Decisions · Runbooks detail composition from the monolithic `index.tsx` into dedicated modules with pure helpers, enriched properties, honest not-found, and composition tests — while preserving document layout and maximize-only format dock.

**Technical approach**:

1. **Shared document shell** (`knowledge-document-shell.tsx` or keep `DocumentCanvas` + `DocumentProperties` in a shared module): `DetailPanel.Body layout="document"`, properties row, `MarkdownEditor` with `showToolbar` only when `detailPanelStore.mode === "maximized"`.
2. **Kind modules**:
   - `document-detail.tsx` — Doc header (Archive), properties via helpers, shell body
   - `decision-detail.tsx` — Decision header (Accept/Deprecate), status chip, alternatives/consequences reflection
   - `runbook-detail.tsx` — Runbook header (Run/Duplicate/Open terminal), severity chip, steps/commands summary
3. **Helpers** (`knowledge-document-helpers.ts` or per-kind helpers): document status tone, decision status tone, runbook severity tone, property builders, steps/commands summary strings.
4. **Not-found**: When active target scope is Knowledge for a kind but entity is missing, render honest not-found content (header + body message).
5. **index.tsx**: Keep tabs, DataTable collections, filters, create dialog, collection registration; wire `<DocumentDetail>`, `<DecisionDetail>`, `<RunbookDetail>` (or equivalent) with body override handlers and host callbacks.
6. **Tests + vitest**: Add `vitest.config.ts`, package test script, composition/helper tests covering three kinds.
7. **Mocks**: Prefer existing shell fields; only enrich shell mocks if a required property is truly missing (prefer helpers over mock schema changes).

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-knowledge` (owner)
- `@polterware/feature-shell` (DOCUMENT/DECISION/RUNBOOK mocks + types)
- `@polterware/frontend-design-system` (`DetailPanel`, `MarkdownEditor`, `StatusChip`, `Badge`, icons)
- `@polterware/frontend-state` (`createDetailPanelSelector`, `detailPanelStore`)

**Storage**: Session-local body overrides in Knowledge view (existing). No new Core API.

**Testing**: Vitest node in `packages/features/knowledge` — helper unit + composition source contracts for three kinds.

**Target Platform**: Desktop Knowledge page (Docs · Decisions · Runbooks) global DetailPanel

**Constraints**:
- AGENTS/DESIGN document-layout law
- StatusChip for status/severity
- No Spec Kit bleed
- English UI strings
- Forbidden: “already done” because Markdown exists inline

**Scale/Scope**: One feature package modularization; ~3 detail modules + helpers + tests; slim orchestration index.

---

## Constitution Check

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Knowledge operational memory §8.7 |
| **II. Domain Before Framework** | PASS | Pure tone/property helpers |
| **III. Runtime Separation** | PASS | Renderer feature only |
| **IV. Monorepo deps** | PASS | knowledge → shell, design-system, state |
| **V. Contracts-First** | PASS | UI contract in `contracts/` |
| **VI. Security** | PASS | No secrets; mock actions only |
| **VII. Safe execution** | PASS | No live apply; mock toasts |
| **VIII. Local-first** | PASS | Fixtures offline |
| **IX. Performance** | PASS | Composition-only |
| **X. Resilience** | PASS | not-found / empty honest |
| **XI. Observability** | PASS | n/a new telemetry |
| **XII. Testability** | PASS | helpers + composition contracts |
| **XIII. Real code** | PASS | Extend existing package |
| **XIV. SSOT** | PASS | Markdown body remains canvas SSOT |
| **XV. Big-bang** | PASS | Incremental extract |
| **XVI. Docs** | PASS | This feature folder |

**Hard stops**: No Spec Kit edits; no second inspector; no live execution claims.

---

## Project Structure

### Documentation (this feature)

```text
specs/034-knowledge-document-modules/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── contracts/
│   └── knowledge-document-detail-ui.md
└── checklists/
    └── requirements.md
```

### Source (implementation)

```text
packages/features/knowledge/
├── package.json                 # add vitest test script
├── vitest.config.ts             # NEW
├── tsconfig.json
└── src/
    ├── index.tsx                # orchestration only
    ├── knowledge-document.css   # existing document chrome
    ├── knowledge-document-shell.tsx   # NEW shared document layout shell
    ├── knowledge-document-helpers.ts  # NEW pure helpers
    ├── document-detail.tsx      # NEW
    ├── decision-detail.tsx      # NEW
    ├── runbook-detail.tsx       # NEW
    ├── document-detail.test.ts  # NEW (or combined suite)
    ├── decision-detail.test.ts  # NEW
    └── runbook-detail.test.ts   # NEW
```

**Structure Decision**: Dedicated kind modules + shared shell/helpers inside existing package. No new package.

---

## Complexity Tracking

| Item | Why needed | Simpler alternative rejected because |
| --- | --- | --- |
| Shared shell module | Avoid triplicating document layout + format dock | Single mega `knowledge-document-detail` switch still needs shared chrome; shell is clearer |
| Three kind modules | Kind-specific headers/actions/properties | One file with giant switch recreates monolith |
| Composition source tests | Match Access/Architecture detail gate without browser harness | Full e2e not authorized and slower |

---

## Implementation Order

1. Helpers + unit tests (tones, property builders, summaries)
2. Shared shell (document layout + format dock gate)
3. Document / Decision / Runbook detail modules + not-found
4. Wire index orchestration; delete inline detail trees
5. Composition tests + package vitest
6. Rubric self-check + `pnpm --filter @polterware/feature-knowledge test`
