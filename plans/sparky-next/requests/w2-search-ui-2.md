---
summary: Regenerate the lockfile for W2-SEARCH-UI package dependencies
rationale: The search feature needs the existing design system, frozen contracts, and protobuf JSON helpers. The root lockfile is owned by W0-SCAFFOLD.
---

W2-SEARCH-UI requests W0-SCAFFOLD / the integrator to regenerate `pnpm-lock.yaml` after accepting the new search-package dependencies. Do not invent versions; reuse the workspace resolutions already locked for W2-MEMORIES / W2-REVIEW.

Exact importer delta required at `packages/features/sparky-search` (leave existing `@polterware/feature-sparky-shell`, solid-js, and vitest entries intact):

```yaml
dependencies:
  '@bufbuild/protobuf':
    specifier: ^2.15.0
  '@polterware/frontend-design-system':
    specifier: workspace:*
    version: link:../../frontend/design-system
  '@polterware/sparky-contracts':
    specifier: workspace:*
    version: link:../../sparky-contracts
devDependencies:
  jsdom:
    specifier: ^30.1.1
  vite-plugin-solid:
    specifier: ^2.11.14
```

Populate omitted version fields with `pnpm install --lockfile-only` and confirm unrelated package versions do not drift. The package manifest is committed in W2-SEARCH-UI. Local validation used `pnpm install --lockfile=false`; the shared lockfile was left untouched.

W3-E2E owns the packaged Playwright runner. `apps/sparky-desktop/tests/e2e/search.spec.ts` is the smoke spec for this workstream.
