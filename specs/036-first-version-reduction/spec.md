# Feature Specification: First Version Reduction

**Status**: Implemented in `feature/first-version`  
**Normative product spec**: `docs/spec.md`

## User stories

### 1. Understand a local project

As a developer, I open a local project directly on Architecture / Canvas and move between Architecture, Resources, Knowledge, Operations, Terminal, and Integrations without unrelated product areas.

### 2. Rehearse maintenance safely

As a developer, I can inspect visual resources, run records, workflows, schedules, and terminal sessions while every action stays demonstrative and in memory.

### 3. Preview provider login

As a developer, I can choose any supported provider and see its official CLI login command in the shared terminal UI, including unavailable, ready, authenticating, connected, and failed states, without initiating authentication.

## Functional requirements

- FR-001: Architecture / Canvas is the default route.
- FR-002: The project sidebar contains exactly six retained surfaces and footer Settings.
- FR-003: Local project creation accepts name, directory, and environments.
- FR-004: Knowledge contains Docs and Runbooks.
- FR-005: Operations contains Runs, Workflows, and Schedules with direct local steps.
- FR-006: Terminal page and drawer use shared mock sessions.
- FR-007: Integrations use one registry with exactly nine CLI-capable providers.
- FR-008: Provider setup uses the shared visual terminal component.
- FR-009: Connected integration detail contains Overview, Resources, CLI, and Sync.
- FR-010: Desktop must not execute commands, launch authentication, or connect to the core.
- FR-011: The retained core requires a technical session token and uses one clean SQLite baseline.
- FR-012: Prior product specifications remain accessible only as non-normative future material.

## Verification contract

Tests are maintained for routes, fixtures, provider registry, terminal presentation, retained contracts, and baseline migration. This implementation stage runs static checks only; automated tests, builds, and browser validation require separate authorization.
