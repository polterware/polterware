# ADR 0002: Single Desktop and optional Cloud connection

- Status: Accepted; amended by [ADR 0003](0003-web-repository-boundary.md)
- Date: 2026-08-15

## Context

Polterware is local-first and distributed as a Desktop application, while the
planned commercial product is a managed Cloud service with a Web interface.
Splitting the Desktop into reduced and paid binaries would create duplicated
release channels, unclear source-build behavior, and pressure to weaken the
local open-source product.

At the same time, publishing Cloud interface code does not require publishing
the hosted control plane or making the complete Cloud platform self-hostable.

## Decision

1. Polterware has one Desktop product, codebase, and distribution model. There
   is no separate Tiny or Pro binary.
2. Polterware Community is the useful local open-source product. It works
   without an account and is not a time-limited Cloud trial.
3. The same Desktop may connect to Polterware Cloud through optional sign-in.
4. Cloud capabilities are supplied by the backend and composed into the
   Desktop only after authentication.
5. Unavailable capabilities are hidden from routes, navigation, Commander, and
   commands instead of being represented by dead pages.
6. The future Polterware Web App is an authenticated hosted product surface.
   Whether it has a free tier, trial, or paid-only access is commercial policy
   rather than an architecture boundary.
7. Administrative workflows for organizations, teams, access, usage, plans,
   and billing belong primarily to the Web App. The Desktop may display
   relevant context and deep-link to Web App management.
8. Hosted frontend source may be public. Identity, authorization, tenants,
   entitlements, billing, metering, provisioning, production infrastructure,
   and hosted data operations remain in the private Cloud backend.
9. Self-hosted installations expose only capabilities implemented by their
   backend. Polterware does not promise Cloud parity for self-hosting.
10. Frontend flags and client-side capability state never authorize hosted
    operations.

## Consequences

The Desktop becomes the entry point for both independent local use and an
optional Cloud relationship. Users can download or build a genuinely useful
open-source product, then connect that installation when they need managed
synchronization, backups, continuity, or collaboration.

The team maintains one Desktop experience instead of edition-specific forks.
Cloud UI can be shared publicly, while the managed service retains its
commercial and security boundary in the private control plane.

Billing is not treated as a Desktop feature gate. Plans determine hosted
entitlements on the server, and account administration can evolve on the Web
App without fragmenting the Desktop distribution.

## Current scope

This ADR defines the intended product and deployment boundary. The active first
version remains local and SQLite-backed through `polterware-core`; authentication,
Cloud connection, self-hosting, managed backups, organizations, teams, and
billing are not claimed as implemented. Local bounded integration sync is not
the Cloud synchronization capability described by this ADR.
