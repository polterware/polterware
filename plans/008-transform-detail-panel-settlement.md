# 008 — Move DetailPanel settlement to a transform trajectory

- **Status**: DONE
- **Commit**: 715655e
- **Severity**: HIGH
- **Category**: Performance, purpose, and interruptibility
- **Estimated scope**: 3 files, medium

## Problem

The button-only DetailPanel animates its full layout height and viewport padding for every mode change:

```css
/* packages/frontend/design-system/src/detail-panel.css:63 — current */
.pw-detail-panel__panel {
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  width: 100% !important;
  height: 50%;
  overflow: hidden;
  pointer-events: none;
  transition: height var(--detail-panel-snap-duration) var(--detail-panel-snap-ease);
}
```

```css
/* packages/frontend/design-system/src/detail-panel.css:113 — current */
.pw-detail-panel__viewport {
  width: 100%;
  height: 100%;
  padding: var(--detail-panel-gutter);
  transition: padding var(--detail-panel-snap-duration) var(--detail-panel-snap-ease);
}
```

That recalculates the detail layout on every frame across a large surface. The body is also hidden as soon as the target mode becomes collapsed, before the 280ms trajectory finishes:

```tsx
// packages/frontend/design-system/src/detail-panel.tsx:327 — current
<div
  class={`pw-detail-panel__body ${props.class ?? ""}`}
  hidden={context.mode() === "collapsed"}
>
```

The settlement lifecycle duplicates the CSS duration in TypeScript and CSS:

```tsx
// packages/frontend/design-system/src/detail-panel.tsx:48 — current
const DETAIL_PANEL_SETTLEMENT_DURATION = 280;
const DETAIL_PANEL_SETTLEMENT_SAFETY_MARGIN = 40;
```

```css
/* packages/frontend/design-system/src/detail-panel.css:14 — current */
--detail-panel-snap-duration: 280ms;
--detail-panel-snap-ease: cubic-bezier(0.16, 1.22, 0.3, 1);
```

## Target

Keep the documented terminal states and the existing deliberate `280ms cubic-bezier(0.16, 1.22, 0.3, 1)` curve, but move the large trajectory to `transform`:

```css
.pw-detail-panel-root {
  container-type: size;
  --detail-panel-viewport-height: 50cqh;
}

.pw-detail-panel-root[data-viewport-mode="collapsed"] {
  --detail-panel-viewport-height: 68px;
}

.pw-detail-panel-root[data-viewport-mode="maximized"] {
  --detail-panel-viewport-height: 100cqh;
}

.pw-detail-panel__panel {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  transform: translateY(50cqh);
  transition: transform 280ms cubic-bezier(0.16, 1.22, 0.3, 1);
  will-change: transform;
}

.pw-detail-panel-root[data-mode="collapsed"] .pw-detail-panel__panel {
  transform: translateY(calc(100cqh - 68px));
}

.pw-detail-panel-root[data-mode="maximized"] .pw-detail-panel__panel {
  transform: translateY(0);
}

.pw-detail-panel__viewport {
  height: var(--detail-panel-viewport-height);
  padding: var(--detail-panel-gutter);
}
```

`data-viewport-mode` controls the non-animated terminal layout inside the moving compositor layer:

- Expansion (`collapsed → open/maximized`, `open → maximized`): switch `data-viewport-mode` to the destination before the transform begins. The larger viewport starts below the clipped host and is progressively revealed.
- Contraction (`maximized → open/collapsed`, `open → collapsed`): keep the source viewport mode during the transform and switch to the destination only after the transform ends. The host clips the excess content during travel.
- `DetailPanel.Body.hidden` follows `viewportMode`, not the target `mode`, so content remains mounted while contracting and is available before expansion begins.
- Gutter and radius follow `viewportMode`. They change once at the appropriate trajectory boundary and do not animate `padding`.
- Reduced motion switches both target mode and viewport mode immediately, without a positional transition.

Replace the raw TypeScript duration with the actual CSS lifecycle:

- `transitionrun` for `transform` marks settlement active.
- `transitionend` for `transform` commits a pending contraction viewport mode and clears settlement.
- `transitioncancel` clears only the cancelled trajectory; the following `transitionrun` may activate the retargeted one.
- Keep a defensive fallback derived from `getComputedStyle(panel).transitionDuration`, plus the existing exact `40ms` safety margin. Do not hard-code `280` in TypeScript.

## Repo conventions to follow

- The terminal geometry and deliberate spring curve are settled in `DESIGN.md:424`; preserve them.
- Shared duration/easing tokens live in `packages/configuration/tailwind/base.css:102-107`; do not create a parallel global scale.
- The component already uses CSS transitions, which retarget from the current transform when buttons are pressed rapidly.
- Detail content must remain opaque. Use clipping by the existing host overflow; do not cross-fade body content.

## Steps

1. In `packages/frontend/design-system/src/detail-panel.tsx`, add a `viewportMode` signal initialized from `props.mode`, a terminal-rank helper, a pending contraction mode, and a panel element ref.
2. On mode changes, set `viewportMode` before expansions; retain it during contractions until the transform transition finishes. In reduced motion, commit it immediately.
3. Replace the settlement duration constant with `transitionrun`, `transitionend`, and `transitioncancel` handlers scoped to `event.propertyName === "transform"`. Derive the fallback timeout from computed CSS transition durations and retain only the `40ms` safety margin constant.
4. Expose `viewportMode` through the DetailPanel context and root `data-viewport-mode` attribute. Make `DetailPanel.Body.hidden` depend on it.
5. In `packages/frontend/design-system/src/detail-panel.css`, make the root a size container, keep the panel at 100% height, and animate only its transform between the three exact positions shown above.
6. Make viewport height, gutter, and radius derive from `data-viewport-mode`. Remove the `height` and `padding` transitions. Keep the decorative opacity transition and radius transition only if they do not move content.
7. Update `packages/frontend/design-system/src/detail-panel.test.tsx` to cover the transform contract, viewport-mode sequencing for expansion/contraction, body presence during contraction, reduced-motion immediate settlement, transition cancellation, and absence of animated `height`/`padding`.

## Boundaries

- Do NOT change public DetailPanel props, global store contracts, control labels, focus restoration, Escape behavior, terminal sizes, gutter endpoints, radius endpoints, fade material, or button order.
- Do NOT reintroduce Corvu, pointer resize, numeric persistence, `requestAnimationFrame`, a motion dependency, keyframes, or `transition: all`.
- Do NOT animate detail text/content opacity.
- Do NOT run tests or open a browser under the current authorization.
- Preserve unrelated dirty-worktree changes. If the cited structure has materially drifted, stop and report instead of replacing it wholesale.

## Verification

- **Mechanical**: update coverage but do not run it; run `pnpm design:lint`, `pnpm typecheck`, and `git diff --check`. Report existing unrelated blockers exactly.
- **Feel check**: when browser authorization is later granted, trigger every mode pair at normal speed and 10% playback. The body must remain visible and clipped while contracting, appear before expansion, and never leave a blank moving shell. Rapidly reverse direction and confirm the transform retargets without restarting from a terminal position.
- **Reduced motion**: emulate `prefers-reduced-motion: reduce`; mode and viewport geometry must settle immediately with no travel.
- **Done when**: the large panel trajectory animates `transform`, terminal layout changes once per transition, body presence follows the visual viewport, and JavaScript no longer duplicates `280ms`.
