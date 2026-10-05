# W1-DS request 1: gallery workspace registration and source coverage

Owner: W0-SCAFFOLD / integrator. The gallery implementation is owned by W1-DS; the shared lockfile and lint scanner are not.

## Required changes

1. Run `pnpm install --lockfile-only` after merging `apps/gallery/package.json`; commit only the new `apps/gallery` importer in `pnpm-lock.yaml`. Its dependencies use existing workspace/resolved versions; no dependency upgrade is requested. W1-DS uses `pnpm install --lockfile=false` locally and leaves the frozen shared lockfile unchanged.
2. Add gallery to the exact `SOURCE_ROOTS` list in `tooling/check-ui-opacity.mjs`:

```diff
 const SOURCE_ROOTS = [
+  "apps/gallery/src",
   "apps/desktop/src",
```

3. Add the missing Sparky source glob in `packages/configuration/tailwind/base.css` (W1-DS owns additive tokens only; its own gallery stylesheet scans gallery sources):

```diff
 @source "../../../apps/desktop/src/**/*.{html,ts,tsx}";
+@source "../../../apps/sparky-desktop/src/**/*.{html,ts,tsx}";
```

These registration subtasks stop here per AGENT-PROTOCOL §2. No rule/material/opacity allowlist change or DESIGN.md edit is needed.
