# 001 — Add shared presence motion and animate the selection toolbar

- **Status**: TODO
- **Commit**: 497766a
- **Severity**: MEDIUM
- **Category**: Missed opportunities; cohesion and tokens
- **Estimated scope**: 10 files, approximately 220–300 lines including tests and policy updates

## Problem

`DataTable.SelectionToolbar` is a shared, transient action surface used throughout Organizations, Inbox, Knowledge, Operations, Integrations, and Access. It currently mounts and unmounts immediately when the selected row count crosses zero:

```tsx
// packages/frontend/design-system/src/table.tsx:747 — current
const toolbar = () => (
  <Show when={state.selectedCount() > 0}>
    <div
      class={`pw-data-table-selection-toolbar surface-panel-float ${props.class ?? ""}`}
      role="toolbar"
      aria-label="Selected row actions"
    >
      <span class="pw-data-table-selection-summary" aria-live="polite">
        {state.selectedCount()} selected
      </span>
      {/* actions and Clear selection */}
    </div>
  </Show>
);
```

The toolbar is positioned 12px above the `MainSurface` bottom edge, so the instantaneous mount has no spatial explanation. A short rise/fall transition will connect the toolbar to that edge without moving any table row or operational data.

Opacity animation cannot be added directly to `table.css` or feature CSS. The repository allows it only in centralized design-system presence CSS:

```js
// tooling/check-ui-opacity.mjs:41 — current
const PRESENCE_MOTION_PATHS = new Set([
  "packages/frontend/design-system/src/dialog.css",
  "packages/frontend/design-system/src/drawer.css",
  "packages/frontend/design-system/src/dropdown-menu.css",
  "packages/frontend/design-system/src/popover.css",
  "packages/frontend/design-system/src/toast.css",
]);
```

Plans 003, 004, and 005 also need the same lifecycle. This plan must therefore add one Ark-backed local primitive rather than four feature-owned presence implementations.

## Target

Add a design-system `MotionPresence` wrapper over Ark UI 5.38.0 `Presence`, imported from `@ark-ui/solid/presence`. The installed package already exports `present`, `lazyMount`, `unmountOnExit`, `onExitComplete`, `asChild`, and `data-state="open|closed"`; do not add or change dependencies.

The public contract must be:

```tsx
export type MotionPresenceMotion = "rise" | "lift" | "swap" | "alert";

export type MotionPresenceProps = Omit<JSX.HTMLAttributes<HTMLDivElement>, "children"> & {
  present: boolean;
  motion: MotionPresenceMotion;
  children: JSX.Element;
  asChild?: boolean;
  onExitComplete?: () => void;
};

export function MotionPresence(props: MotionPresenceProps): JSX.Element;
```

The wrapper must:

- import Ark only inside `@polterware/frontend-design-system`;
- pass `present`, `lazyMount`, `unmountOnExit`, `asChild`, and `onExitComplete` to Ark Presence;
- render `class="pw-motion-presence ..."` and `data-motion={props.motion}`;
- set `inert={!props.present}` and `aria-hidden={!props.present ? "true" : undefined}` so exiting controls cannot receive input;
- forward remaining HTML props and merge caller classes without overwriting them.

Create `packages/frontend/design-system/src/presence.css` with only `transform` and `opacity` animation. Use the existing repository tokens exactly; do not add parallel ease or duration tokens.

| Motion | Open keyframe | Closed keyframe | Open timing | Closed timing |
| --- | --- | --- | --- | --- |
| `rise` | `opacity: 0; transform: translateY(4px)` → settled | settled → `opacity: 0; transform: translateY(4px)` | `160ms var(--ease-enter, cubic-bezier(0.16, 1, 0.3, 1))` | `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))` |
| `lift` | `opacity: 0; transform: translateY(4px)` → settled | settled → `opacity: 0; transform: translateY(-4px)` | `160ms var(--ease-enter, cubic-bezier(0.16, 1, 0.3, 1))` | `160ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))` |
| `swap` | `opacity: 0; transform: translateY(4px)` → settled | settled → `opacity: 0; transform: translateY(-4px)` | `160ms var(--ease-enter, cubic-bezier(0.16, 1, 0.3, 1))` | `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))` |
| `alert` | `opacity: 0; transform: translateY(-2px)` → settled | settled → `opacity: 0; transform: translateY(-2px)` | `120ms var(--ease-enter, cubic-bezier(0.16, 1, 0.3, 1))` | `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))` |

Use explicit `@keyframes` per direction because Ark Presence 5.38.0 delays `unmountOnExit` by observing `animationstart`, `animationcancel`, and `animationend`. Do not replace these animations with CSS transitions unless the Ark lifecycle is also changed and proven.

Reduced motion must retain opacity feedback while removing movement:

```css
@keyframes pw-motion-presence-static-in {
  from { opacity: 0; transform: translateY(0) scale(1); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

@keyframes pw-motion-presence-static-out {
  from { opacity: 1; transform: translateY(0) scale(1); }
  to { opacity: 0; transform: translateY(0) scale(1); }
}

@media (prefers-reduced-motion: reduce) {
  .pw-motion-presence[data-state="open"] {
    animation-name: pw-motion-presence-static-in;
    animation-duration: var(--duration-fast, 120ms) !important;
    animation-timing-function: var(--ease-standard, cubic-bezier(0.2, 0, 0, 1));
  }

  .pw-motion-presence[data-state="closed"] {
    animation-name: pw-motion-presence-static-out;
    animation-duration: var(--duration-fast, 120ms) !important;
    animation-timing-function: var(--ease-standard, cubic-bezier(0.2, 0, 0, 1));
  }
}
```

For the selection toolbar, use `motion="rise"`. Preserve the last non-empty selection snapshot during exit so the toolbar does not flash `0 selected` or change its actions while fading out.

## Repo conventions to follow

- Motion tokens are defined at `packages/configuration/tailwind/base.css:78`: `--ease-standard`, `--ease-enter`, `--duration-fast`, `--duration-control`, and `--duration-enter`.
- `packages/frontend/design-system/src/toast.tsx:29` demonstrates delayed unmount with a visible/exiting phase; Ark Presence replaces that manual lifecycle for this new shared primitive.
- `packages/frontend/design-system/src/dropdown-menu.css:47` demonstrates open/closed keyframes, exact token fallbacks, and a gentler reduced-motion animation.
- `packages/frontend/design-system/src/dialog.tsx:9` demonstrates the required rule that Ark imports remain inside the design-system package.
- `DESIGN.md:148` and `.specify/memory/constitution.md:179` require element-opacity animation to remain centralized.
- Official Ark UI 5.38.0 Presence API: `https://ark-ui.com/docs/utilities/presence`.

## Steps

1. Create `packages/frontend/design-system/src/presence.tsx` with the exact `MotionPresence` contract above. Use `splitProps` to separate `present`, `motion`, `children`, `asChild`, `onExitComplete`, and `class`; forward every other HTML prop to Ark Presence.
2. Create `packages/frontend/design-system/src/presence.css` with separate open/closed keyframes for `rise`, `lift`, `swap`, and `alert`, plus the static reduced-motion keyframes. Animate only `transform` and `opacity`. Do not add `will-change`, blur, layout-property animation, or `transition: all`.
3. Export `MotionPresence`, `MotionPresenceProps`, and `MotionPresenceMotion` from `packages/frontend/design-system/src/index.ts`.
4. Add `packages/frontend/design-system/src/presence.css` to `PRESENCE_MOTION_PATHS` in `tooling/check-ui-opacity.mjs`. Update `tooling/check-ui-opacity.test.mjs` so the allowlist test proves opacity is accepted in `presence.css` and still rejected in feature CSS.
5. Update `DESIGN.md` in the Motion section and `.specify/memory/constitution.md` in the opacity/presence rule to name `MotionPresence`/`presence.css` as the shared Ark-backed path for non-overlay conditional presence. Do not loosen the rule for feature-owned opacity.
6. In `packages/frontend/design-system/src/table.tsx`, import `MotionPresence`. Inside `DataTableSelectionToolbar`, keep a signal containing the last non-empty `context.selectedRows()` array. Derive the displayed `selectedRows`, IDs, and count from that snapshot; update it whenever the live selection is non-empty, and clear it only from `onExitComplete` when the live selection is still empty.
7. Replace the `<Show when={state.selectedCount() > 0}>` wrapper with `<MotionPresence present={context.selectedRows().length > 0} motion="rise" ...>`. Put the existing toolbar classes, `role`, and `aria-label` on `MotionPresence` itself so the portal/host layout does not gain an extra box. Keep all existing action semantics and `Clear selection` behavior.
8. Add `packages/frontend/design-system/src/presence.test.tsx`. Verify lazy mount, `data-state`, `data-motion`, inert/`aria-hidden` during exit, unmount after the closing animation, `onExitComplete`, `asChild`, and reopening before exit completion.
9. Extend `packages/frontend/design-system/src/table.test.tsx` with a hosted selection-toolbar test. Select a row, confirm the toolbar mounts with the correct count, clear selection, confirm the outgoing toolbar retains the previous count and becomes inert, dispatch the closing animation end, and confirm it unmounts. Re-select before animation end and confirm it returns to open state without a stale unmount.

## Boundaries

- Do NOT animate table rows, selection checkboxes, sorting, filtering, or route/tab content.
- Do NOT change `DataTable` selection semantics, portal ownership, focus behavior, or action eligibility.
- Do NOT import Ark Presence from feature packages.
- Do NOT add a dependency; `@ark-ui/solid` 5.38.0 is already declared by the design-system package.
- Do NOT add feature CSS to the opacity allowlist. Only the new centralized `presence.css` is allowed.
- Do NOT change the settled repository easing tokens to the similarly named curves from the generic audit catalog.
- If Ark Presence no longer emits `data-state="open|closed"` at execution time, STOP and report version drift instead of inventing a custom lifecycle.

## Verification

- **Mechanical**:
  - `node --test tooling/check-ui-opacity.test.mjs` — expected exit 0.
  - `node tooling/check-ui-opacity.mjs` — expected no new opacity violations.
  - `pnpm exec vitest run packages/frontend/design-system/src/presence.test.tsx packages/frontend/design-system/src/table.test.tsx` — expected all focused tests pass.
  - `pnpm exec tsc --noEmit -p packages/frontend/design-system/tsconfig.json` — expected exit 0; do not use the package script because it contains `|| true`.
  - `pnpm design:lint` — expected exit 0. If the known pnpm signature baseline blocks startup, preserve the exact output and do not claim this gate passed.
  - `git diff --check` — expected no whitespace errors.
- **Feel check**: only when Erick explicitly authorizes launching the app, run `pnpm dev:desktop`, select and clear rows in Organizations, Inbox, and one multi-select table, and confirm:
  - the toolbar rises only 4px and never moves rows;
  - the count and actions remain stable during exit;
  - clearing and immediately reselecting does not flash, jump, or unmount the reopened toolbar;
  - keyboard focus stays on the selected row or control; the toolbar never steals focus;
  - in DevTools Animations at 10% speed, entry is 160ms ease-enter and exit is 120ms ease-standard;
  - with `prefers-reduced-motion: reduce`, translation disappears but the 120ms opacity cue remains.
- **Done when**: all shared presence and table tests pass, the opacity gate accepts only centralized `presence.css`, the toolbar has symmetric edge-connected presence, and selection behavior is unchanged.
