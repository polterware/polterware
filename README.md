# Polterware

Polterware is a local desktop workspace for understanding and maintaining software systems. The first version keeps system architecture, local resources, operational knowledge, visual operations, terminal sessions, CLI-oriented integrations, and project settings in one Electron application.

## Current product

The Desktop opens at **Architecture / Canvas** and supports multiple local projects and environments. Its project navigation is:

- Architecture — Canvas, Components, Views
- Resources — Inventory, Nodes, Processes
- Knowledge — Docs, Runbooks
- Operations — Runs, Workflows, Schedules
- Terminal — live local sessions (page and drawer)
- Integrations — Connected, Catalog
- Settings — General, Environments, Advanced

The Electron Main process starts `polterware-core`. The renderer reads and writes only through `window.polterware`. Project data persists in local SQLite (projects, environments, architecture layout, resources and nodes, knowledge, operations, workflows, schedules, and integration connections).

Terminal opens interactive local PTY sessions on the Terminal page and in the quick drawer; Expand hands the live session off to the full page. Integrations detect CLI binaries, run provider login in an in-app PTY from the setup wizard, and probe connection status. Credentials stay in the OS/CLI credential store; the app does not request, display, or persist provider tokens in SQLite.

Operations run locally through the core. Active schedules are evaluated by the core runner while the core is running; missed times while the core was offline are skipped.

Integration Sync imports a bounded inventory through the local authenticated CLI (replace-on-success; failures leave prior resources). Processes support real Start/Stop on the local host only. Sync does not auto-create Architecture inventory rows, crawl full cloud graphs, or run on a background ticker.

The only active deployment mode is **local**. The public architecture defines possible `self-hosted` and `cloud` modes, but neither is implemented today. See [the deployment model](docs/architecture/deployment-model.md) and [repository map](docs/architecture/repository-map.md).

## Open-source and Cloud

This repository contains the public product, including local and self-hosted foundations, public contracts, and UI that may be used by the hosted product. Unavailable surfaces are hidden according to backend-provided capabilities.

Polterware Cloud is a managed service built on a separate private control plane. Its identity, organization, billing, metering, tenant provisioning, internal administration, production infrastructure, and operational runbooks are not part of this repository. Frontend flags never grant authorization.

Polterware ships one Desktop product rather than separate Tiny and Pro
editions. The local Community experience works without an account; optional
sign-in connects the same Desktop to server-authorized Cloud capabilities.
Polterware Web is an authenticated hosted surface, while pricing and free-tier
policy remain independent commercial decisions. See
[ADR 0002](docs/adr/0002-single-desktop-and-cloud-connection.md).

## Repository

- `apps/desktop` — Electron Main, preload bridge, and Solid renderer
- `packages/features` — retained product surfaces
- `packages/frontend` — shared design system, canvas, and state
- `backend` — local Go runtime (`polterware-core`)
- `schemas` — Protobuf contracts and generated Go code
- `docs/spec.md` — normative product scope
- `docs/architecture` — deployment and repository boundaries
- `docs/adr` — architecture decisions
- `docs/future` — archived platform vision and previous specifications

## Development

```bash
pnpm install
pnpm dev:desktop
```

`pnpm dev:desktop` starts the Desktop together with `polterware-core`. For standalone core development:

```bash
pnpm dev:core
```

Static checks:

```bash
pnpm typecheck
pnpm design:lint
cd schemas && buf lint
cd ../backend && go vet ./...
git diff --check
```

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [TRADEMARKS.md](TRADEMARKS.md), and [SECURITY.md](SECURITY.md).

The selective migration policy for hosted-only interface is documented in [docs/architecture/hosted-ui-porting.md](docs/architecture/hosted-ui-porting.md).
