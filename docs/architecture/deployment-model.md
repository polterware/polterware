# Deployment Model

## Product model

Polterware has one Desktop product and one Desktop codebase. It does not ship a
separate Tiny, Community, or Pro binary.

The Desktop must remain useful without an account:

- **Polterware Community** is the local, open-source product. It does not
  require sign-in and is not a time-limited Cloud trial.
- **Polterware Cloud** is the managed service. An optional sign-in connects the
  same Desktop installation to hosted projects and capabilities.
- **Polterware Web** is an authenticated Cloud surface for hosted projects and
  administration. Its availability and pricing are separate decisions: a
  future free tier or trial does not change its Cloud deployment model.
- **Polterware Self-hosted** exposes only the public capabilities that an
  installation actually implements. It does not imply parity with Polterware
  Cloud.

Cloud is an addition to the local product, not an unlock for local features.
The commercial value comes from the managed service: collaboration,
synchronization, managed backups, hosted project continuity, and operated
infrastructure.

## Public contract

```ts
export type DeploymentMode = "local" | "self-hosted" | "cloud";

export type PlatformCapabilities = {
  organizations: boolean;
  teams: boolean;
  billing: boolean;
  usage: boolean;
  managedBackups: boolean;
  cloudSync: boolean;
};
```

Build and runtime configuration select the deployment composition.
Backend capabilities select the available product surfaces.
Backend authorization decides whether an operation is allowed.

The deployment mode is an initial composition hint. It is never proof of a
subscription or authorization.

## Modes

The only active first-version mode is `local`, which uses local SQLite persistence and `polterware-core` for execution (not fixture-only).

`self-hosted` exposes only the capabilities implemented by that installation.

`cloud` may expose Organizations, Teams, Access, Billing, Usage, managed backups, and synchronization. Every Cloud request must validate authentication, organization membership, permission, tenant, plan, entitlement, and billing state on the server.

These modes define a public boundary; they do not claim that self-hosted or Cloud functionality exists today.

## Desktop behavior

The Desktop uses progressive Cloud connection instead of separate editions:

1. It starts as a local product without requiring an account.
2. It presents one quiet entry point to connect to Polterware Cloud.
3. After authentication, it obtains capabilities from the Cloud backend.
4. It composes navigation, commands, project selection, and data access from
   those capabilities.
5. Signing out removes Cloud composition and returns the application to its
   local behavior without affecting local project access.

Unavailable Cloud functionality must not appear as a collection of dead pages
or repeated upgrade placeholders. Routes, sidebar items, Commander entries,
and commands are absent when their required capability is unavailable. Direct
navigation to an unavailable route resolves to a safe available route.

The Desktop may expose Cloud project selection, synchronization, managed
backup state, and collaboration context. Organization administration, team
administration, access management, usage, plans, and billing primarily belong
to Polterware Web. The Desktop may deep-link to the corresponding authenticated
Web surface instead of duplicating its management workflows.

## Web behavior

Polterware Web is always backed by the hosted control plane and therefore
requires authentication. It is not the browser packaging of the local Desktop
and is not promised as a standalone self-hosted application.

The Web frontend and hosted-only interface packages may remain in the public
monorepo. Public interface code does not grant access to Polterware Cloud. The
private backend remains authoritative for identity, tenants, membership,
permissions, plans, entitlements, billing state, metering, provisioning, and
managed infrastructure.

## Distribution rules

- Official Desktop releases and source builds share the same product boundary.
- There is no reduced Tiny download and no separate Pro feature fork.
- Local functionality must not require a Cloud entitlement.
- Cloud functionality requires a compatible Cloud service, authentication,
  and server-issued capabilities.
- Changing frontend code, a build flag, local storage, or a capability response
  in the client must not grant a hosted operation.
- Plan names and prices are commercial policy. They must not be encoded as
  deployment modes.

## Capability authority

Capabilities control composition, not security. For every hosted request, the
Cloud backend must independently validate:

- authentication;
- organization and team membership;
- permission and tenant scope;
- plan and entitlements;
- billing state when relevant;
- ownership of the requested hosted resource.

The client must treat a denied or expired capability as a normal state and
remove or disable the affected Cloud composition without degrading the local
product.
