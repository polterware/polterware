# W1-ELECTRON request 2: scaffold-owned renderer bridge and CSP

Owner: W0-SCAFFOLD / integrator. Required changes outside W1-ELECTRON Owns.

Exact replacement for `apps/sparky-desktop/src/renderer/window.d.ts`:

```ts
import type { SparkyBridge } from "../shared/bridge.js";
declare global {
  interface Window { sparky: SparkyBridge; }
}
export {};
```

The current declaration exposes only `app.name`; W2 features need the shared RPC bridge type.

Remove the complete `<meta http-equiv="Content-Security-Policy" ... />` element from `apps/sparky-desktop/src/renderer/index.html` after verifying Main's CSP is applied to dev and packaged renderer responses. Main supplies CSP; the existing meta independently blocks `sparky-file:` previews and the optional OpenFreeMap host, so enabling maps in Main cannot work until this is resolved. Keep a strict packaged CSP; never widen the meta independently of the `map.online` setting.

These renderer changes are stopped pending the owner's integration.

The stub `src/renderer/App.tsx` has no router. W2-SHELL must subscribe to `app.onDeepLink`, consume `app.getDeepLink()` on mount (cold start), and route the validated `DeepLink.route`. Main routing/renderer event dispatch can be tested now; end-to-end renderer router acceptance remains blocked until that workstream lands.
