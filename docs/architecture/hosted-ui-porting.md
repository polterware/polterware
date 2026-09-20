# Hosted UI Porting Boundary

## Decision

Hosted composition that belongs inside the Desktop may live in `polterware/polterware`. Browser-specific hosted interface belongs in the future Polterware Web App inside `polterware/polterware-web`. Production adapters, the hosted control plane, commercial services, tenant operations, and production infrastructure remain private in `polterware/cloud`.

The historical `refactor/new` branch is preserved only in `polterware/product-archive`. It must not be merged wholesale into the clean repository. UI is ported as independently specified feature units after its data contracts and states are approved.

## Candidate inventory

| Historical source | Product destination | Gate |
| --- | --- | --- |
| `create-organization.tsx`, `organizations-index.tsx`, `organization-home.tsx`, `organization-settings.tsx` | `packages/features/organizations` | `organizations` |
| `account-page.tsx` | `packages/features/account` | Hosted authenticated composition; ported with the Organizations unit |
| `organization-team.tsx` and organization membership details | `packages/features/organizations` | `teams` |
| `packages/features/access/src/*` | `packages/features/access` | `teams` plus server-provided permissions |
| `organization-billing.tsx` | `packages/features/billing` | `billing` |
| Future usage surfaces | `packages/features/billing` or a separately approved usage package | `usage` |
| Future managed backup surfaces | Separately approved feature package | `managedBackups` |
| Future synchronization surfaces | Separately approved feature package | `cloudSync` |

Desktop destination paths describe ownership, not implemented packages. A Web App implementation may instead define browser-specific destinations in `polterware-web`; empty packages are not created before an approved feature specification.

## What may be reused

- information architecture and visual composition;
- shared design-system primitives and established layout patterns;
- accessibility behavior that still matches the current product;
- domain-neutral presentation helpers.

## What must be replaced

- fixture mutations that imply a successful hosted operation;
- organization, membership, access, billing, or usage mock data presented as real;
- local availability stubs such as `Billing unavailable`;
- direct dependencies on historical workspace mocks;
- client-side checks presented as authorization.

Every port must define typed loading, empty, unavailable, permission-denied, error, and backend-driven ready states as applicable.

## Availability and authority

Routes, sidebar items, Commander items, and data requests declare the same required capability. If the backend does not provide it, the surface is absent and direct navigation resolves to the safe local route.

Visibility is composition, not authorization. A Cloud request still validates authentication, organization membership, permission, tenant, plan, entitlement, and billing state on the server.

## Port sequence

1. Organizations and Account.
2. Teams and Access.
3. Billing and Usage.
4. Managed backups and synchronization only after their own specifications.

Each unit is reviewed and validated independently. The active first version remains local and SQLite-backed through `polterware-core`; none of these hosted surfaces is active today.
