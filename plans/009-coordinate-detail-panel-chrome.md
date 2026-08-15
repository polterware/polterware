# 009 — Coordinate collapsed title chrome and hover

- **Status**: DONE
- **Commit**: 715655e
- **Severity**: MEDIUM
- **Category**: Cohesion, easing, and duration
- **Estimated scope**: 3 files, small

## Problem

Collapsed title geometry and chrome switch immediately from mode state:

```tsx
// packages/frontend/design-system/src/detail-panel.tsx:223 — current
<div
  class="pw-detail-panel__identity-content"
  classList={{
    "control-field": context.mode() === "collapsed",
    "control-field-sm": context.mode() === "collapsed",
    "!rounded-full": context.mode() === "collapsed",
  }}
>
```

```css
/* packages/frontend/design-system/src/detail-panel.css:267 — current */
.pw-detail-panel-root[data-mode="collapsed"]
  .pw-detail-panel__identity-content {
  gap: 6px;
  padding-inline: 1rem;
}
```

The padding, radius, surface, and icon dimensions therefore jump at the start of a 280ms panel transition. Collapsed hover also drives two decorative layers at different speeds: the full panel chrome uses the dedicated 120ms path while `.control-field` uses 160ms:

```css
/* packages/frontend/design-system/src/materials.css:79 — current */
transition:
  background var(--duration-control) var(--ease-standard),
  box-shadow var(--duration-control) var(--ease-standard);
```

```css
/* packages/frontend/design-system/src/detail-panel.css:159 — current */
transition:
  opacity var(--detail-panel-hover-duration) var(--detail-panel-hover-ease),
  background-color var(--detail-panel-hover-duration) var(--detail-panel-hover-ease);
```

## Target

Keep title and icon layout stable across all modes. Paint the collapsed control chrome with a dedicated decorative pseudo-element whose visual padding extends `1rem` around the unchanged content without moving it.

```css
.pw-detail-panel__identity-content {
  position: relative;
  isolation: isolate;
  height: var(--control-height-sm);
  gap: 8px;
  border-radius: 9999px; /* supplied by the existing !rounded-full utility in markup */
}

.pw-detail-panel__identity-content::before {
  position: absolute;
  z-index: -1;
  inset-block: 0;
  inset-inline: -1rem;
  border-radius: inherit;
  background: var(--color-surface-subtle);
  box-shadow:
    inset 0 1px 0 var(--edge-light-field-highlight),
    inset 0 0 0 1px var(--edge-light-field-rim);
  content: "";
  opacity: 0;
  pointer-events: none;
  transition:
    opacity var(--detail-panel-snap-duration) var(--detail-panel-snap-ease),
    background-color var(--duration-fast) var(--ease-standard);
}

.pw-detail-panel-root[data-mode="collapsed"]
  .pw-detail-panel__identity-content::before {
  opacity: 1;
}
```

Exact interaction timing:

- Collapsed title default chrome follows the structural `280ms` opacity trajectory.
- Once collapsed, hover/focus/pressed background feedback uses `var(--duration-fast)` = `120ms` and `var(--ease-standard)` = `cubic-bezier(0.2, 0, 0, 1)`, matching `packages/features/shell/src/sidebar.css:54-56`.
- The settled collapsed title pseudo-element is the only hover chrome. Remove the full-surface hover/focus restoration that sets `--detail-panel-chrome-alpha: 1`.
- Keep the full-width bottom fade visible and keep structural card/fade interpolation for collapse/expansion.
- Keep the icon's 16px layout box in every mode. Apply the requested collapsed 14px appearance with `transform: scale(0.875)` on the SVG over the structural 280ms curve; do not change width/height/flex-basis by mode.
- Focus remains visible around the title control chrome. Detail text stays fully opaque.

## Repo conventions to follow

- `packages/configuration/tailwind/base.css:102-107` owns `--ease-standard`, `--duration-fast`, and `--duration-control`.
- `packages/frontend/design-system/src/materials.css:71-81` supplies the exact shared field surface and edge-light tokens; reuse those values rather than inventing colors.
- `packages/features/shell/src/sidebar.css:54-56` is the requested 120ms hover exemplar.
- Decorative opacity in the DetailPanel is allowed by `DESIGN.md:150`; text and controls remain opaque.

## Steps

1. In `packages/frontend/design-system/src/detail-panel.tsx`, remove mode-dependent `control-field` and `control-field-sm` class toggles. Keep `!rounded-full` as a stable class so the decorative layer inherits the existing pill radius.
2. In `packages/frontend/design-system/src/detail-panel.css`, keep one stable 16px icon/title layout and remove collapsed padding/gap/dimension changes.
3. Add the title chrome pseudo-element with the exact shared surface/edge-light tokens and `1rem` visual inline extension shown above.
4. Scale only the SVG to `0.875` when collapsed and transition `transform` with the existing structural 280ms curve.
5. Route collapsed title hover, focus, and pressed state to the pseudo-element at 120ms standard easing. Remove the full-surface hover/focus chrome restoration selectors while retaining the structural decorative fade/card interpolation.
6. Update `DESIGN.md` and `AGENTS.md` so the collapsed title control documents its shared field chrome and 120ms hover instead of the stale no-border statement.
7. Update `packages/frontend/design-system/src/detail-panel.test.tsx` to assert stable layout, decorative-only opacity, 120ms hover parity, transformed 14px icon appearance, focus visibility, and absence of full-surface collapsed hover restoration.

## Boundaries

- Do NOT change title font size/weight/line-height, visible `1rem` inline padding, header height, global actions, panel geometry, fade extent, mode behavior, focus restoration, Tooltip timing, or public API.
- Do NOT animate padding, width, height, text opacity, or whole-element opacity.
- Do NOT add raw alpha colors, a new material token, a dependency, `transition: all`, or keyframes.
- Do NOT run tests or open a browser under the current authorization.
- Execute after plan 008 and preserve its transform/viewport lifecycle.

## Verification

- **Mechanical**: update coverage but do not run it; run `pnpm design:lint`, `pnpm typecheck`, and `git diff --check`.
- **Feel check**: with later browser authorization, inspect collapse/expand at 10% speed. Text must not shift horizontally, the icon must scale smoothly, the title chrome must cross-fade without bringing back full-card hover chrome, and hover in/out must finish in 120ms.
- **Reduced motion**: the title/icon movement becomes immediate while hover/focus state remains visible.
- **Done when**: title layout is stable, structural chrome is decorative-only, and collapsed hover has one 120ms response path.
