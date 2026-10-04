---
summary: Regenerate the lockfile for W2-MAP maplibre-gl and supercluster
rationale: The shared MapView needs MapLibre GL JS and supercluster. The root lockfile is owned by W0-SCAFFOLD / the integrator.
---

W2-MAP requests W0-SCAFFOLD / the integrator to regenerate `pnpm-lock.yaml` after accepting the new `@polterware/frontend-map` dependencies. Do not invent versions; reuse the workspace resolutions already locked for Solid/Vite where they overlap. `maplibre-gl` and `supercluster` versions match the W0-MAP spike (`spikes/map/package.json`).

Exact importer delta required at `packages/frontend/map` (leave existing `solid-js` peer/dev and `vitest` entries intact):

```yaml
dependencies:
  maplibre-gl:
    specifier: ^5.6.1
  supercluster:
    specifier: ^8.0.1
devDependencies:
  jsdom:
    specifier: ^30.1.1
  vite-plugin-solid:
    specifier: ^2.11.14
```

Exact importer delta required at `packages/features/sparky-map` (leave existing `@polterware/feature-sparky-shell`, solid-js, and vitest entries intact):

```yaml
dependencies:
  '@bufbuild/protobuf':
    specifier: ^2.15.0
  '@polterware/frontend-design-system':
    specifier: workspace:*
    version: link:../../frontend/design-system
  '@polterware/frontend-map':
    specifier: workspace:*
    version: link:../../frontend/map
  '@polterware/sparky-contracts':
    specifier: workspace:*
    version: link:../../sparky-contracts
devDependencies:
  jsdom:
    specifier: ^30.1.1
  vite-plugin-solid:
    specifier: ^2.11.14
```

Populate omitted version fields with `pnpm install --lockfile-only` and confirm unrelated package versions do not drift. The package manifests are committed in W2-MAP. Local validation used `pnpm install --no-frozen-lockfile`; **do not commit `pnpm-lock.yaml` from this workstream**.

W3-E2E owns the packaged Playwright runner. `apps/sparky-desktop/tests/e2e/map.spec.ts` is the smoke spec for this workstream (network log while `map.online` is false).
