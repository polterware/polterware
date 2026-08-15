# Polterware

Polterware is a local desktop workspace for understanding and maintaining software systems. The first version keeps system architecture, local resources, operational knowledge, visual operations, terminal sessions, CLI-oriented integrations, and project settings in one Electron application.

## Current product

The Desktop opens at **Architecture / Canvas** and supports multiple local projects and environments. Its project navigation is:

- Architecture — Canvas, Components, Views
- Resources — Inventory, Nodes, Processes
- Knowledge — Docs, Runbooks
- Operations — Runs, Workflows, Schedules
- Terminal — visual session page and drawer
- Integrations — Connected, Catalog
- Settings — General, Environments, Advanced

Desktop data is demonstrative and held in the current session. It does not launch, query, or supervise the Go core. Integration setup renders a guided terminal transcript but never starts a command, PTY, browser, or authentication flow.

The only active deployment mode is local and fixture-backed. The public architecture defines possible `self-hosted` and `cloud` modes, but neither is implemented today. See [the deployment model](docs/architecture/deployment-model.md) and [repository map](docs/architecture/repository-map.md).

## Open-source and Cloud

This repository contains the public product, including local and self-hosted foundations, public contracts, and UI that may be used by the hosted product. Unavailable surfaces are hidden according to backend-provided capabilities.

Polterware Cloud is a managed service built on a separate private control plane. Its identity, organization, billing, metering, tenant provisioning, internal administration, production infrastructure, and operational runbooks are not part of this repository. Frontend flags never grant authorization.

## Repository

- `apps/desktop` — Electron Main, preload bridge, and Solid renderer
- `packages/features` — retained product surfaces
- `packages/frontend` — shared design system, canvas, and state
- `backend` — reduced Go core retained for future local runtime work
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

Static checks:

```bash
pnpm typecheck
pnpm design:lint
cd schemas && buf lint
cd ../backend && go vet ./...
git diff --check
```

The standalone core can be run explicitly for backend development:

```bash
pnpm dev:core
```

The Desktop remains fixture-only until a later version deliberately reconnects it to the core.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [TRADEMARKS.md](TRADEMARKS.md), and [SECURITY.md](SECURITY.md).

The selective migration policy for hosted-only interface is documented in [docs/architecture/hosted-ui-porting.md](docs/architecture/hosted-ui-porting.md).
