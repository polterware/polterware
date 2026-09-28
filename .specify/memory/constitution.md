# Polterware Constitution

Version: 4.0.0
Ratified: 2026-08-14
Last amended: 2026-09-19

## I. Local first-version scope

The implemented product is the local Desktop described in `docs/spec.md`. Architecture, Resources, Knowledge, Operations, Terminal, Integrations, and footer Settings are the complete active information architecture. `docs/product-vision.md` governs product direction without claiming that planned capabilities are implemented. Earlier specifications under `docs/future/` are non-normative historical material.

## II. Honest product behavior

The Desktop must describe actions according to what actually occurred. SQLite mutations, local operations, terminal sessions, CLI authentication hand-offs, and bounded integration sync may be reported as real only when the core completed them. Previews, demonstrations, unavailable capabilities, skipped schedules, and partial imports must remain explicit. No surface may imply a remote deployment, complete provider graph, or Cloud capability that did not occur.

## III. Runtime separation

Electron Main starts and supervises the local core (`polterware serve`). The renderer uses only the typed `window.polterware` preload bridge and never receives raw Electron, Node.js, filesystem, shell, or credential-store access.

The Go core is the local authority for persistence and execution. Its active domains and Protobuf services are limited to Projects, Architecture, Resources, Knowledge, Operations, Workflows, Integrations, Process, Terminal, Logs, Health, and Maintenance.

## IV. Local project model

A project is a local directory with environments. Architecture / Canvas is the default entry. Project creation and switching require no remote ownership or hosted context.

## V. Supported integrations only

The catalog contains exactly nine providers with an official CLI login path: GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. No placeholder or speculative provider is allowed. Integration flows may detect the CLI, perform the supported login hand-off, probe connection state, and import a bounded inventory without storing provider tokens in the app.

## VI. Contracts and storage

Domain contracts precede framework adapters. Protobuf generates Go services and TypeScript messages consumed through the Electron Main adapter. SQLite begins at one clean baseline and does not preserve compatibility with earlier databases. Core access uses a technical bearer session token without a person or account model.

## VII. One product UI system

`DESIGN.md` and `@polterware/frontend-design-system` are the UI source of truth. Product features compose the shared Solid primitives, Ark-backed behavior, Hugeicons set, `DataTable`, and global `DetailPanel`. Parallel component chrome and feature-owned interaction infrastructure are forbidden.

## VIII. Security by boundary

The renderer has no unrestricted Node.js, Electron, shell, PTY, privileged network bridge, or filesystem capability. Core-owned PTY and local execution are exposed only through declared, typed, validated IPC and domain controllers. App-owned records must not deliberately copy provider tokens into SQLite, structured logs, fixtures, or documentation. Raw terminal and command output is not assumed to be sanitized and must be treated as potentially sensitive.

## IX. Maintainability over breadth

Add no surface, package, runtime, provider, or abstraction without an accepted specification and concrete use. The Polterware Project Model is a product direction, not permission to add speculative schemas or infrastructure. Remove dead consumers and references together. Prefer small direct models to transitional dual implementations.

## X. Verification gates

Changes must preserve static correctness and repository hygiene. The standard gates are typecheck, design lint, Buf lint, Go vet, and `git diff --check`. Risk-proportional tests must be maintained for domain, contract, security, and critical boundary changes; execution follows the current task authorization.

## Governance

Changing implemented behavior requires an explicit update to `docs/spec.md`. Changing product-direction invariants requires an explicit update to `docs/product-vision.md`. This Constitution overrides archived specifications. Amendments must update this file and every affected source-of-truth document in the same change.
