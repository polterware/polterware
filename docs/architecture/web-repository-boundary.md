# Web Repository Boundary

## Decision

`polterware/polterware-web` remains an active repository. It owns the current Polterware Website and will also own the future Polterware Web App frontend.

There is no planned migration to `apps/website` and no planned archival of the web repository.

## Current surface: Polterware Website

The Website is public and unauthenticated. Its current responsibility is to explain the local Polterware Desktop and the product direction without presenting planned capabilities as shipped.

The current site has one public manifesto route. It does not provide authentication, hosted project access, account administration, downloads, a waitlist, or a contact workflow.

## Future surface: Polterware Web App

The Web App will be an authenticated surface for hosted projects, collaboration, continuity, and administration. It may add authenticated routes and application composition to the same repository after accepted feature and architecture specifications exist.

The existence of the Website does not imply that the Web App is available. Public copy, route documentation, and metadata must preserve that distinction.

## Hosted authority

The web frontend may render server-provided capabilities, but it never authorizes hosted operations. `polterware/cloud` remains authoritative for:

- identity and sessions;
- organizations, teams, membership, and permissions;
- tenant and project scope;
- plans, entitlements, billing state, and usage;
- synchronization, provisioning, and managed infrastructure;
- hosted project reads and mutations.

Changing client code, route visibility, local storage, or a capability response cannot grant a hosted operation.

## Shared product boundaries

The Web App should reuse public contracts and compatible shared interface packages deliberately. Sharing does not require moving the Website into the Desktop repository or duplicating Cloud authority in a public frontend package.

Desktop-specific Electron behavior remains in `polterware/polterware`. Browser-specific routing and hosted frontend composition remain in `polterware/polterware-web`. Private service adapters and production operations remain in `polterware/cloud`.

## Naming

- **Polterware Website** means the public landing and future public marketing/documentation routes.
- **Polterware Web App** means the future authenticated hosted application.
- **Polterware Cloud** means the managed service and private hosted authority.

Use “Polterware Web” only as an informal umbrella for the web repository, never as the precise name of a product surface.

## Translation protection

Public and authenticated roots must protect the React DOM from automatic browser translation:

```html
<html lang="en" translate="no">
<meta name="google" content="notranslate" />
```
