# Polterware — First Version Specification

Status: Active  
Scope: Local SQLite-backed Desktop with Go core execution authority

## Objective

Deliver a maintainable first version focused on understanding and rehearsing maintenance work for local software projects. The Desktop reads and writes through `polterware-core` and may execute local operations, interactive terminals, and CLI-backed integration probes on the host machine.

## Deployment boundary

Only the `local` deployment mode is active in this first version. Self-hosted and Cloud functionality is not implemented today. The public deployment contract and capability boundary are defined in [deployment-model.md](architecture/deployment-model.md); repository ownership is defined in [repository-map.md](architecture/repository-map.md) and [ADR 0001](adr/0001-open-source-cloud-boundary.md); the single-Desktop distribution and optional Cloud connection are defined in [ADR 0002](adr/0002-single-desktop-and-cloud-connection.md).

## Information architecture

Architecture / Canvas is the initial route and the destination after entering or changing a project.

| Surface | Tabs |
| --- | --- |
| Architecture | Canvas, Components, Views |
| Resources | Inventory, Nodes, Processes |
| Knowledge | Docs, Runbooks |
| Operations | Runs, Workflows, Schedules |
| Terminal | Local interactive sessions |
| Integrations | Connected, Catalog |
| Settings | General, Environments, Advanced |

The sidebar contains the six product surfaces. Settings is fixed in its footer. The title bar retains local project and environment switchers, Commander, history controls, and quick Terminal access.

## Local project model

The application supports several local projects. A project has an identifier, name, directory, environments, and an active/archived status. Project creation asks only for name, directory, and environments. Mutations persist in the local SQLite store through `polterware-core`.

## Execution authority

The Electron Main process starts `polterware-core`. The renderer reads and writes through `window.polterware` only. Go core owns local execution:

- Terminal sessions create a local PTY/shell child process and stream live output. They must not contain real secrets in persisted transcripts.
- Operations and workflow steps run locally via `os/exec`, transitioning queued runs through running to terminal states while appending logs.
- Runbook Run creates local operations through the same runner.

## Integration CLI auth

The provider registry stores provider id, label, executable name, authentication commands, and connection status. Allowed states are `cli-unavailable`, `ready`, `authenticating`, `connected`, and `failed`.

The catalog contains exactly GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. Flows may detect the CLI binary, open a provider login browser or CLI hand-off, and probe connection status. Credentials remain in the OS/CLI credential store. No flow requests, displays, or persists provider tokens in the app or SQLite.

Connected detail contains Overview, Resources, CLI, and Sync.

## Reduced core

The repository retains one Go runtime, `polterware-core`. Its active domains are Projects, Architecture, Resources, Knowledge, Operations, Workflows, Integrations, Process, Terminal, Logs, Health, and Maintenance.

Operations and workflow steps are direct local models. Core requests use one technical bearer session token without an account directory. The database is initialized from a clean SQLite baseline with no compatibility guarantee for earlier databases.

## Acceptance

- The initial route is Architecture / Canvas.
- The sidebar exposes exactly six surfaces plus footer Settings.
- Knowledge exposes only Docs and Runbooks.
- Operations exposes Runs, Workflows, and Schedules with local execution states.
- Terminal opens interactive local sessions through core-owned PTY.
- The provider catalog contains exactly the nine specified providers.
- Provider connect may probe CLI availability and authentication status without storing tokens.
- Product actions that execute locally report real status and logs.
- Static validation passes: typecheck, design lint, Buf lint, Go vet, and `git diff --check`.

Previous specifications and the broader platform direction are archived under `docs/future/` and are non-normative.
