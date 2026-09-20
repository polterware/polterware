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

Validate against `.specify/memory/constitution.md` (Polterware Constitution v4.0):

- **I. Local first-version scope**: `docs/spec.md` defines implemented behavior;
  `docs/product-vision.md` defines direction. Planned PPM, agent, Web App, and
  Cloud capabilities are not presented as shipped.
- **II. Honest product behavior**: Live, queued, failed, preview, seeded, bounded,
  and unavailable states match what actually occurred. Local work never implies
  a remote deployment or complete provider graph.
- **III. Runtime separation**: Electron Main supervises `polterware-core`; the
  renderer uses only `window.polterware`. Local persistence, PTY, process, and
  operation authority remain in the Go core.
- **IV. Local project model**: Projects are local directories with environments,
  and Architecture / Canvas remains the default entry unless `docs/spec.md`
  explicitly changes it.
- **V. Supported integrations only**: The catalog remains the nine approved CLI
  providers. No placeholders, provider-token fields, or claims of complete cloud
  discovery.
- **VI. Contracts and storage**: Domain contracts precede adapters. Protobuf is
  the Go/TypeScript cross-runtime contract; SQLite uses the accepted clean
  baseline and technical bearer process authentication.
- **VII. One product UI system**: Use `DESIGN.md` and shared frontend primitives;
  do not create parallel control chrome, entity inspectors, or interaction
  infrastructure.
- **VIII. Security by boundary**: The renderer stays unprivileged. Core access is
  typed and validated. App-owned records do not deliberately persist provider
  tokens; raw terminal output is treated as potentially sensitive.
- **IX. Maintainability over breadth**: Add no surface, provider, package, runtime,
  or PPM abstraction without an accepted specification and concrete consumer.
- **X. Verification gates**: Preserve static correctness and plan
  risk-proportional tests, respecting the current task's authorization to run
  them.

If any gate fails, redesign or record a concrete justified violation in
Complexity Tracking. Unresolved product-boundary, runtime, security, contract,
or UI-system conflicts block implementation.

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
apps/desktop/                  # Electron Main, preload, shared IPC, Solid renderer
packages/features/<feature>/   # Product surface and feature composition
packages/frontend/             # Shared design system, canvas, platform, and state
packages/contracts/            # Generated TypeScript messages
backend/domains/               # Local domain models and invariants
backend/providers/             # Local adapters such as SQLite, PTY, CLI, and process
backend/compositions/          # Core service composition
backend/runtimes/core/         # The active Go runtime
schemas/                       # Protobuf source and generation configuration
```

**Structure Decision**: [List only paths this feature will actually create or
modify. State the runtime owner, contract changes, shared UI primitives, and why
any new package is required now. Do not add Web App, Cloud, MCP, or PPM modules
unless the accepted feature specification explicitly puts them in scope.]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
