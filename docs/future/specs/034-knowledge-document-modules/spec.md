# Feature Specification: Knowledge Document Detail Modules

**Feature Branch**: `034-knowledge-document-modules`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Extract and professionalize Knowledge document details (Docs · Decisions · Runbooks) out of the monolithic knowledge index; keep document layout + maximize format dock; enrich properties; composition tests per kind; do not touch Spec Kit."

**Related product docs**: `docs/spec.md` §8.7 Knowledge (documents, architectural decisions, runbooks, ownership, links to systems/components/resources/operations), AGENTS.md document-layout law (DetailPanel.Body `layout="document"`, properties row + Markdown body, maximize for writing surface), DESIGN.md (StatusChip, DetailPanel, no second entity page).

**Extends**: Existing Knowledge feature (`packages/features/knowledge`) which already opens Docs / Decisions / Runbooks in the global DetailPanel with MarkdownEditor and a properties row — all inline in `index.tsx` (~750 lines). This feature modularizes and densifies those details; it does **not** invent a new product surface.

**Parallelism boundary**: Other P2 agents own other surfaces. This feature owns **only** Knowledge Docs · Decisions · Runbooks document detail modules under `specs/034-knowledge-document-modules/`. **MUST NOT** touch Spec Kit package/tabs/kinds, Overview, Activity, Audit, or Architecture.

---

## Summary

Operators inspecting project knowledge open Docs, Decisions, or Runbooks in the single global bottom DetailPanel as professional document details: fixed properties (owner, status as filled chip, environment, linked components as quiet badges, updated/author), kind-specific properties (decision status + alternatives/consequences reflection; runbook severity + steps/commands summary without live execution), Markdown body with format dock only when maximized, and honest empty/not-found states.

`index.tsx` becomes orchestration (tabs, collection registration, wiring). Kind detail composition lives in dedicated modules with helpers and composition tests. Spec Kit remains a separate SYSTEM destination and is out of scope.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Modular document details for all three kinds (Priority: P1)

As an operator, when I select a Doc, Decision, or Runbook from the Knowledge collection, I see a kind-appropriate document detail in the global DetailPanel (document layout: properties row + Markdown body), not a blank or second page. The page index only orchestrates tabs and collection selection.

**Why this priority**: §8.7 Knowledge is operational memory; AGENTS.md requires document layout for Knowledge artifacts. Today detail JSX for all three kinds is inlined in a monolithic index, blocking modular tests and consistent enrichment.

**Independent Test**: Open one fixture of each kind from Docs / Decisions / Runbooks tabs; each renders document layout with properties + body; index source does not embed the full three-kind detail trees.

**Acceptance Scenarios**:

1. **Given** the Docs tab and a selected document, **When** detail opens, **Then** the global DetailPanel shows the document title, domain icon, properties row, and Markdown body under document layout.
2. **Given** the Decisions tab and a selected decision, **When** detail opens, **Then** decision status is a filled StatusChip and the body remains Markdown (ADR content).
3. **Given** the Runbooks tab and a selected runbook, **When** detail opens, **Then** severity is a filled StatusChip and the body remains Markdown (procedure content).
4. **Given** any of the three kinds, **When** the panel is not maximized, **Then** the Markdown format dock/toolbar is hidden; **When** maximized, **Then** the format dock is available (existing maximize-for-format behavior preserved).

---

### User Story 2 - Enriched properties and honest empty/not-found (Priority: P1)

As an operator, properties expose ownership, status, environment, linked components, and timestamps/author without leaving the first fold; missing entity ids show an honest not-found state; empty collections stay honest. Linked component badges remain quiet relation links (clickable when the host supplies navigation).

**Why this priority**: §8.7 requires ownership and links to components/resources/operations; shallow properties under-serve operator inspection compared to other polished details.

**Independent Test**: Open a rich document with components and owner; open a decision with alternatives/consequences reflected; open a runbook with steps/commands summary counts; force a missing id and confirm not-found copy.

**Acceptance Scenarios**:

1. **Given** a published document with owner, environment, components, author, and updated time, **When** detail renders, **Then** those properties appear; status uses StatusChip; components render as badges (clickable only when host handler exists).
2. **Given** a decision with alternatives and consequences, **When** detail renders, **Then** status is StatusChip and alternatives/consequences are reflected in the properties row and/or Markdown body SSOT without inventing a second data source.
3. **Given** a runbook with steps and commands, **When** detail renders, **Then** severity is StatusChip and a steps/commands summary appears as properties (counts or short summary) without claiming live remote execution.
4. **Given** an active target id that is not in the mock/collection set, **When** detail would render, **Then** the operator sees an honest not-found state (not a blank panel).
5. **Given** filters that match nothing, **When** the collection renders, **Then** empty copy remains honest (existing collection empty states preserved).

---

### User Story 3 - Composition tests and index orchestration (Priority: P2)

As a maintainer, each Knowledge kind has dedicated detail modules (or a shared document shell with kind-specific modules) plus helpers and composition tests so regressions in layout, StatusChip usage, and wiring are caught without opening the desktop app.

**Why this priority**: Peer entity details (Access agents, Architecture nodes, Operations runs) already use helpers + composition contracts; Knowledge must match that maintainability bar.

**Independent Test**: Package vitest runs and passes with coverage for all three kinds (one suite covering three kinds or one test file per kind); index imports dedicated detail modules rather than defining full detail trees inline.

**Acceptance Scenarios**:

1. **Given** the knowledge package test config, **When** tests run, **Then** composition contracts assert document layout, StatusChip for status/severity, maximize-gated format toolbar behavior, and wiring from index.
2. **Given** pure helpers for status/severity tones and property builders, **When** unit tests run, **Then** tones/labels are stable for published/draft, proposed/accepted/deprecated/superseded, and runbook severities.
3. **Given** Spec Kit artifacts, **When** this feature is reviewed, **Then** no Spec Kit tabs, kinds, or package files are modified.

---

### Edge Cases

- Active target scope matches a Knowledge tab but entity id is missing → not-found, not blank.
- Document archived from detail → leaves collection (existing mock archive behavior) without crashing detail wiring.
- Decision without component links → Components property omitted; body still shows ADR markdown.
- Runbook without operation/component links → those properties omitted; severity and steps/commands summary still show.
- Body session overrides → edits remain session-local mock; no claim of Core persistence.
- Format dock: open (half) panel never shows Markdown format toolbar; maximized does.
- Spec Kit destination and its kinds remain untouched by this feature.
- Parallel agents own other packages — no edits outside `packages/features/knowledge/**` except read-only use of shell DOCUMENT/DECISION/RUNBOOK mocks/types.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Knowledge document details (Doc, Decision, Runbook) MUST render inside the single global DetailPanel using document layout (fixed properties row + Markdown body). MUST NOT introduce a second entity-detail page, modal, drawer, or permanent split editor.
- **FR-002**: Markdown format toolbar/dock MUST appear only when the DetailPanel mode is maximized; half-open and collapsed MUST NOT show the format dock.
- **FR-003**: Document (Doc) properties MUST include status as StatusChip, owner, author, updated, environment when present, tags when present, and linked components as quiet badges (clickable when host provides navigation).
- **FR-004**: Decision properties MUST include status as StatusChip (proposed / accepted / deprecated / superseded tones), author, date, id, linked components when present, and reflection of alternatives/consequences via properties and/or Markdown body SSOT.
- **FR-005**: Runbook properties MUST include severity as StatusChip, service, owner, last validated, steps/commands summary (counts or short summary without live execution claims), and optional component/operation links when present.
- **FR-006**: Missing entity for the active Knowledge target MUST show an honest not-found detail state; loading/error collection states MUST remain honest where already modeled.
- **FR-007**: Detail composition MUST live in dedicated modules (e.g. `document-detail.tsx`, `decision-detail.tsx`, `runbook-detail.tsx`, and/or a shared knowledge document shell + helpers) — `index.tsx` MUST be orchestration (tabs, tables, collection registration, wiring) only, not the sole home of all three detail JSX trees.
- **FR-008**: Pure helpers MUST centralize status/severity label and tone mapping and property list building where practical.
- **FR-009**: Composition (and helper) tests MUST cover all three kinds (≥1 test file per kind **or** one suite covering three kinds) and MUST pass under the knowledge package vitest config.
- **FR-010**: MUST NOT mix Spec Kit tabs, kinds, routes, or package code into this feature; Spec Kit remains a separate SYSTEM destination.
- **FR-011**: Existing collection UX (Docs · Decisions · Runbooks tabs, DataTable rows, filters, create document mock dialog, header actions Archive / Accept / Deprecate / Run / Duplicate / Open terminal) MUST remain wired and mock-honest.
- **FR-012**: MUST NOT claim live Core knowledge persistence, live runbook execution, or multiplayer editing.

### Key Entities

- **Knowledge document (Doc)**: Project operational document — title, Markdown body, status (draft/published), owner, author, updated, environment, tags, linked component ids/names.
- **Architectural decision**: ADR-shaped record — title, status (proposed/accepted/deprecated/superseded), author, date, structured context/decision/alternatives/consequences with Markdown body as canvas SSOT, linked components, optional supersedes/related ids.
- **Runbook**: Operational procedure — title, severity, service, owner, last validated, prerequisites/steps/commands/rollback (body SSOT), optional component/resource/node/operation links. Presentation is documentary; Run action is mock-only.
- **Document properties row**: Fixed metadata strip above the Markdown body in document layout (chips, text values, quiet badges).

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Collaboration / audit / understand — project operational memory (Knowledge), not a generic notebook.
- **Runtime owner (III)**: Desktop Renderer feature package only (`packages/features/knowledge`).
- **Domain vs adapter (II, IV)**: Presentation + pure helpers over existing shell mock types; no domain→Core rewrite.
- **Contracts (V)**: UI composition contract in this feature’s `contracts/`; mock types remain in shell workspace mocks.
- **Product loop impact**: Knowledge document inspection/editing surface inside global DetailPanel.
- **Evidence expectations**: n/a for live evidence; links are declared mock relations.
- **Mutation safety (VII)**: Session-local body edits and mock archive/accept/run toasts only — no silent Core mutation.
- **Operation audit fields**: n/a for this mock polish (no new apply path).
- **Security boundary (VI)**: Renderer only; no secrets; no unrestricted OS.
- **Local vs cloud authority (VIII)**: Local fixtures/mocks; offline-capable presentation.
- **Performance (IX)**: Composition split only; body scroll stays inside DetailPanel.Body.
- **Resilience (X)**: Honest not-found / empty / mock toast failures.
- **Observability (XI)**: n/a new telemetry.
- **Test plan (XII)**: Helper unit + composition source contracts for three kinds; package vitest.
- **Structure created now (XIII)**: Modules inside existing `packages/features/knowledge` only; no new scenic package.
- **Single source of truth (XIV)**: Markdown body remains SSOT for long form; structured fields may mirror summaries without dual persistence.
- **Big-bang / refactor context (XV)**: Incremental modularization on refactor branch workstream.
- **Non-goals (I, XIII)**: Spec Kit, wiki platform, embeddings, live execution, multiplayer, second inspector.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any of the three Knowledge kinds and see properties + body in one document detail surface without navigating to a second page.
- **SC-002**: 100% of status (docs/decisions) and severity (runbooks) presentations in kind details use filled status chips (not chip-less plain status alone).
- **SC-003**: Format dock is available in maximized detail and unavailable in non-maximized detail for document bodies.
- **SC-004**: Missing-id selection yields an honest not-found message instead of an empty panel.
- **SC-005**: Knowledge package automated tests covering all three kinds pass.
- **SC-006**: Index orchestration does not concentrate the full three-kind detail JSX trees (dedicated modules own composition).
- **SC-007**: Spec Kit package and Spec Kit product destination remain unmodified by this feature’s delivery.

## Assumptions

- Shell `DOCUMENT_MOCKS` / `DECISION_MOCKS` / `RUNBOOK_MOCKS` and types are the data source for this polish; enrichment may use existing fields (steps, commands, alternatives, consequences) without inventing live Core APIs.
- Host callbacks (`onOpenComponent`, `onOpenOperation`, `onOpenTerminal`, tab open handlers) already exist on `KnowledgeView` and remain the only navigation hooks.
- Maximize-for-format-dock behavior already works via detail panel mode selector and must be preserved, not redesigned.
- Parallel agents will not edit `packages/features/knowledge/**` concurrently.
- English UI strings; Portuguese chat only.

## Out of Scope

- Spec Kit package, tabs, kinds, fixtures, or detail experience.
- Overview, Activity, Audit, Architecture, Resources, Operations (beyond optional runbook operation link badges).
- Live Core knowledge CRUD, search backend, or multiplayer editing.
- Live runbook execution, terminal command injection, or claiming mock Run as real apply.
- Redesign of Knowledge collection tables or tab set (Docs · Decisions · Runbooks stays).
- Ownership as a separate Knowledge tab (ownership remains metadata on documents).

## Acceptance Rubric (delivery gate)

Any fail → FAILED:

- [x] `index.tsx` does **not** concentrate the full JSX of all three details (dedicated modules)
- [x] `layout="document"` preserved for Knowledge document details
- [x] StatusChip on status/severity where applicable
- [x] ≥1 test file per kind **or** one suite covering three kinds
- [x] Spec Kit tabs/kinds/package **not** mixed into this delivery
- [x] Forbidden: claim “already implemented” solely because MarkdownEditor already exists in the monolith
