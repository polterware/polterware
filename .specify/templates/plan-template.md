# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]

**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]

**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]

**Storage**: [if applicable, e.g., PostgreSQL, CoreData, files or N/A]

**Testing**: [e.g., pytest, XCTest, cargo test or NEEDS CLARIFICATION]

**Target Platform**: [e.g., Linux server, iOS 15+, WASM or NEEDS CLARIFICATION]

**Project Type**: [e.g., library/cli/web-service/mobile-app/compiler/desktop-app or NEEDS CLARIFICATION]

**Performance Goals**: [domain-specific, e.g., 1000 req/s, 10k lines/sec, 60 fps or NEEDS CLARIFICATION]

**Constraints**: [domain-specific, e.g., <200ms p95, <100MB memory, offline-capable or NEEDS CLARIFICATION]

**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*
*Critical conflicts MUST block implementation (Governance §5).*

Validate against `.specify/memory/constitution.md` (Polterware Constitution v2.x):

- **I. Operational Architecture Product**: Strengthens architecture understanding,
  resource organization, safe execution, collaboration, auditability, or external
  tool connection. Not a static diagram-only feature or incomplete clone of a
  specialized platform. Domain root is System; machines are Targets.
- **II. Domain Before Framework**: Business rules/invariants live outside Electron,
  UI frameworks, DBs, provider SDKs, and transports. Domains define ports;
  adapters implement them. No domain → concrete provider/UI/runtime imports.
- **III. Strict Runtime Separation**: Owning runtime is explicit (Main / Preload /
  Renderer / Web / polter-core / polter-server / polter-node / CLI / MCP). No
  permanent duplication of business rules. Unimplemented runtimes not claimed live.
  MCP cannot bypass human policies.
- **IV. Modular Monorepo And Directed Dependencies**: Dependency direction apps →
  features → shared frontend → contracts; backend runtimes → compositions →
  domains → ports; providers implement ports. No cycles; no deep internal imports.
- **V. Contracts-First**: Cross-runtime APIs/events/commands/streams use shared
  versioned contracts. No hand-maintained duplicate types. Breaking changes have
  migration strategy.
- **VI. Security By Default**: Least privilege, boundary validation, authz scope,
  secrets as references, no unrestricted renderer Node/OS, no silent policy bypass.
- **VII. Safe And Auditable Execution**: Mutations declare plan → review → approve →
  apply → logs → audit. Operations carry actor, target, scope, policy, approval,
  state, timestamps, result/error, correlation/operation ID. No silent destructive
  ops; retries are idempotent-safe.
- **VIII. Local-First And Cloud-Capable**: Desktop remains useful offline/local.
  Cloud adds collaboration without mandatory network for pure local capability.
  Data authority (local/cloud/synced) is explicit; sync conflicts not silent.
- **IX. Performance As Architecture**: No heavy work on Electron Main or UI thread.
  Bounded queues, async IPC, backpressure, timeouts, cancellation, pagination/
  virtualization when lists/streams apply.
- **X. Resilience And Consistency**: Integrations define timeouts, retries, error
  classes, offline/recovery, limits. Primary state and related events stay
  consistent (outbox or equivalent when needed).
- **XI. Internal Observability**: Structured logs/metrics/traces correlate actor,
  org, system, environment, resource, operation, approval, trace. Not a product
  replacement for external APM platforms.
- **XII. Testability And Quality**: Domain rules testable without Electron/HTTP/
  real providers. Risk-proportional tests planned. Task not complete if required
  checks fail. Mocks do not replace all critical-boundary integration tests.
- **XIII. Real Code, No Scenic Architecture**: Only packages with real API +
  consumers. No empty scaffolds or speculative generic layers.
- **XIV. Single Source Of Truth**: No permanent dual implementations. Temporary
  dual paths only inside authorized migration branch and removed before done.
- **XV. Decisive Architectural Refactors**: If this work is part of an authorized
  big-bang refactor, target architecture is complete — not scaffolding and not
  permanent dual systems — with continuous validation and deliberate data/behavior
  migration.
- **XVI. Documentation And Decisions**: Public module responsibility is clear;
  contracts/boundaries/threats/decisions stay current.

**Hard stops (cannot approve for convenience alone):** VI Security, IV dependency
direction, XIV single source of truth.

If any gate fails, redesign or record a justified violation in Complexity
Tracking. Critical unresolved conflicts block implementation.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Prefer active Polterware paths. Delete unused branches. The
  delivered plan must not include Option labels.
-->

```text
apps/                        # compose shared capabilities; no domain reimplementation
├── desktop/                 # Main | Preload | Renderer (retain workbench features)
├── web/                     # only when a real web consumer is in scope
└── presentation/            # narrative only — not product runtime

# Backend runtimes / binaries — create only with real responsibility + consumers
polter-core / polter-server / polter-node / CLI / MCP

packages/ or crates/         # directed deps: domains → ports; providers implement ports
├── domains/ ports/ compositions/
├── contracts/ protocol-types/   # contracts-first shared types
├── providers/ platform/         # adapters only
└── ui/ feature-*                # browser-safe shared frontend only

tests: unit (domain) | contract | integration | e2e | security | performance
       proportional to risk (Constitution XII)
```

**Structure Decision**: [List only paths this feature will actually create or
modify. See `docs/ARCHITECTURE.md` and Constitution IV/XIII. State runtime owner,
domain vs adapter placement, contracts touched, and why any new package is
required now — no scenic scaffolds.]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
