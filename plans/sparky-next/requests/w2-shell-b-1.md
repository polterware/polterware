# W2-SHELL-B request 1 — desktop typecheck blocked by Minds Window.sparky

- Owner: W2-MINDS / integrator.
- Gate: `pnpm typecheck` and `pnpm --filter @polterware/sparky-desktop typecheck`.
- Evidence: the failure is pre-existing on current `sparky-next` (confirmed with a clean tree). Desktop compiles `@polterware/feature-sparky-minds` source, which declares a second `Window.sparky` in `packages/features/sparky-minds/src/window.d.ts`. That declaration intersects the desktop protobuf-JSON bridge and makes `minds.list` / `memories.list` / `minds.delete` resolve to `JsonValue`.
- Errors (unchanged by this workstream): `packages/features/sparky-minds/src/index.tsx` lines 14, 60, 69, and 76.
- Requested fix (outside W2-SHELL-B Owns): remove the feature-local `Window.sparky` type. Reach the core through `window.sparky` protobuf JSON, validating with `createShellBridge` from `@polterware/feature-sparky-shell` (do not declare a second `Window.sparky`). `@polterware/feature-sparky-minds` typecheck already passes in isolation.
- No file outside W2-SHELL-B ownership was edited to work around this.
