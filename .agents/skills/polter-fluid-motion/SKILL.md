---
name: polter-fluid-motion
description: >-
  Fluid, spring-based UI motion for Polterware's own stack (SolidJS, vanilla
  `motion`, Ark UI Solid, Corvu, Electron renderer, our CSS tokens), modeled on
  the fluid-functionalism library. Use whenever the user asks to animate,
  polish, "make it feel fluid/smooth/alive", fix janky or stiff transitions, add
  hover/press/drag/enter/exit motion, or to audit or refactor animation across
  the app (animacao, animações, transição, mola, spring, hover fluido, indicador
  deslizante, tabs, switch, dropdown, dialog, drawer, tooltip, popover, tabela,
  sidebar, lista). Also use before hand-writing any CSS transition, keyframe,
  hover background, or `animate()` call in packages/frontend/design-system or
  packages/features, so new motion follows the shared spring tokens instead of
  inventing durations. Use it even when the user never says "fluid".
---

# Polter fluid motion

Polterware's UI should move like [fluid-functionalism](https://www.fluidfunctionalism.com):
every state change is legible, interruptible, and cheap. That library is React +
framer-motion + Radix. Ours is **SolidJS + vanilla `motion` + Ark UI Solid** inside
Electron, so this skill keeps the *behavior and numbers* and swaps the *mechanism*.

The reference implementation already in the repo is the Switch:
`packages/frontend/design-system/src/switch.tsx` (+ `switch.css`) and the shared
tokens in `packages/frontend/design-system/src/springs.ts`. Read both before
writing new motion; they show the house style end to end.

## Priority: the fluid result comes first

The goal is motion that feels fluid and beautiful, the way fluid-functionalism does.
Everything else serves that. So:
- **Decide the target behavior first** (the glide, the stretch, the shared highlight,
  the interruptible spring) using `references/principles.md` and the upstream craft, then
  pick whatever mechanism reaches it. Never shrink the effect to fit what a primitive's own
  animation hooks can do (for example Ark's CSS keyframe/`--height` model); if the
  primitive's way is stiffer than the target, animate it yourself with `motion`.
- **Primitives (Ark, Corvu) own behavior and accessibility, not the look of the motion**:
  state, focus, keyboard, `aria-*`, placement, dismissal. Use what they expose
  (`data-state`, `data-placement`, measured values or CSS variables) only when it makes the
  fluid result easier or more robust, and never as a reason to skip the spring.
- Do not install a new package (any Corvu or Ark add-on, or another animation lib) without
  asking Erick; Ark is already fully available, Corvu only has `@corvu/resizable`.

## Stack facts (verify if the task depends on them)

- Renderer: SolidJS 1.9 (fine-grained; no virtual DOM re-render). Motion library:
  `motion` (vanilla; `animate`, `motionValue`), a dependency of the design system.
  Never import `framer-motion` React APIs, `motion/react`, or `solid-motionone`
  unless Erick asks; do not add another animation dependency.
- Primitives: Ark UI Solid (`menu`, `tooltip`, `tabs`, `switch`, `popover`,
  `drawer`, `dialog`) for behavior and a11y; Corvu (`@corvu/resizable`) for panels;
  TanStack Solid Virtual for long lists. Ark parts expose `data-state`,
  `data-placement`, `data-focus-visible`, `data-disabled`: style off those.
- CSS tokens (in `packages/configuration/tailwind/base.css`): `--duration-fast`
  120ms, `--duration-control` 160ms, `--duration-enter` 200ms, `--ease-standard`,
  `--ease-enter`, radius tokens. Motion tokens for JS springs: `spring.fast /
  moderate / slow` (+ `.exit`) from `springs.ts`.
- Electron: one renderer per window, no SSR/hydration, `prefers-reduced-motion`
  comes from the OS. macOS shell chrome may use vibrancy; product chrome is opaque.

## The system in eight rules (why each exists is in references/principles.md)

1. **Three spring tiers, never a hand-written duration.** fast 0.08s (hover, focus,
   fades, small indicators), moderate 0.16s critically damped (short travel: switch
   thumb, tab/selection indicators, drawers), slow 0.24s bounce 0.12 (dialogs, big
   panels). The bigger the thing that moves, the slower the spring. Springs (not
   tweens) for enters, so an interrupted animation reverses from where it is.
2. **Exits are quicker tweens**, one tier faster (`spring.<tier>.exit`), so a
   dismissal reads crisp instead of replaying the entrance backwards.
3. **Move `transform` and `opacity`.** Position and size changes on the compositor;
   layout properties only when the value is genuinely tiny and bounded (the switch
   thumb animates width/height by 2-4px on purpose).
4. **One hover highlight per list**, gliding to the nearest item, never per-row
   `:hover` backgrounds next to components that glide. One selection indicator that
   travels between items (tabs, menus, radio-like groups), never fade-out/fade-in.
5. **Weight without reflow**: ghost-span sizing + variable font axis; icon strokes
   thicken on hover; icon swaps crossfade in one fixed grid cell.
6. **Collapse to a measured pixel height**, never `auto`; snap when a child caused
   the resize (no stacked springs).
7. **Gestures are honest**: pointer capture, a small dead zone, suppress the click
   after a drag, hover only for `pointerType === "mouse"`, no animation on first paint.
8. **Reduced motion means fewer and gentler, not none**: drop travel/scale, keep
   opacity and colour fades.

## Choose the mechanism

| Situation | Use |
|---|---|
| Colour, background, border, opacity fade, and enter/exit presence that already reads as fluid | CSS with `--duration-*` / `--ease-*` tokens, keyed off Ark `data-state` |
| Anything that must be interruptible, share velocity, or travel between targets (thumb, indicator, highlight, drag) | `motion`: `motionValue` + `animate(value, target, spring.<tier>)`, painted with `value.on("change", ...)` |
| Height/width collapse | `animate` a `motionValue` to a measured `offsetHeight` |
| Exit of a Show/Presence region | keep it mounted until the exit finishes (Ark `Presence`/`data-state="closed"` or an `animate(...).finished` then unmount), with a timeout fallback |
| Purely decorative loops (shimmer, spinner) | CSS keyframes; must be off under reduced motion |

Do not reach for `motion` when a 120ms colour transition is enough; it costs JS and
loses the compositor path. Do reach for it whenever the target is the fluid feel and CSS
would give the "stiff" one (fixed cubic-bezier, no velocity carry-over, competing
transitions, no shared indicator), even if the primitive ships a CSS animation for it.

## Solid-specific traps (each one bit us)

- **`For` recreates rows when items are new objects.** A rebuilt list (filters,
  refetched data) destroys the switch/indicator and the new one appears already in
  its final state, so nothing animates. Use `Index` (rows persist, item is an
  accessor) or keyed identity for any list whose items animate on state change.
- **Effects must clean up**: `onCleanup(() => controls.stop())`, unsubscribe
  `motionValue.on("change")`, cancel rAF; the first effect run must snap (no
  animation) and enable springs after first paint.
- **Never call `getComputedStyle`/measure in `pointermove` or per frame**; cache
  geometry (constants or a measurement on mount/resize).
- **Spreading handlers on an Ark part overrides the part's own handlers.** Compose
  (call both) or attach to a wrapper.
- **Read reactive values inside the effect** (`props.checked`, signals) or the
  target will not update; do not read them in cleanup or event handlers expecting
  tracking.
- Reduced motion helper: check `matchMedia("(prefers-reduced-motion: reduce)")`
  where you call `animate`; CSS gets `@media (prefers-reduced-motion: reduce)`.

## Two modes

**Implement** (the user names a component or interaction): read the upstream craft
for it (`references/component-index.md` -> `references/upstream-craft.md` section),
read our counterpart and its CSS, then port the behavior with the recipes in
`references/solid-motion-recipes.md`. Reuse `springs.ts`; add new shared pieces (for
example a `createFluidHover`) to the design system, not inside a feature.

**Audit** (the user asks where to improve): follow `references/polterware-audit.md`.
Grep for the smells below, group findings by surface, rate each by *what the person
sees* (frequency x visibility) and effort, and return 3-7 ranked proposals. Propose
before changing; a motion the product never had is a question for Erick, fixing one
it already does badly is a finding.

Smells worth grepping: per-row `:hover` backgrounds (`hover:bg`, `:hover` in list/table
CSS); literal durations/easings instead of tokens (`transition: ... 200ms`,
`cubic-bezier(` outside `base.css`); `transition: all`; animating `width|height|top|left`
in CSS; `For` around animated rows; selection/active states that fade instead of glide;
popups without side-aware origin; no `prefers-reduced-motion` guard on keyframes;
`height: auto` collapses; icon swaps that pop.

## Polterware rules that override the upstream look

- Follow `AGENTS.md` and `DESIGN.md`: shared components first, radius and colour
  tokens only, opaque structural surfaces, Hugeicons Free Stroke Rounded only.
  `pnpm design:lint` rejects `opacity`/alpha colours in authored CSS: express dimming
  through colour tokens, or through `motion` values on `opacity` in JS when it is a
  real transition (upstream uses opacity for hover pills; check the linter first).
- All app strings in English; code, comments, commits in English.
- Do not run a browser, tests, dev servers, or builds unless Erick asked in the
  current request. Validate with typecheck, `pnpm design:lint`, `git diff --check`,
  and code reading. When Erick does ask for browser validation, sample the animated
  element with `requestAnimationFrame` (see Verification) instead of judging by eye.
- Preserve unrelated diffs; the worktree usually has other agents' changes.

## Verification

- Static: `pnpm typecheck`, `pnpm design:lint`, `git diff --check`.
- With browser permission: log `getBoundingClientRect()` of the moving part per
  frame around a real click and check (a) it starts on the frame after the input,
  (b) the curve matches the tier (moderate: ~67% at 40ms, ~94% at 80ms, done at
  160ms, no overshoot), (c) the element is the *same node* before and after (a
  replaced node means `For` recreated it), (d) frames are ~8-16ms apart.
  Note: a scratch Vite server under `node_modules/` does not see file edits (restart
  it), and Playwright's click cannot hold the mouse down (press/drag need a real
  gesture or a manual check).

## References (read on demand)

- `references/principles.md`: the full system with reasons and numbers.
- `references/solid-motion-recipes.md`: copy-ready Solid + `motion` patterns.
- `references/component-index.md`: every upstream component/system with source, docs,
  and registry URLs, tier, craft section, and our counterpart.
- `references/upstream-craft.md`: upstream per-component craft, verbatim (MIT, pinned).
- `references/polterware-audit.md`: our components mapped to fluid upgrades, ranked.

Upstream is pinned to commit `1add7762eea3b5bbb3a1f44bea296e13c3f119c0`. For anything
this skill does not cover, fetch the raw source URL from the index rather than guessing.
