# Deployment Model

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

## Modes

The only active first-version mode is `local`, and it is fixture-backed.

`self-hosted` exposes only the capabilities implemented by that installation.

`cloud` may expose Organizations, Teams, Access, Billing, Usage, managed backups, and synchronization. Every Cloud request must validate authentication, organization membership, permission, tenant, plan, entitlement, and billing state on the server.

These modes define a public boundary; they do not claim that self-hosted or Cloud functionality exists today.
