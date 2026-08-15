# 006 — Make DetailPanel settlement interruptible

- **Status**: DONE
- **Commit**: f8a1129
- **Severity**: HIGH
- **Category**: Interruptibility and physicality
- **Estimated scope**: 3 files, medium

## Problem

The DetailPanel has two competing notions of size while a 280ms snap is running. CSS displays an intermediate height, but pointer-down resets the drag origin from the terminal mode instead of the currently painted panel:

```tsx
// packages/frontend/design-system/src/detail-panel.tsx:336 — current
beginResize: (event) => {
  dragging = true;
  didDrag = false;
  dragStartedMaximized = mode() === "maximized";
  setIsClosingFromMaximized(false);
  dragStartCoordinate = pointerCoordinate(event);
  currentSize = sizeForMode(mode(), axisSize());
  dragStartSize = currentSize;
},
```

The settlement path also jumps `renderedSize` directly to a terminal target and lets several layout properties transition independently:

```tsx
// packages/frontend/design-system/src/detail-panel.tsx:253 — current
function applySettledSize() {
  const availableSize = axisSize();
  if (availableSize > 0) setAvailableAxisSize(availableSize);
  const settled = resolveDetailPanelSize(currentSize, availableSize);
  if (settled.mode === "collapsed") {
    currentSize = 0;
    setRenderedSize(0);
    props.onCollapse();
    return;
  }
```

```css
/* packages/frontend/design-system/src/detail-panel.css:49 — current */
.pw-detail-panel__panel {
  height: var(--detail-panel-size) !important;
  transition: height var(--detail-panel-snap-duration) var(--detail-panel-snap-ease);
}
```

Grabbing the header during a snap can therefore jump to the logical destination. Pointer drag, keyboard actions, header controls, Escape, and release settlement do not all begin from the same visual size.

## Target

Use `renderedSize` as the single source for every DetailPanel trajectory and sample the painted panel height before a new pointer drag starts.

- Keep the current three terminal targets: collapsed `0px` model size with a visible `68px` strip, open at `50%`, maximized at `100%`.
- Keep the `12px` drag activation threshold.
- Keep settlement at `280ms` with `cubic-bezier(0.16, 1.22, 0.3, 1)`; do not add a motion dependency or a JavaScript `requestAnimationFrame` loop.
- Route pointer release, Arrow/Home/End, Enter/Space, the header collapse control, and Escape through one mode-transition helper that first updates `currentSize`, `renderedSize`, and Corvu panel sizes from the same target, then invokes the existing public callback.
- At pointer-down, read the current painted height from `.pw-detail-panel__panel` using `getBoundingClientRect().height`, clamp it to the current axis, and use that value for `currentSize`, `dragStartSize`, `renderedSize`, and Corvu sizes before direct dragging continues.
- Add a bounded settlement flag only if needed to keep CSS transitions enabled during programmatic settlement and disabled during direct drag. Clear it on the panel height `transitionend`, with a `280ms` fallback timer plus a small safety margin; clean the timer in `onCleanup`.
- Preserve all geometry calculations from `renderedSize`: collapse progress from `220px` to `68px`, the `8px` gutter and `14px` floating radius, and the 60%–80% merge to zero.

## Repo conventions to follow

- Motion values stay local to `packages/frontend/design-system/src/detail-panel.css`: `--detail-panel-snap-duration: 280ms` and `--detail-panel-snap-ease: cubic-bezier(0.16, 1.22, 0.3, 1)`.
- Shared control motion tokens remain in `packages/configuration/tailwind/base.css`; do not introduce a second global motion scale.
- Corvu remains the resize behavior owner. Continue using `resizableContext.setSizes` through the existing `setPanelSizes` adapter at `packages/frontend/design-system/src/detail-panel.tsx:416`.
- The existing direct-drag rule at `packages/frontend/design-system/src/detail-panel.css:331` disables settlement transitions while the Corvu handle has `data-dragging`.

## Steps

1. In `packages/frontend/design-system/src/detail-panel.tsx`, add a small helper that reads and clamps the painted `.pw-detail-panel__panel` height. Keep the selector local to `rootElement` and fall back to `renderedSize()` before falling back to `sizeForMode(...)`.
2. Replace the mode-derived assignment in `beginResize` with the painted height. Synchronize `currentSize`, `dragStartSize`, `renderedSize`, `availableAxisSize`, and Corvu sizes before processing subsequent drag values.
3. Extract one internal transition helper for collapsed/open/maximized targets. It must update the size source and Corvu ratio before calling `onCollapse`, `onRestore`, or `onMaximize`; preserve focus restoration only on collapse.
4. Route `applySettledSize`, `stepUp`, `stepDown`, Escape, and the header collapse/expand action through that helper. Preserve the current public props and keyboard semantics.
5. In `packages/frontend/design-system/src/detail-panel.css`, keep direct drag transition-free while ensuring a pointer-down during settlement continues from the sampled painted height instead of snapping to the old target. Do not add `transition: all`, keyframes, or new layout animations.
6. In `packages/frontend/design-system/src/detail-panel.test.tsx`, extend the existing Corvu interaction coverage with structural/behavioral assertions for a pointer-down during an in-flight settlement. Mock the panel rectangle at an intermediate height and verify the next drag uses that intermediate height, not `0`, `50%`, or `100%`.

## Boundaries

- Do NOT change DetailPanel public props, state-store contracts, snaps, focus rules, Corvu ownership, gutter, radius, fade geometry, or resize threshold.
- Do NOT change feature consumers, Tooltip, MainSurface, documentation, or opacity policy.
- Do NOT add a dependency, a spring library, `requestAnimationFrame`, or keyframes.
- Do NOT run tests or open a browser under the current task authorization.
- Preserve every unrelated dirty-worktree change. If the cited blocks have materially drifted from commit `f8a1129`, STOP and report instead of replacing them wholesale.

## Verification

- **Mechanical**: update coverage but do not run tests; run `pnpm design:lint`, `pnpm typecheck`, and `git diff --check`. Report the exact failures if repository-wide concurrent errors remain.
- **Feel check**: only with explicit browser authorization, begin collapse or maximize, grab the header midway through the 280ms snap, and confirm the panel stays under the pointer without jumping. Repeat rapidly in both directions and at 10% playback speed. Confirm direct drag remains immediate and release still selects the nearest of collapsed, half, or full.
- **Reduced motion**: with explicit browser authorization, enable `prefers-reduced-motion: reduce` and confirm terminal size changes are immediate while drag remains continuous.
- **Done when**: every interaction path targets the same size source, and interrupting a snap starts from the currently painted height.
