# Polterware Constitution

Version: 3.0.0
Ratified: 2026-08-14

## I. Local first-version scope

The normative product is the reduced local Desktop described in `docs/spec.md`. Architecture, Resources, Knowledge, Operations, Terminal, Integrations, and footer Settings are the complete active information architecture. Earlier platform specifications are non-normative material under `docs/future/`.

## II. Honest visual behavior

The Desktop is fixture-only. It must never present a visual action as real execution, authentication, synchronization, or persistence. Session memory is the only mutation boundary. Terminal and integration transcripts are synthetic and contain no real credential material.

## III. Runtime separation

The Desktop does not start, supervise, discover, or call the Go core. Electron Main remains thin and the renderer uses only the typed preload bridge. No raw operating-system access enters the renderer.

The repository retains a separate `polterware-core` for future work. Its active domains and Protobuf services are limited to Projects, Architecture, Resources, Knowledge, Operations, Workflows, Integrations, Process, Logs, Health, and Maintenance.

## IV. Local project model

A project is a local directory with environments. Architecture / Canvas is the default entry. Project creation and switching require no remote ownership or hosted context.

## V. Supported integrations only

The catalog contains exactly nine providers with a represented CLI login path: GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. No placeholder or speculative provider is allowed. CLI flows are visual only.

## VI. Contracts and storage

Domain contracts precede framework adapters. Protobuf generates Go code only while the Desktop has no core client. SQLite begins at one clean baseline and does not preserve compatibility with earlier databases. Core access uses a technical bearer session token without a person or account model.

## VII. One product UI system

`DESIGN.md` and `@polterware/frontend-design-system` are the UI source of truth. Product features compose the shared Solid primitives, Ark-backed behavior, Hugeicons set, `DataTable`, and global `DetailPanel`. Parallel component chrome and feature-owned interaction infrastructure are forbidden.

## VIII. Security by boundary

The renderer has no unrestricted Node.js, Electron, shell, PTY, privileged network bridge, or filesystem capability. IPC is declared, typed, minimal, and validated. Tokens and sensitive provider material are never written to logs, fixtures, transcripts, or documentation.

## IX. Maintainability over breadth

Add no surface, package, runtime, provider, or abstraction without concrete use in the active product. Remove dead consumers and references together. Prefer small direct models to transitional dual implementations.

## X. Verification gates

Changes must preserve static correctness and repository hygiene. The standard gates are typecheck, design lint, Buf lint, Go vet, and `git diff --check`. Risk-proportional tests must be maintained for domain, contract, security, and critical boundary changes; execution follows the current task authorization.

## Governance

Changing the active product boundary requires an explicit specification change. This Constitution overrides archived specifications. Amendments must update this file, `docs/spec.md`, and any affected source-of-truth document in the same change.
