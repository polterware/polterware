---
summary: Allow sparky://map in Electron parseDeepLink
rationale: The Map feature command calls app.openDeepLink("sparky://map"). Shell deepLinkPath already accepts /map via SHELL_ROUTES, but Main parseDeepLink currently rejects the hostname.
---

W2-MAP requests W1-ELECTRON:

Add `case "map":` to `apps/sparky-desktop/src/main/navigation-policy.ts` `parseDeepLink`, same shape as `today` / `focus` (pathname `/` or empty, no query params), routing to `/map`.

W2-MAP does not edit Main. Commander already has "Go to Map" from SHELL_ROUTES; this unblocks `sparky://map` and the feature command `map.open`.
