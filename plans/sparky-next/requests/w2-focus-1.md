---
summary: Route Start focus deep links to the Focus page and allow sparky://focus
rationale: W2-FOCUS can consume sparky://memory/<id>?action=focus through SparkyFeature.onDeepLink and start the timer, but it cannot navigate. The shell currently opens the memory detail instead of /focus. Commander Start focus has the same gap.
---

W2-FOCUS requests W2-SHELL and W1-ELECTRON / W3-PACKAGING:

1. After a feature `onDeepLink` starts a memory-bound session for `sparky://memory/<ulid>?action=focus`, navigate to `/focus` (keep the memory selected if useful). Today `onDeepLink` returning true skips navigation entirely, and returning false opens `/memory/:id`.
2. Allow `sparky://focus` in `apps/sparky-desktop/src/main/navigation-policy.ts` `parseDeepLink` and in `packages/features/sparky-shell/src/route.ts` `deepLinkPath`, so the Start focus command can call `app.openDeepLink("sparky://focus")`.
3. Copy `apps/sparky-desktop/resources/sounds/*.webm` into the renderer/public extraResources (they are not in `electron-builder.yml` today) so Web Audio can `fetch("/sounds/<name>.webm")`. W2-FOCUS synthesizes a short oscillator when the file is missing.

W2-FOCUS does not edit those shared files.
