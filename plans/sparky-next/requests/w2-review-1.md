---
summary: Serve mood character PNGs at /characters for the Review summary card
rationale: W2-REVIEW must show streak/completed mood art from apps/sparky-desktop/resources/characters/ without editing Electron Main, electron-builder.yml, or the renderer public root.
---

W2-REVIEW requests W1-ELECTRON / W1-ASSETS / W2-SHELL:

Character files already exist (do not regenerate):

- `apps/sparky-desktop/resources/characters/me-streak-{sad,hopeful,happy,euphoric}.png` plus `@2x` / `@3x`
- `apps/sparky-desktop/resources/characters/me-completed-{sad,hopeful,happy,euphoric}.png` plus `@2x` / `@3x`
- `apps/sparky-desktop/resources/characters/sparky-character.webp`

The Review renderer loads:

```
/characters/me-streak-<mood>.png
/characters/me-streak-<mood>@2x.png
/characters/me-streak-<mood>@3x.png
/characters/me-completed-<mood>.png
```

(`<mood>` is the core `streakMood` / `completedMood` string: `sad` | `hopeful` | `happy` | `euphoric`.)

Exact serving change:

1. Dev (electron-vite renderer): copy or alias `apps/sparky-desktop/resources/characters` to the renderer public root as `/characters/` (for example `publicDir` + `public/characters`, or a Vite `server.fs` / `publicDir` pointing at `resources`).
2. Packaged app: add to `apps/sparky-desktop/electron-builder.yml` `extraResources`:

```yaml
  - from: resources/characters
    to: characters
```

and serve those files from Electron Main at `http://.../characters/...` (same pattern as `/sounds/*.webm` requested by W2-FOCUS). Do not expose `file://`.

Until this lands, Review shows the numeric streak/completed values and the caption "Character art unavailable" when the image 404s. W2-REVIEW does not edit those shared files.
