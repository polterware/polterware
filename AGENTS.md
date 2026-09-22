# Polterware Agent Instructions

## Communication and editing

- Speak to Erick in Brazilian Portuguese.
- Write code, UI strings, technical documentation, commits, logs, and identifiers in English.
- Preserve unrelated work and make the smallest safe change.
- Do not create branches or commits unless explicitly requested.
- Do not run browsers, automated tests, builds, previews, distributions, or development servers unless explicitly requested in the current task.

## Current product boundary

Polterware's first version is a local SQLite-backed Desktop with six project surfaces: Architecture, Resources, Knowledge, Operations, Terminal, and Integrations. Settings is fixed in the sidebar footer. Architecture / Canvas is the default route.

The Desktop supports multiple local projects and environments. It has no hosted or shared workspace model. The Electron Main process starts `polterware-core`, and the renderer reads and writes through `window.polterware` only.

The Desktop may execute on the local machine through `polterware-core` only. The renderer never spawns shells or touches Node APIs; it talks through `window.polterware`.

- Terminal: interactive local and SSH sessions via PTY owned by Go core, with live output streams. SSH uses saved host profiles (alias, user, host, port, optional identity file path); private keys are never stored in SQLite.
- Operations / Workflows / Runbooks: local runner (`os/exec`) that transitions queued runs to running/succeeded/failed/cancelled and appends logs.
- Integrations: detect CLI binaries, start provider authentication commands in a core-owned PTY, optionally open provider documentation through Main `openExternal`, and probe connection status. Credentials stay in the OS/CLI credential store.

The integration catalog contains exactly GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. Do not add placeholders or app-owned provider-token fields, and never persist provider credentials in SQLite. Raw PTY output is not guaranteed to be sanitized; do not describe it as redacted.

## Architecture boundaries

- Go local authority: `backend/runtimes/core` and retained domains only.
- Desktop: `apps/desktop` with thin Electron Main, typed preload bridge, and Solid renderer.
- Features: `packages/features/<feature>`.
- Shared frontend: `packages/frontend`.
- Contracts: `schemas` to generated Go code in `backend/platform/gen` and TypeScript in `packages/contracts`.
- Renderer access: `window.polterware` only; no raw Electron or Node.js.
- The core uses a technical bearer session token without an account directory.
- There is one clean SQLite baseline and no compatibility requirement for earlier databases.

## Product navigation

- Architecture: Canvas, Components, Views
- Resources: Inventory, Nodes, Processes
- Knowledge: Docs, ADRs, Runbooks, Specs
  - Specs hold one Constitution per project (`constitution`) and per-feature Spec / Plan / Tasks / Clarify / Checklist items linked by `featureKey`. Mark ready is blocked while Checklist still has unchecked `- [ ]` lines.
- Operations: Runs, Workflows, Schedules
- Terminal: local and SSH interactive sessions
- Integrations: Connected, Catalog
- Settings: General, Environments, Shortcuts, Agents, Advanced

Keep project and environment switchers, Commander, history controls, Settings, and quick Terminal access. Do not introduce additional product surfaces without an approved specification update.

## UI law

`DESIGN.md` and `@polterware/frontend-design-system` are the visual source of truth. Product features must use shared `Input`, `Textarea`, `Select`, `Field`, `Button`, `Card`, `Surface`, Ark-backed overlays, `PageToolbar`, `FilterChip`, `DataTable`, `DitherIconTile`, `StatusChip`, and the global `DetailPanel` where applicable.

Do not hand-roll shared control chrome, menus, dialogs, operational tables, or entity panels. Use only radius and color tokens. Structural surfaces and text are opaque; alpha is limited to shared border, shadow, scrim, and transition tokens. Hugeicons Free Stroke Rounded is the only product icon set.

## Development judgments

`pnpm jev` is a local development aid. It reads `TYPESAFE_API_KEY` from the repository `.env` and calls TypeSafe Jev. Do not add this call to the Desktop, the renderer, or `polterware-core`, and do not print the key.

Use it when a closed semantic judgment avoids a wide search or a wrong implementation:

- `pnpm jev triage "<task>"` before opening files, when the task does not already name a file or package.
- `pnpm jev pick "<task>" -- <candidate...>` when search returned several plausible paths and one starting point is enough.
- `pnpm jev review` once, on the finished diff, when the change touches the renderer boundary, integrations, product copy, Knowledge scope, or shared controls.
- `pnpm jev copy "<text>"` when new user-facing copy might describe planned behavior as shipped.

Skip it when the change is already located in a known file, and when typecheck, design lint, Buf, or `go vet` already decides the question.

`hold` exits 2 and stops that step for Erick. `review` means inspect that line before continuing. `clear` means proceed.

## Verification

After relevant changes, use static checks allowed by the task: typecheck, `pnpm design:lint`, `buf lint`, `go vet ./...`, and `git diff --check`. Maintain risk-proportional tests, but do not run them without explicit permission.

Previous specifications live under `docs/future/` and are non-normative.
`docs/spec.md` defines implemented behavior. `docs/product-vision.md` defines product direction; never present its planned PPM, agent, spec-gate, Web App, or Cloud capabilities as already shipped.
