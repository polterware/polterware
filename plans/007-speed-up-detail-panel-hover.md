# 007 — Speed up collapsed DetailPanel hover

- **Status**: DONE
- **Commit**: f8a1129
- **Severity**: MEDIUM
- **Category**: Easing and duration
- **Estimated scope**: 2 files, small

## Problem

The collapsed DetailPanel restores its chrome on hover and focus through the same overshooting 280ms curve used for large panel settlement:

```css
/* packages/frontend/design-system/src/detail-panel.css:14 — current */
.pw-detail-panel-root {
  --detail-panel-snap-duration: 280ms;
  --detail-panel-snap-ease: cubic-bezier(0.16, 1.22, 0.3, 1);
}
```

```css
/* packages/frontend/design-system/src/detail-panel.css:100 — current */
.pw-detail-panel__surface.surface-panel-float::before,
.pw-detail-panel__surface.surface-panel-float::after {
  transition:
    opacity var(--detail-panel-snap-duration) var(--detail-panel-snap-ease),
    background var(--duration-control) var(--ease-standard);
}
```

Hover is a high-frequency control response. A delayed, overshooting opacity crossfade feels sticky and makes the now-working hover affordance appear unreliable.

## Target

Separate pointer/focus feedback from large settlement motion.

```css
/* target values */
--detail-panel-hover-duration: var(--duration-fast); /* 120ms */
--detail-panel-hover-ease: var(--ease-standard); /* cubic-bezier(0.2, 0, 0, 1) */
```

- Chrome/fade interpolation during a collapse or expansion keeps `280ms cubic-bezier(0.16, 1.22, 0.3, 1)`.
- Once the panel is settled collapsed, hover, hover-out, `focus-within`, resize-handle hover, and resize-handle `focus-visible` crossfade chrome/fade in `120ms cubic-bezier(0.2, 0, 0, 1)`.
- Background color continues to use `160ms var(--ease-standard)`.
- No scale, bounce, movement, or new decorative treatment is added.

## Repo conventions to follow

- `packages/configuration/tailwind/base.css:102` defines `--ease-standard: cubic-bezier(0.2, 0, 0, 1)`.
- `packages/configuration/tailwind/base.css:105` defines `--duration-fast: 120ms`; use the token instead of a new raw duration.
- Keep the existing equivalent selectors at `packages/frontend/design-system/src/detail-panel.css:128`: surface hover, surface `focus-within`, resize-handle hover through `:has(...)`, and resize-handle `focus-visible` through `:has(...)`.

## Steps

1. In `packages/frontend/design-system/src/detail-panel.css`, define local aliases for the hover duration and easing using `--duration-fast` and `--ease-standard`.
2. Give the collapsed settled hover/focus crossfade its own 120ms opacity transition in both directions. If plan 006 introduces a settlement data attribute, use it to distinguish an in-flight collapse from settled hover state so the 280ms collapse crossfade is preserved.
3. Preserve the existing `surface-hover`, edge-light, shadow, fade removal, and all four pointer/focus selectors. Do not change their alpha endpoints (`1` for chrome and `0` for fade).
4. In `packages/frontend/design-system/src/detail-panel.test.tsx`, update the CSS contract coverage to assert the hover token aliases, the 120ms/standard-ease crossfade path, the 280ms settlement path, and selector parity for surface and resize-handle hover/focus.

## Boundaries

- Do NOT change geometry, markup, snaps, controls, Tooltip timing, colors, materials, opacity allowlist, or shared motion token values.
- Do NOT add movement, scale, keyframes, dependencies, or `transition: all`.
- Do NOT run tests or open a browser under the current task authorization.
- Execute after plan 006 and reuse its settlement state if present; do not create a second lifecycle mechanism.
- Preserve every unrelated dirty-worktree change. If the cited blocks have materially drifted from commit `f8a1129`, STOP and report instead of improvising.

## Verification

- **Mechanical**: update coverage but do not run tests; run `pnpm design:lint`, `pnpm typecheck`, and `git diff --check`. Report the exact failures if repository-wide concurrent errors remain.
- **Feel check**: only with explicit browser authorization, settle the panel collapsed and move the pointer quickly across the strip. Chrome should appear and disappear promptly without bounce or lag. Repeat with keyboard focus and at 10% playback speed; the opacity path must be shorter than the panel snap.
- **Reduced motion**: with explicit browser authorization, confirm focus and hover retain safe color/opacity feedback without positional movement.
- **Done when**: collapsed hover/focus uses the shared 120ms control response while the structural collapse/expand trajectory remains 280ms.
