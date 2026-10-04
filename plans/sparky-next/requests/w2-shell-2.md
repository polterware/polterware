# W2-SHELL request 2 — shared lockfile importer

- Owner: W0-SCAFFOLD / integrator.
- Blocked sub-task: updating shared `pnpm-lock.yaml` for owned shell-package dependencies.
- Why: the shell now consumes the existing design system and frozen contracts. Component behavior tests need the already-used Solid Vite plugin and jsdom. The root lockfile is outside Owns.
- Exact importer delta required at `packages/features/sparky-shell` (leave existing solid-js and vitest entries intact; reuse already-locked resolutions):

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

Populate the omitted version fields using the existing workspace resolutions (`pnpm install --lockfile-only`, review that unrelated package versions do not drift). The package manifest is committed in W2-SHELL. Local validation used `pnpm install --lockfile=false`; the shared lockfile was left untouched.
