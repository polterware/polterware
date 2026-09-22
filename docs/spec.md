# Polterware — First Version Specification

Status: Active  
Scope: Local SQLite-backed Desktop with Go core execution authority

## Current scope versus product direction

This document is the source of truth for behavior implemented in the current product. [`product-vision.md`](product-vision.md) defines the intended Polterware Project Model, agent context, structured spec-driven development, Understanding Gates, Web App, and Cloud direction. Those capabilities are not part of this specification unless they are explicitly added here by a later accepted change.

Historical specifications under `docs/future/` describe earlier states or superseded plans and are not evidence of current behavior.

## Objective

Deliver a maintainable first version focused on understanding, documenting, and performing bounded maintenance work for local software projects. The Desktop reads and writes through `polterware-core` and may execute local operations, interactive terminals, and CLI-backed integration probes on the host machine.

## Deployment boundary

Only the `local` deployment mode is active in this first version. Self-hosted and Cloud functionality is not implemented today. The public deployment contract and capability boundary are defined in [deployment-model.md](architecture/deployment-model.md); repository ownership is defined in [repository-map.md](architecture/repository-map.md) and [ADR 0001](adr/0001-open-source-cloud-boundary.md); the single-Desktop distribution and optional Cloud connection are defined in [ADR 0002](adr/0002-single-desktop-and-cloud-connection.md).

## Information architecture

Architecture / Canvas is the initial route and the destination after entering or changing a project.

| Surface | Tabs |
| --- | --- |
| Architecture | Canvas, Components, Views |
| Resources | Inventory, Nodes, Processes |
| Knowledge | Docs, ADRs, Runbooks, Specs |
| Operations | Runs, Workflows, Schedules |
| Terminal | Local and SSH interactive sessions |
| Integrations | Connected, Catalog |
| Settings | General, Environments, Shortcuts, Agents, Advanced |

The sidebar contains the six product surfaces. Settings is fixed in its footer. The title bar retains local project and environment switchers, Commander, history controls, and quick Terminal access.

## Local project model

The application supports several local projects. A project has an identifier, name, directory, environments, and an active/archived status. Project creation asks only for name, directory, and environments. Mutations persist in the local SQLite store through `polterware-core`.

## Minimal Polterware Project Model (PPM)

The core maintains a thin entity registry (`ppm_entities`) that projects architecture canvas nodes, inventory resources, and environments into typed refs with provenance (`truth_class`, `source`, `actor`, `observed_at`). Typed relationships (`ppm_relationships`) support predicates `DOCUMENTS`, `OPERATES`, `RELATED_TO`, and `DEFINES`. The canvas `architecture_layouts.body_json` remains the visual graph SSOT; the registry does not duplicate canvas geometry. Sync runs when layouts, resources, or environments are saved and via `SyncProjectModel`.

## Knowledge Specs

Knowledge Specs store product features as five linked knowledge items (`spec`, `plan`, `tasks`, `clarify`, `checklist`) that share a `featureKey` in `extra_json`. Creating a feature writes all five bodies from English templates. The Specs table lists one row per feature. The detail panel switches among Spec, Plan, Tasks, Clarify, and Checklist for that feature. Allowed feature statuses are `draft` and `ready`. Marking a feature ready is blocked while the Checklist body still contains unchecked Markdown items (`- [ ]`). Clarify holds open questions and does not block ready.

Each project may also store one Constitution knowledge item (`kind = constitution`, no `featureKey`). The Specs tab shows that singleton above the feature table; creating it seeds an English principles template titled "Project constitution". Archiving removes the singleton so Specs returns to the empty constitution CTA. Specs do not run Spec Kit CLI and do not bind to `.specify/` on disk.

## Knowledge links, ADRs, and search

Knowledge items may link to PPM entities through `knowledge_entity_links` with the same predicates as relationships. Docs, ADRs, and Runbooks expose linked entities in the detail panel. Architecture node detail shows knowledge linked to the matching component entity.

Knowledge kinds include `doc`, `adr`, `runbook`, `spec`, `plan`, `tasks`, `clarify`, `checklist`, and `constitution`. ADRs are listed on the Knowledge ADRs tab.

`SearchKnowledge` provides exact id match and SQLite FTS5 full-text search over title, body, summary, and tags within a project. Results carry basic provenance fields (`truth_class`, `source`).

## Create templates

When a Knowledge Docs, ADRs, Runbooks, or Specs list is empty, the Desktop shows local inspiration template cards. The matching create dialog also offers those templates. Choosing a template creates the item immediately with a suggested title and, for Docs, ADRs, and Runbooks, a suggested body or runbook `extra_json`. Spec feature templates only supply a title; Spec, Plan, Tasks, Clarify, and Checklist bodies still come from the English Spec templates. Blank create remains available from the toolbar and create dialog form.

When a Resources Inventory, Nodes, or Processes list is empty, the Desktop shows local inspiration template cards. The matching dialog also offers those templates. Choosing a template creates the resource or node, or starts the process, immediately with suggested fields. Nodes templates create SSH host profiles (alias, user, host, port, optional identity file path). Process templates use short local commands (`sleep`, `echo`, `git status`). Blank create remains available from the toolbar and dialog form.

When Architecture Canvas or Components is empty, the Desktop shows featured component templates (API, Service, PostgreSQL, Redis, Worker). Choosing one places the component immediately. Architecture Views empty shows Overview, Applications, and Data plane templates that create a saved view immediately. The existing Architecture catalog dialog remains the full blank/browse path.

When Operations Runs, Workflows, or Schedules is empty, the Desktop shows inspiration templates. Run templates start a safe local command immediately. Workflow templates create a named draft immediately. Schedule templates create a cadence against the first available workflow when workflows exist; otherwise the empty state asks for a workflow first. Matching create dialogs also offer those templates.

When Settings Environments is empty, the Desktop shows Development, Staging, and Production templates that create the environment immediately. The create dialog also offers those templates.

Integrations do not use content templates; they use the fixed provider catalog. Terminal empty offers action cards to open a local session on this machine or add an SSH host profile.

## Context Compiler and experimental MCP

`CompileContext` assembles a bounded context package from entity resolution, knowledge links, one-hop relationships, and full-text hits under a token budget. Items include `kind`, `id`, `title`, `excerpt`, `truth_class`, `source`, and `path`. There is no embedding model, GraphRAG, or fixed PCTX serialization.

`polterware-core mcp` serves an experimental MCP stdio server over the same local SQLite store. Provisional tool names (`search`, `compile_context`, `get_entity`) are exploration only and are not a product API contract.

Settings → Agents is the Desktop operator surface for this access: start/stop a parked local MCP helper, copy Cursor MCP config, copy or download the Polterware AI skill (markdown that teaches agents how to use those tools), preview `CompileContext` for the current project, and inspect a light local tool-call history (`agent_tool_calls`). History is not a full agent audit trail.

## Execution authority

The Electron Main process starts `polterware-core`. The renderer reads and writes through `window.polterware` only. Go core owns local execution:

- Terminal sessions create a core-owned PTY child process and stream live output. A session may be local (shell on this machine) or remote (system `ssh` to a saved host profile). They must not contain real secrets in persisted transcripts.
- Resources Nodes may store SSH host profiles (`connection_kind = ssh`): alias, user, hostname/address, port, and optional identity file path. Private key material is never persisted in SQLite; authentication uses the OS `ssh-agent`, default keys, or an interactive prompt inside the PTY. A `polterware-node` daemon / mTLS agent is not implemented.
- Operations and workflow steps run locally via `os/exec`, transitioning queued runs through running to terminal states while appending logs.
- Runbook Run creates local operations through the same runner.

## Integration CLI auth

The provider registry stores provider id, label, executable name, authentication commands, and connection status. Allowed states are `cli-unavailable`, `ready`, `authenticating`, `connected`, and `failed`.

The catalog contains exactly GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. Flows may detect the CLI binary, start a fixed provider authentication command in the core-owned PTY, optionally open the provider documentation URL through Electron Main, and probe connection status. Credentials remain in the OS/CLI credential store. No app-owned field requests or persists provider tokens in SQLite. Raw CLI output is shown as terminal output and is not guaranteed to be sanitized.

Connected detail contains Overview, Resources, CLI, and Sync.

## Reduced core

The repository retains one Go runtime, `polterware-core`. Its active domains are Projects, Architecture, Resources, Knowledge, Project Model (PPM), Context, Operations, Workflows, Integrations, Process, Terminal, Logs, Health, and Maintenance.

Operations and workflow steps are direct local models. Core requests use one technical bearer session token without an account directory. The database is initialized from a clean SQLite baseline with no compatibility guarantee for earlier databases.

## Acceptance

- The initial route is Architecture / Canvas.
- The sidebar exposes exactly six surfaces plus footer Settings.
- Knowledge exposes Docs, ADRs, Runbooks, and Specs.
- Specs store one feature as five local knowledge items (`spec`, `plan`, `tasks`, `clarify`, `checklist`) linked by `featureKey` in `extra_json`.
- Specs also support one project Constitution (`kind = constitution`) without a `featureKey`.
- Marking a feature ready requires the Checklist body to have no unchecked `- [ ]` items; Clarify does not gate ready.
- Docs, ADRs, and Runbooks can link to PPM entities; Architecture shows linked knowledge for the selected component.
- `SearchKnowledge` returns exact and FTS hits with provenance fields.
- Knowledge, Resources, Architecture, Operations, and Settings Environments empty states show inspiration template cards that create the item immediately where applicable; create dialogs also offer templates, and blank create remains on the toolbar and form.
- `CompileContext` returns a bounded package with provenance fields.
- `polterware-core mcp` exposes experimental search/compile/get_entity tools over stdio.
- Settings → Agents exposes MCP helper start/stop, Cursor config copy, the Polterware AI skill, CompileContext preview, and local tool-call history.
- Operations exposes Runs, Workflows, and Schedules with local execution states.
- Terminal opens interactive local sessions and SSH sessions to saved host profiles through core-owned PTY.
- Nodes may be SSH connection profiles; private keys are not stored in SQLite.
- The provider catalog contains exactly the nine specified providers.
- Provider connect may probe CLI availability and authentication status without storing tokens.
- Product actions that execute locally report real status and logs.
- Static validation passes: typecheck, design lint, Buf lint, Go vet, and `git diff --check`.

Previous specifications are archived under `docs/future/` and are non-normative. Product direction is defined separately by `docs/product-vision.md`.
