# Fluid motion principles (with reasons)

Distilled from fluid-functionalism (`motion-guidelines.md`, `custom-motion.md`,
`craft.md`, pinned in SKILL.md) and from building Polterware's Switch. Numbers are
upstream's; keep them unless you have measured a reason to differ.

Contents: springs · exits · transform-only · fluid hover · shared indicators ·
weight without reflow · icon swaps · collapse · press geometry · gestures ·
selection acknowledgment · reduced motion · concentric radii · when not to animate

## Springs, not durations

| Token | Duration | Bounce | Use |
|---|---|---|---|
| `spring.fast` | 0.08s | 0 | hover, focus rings, fades, tooltips, small selection indicators |
| `spring.moderate` | 0.16s | 0 | short travel: switch thumb, tab/selection indicators, drawers, dropdown highlight |
| `spring.slow` | 0.24s | 0.12 | large surfaces: dialogs, side panels, stepped flows |

- **Rule**: the bigger the thing that moves, the slower the spring. Unrelated parts of
  the UI then move at consistent magnitudes.
- Spring enters are *interruptible*: a new target keeps the current velocity, so a
  half-finished animation reverses from where it is instead of finishing first. A CSS
  `transition` with a fixed cubic-bezier restarts its curve and feels stiff by
  comparison. That difference is what people perceive as "travado".
- `moderate` is critically damped: same perceived speed as a bouncy tier, lands with no
  overshoot. Measured curve for `{ duration: 0.16, bounce: 0 }`: 0ms 0, 40ms 0.67,
  80ms 0.94, 120ms 0.99, 160ms 1.00. Identical in `motion` v12 and v13.
- Retrofitting: most hand-written values sit near a tier; adopt the token. A value that
  is deliberately outside the tiers (a cinematic entrance) may stay, but name it once
  next to its owner so it cannot drift. Recommend, do not bulldoze.

## Exits are quicker tweens

`spring.fast.exit` 0.06s, `moderate.exit` 0.12s, `slow.exit` 0.16s, plain tweens with no
bounce. A dismissal is a decision already made, so it should read crisp and final rather
than replay the entrance in reverse. In CSS, exit keyframes use `--duration-fast` (120ms)
where the enter uses `--duration-control`/`--duration-enter`. An animation that flips a
target instead of unmounting picks its transition off state:
`open ? spring.fast : { type: "tween", ...spring.fast.exit }`.

## Transform and opacity only

Compositor-friendly and reduced-motion friendly. Layout properties (`top`, `left`,
`width`, `height`) trigger layout and are not neutralised by reduced-motion handling.
Bounded exceptions: the switch thumb's width/height (2-4px) and measured-height
collapses. A plain CSS transition on a positional property is exempt from any JS
reduced-motion handling, so guard it with `@media (prefers-reduced-motion: reduce)`.

## Fluid hover: one highlight per list

A per-row `:hover` background blinks off in the gap between rows and looks like separate
events. Instead one absolutely positioned highlight travels (on `transform`) to the item
nearest the pointer.
- Picking rule (a pure function): an item the pointer is inside always wins; otherwise
  the item whose centre is nearest wins; ties keep the first. Axis: `y` lists, `x` strips,
  `xy` grids (Euclidean distance to centres).
- Fresh pointer entry fades the highlight in at the nearest row (a session counter),
  rather than sliding from wherever it was last. `from` seeds a start rect (a menu's
  checked row).
- **Gap click**: a click in a gap, padding, or past the last row is routed to the lit
  item's activator: what is lit is what a click hits. Cap it (`maxDistance`) or disable
  it where empty space must stay inert (destructive rows, roomy grids).
- Only click targets register (a non-clickable card must not promise a click). Disabled
  rows stay registered but are skipped. One list per group of alternatives; a divider
  between different kinds of rows starts a new list; children ride the parent's list.
- Gate the overlay on "measured": mounting against a rect a later pass corrects makes the
  highlight slide in from another row. Measurement coalesces into one rAF pass and never
  publishes zeroed rects.
- Skip it when a wrong click would hurt, when only some items are clickable, when there
  is a lot of empty space, or when rows move as you scroll.

## Shared indicators glide

A selection/active indicator is one element that springs (`moderate`) between items
instead of fading out on one and in on another. Tabs: the active pill travels; the hover
pill is born at the selected pill and returns to it on leave; the active pill dims
slightly while another tab is hovered. Runs of adjacent selected rows merge into one
block that merges and splits as picks bridge or break the run (stable run ids so blocks
morph instead of exit+enter).

## Weight without reflow

Selected/active text gets heavier; heavier is wider; animating weight on a bare text node
reflows neighbours. Reserve the width with an invisible copy at the heaviest weight (a
2-layer `inline-grid`) and animate `font-variation-settings` on the visible one. Resting
`normal` -> active `semibold` is 400 -> 550, with a tighter optical size paired with the
heavier weight so advance width barely changes. The transition list must include
`font-variation-settings`. Requires a variable font with `wght` (and ideally `opsz`);
check the app font before adopting. Also thicken icon strokes 1.5 -> 2 on hover/selected
and shift muted -> foreground over 80ms.

## Icon swaps

Two glyphs mounted in one icon-sized grid cell so the slot never resizes; crossfade
opacity 1<->0, blur 0<->4px, scale 1<->0.6. The arriving glyph rides the tier's full
duration (`ease-out`), the leaving one its exit token (`ease-in`), so an appear always
outlasts a disappear. Tweens on purpose: a critically damped spring settles early and
would invert that order, and blur is a `filter` string a spring cannot drive. Keep the
accessible name stable (hidden live-region text for "Copied").

## Collapse to a measured pixel height

Animate to `element.offsetHeight` (transform-immune), not `auto`: an `auto` target is
resolved from the visual size, which is wrong under a scaled ancestor and overshoots.
- Spring only when *this* wrapper toggles; if a child's collapse changed the measurement,
  snap (`duration 0`). Otherwise the outer wrapper chases a moving target every frame and
  lands late (upstream measured sub-menu 218ms vs group 326ms, ~80px apart mid-flight).
- Keep closed panels mounted for measurement and mark them `hidden` only after the exit
  finishes, so the animation is not cut and the a11y tree stays correct.
- A panel open at mount snaps to its first measured height instead of animating.
- Opacity runs slightly ahead of height so content dissolves instead of being sliced.

## Press geometry, not scale

`scale` warps (2% of a 400px button is 8px sideways but under 1px vertically). Upstream
Button insets the surface 1px and collapses a same-colour 1px box-shadow spread on press
so the surface shrinks exactly 1px per side at any width. Press releases slowly (180ms) and
presses fast (80ms). Switch thumb: hover widens +2px, press widens +4px and squashes -4px
height (recentred); when checked the extra width grows leftward so the thumb stays flush
right (compact: +2 / +3 / -3).

## Gestures

- Pointer capture on the row, a 2px dead zone to tell drag from click, thumb clamped to
  the track, decision at the track midpoint on release, spring to the resting place.
- A `didDrag` flag swallows the click that follows the same pointerup; reset on the next
  animation frame. A cancelled gesture snaps back without toggling.
- Hover state only for `pointerType === "mouse"` so touch never sticks in a hover shape.
- Initial mount sets position with no animation, so a pre-checked control never animates
  on load.
- Cache geometry; never measure in `pointermove`.

## Selection acknowledgment

When a pick closes a popup, hold it ~300ms so the check drawing in and the marker
springing to the picked row are seen instead of cut by the close fade. Escape, outside
press and trigger toggle still close immediately. The check mark draws itself
(`pathLength` 0 -> 1 in 0.08s ease-out, back in 0.04s ease-in); its slot has fixed width so
the row never resizes.

## Reduced motion

Fewer and gentler, not none. Keep opacity and colour fades (they aid comprehension); drop
transform travel, scale, stretch, and layout shifts. In JS check
`matchMedia("(prefers-reduced-motion: reduce)")` at the `animate` call (set values directly
instead); in CSS use `@media (prefers-reduced-motion: reduce)`. Autoplay and infinite loops
do not run.

## Concentric radii

Nested rounded surfaces: `inner = max(0, outer - inset - border)`. Past ~24px, with
asymmetric gaps, or when the inner surface does not reach the corner, tune by eye. We only
have radius tokens (`--radius-control|item|card|surface`); do not invent pixel radii.

## When not to animate

High-frequency keyboard actions (list navigation with held keys), bulk resets (expand all),
first paint, and anything the user did on purpose thousands of times a day should snap or
use `fast` at most. Motion must explain a state change, not decorate it.
