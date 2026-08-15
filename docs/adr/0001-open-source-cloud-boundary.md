# ADR 0001: Open-source and Cloud boundary

- Status: Accepted
- Date: 2026-08-15

## Decision

1. Public source may include hosted-only UI.
2. Local and self-hosted distributions hide unavailable routes.
3. The hosted control plane is private.
4. Project data contracts and import/export formats are public.
5. Cloud subscriptions, invoices, metering, tenant provisioning, internal administration, and production infrastructure are not part of the self-hosted product.
6. One public product monorepo and one private Cloud monorepo are the only active repositories.

## Consequences

The public product monorepo can share product surfaces, contracts, and SDKs without exposing hosted control-plane implementation or production infrastructure. Route availability follows backend capabilities, while backend authorization remains the authority for operations.

The current first version remains local and fixture-backed. This decision does not claim that self-hosted or Cloud functionality exists today.
