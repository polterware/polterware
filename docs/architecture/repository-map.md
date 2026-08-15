# Repository Map

| Repository | Visibility | Responsibility |
| --- | --- | --- |
| `polterware/polterware` | Public after audit | Product monorepo: Desktop, web product, website, docs, public frontend, local/self-hosted runtimes, contracts, SDKs, and hosted-only UI source |
| `polterware/cloud` | Private | Hosted Platform API, accounts, organizations, teams, billing, metering, tenant provisioning, internal administration, and production infrastructure |
| `polterware/product-archive` | Private and archived after verification | Exact historical repository, including branches, tags, releases, issues, and pull requests |
| `polterware/polterware-web` | Private until website migration; archived afterward | Existing website source retained until `apps/website` is accepted in the public monorepo |

Only `polterware/polterware` and `polterware/cloud` are active repositories. The archive and legacy website repository remain private until their stated archival conditions are met.

## Public product monorepo ownership

`polterware/polterware` owns the product source that can be distributed locally or self-hosted. It contains public documentation; no standalone documentation repository is planned.

| Area | Public ownership |
| --- | --- |
| Apps | Desktop, future web product, website, and product documentation |
| Packages | Product features, including hosted-only UI; frontend design system, platform utilities, and state; public contracts and SDKs |
| Local and self-hosted runtime | Go local/self-hosted runtimes, public schemas, project data contracts, and import/export formats |
| Documentation | Product, architecture, contract, and self-hosted documentation that belongs with the public product source |

Hosted-only UI source may be public, but local and self-hosted distributions hide routes for capabilities that their backend does not provide.

The public monorepo owns one Desktop application rather than separate Tiny,
Community, and Pro editions. It also owns the public Web frontend and Cloud
client integration. The private Cloud repository owns the hosted authority
that makes those surfaces operational.

Historical hosted UI is ported selectively under [hosted-ui-porting.md](hosted-ui-porting.md); the archive branch is never merged wholesale.

The legacy studio website remains separate until the product-only migration gate in [website-migration.md](website-migration.md) is satisfied.

## Private Cloud monorepo ownership

`polterware/cloud` owns the hosted control plane and its private operational material.

| Area | Private ownership |
| --- | --- |
| Hosted services | Platform API; identity and accounts; organizations, teams, and access; project registry, synchronization, and tenant provisioning; billing, metering, entitlements, and notifications; internal administration |
| Production operations | Production infrastructure, deployment operations, private runbooks, and operational documentation |

Private operational documentation lives with the Cloud repository. Cloud subscriptions, invoices, metering, tenant provisioning, internal administration, and production infrastructure are not part of the self-hosted product.

The product and distribution decision is recorded in
[ADR 0002](../adr/0002-single-desktop-and-cloud-connection.md). Pricing tiers
do not create additional repositories or Desktop forks.
