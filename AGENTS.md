# Polterware Agent Instructions

## Communication and editing

- Speak to Erick in Brazilian Portuguese.
- Write code, UI strings, technical documentation, commits, logs, and identifiers in English.
- Preserve unrelated work and make the smallest safe change.
- Do not create branches or commits unless explicitly requested.
- Do not run browsers, automated tests, builds, previews, distributions, or development servers unless explicitly requested in the current task.

## Current product boundary

Polterware's first version is a local fixture-backed Desktop with six project surfaces: Architecture, Resources, Knowledge, Operations, Terminal, and Integrations. Settings is fixed in the sidebar footer. Architecture / Canvas is the default route.

The Desktop supports multiple local projects and environments. It has no hosted or shared workspace model. All apparent mutations remain in renderer memory and must be presented as demonstrative.

The Desktop must not start or query the Go core. It must not execute shell commands, create PTYs, open provider login browsers, detect CLI binaries, authenticate, or synchronize data. The terminal and integration flows are visual transcripts only.

The integration catalog contains exactly GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. Do not add placeholders. Never request, display, or persist real tokens in fixtures.

## Architecture boundaries

- Go local authority: `backend/runtimes/core` and retained domains only.
- Desktop: `apps/desktop` with thin Electron Main, typed preload bridge, and Solid renderer.
- Features: `packages/features/<feature>`.
- Shared frontend: `packages/frontend`.
- Contracts: `schemas` to generated Go code in `backend/platform/gen`.
- Renderer access: `window.polterware` only; no raw Electron or Node.js.
- The core uses a technical bearer session token without an account directory.
- There is one clean SQLite baseline and no compatibility requirement for earlier databases.

## Product navigation

- Architecture: Canvas, Components, Views
- Resources: Inventory, Nodes, Processes
- Knowledge: Docs, Runbooks
- Operations: Runs, Workflows, Schedules
- Terminal: visual sessions
- Integrations: Connected, Catalog
- Settings: General, Environments, Advanced

Keep project and environment switchers, Commander, history controls, Settings, and quick Terminal access. Do not introduce additional product surfaces without an approved specification update.

## UI law

`DESIGN.md` and `@polterware/frontend-design-system` are the visual source of truth. Product features must use shared `Input`, `Textarea`, `Select`, `Field`, `Button`, `Card`, `Surface`, Ark-backed overlays, `PageToolbar`, `FilterChip`, `DataTable`, `DitherIconTile`, `StatusChip`, and the global `DetailPanel` where applicable.

Do not hand-roll shared control chrome, menus, dialogs, operational tables, or entity panels. Use only radius and color tokens. Structural surfaces and text are opaque; alpha is limited to shared border, shadow, scrim, and transition tokens. Hugeicons Free Stroke Rounded is the only product icon set.

## Verification

After relevant changes, use static checks allowed by the task: typecheck, `pnpm design:lint`, `buf lint`, `go vet ./...`, and `git diff --check`. Maintain risk-proportional tests, but do not run them without explicit permission.

Previous specifications live under `docs/future/` and are non-normative.
