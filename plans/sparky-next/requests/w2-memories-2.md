# W2-MEMORIES: lockfile importers

Owner: integration / W0-SCAFFOLD.

Update only the `packages/features/sparky-memories` importer in `pnpm-lock.yaml` after merging its package.json. Runtime additions: `@bufbuild/protobuf: ^2.15.0`, `@polterware/frontend-design-system: workspace:*`, `@polterware/sparky-contracts: workspace:*`. Dev additions: `jsdom: ^30.1.1`, `vite-plugin-solid: ^2.11.14`. These versions already exist elsewhere in the workspace; no new resolved dependency version is needed. Virtualization uses the shared DataTable's existing `@tanstack/solid-virtual` implementation.

Command: `pnpm install --lockfile-only` in the integration checkout; review that only the memory importer changes. W2-MEMORIES installs with `--lockfile=false` locally to verify without editing the shared lockfile.
