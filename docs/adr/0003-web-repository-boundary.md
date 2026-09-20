# ADR 0003: Website and Web App repository boundary

- Status: Accepted
- Date: 2026-09-19
- Amends: ADR 0001 and ADR 0002

## Context

The active `polterware/polterware-web` repository now contains the public Polterware landing page. Earlier documentation treated it as a legacy studio website that would be migrated into `polterware/polterware` and archived. The product direction also needs an authenticated browser application backed by Polterware Cloud.

Using one active web repository for the public Website and future authenticated Web App keeps browser deployment and routing together without moving hosted authority into frontend code.

## Decision

1. `polterware/polterware-web` remains an active product repository.
2. The public unauthenticated surface is named **Polterware Website**.
3. The future authenticated hosted surface is named **Polterware Web App**.
4. Both surfaces are owned by `polterware/polterware-web`.
5. There is no planned migration to `apps/website` and no planned archival of the web repository.
6. The Web App is not implemented merely because the Website exists in its future repository.
7. `polterware/cloud` remains authoritative for identity, tenants, permissions, entitlements, billing, provisioning, managed infrastructure, and hosted project operations.
8. Public or shared frontend code never grants hosted authorization.

## Consequences

Polterware has three active product repositories: the Desktop/local product repository, the web frontend repository, and the private Cloud repository. Documentation and public copy must distinguish the Website from the Web App and must describe the latter as future work until implemented.

ADR 0001 remains valid for the open-source and private Cloud boundary except for its two-repository statement. ADR 0002 remains valid for the single Desktop and optional Cloud relationship, with “Polterware Web” clarified as the future Polterware Web App.
