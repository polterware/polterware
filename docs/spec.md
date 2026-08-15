# Polterware — First Version Specification

Status: Active  
Scope: Local fixture-backed desktop preview and reduced future core

## Objective

Deliver a maintainable first version focused on understanding and rehearsing maintenance work for local software projects. The Desktop is complete as a visual product slice while runtime integration remains deliberately deferred.

## Deployment boundary

Only the `local` deployment mode is active in this first version, and its behavior is fixture-backed. Self-hosted and Cloud functionality is not implemented today. The public deployment contract and capability boundary are defined in [deployment-model.md](architecture/deployment-model.md); repository ownership is defined in [repository-map.md](architecture/repository-map.md) and [ADR 0001](adr/0001-open-source-cloud-boundary.md); the single-Desktop distribution and optional Cloud connection are defined in [ADR 0002](adr/0002-single-desktop-and-cloud-connection.md).

## Information architecture

Architecture / Canvas is the initial route and the destination after entering or changing a project.

| Surface | Tabs |
| --- | --- |
| Architecture | Canvas, Components, Views |
| Resources | Inventory, Nodes, Processes |
| Knowledge | Docs, Runbooks |
| Operations | Runs, Workflows, Schedules |
| Terminal | Dynamic visual sessions |
| Integrations | Connected, Catalog |
| Settings | General, Environments, Advanced |

The sidebar contains the six product surfaces. Settings is fixed in its footer. The title bar retains local project and environment switchers, Commander, history controls, and quick Terminal access.

## Local project model

The application supports several local projects. A project has an identifier, name, directory, and environments. Project creation asks only for name, directory, and environments. All project and environment changes live in renderer memory for the current session.

## Data honesty

Every Desktop surface uses fixtures. No renderer path starts, discovers, supervises, or queries the Go core. Any action that resembles execution must say that it is a preview or fixture and must only update in-memory state.

Terminal sessions are visual. They do not create a PTY, shell, SSH session, or child process. Output is synthetic and must not contain real secrets.

## Integration CLI preview

The single provider registry stores provider id, label, executable name, authentication commands, mock transcript, and visual state. The only allowed visual states are `cli-unavailable`, `ready`, `authenticating`, `connected`, and `failed`.

The catalog contains exactly GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. Every catalog row opens the shared terminal-guided preview. Connected detail contains only Overview, Resources, CLI, and Sync.

No flow detects a binary, starts a command, opens a browser, contacts a provider, accepts credentials, or persists authentication material.

## Reduced core

The repository retains one Go runtime, `polterware-core`, for later evolution. Its active domains are Projects, Architecture, Resources, Knowledge, Operations, Workflows, Integrations, Process, Logs, Health, and Maintenance.

Operations and workflow steps are direct local models. Core requests use one technical bearer session token without an account directory. The database is initialized from a clean SQLite baseline with no compatibility guarantee for earlier databases.

The Desktop package excludes the core binary and has no generated TypeScript API client.

## Acceptance

- The initial route is Architecture / Canvas.
- The sidebar exposes exactly six surfaces plus footer Settings.
- Knowledge exposes only Docs and Runbooks.
- Operations exposes Runs, Workflows, and Schedules with direct local states.
- The provider catalog contains exactly the nine specified providers.
- Every provider opens the shared visual terminal login preview.
- The Desktop code graph contains no core client or supervisor.
- Active Protobuf and SQLite schemas contain only retained domains.
- Product actions never claim real execution or authentication.
- Static validation passes: typecheck, design lint, Buf lint, Go vet, and `git diff --check`.

Previous specifications and the broader platform direction are archived under `docs/future/` and are non-normative.
