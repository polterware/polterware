# W3-SECURITY-1: remediate the shared MapLibre sanitizer vulnerability

- Finding SEC-09, Critical dependency advisory: `pnpm audit --prod` reports `maplibre-gl@5.24.0`, GHSA-jrc7-96c5-q579 / CVE-2026-85061. Versions <=6.4.0 are affected; 6.4.1 is patched.
- Primary evidence: https://github.com/maplibre/maplibre-gl-js/security/advisories/GHSA-jrc7-96c5-q579 and the installed `src/util/dom.ts` live attribute iterator.
- Reachable surface: shared `packages/frontend/map/src/engine.ts` renders third-party OpenFreeMap/TileJSON attribution through MapLibre. Sparky's enforced `script-src 'self'` blocks injected event handlers, but retaining the dependency is not a completed supply-chain fix.
- Ownership blocker: `packages/frontend/map/**` and `pnpm-lock.yaml` are explicitly excluded from W3-SECURITY. Stop this subtask; request the integrator's change.

## Exact requested change

Preferred minimal remediation: patch the installed dependency through a checked-in pnpm patch changing `for (const {name, value} of elem.attributes)` to `for (const {name, value} of Array.from(elem.attributes))` in source and the runtime distribution, register that patch in the shared pnpm configuration, and update the lockfile integrity. Verify attribution payloads with adjacent event-handler attributes cannot execute under either product.

Alternatively update `packages/frontend/map/package.json` from `"maplibre-gl": "^5.24.0"` to an exact `"maplibre-gl": "6.4.1"` and regenerate the shared lockfile. This is a major upgrade: validate MapLibre options/types, CSS, attribution, markers, clustering, offline-provider behavior, worker/CSP behavior, both Desktop products and the packaged renderer before accepting it. W3-SECURITY did not perform or approve a major bump.

Re-run `pnpm audit --prod`, shared-map/Sparky-map tests and typechecks, both product checks, and the W3 Electron security PoC. No Sparky contract change is required.
