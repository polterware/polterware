# W2-ATTACH-UI request 2 — lockfile importer for the attachments package

Owner: W0-SCAFFOLD / integrator. No frozen RPC change.

`packages/features/sparky-attachments/package.json` now depends on the same test/runtime set as `sparky-memories`. Do not commit `pnpm-lock.yaml` from this worktree. Please regenerate the root lockfile with this importer delta:

```json
{
  "name": "@polterware/feature-sparky-attachments",
  "dependencies": {
    "@bufbuild/protobuf": "^2.15.0",
    "@polterware/feature-sparky-shell": "workspace:*",
    "@polterware/frontend-design-system": "workspace:*",
    "@polterware/sparky-contracts": "workspace:*"
  },
  "devDependencies": {
    "jsdom": "^30.1.1",
    "solid-js": "^1.9.15",
    "vite-plugin-solid": "^2.11.14",
    "vitest": "^5.0.1"
  }
}
```

Removed: none. The previous stub only listed `@polterware/feature-sparky-shell` and `solid-js`/`vitest`.
