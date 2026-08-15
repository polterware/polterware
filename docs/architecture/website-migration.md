# Website Migration Assessment

## Current source

The private `polterware/polterware-web` repository is an active Next.js studio website. It is not currently a product website for the Polterware control plane.

## Content classification

| Area | Classification | Migration decision |
| --- | --- | --- |
| Polterware wordmark and ghost assets | Reusable shared brand assets | Eligible after brand ownership and final asset selection are confirmed |
| Translation hardening in the root layout | Reusable web behavior | Required in any future `apps/website` implementation |
| Hero copy about a solo studio | Studio positioning | Not moved into the product monorepo |
| Erick profile and portfolio links | Personal/portfolio content | Not moved into the product monorepo |
| DOST, Duplizen, Converge, and Kit listings | Broader studio portfolio | Not moved into the product monorepo |
| Contact form and Supabase migration | Studio contact infrastructure | Not moved; its data and deployment ownership are unrelated to the product |
| `package-lock.json` | Obsolete mixed package-manager state | Never moved into the pnpm monorepo |
| Next.js component and styling implementation | Potential reference | Reuse requires an approved product-website specification and dependency audit |

## Decision

No current route is copied into `apps/website` during the repository migration. Moving the existing application wholesale would mix personal portfolio content and unrelated contact infrastructure into the public product repository.

`polterware/polterware-web` remains private and active as a rollback source. It is not archived until:

1. a product-website content specification is accepted;
2. approved brand assets and product-only content are migrated to `apps/website`;
3. the pnpm workspace and deployment owner are established;
4. static validation and an explicit acceptance checkpoint succeed.

The future website root must use:

```html
<html lang="en" translate="no">
<meta name="google" content="notranslate" />
```

This assessment prevents an organizational migration from silently becoming a redesign or a publication of unrelated personal material.
