# Repository Map

Polterware currently has three active product repositories and one historical archive.

| Repository | Responsibility |
| --- | --- |
| `polterware/polterware` | Public product repository: Desktop, local Go runtime, shared product packages, schemas, documentation, and local/self-hosted foundations |
| `polterware/polterware-web` | Polterware Website today and the future Polterware Web App frontend |
| `polterware/cloud` | Private hosted control plane, tenant authority, managed infrastructure, and private operations |
| `polterware/product-archive` | Historical repository retained for provenance; not an active product source |

Repository visibility and commercial packaging are separate policy decisions from ownership. Public frontend code never grants access to hosted operations.

## Product repository

`polterware/polterware` owns the distributable local product and contracts that may also support self-hosted or hosted compositions.

| Area | Ownership |
| --- | --- |
| Desktop | Electron Main, typed preload bridge, Solid renderer, and current product surfaces |
| Local runtime | `polterware-core`, `polterware` CLI, SQLite persistence, local execution, and provider CLI adapters |
| Shared packages | Product features, frontend design system, platform utilities, state, and contracts |
| Documentation | Current product specification, product vision, ADRs, architecture, security, and contribution guidance |
| Public contracts | Protobuf schemas, generated clients, SDK boundaries, and future import/export contracts |

The repository owns one Desktop product rather than separate Tiny, Community, or Pro binaries. Hosted capabilities may compose into the Desktop only when a compatible backend authorizes them.

## Web repository

`polterware/polterware-web` owns both browser-facing surfaces:

- **Polterware Website** — the current public, unauthenticated landing page;
- **Polterware Web App** — the future authenticated frontend for hosted projects and administration.

The Website exists today. The Web App is product direction, not implemented behavior. Both remain in the active web repository; there is no planned migration to `apps/website` and no planned archival of `polterware-web`.

Frontend code does not own hosted identity, authorization, tenant membership, entitlements, billing state, metering, provisioning, or production data operations. Those remain server-authoritative in `polterware/cloud`.

See [web-repository-boundary.md](web-repository-boundary.md) and [ADR 0003](../adr/0003-web-repository-boundary.md).

## Cloud repository

`polterware/cloud` owns the private hosted control plane and operational material.

| Area | Ownership |
| --- | --- |
| Hosted services | Identity, accounts, organizations, teams, access, project registry, synchronization, billing, metering, entitlements, notifications, and tenant provisioning |
| Production operations | Managed infrastructure, deployment operations, private runbooks, internal administration, and incident material |

Cloud subscriptions, invoices, metering, tenant provisioning, internal administration, and production infrastructure are not part of the local or self-hosted product.

## Archive

`polterware/product-archive` preserves historical source, branches, tags, releases, issues, and pull requests. It must not be used as a current implementation or documentation source without a new accepted specification.
