# Feature Specification: Workflow Definition Depth

**Feature Branch**: `027-workflow-definition-depth`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Elevate Operations → Workflows (builder + model) to parity with docs/spec.md domain §25 workflows + run-detail quality. Enriched WorkflowDef, type-specific step config, version immutability honesty, failure/retry/approval semantics, StatusChip, recent runs link, empty/not-found states, composition tests, rich fixtures. Sequential pipeline only — no freeform canvas. Do not rewrite Run detail or Schedule detail."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Author a deep sequential workflow definition (Priority: P1)

An operator opens **Operations → Workflows**, selects a workflow (or creates a draft), and the global **DetailPanel** shows a sequential pipeline builder with: workflow name, description, trigger, definition **version**, definition **status** (Draft / Ready / …), ordered steps with **type-specific configuration**, optional **failure strategy**, per-step **retry policy** where applicable, and **approval step semantics** that make clear approval is not execution. Save increments version honestly for existing definitions; create starts at version 1. The product remains a linear Polter pipeline — not a freeform n8n-style canvas.

**Why this priority**: Domain §25 requires versioned definitions with steps, conditions, retries, failure strategy, and approval semantics. A pipeline diagram alone is not depth.

**Independent Test**: Open a rich fixture workflow; inspect header (version + status chip), trigger panel, at least three step types with distinct config fields, failure strategy, and an approval step with reviewers/condition/expiration; Save on an existing definition increments version by 1 in the UI.

**Acceptance Scenarios**:

1. **Given** a fixture workflow with steps, **When** the operator activates the row, **Then** DetailPanel shows the sequential pipeline and a configuration panel for the selected step or trigger.
2. **Given** an existing workflow at version N, **When** the operator saves after edits, **Then** the saved definition version becomes N+1 and the footer/header reflects the new version without claiming core durability.
3. **Given** step types validate, approval, and command (or deploy/notify), **When** each is selected, **Then** the config panel exposes distinct fields (not only a generic name/label).

---

### User Story 2 - Trust version immutability and definition honesty (Priority: P1)

The operator sees that **runs bind to an immutable definition version**. Editing the workflow does not rewrite historical runs. Fixture `dataSource` remains honest. Status uses shared **StatusChip** (Draft, Ready, Running, Success, Failed, Archived) — not hand-rolled pills and not run-status mapping misuse for definition lifecycle.

**Why this priority**: Domain invariant: edit does not alter in-flight or historical runs; version is first-class.

**Independent Test**: Header or metadata shows version and StatusChip; copy or metric states that runs reference the version at start; fixtures stay `dataSource: "fixture"`.

**Acceptance Scenarios**:

1. **Given** any open workflow, **When** detail renders, **Then** version is visible and status is a StatusChip with definition-appropriate labels.
2. **Given** a saved workflow linked to run fixtures, **When** the operator views definition metadata, **Then** wording does not claim that editing rewrites past runs.

---

### User Story 3 - Inspect approval, retry, and failure semantics (Priority: P1)

On approval steps, the operator configures (or sees fixture) condition, reviewers, expiration, and honesty that **approval is not execution**. On executable steps (command/deploy/health_check/build), optional **retry policy** (max attempts, backoff) is visible and copy states **retry never skips policy/approval**. Workflow-level **failure strategy** (stop / continue / fail pipeline) is editable on the trigger/definition panel.

**Why this priority**: Domain §25 and §114 list failure strategy, retries, and approval semantics as definition content.

**Independent Test**: Open a fixture with approval + a fixture with retries; both show structured fields in the config panel, not free text dump only.

**Acceptance Scenarios**:

1. **Given** an approval step, **When** selected, **Then** condition, reviewers, and expiration fields are present.
2. **Given** a step with retry policy, **When** selected, **Then** max attempts and backoff are visible.
3. **Given** the trigger/definition panel, **When** opened, **Then** failure strategy is selectable.

---

### User Story 4 - Navigate recent runs and edge states (Priority: P2)

From the workflow builder, the operator can open **recent runs** for this definition (when run fixtures share `workflowId`) via callback into the existing Runs detail — without rebuilding Run detail. Empty pipeline (zero steps) and not-found / missing workflow targets render honest empty or not-found chrome.

**Why this priority**: Closes the definition ↔ execution loop and prevents blank panels on bad targets.

**Independent Test**: Workflow with matching run fixtures shows a recent-runs control; empty draft shows empty pipeline; invalid/missing workflow prop shows not-found content.

**Acceptance Scenarios**:

1. **Given** `wf-deploy` with run fixtures, **When** detail opens, **Then** a Recent runs affordance lists or links at least one run without embedding full run detail chrome.
2. **Given** a draft with no steps, **When** the pipeline column renders, **Then** empty-state copy invites adding the first step.
3. **Given** a missing workflow entity for the active target, **When** the host mounts the builder in not-found mode, **Then** honest not-found content appears (no scenic blank).

---

### User Story 5 - Fixture depth and automated contracts (Priority: P2)

At least two workflows are fixture-rich: one with a full approval step, one with per-step retries. Dedicated unit tests cover workflow status helpers, model/version helpers, step config field contracts (≥3 types distinct), and fixture completeness. Sequential builder existence alone is not “done.”

**Why this priority**: Rubric rejects “already implemented because sequential builder exists.”

**Independent Test**: Operations package tests pass for workflow-status, workflow-model, and builder/fixture composition contracts.

**Acceptance Scenarios**:

1. **Given** the test suite for operations workflow modules, **When** run, **Then** all dedicated workflow-definition tests pass.
2. **Given** WORKFLOW_FIXTURES, **When** inspected, **Then** ≥2 workflows carry deep fields (version, failureStrategy or step retries, approval semantics where applicable).

---

### Edge Cases

- Create mode: version starts at 1; status Draft until first save promotes to Ready (session-local).
- Save without name: default “Untitled workflow” remains valid.
- Removing the only step: empty pipeline state returns; trigger remains selectable.
- Retry fields on approval steps: not required; approval semantics take precedence.
- Secrets: definition stores references/labels only — never literal secrets in fixtures.
- Run detail and Schedule detail modules MUST NOT be rewritten by this feature.
- Freeform graph canvas MUST NOT be introduced.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Workflow definition model MUST include: id, name, optional description, trigger, version (required number), status, ordered steps, optional failure strategy, optional input/output/artifact summaries, dataSource honesty.
- **FR-002**: Each step MUST include id, label, type, and type-specific config; MAY include condition, retry policy, failure strategy override, and approval semantics when type is approval.
- **FR-003**: Step types MUST include at least: validate, approval, build, deploy, health_check, command, notify — sequential order only.
- **FR-004**: Builder config panel MUST show distinct fields for ≥3 step types (e.g. approval ≠ command ≠ validate).
- **FR-005**: Definition version MUST display in the builder; Save on existing definitions MUST increment version by 1; create Save MUST set version 1.
- **FR-006**: Workflow status MUST render with design-system StatusChip using definition lifecycle labels (Draft, Ready, Running, Success, Failed, Archived or equivalent).
- **FR-007**: Failure strategy MUST be editable at definition (trigger) level; values: stop | continue | fail_pipeline (operator-facing labels).
- **FR-008**: Retry policy fields (max attempts, backoff) MUST be available on applicable executable steps; UI copy MUST state retry does not skip policy/approval.
- **FR-009**: Approval steps MUST expose condition, reviewers, expiration; UI copy MUST state approval is not execution.
- **FR-010**: Builder MUST support empty pipeline state and not-found workflow state.
- **FR-011**: Builder MAY offer Recent runs affordance when runs share workflowId; activating a run MUST call host callback (open run) — MUST NOT reimplement Run detail.
- **FR-012**: Fixtures MUST include ≥2 rich workflows (approval-heavy + retry-heavy) with dataSource fixture.
- **FR-013**: Dedicated tests MUST cover workflow status helpers, version/save model helpers, step config field contracts, and fixture depth.
- **FR-014**: Implementation scope is operations feature package workflow surfaces only; Run detail and Schedule detail MUST remain untouched.
- **FR-015**: Pipeline UI remains sequential (trigger → steps → end); freeform multi-edge canvas is out of scope.
- **FR-016**: Mock persistence is session-local only; toasts/copy MUST NOT claim live polter-core workflow engine durability.

### Key Entities

- **Workflow definition (WorkflowDef)**: Versioned reusable sequential orchestration definition.
- **Workflow step**: Ordered unit of work with type, config, optional retry/condition/approval semantics.
- **Workflow status**: Definition lifecycle for operator chrome (distinct from run execution status).
- **Failure strategy**: What the pipeline does when a step fails.
- **Retry policy**: Explicit per-step retry bounds; never bypasses policy/approval.
- **Approval semantics**: Gate configuration on approval steps (not execution).
- **Run binding**: Historical runs reference definition id + version at start; immutable for that run.

### Constitution Constraints *(mandatory)*

- **I Operational product**: Strengthens safe execution authoring and audit continuity.
- **III Runtime**: Renderer feature mock only; no claimed live workflow engine.
- **VI Security**: No secrets in fixtures; secrets as references if mentioned.
- **VII Safe execution**: Version immutability honesty; approval ≠ execution; retry does not skip policy.
- **XII Tests**: Risk-proportional unit tests required for model/helpers and composition contracts.
- **XIII Structure**: Extend `packages/features/operations` only.
- **Non-goals**: Freeform canvas, live scheduler/engine, rewrite of Run/Schedule detail, protobuf/core changes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operator can open a rich workflow and identify version, status chip, trigger, failure strategy, and step count in under 15 seconds.
- **SC-002**: ≥3 step types expose mutually distinct config fields in the builder.
- **SC-003**: Saving an existing definition always increases version by exactly 1 in the session model.
- **SC-004**: 100% of dedicated workflow-definition unit tests pass in the operations package.
- **SC-005**: ≥2 fixtures demonstrate depth (approval semantics and retries) with `dataSource: "fixture"`.
- **SC-006**: Run detail and Schedule detail source files are unchanged by this feature.

## Assumptions

- Sequential pipeline is the only Polter authoring graph for this slice.
- Session-local save is sufficient until core workflow store exists.
- Recent runs resolve from existing RUN_FIXTURES by `workflowId`.
- Definition status may still show last-run-oriented labels (Success/Failed) when fixtures use them; helpers normalize casing.

## Out of Scope

- Rewriting Run detail or Schedule detail.
- Freeform/node-graph canvas (n8n-style).
- Live workflow engine, durable version store, or protobuf contracts.
- Parallel/branching step graphs beyond sequential insert/reorder.
- New packages outside `packages/features/operations`.
