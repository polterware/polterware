# Implementation Plan: Integrations Secondary Flows

**Branch**: `029-integrations-secondary-flows` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/029-integrations-secondary-flows/spec.md`

## Summary

Elevate four thin satellite surfaces in `@polterware/feature-integrations` to professional mock quality: multi-step setup wizard, webhooks DataTable + create, sync center freshness/history/trigger, and explicit conflict resolution (keep-local / keep-remote / merge). Preserve `integration-detail.tsx` as-is; only minimal `index.tsx` hand-off wiring.

## Technical Context

**Language/Version**: TypeScript 5.x · SolidJS 1.9  

**Primary Dependencies**: `@polterware/frontend-design-system`, `@polterware/feature-shell` mocks, `@polterware/frontend-state` (detail hand-off only)  

**Storage**: N/A (session mock signals only)  

**Testing**: Vitest (node) for pure helpers; package `src/**/*.{test,spec}.ts`  

**Target Platform**: Desktop Solid renderer (mock fixtures)  

**Project Type**: Feature package satellite UI  

**Performance Goals**: Instant local mock interactions; no heavy Main/renderer work  

**Constraints**: Design-system only (DataTable, StatusChip, Dialog/A11yDialog, Button, Field, Card); no hand-rolled card chrome; no Actions columns; secrets write-only; no rewrite of IntegrationDetail  

**Scale/Scope**: 4 UI modules + status helpers + ≥2 test files; index wiring only  

## Constitution Check

- **I. Operational product**: Strengthens external tool connection honesty and sync conflict governance — pass.
- **II. Domain before framework**: Status/choice helpers pure TS; UI consumes helpers — pass.
- **III. Runtime separation**: Renderer feature only; mock claims only — pass.
- **IV. Monorepo deps**: integrations → design-system + shell + state; no cycles — pass.
- **V. Contracts-first**: Mock DTOs local/shell; no new cross-runtime contracts this slice — pass.
- **VI. Security**: Secrets write-only; signing secret never printed — pass.
- **VII. Auditable mutations**: Conflict resolve names choice; mock audit copy — pass.
- **VIII. Local-first**: Sync/conflict offline queue visible; no silent overwrite — pass.
- **IX–XI**: N/A scale; structured toasts only — pass.

## Project Structure

### Documentation (this feature)

```text
specs/029-integrations-secondary-flows/
├── spec.md
├── plan.md
├── tasks.md
├── checklists/requirements.md
└── contracts/secondary-flows-ui.md
```

### Source (owned)

```text
packages/features/integrations/src/
├── setup-wizard.tsx          # elevate
├── setup-wizard-helpers.ts   # steps + validation (new)
├── setup-wizard-helpers.test.ts
├── webhooks.tsx              # elevate
├── webhook-status.ts         # tone helpers (new)
├── sync-center.tsx           # elevate
├── sync-status.ts            # health/outcome/stale helpers (new)
├── sync-status.test.ts
├── conflict-resolve.tsx      # elevate
├── conflict-resolve helpers stay in module or conflict-status.ts
├── conflict.test.ts          # expand choices
├── index.tsx                 # finish hand-off only
└── integration-detail.tsx    # DO NOT rewrite
```

### Out of bounds

architecture · operations workflows · org-team · settings · terminal · access · shell route redesign · IntegrationDetail greenfield

## Implementation Approach

1. **Helpers first** — pure status/step/choice functions with unit tests.
2. **Setup wizard** — four steps, StatusChip progress, Field/Button, cancel + finish → Connected + `detailPanelStore.activateTarget` when provider matches `INTEGRATION_MOCKS`.
3. **Webhooks** — DataTable columns (destination, events, status, last delivery); Card-free detail strip or clean bordered section using design-system only; A11yDialog create; empty via DataTable.Empty / PageEmpty.
4. **Sync center** — StatusChip on health, queue status, history outcome; overall last success/fail + stale; Sync now mutates local history mock; empty states; open ConflictResolve on row.
5. **Conflict resolve** — list + detail; Card for local/remote compare; choices `keep-local` | `keep-remote` | `merge`; no default auto-resolve; expand tests.
6. **Index** — `onFinish` activates matching integration by name; open setup from reconfigure already exists; do not touch IntegrationDetail body.

## Rejection Rubric (definition of done)

- [ ] Each of 4 flows uses StatusChip for status/health/result
- [ ] None remain raw lists without empty/error
- [ ] ≥2 test files new/expanded passing
- [ ] `integration-detail.tsx` not rewritten from scratch
- [ ] Forbidden to claim “already implemented” for thin stubs

## Complexity Tracking

No constitution violations requiring justification. Local fixtures in sync-center retained (already present) rather than expanding shell if sufficient for mock UX.
