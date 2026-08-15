# 003 — Animate notification bulk removal into the empty state

- **Status**: TODO
- **Commit**: 497766a
- **Severity**: MEDIUM
- **Category**: Missed opportunities; interruptibility
- **Estimated scope**: 2 files, approximately 100–150 lines including tests

## Problem

The Notifications drawer owns a local read-state overlay. `Mark all as read` applies every update synchronously:

```tsx
// packages/features/shell/src/notification-drawer.tsx:25 — current
<Button
  size="sm"
  variant="ghost"
  onClick={() => {
    setLocalItems(items().map((n) => ({ ...n, read: true })));
    props.onMarkAllRead?.();
  }}
>
  Mark all as read
</Button>
```

When the active filter is `unread`, `filterNotifications` immediately removes every card and the plain empty message mounts in the same render:

```tsx
// packages/features/shell/src/notification-drawer.tsx:54 — current
<div class="flex-1 overflow-auto p-2">
  <For each={visible()}>
    {(n) => (
      <article class={`mb-2 rounded-[var(--radius-card)] border ...`}>
        {/* notification content */}
      </article>
    )}
  </For>
  <Show when={visible().length === 0}>
    <p class="px-2 py-8 text-center text-[12px] text-[var(--color-text-muted)]">
      No notifications
    </p>
  </Show>
</div>
```

The result teleports from a populated list to an empty state. This is an occasional explicit action, so a 160ms exit clarifies what happened without animating routine filter results or operational data.

## Target

**Dependency**: execute plan `001-shared-presence-selection-toolbar.md` first. This plan consumes the exported `MotionPresence` primitive and must not add feature CSS.

Only `Mark all as read` while `filter() === "unread"` receives list-exit motion:

- each currently visible unread card: settled → `opacity: 0; transform: translateY(-4px)` over `160ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))` through `motion="lift"`;
- the empty state: `opacity: 0; transform: translateY(4px)` → settled over `160ms var(--ease-enter, cubic-bezier(0.16, 1, 0.3, 1))` through `motion="swap"`;
- reduced motion: no translation, opacity only over `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))`.

Do not animate the `All` or `Approvals` filter result set. Under those filters the cards remain present after being marked read; update their semantic surface immediately as today.

Use a stable set of exiting IDs:

```tsx
const [exitingUnreadIds, setExitingUnreadIds] =
  createSignal<ReadonlySet<string>>(new Set());

const markingAllRead = () => exitingUnreadIds().size > 0;
```

The action must follow this lifecycle:

1. If the active filter is not `unread`, immediately mark every item read and call `onMarkAllRead` once.
2. If there are no visible unread items, do nothing; do not fire a duplicate callback.
3. Otherwise snapshot the current visible unread IDs into `exitingUnreadIds`; do not mark them read yet.
4. Render each visible item through `MotionPresence present={!exitingUnreadIds().has(n.id)} motion="lift" asChild` so the semantic `<article>` remains the DOM node Ark observes.
5. On each card's `onExitComplete`, mark only that notification read and remove only that ID from the pending set.
6. When the final ID finishes, call `props.onMarkAllRead?.()` exactly once.
7. While exits are pending, disable the footer action and the three filter buttons for at most 160ms. This prevents a filter switch from re-entering half-completed cards.
8. Render `No notifications` through `MotionPresence present={visible().length === 0 && !markingAllRead()} motion="swap" asChild`. It enters only after the last card has committed its read state.

If the drawer closes while exits are pending, a `createEffect` watching `props.open` must finalize every pending read state and call `onMarkAllRead` once before Ark unmounts the drawer content. Also finalize during component cleanup as a last resort. Do not lose the user's explicit action because the surface closed mid-transition.

## Repo conventions to follow

- `packages/features/shell/src/fixtures.ts:220` defines filtering: only `unread` removes items based on `read`; `approvals` filters by category.
- `packages/frontend/design-system/src/toast.tsx:29` is the current exemplar for keeping transient content mounted through an exit phase and providing a fallback completion path.
- Plan 001's `MotionPresence` owns Ark lifecycle, opacity CSS, exact duration/easing values, reduced motion, `inert`, and `aria-hidden`.
- Keep every UI string in English and preserve existing Notification drawer structure and callbacks.

## Steps

1. Add `MotionPresence` to the design-system import in `packages/features/shell/src/notification-drawer.tsx`. Add `createEffect` and `onCleanup` to the Solid import for pending-action finalization.
2. Add the `exitingUnreadIds` signal and small pure helpers for applying read state to one ID or all IDs. Never mutate the existing Set in place; always publish a new Set.
3. Replace the current footer handler with the lifecycle above. Disable the footer button while `markingAllRead()` is true.
4. Disable the filter buttons during the exit window. Preserve their labels, selected paint, and filtering behavior when no bulk action is pending.
5. Wrap each `<article>` in `MotionPresence` with `motion="lift"` and `asChild`. Keep `key={n.id}` behavior through the existing Solid `<For>` identity; do not key by index.
6. Replace the empty-state `<Show>` with `MotionPresence motion="swap" asChild`. Do not add icons, celebration, or explanatory copy.
7. Add `packages/features/shell/src/notification-drawer.test.tsx`. Cover the `All`, `Unread`, and `Approvals` filters; bulk exit staging; one callback after the final exit; disabled controls during exit; `open` changing to false mid-exit; component cleanup finalization; and reduced-motion-independent state semantics. Mock only the timing/animation boundary, not the domain filter function.

## Boundaries

- Do NOT animate filter changes, card insertion, notification arrival, drawer open/close, card background, or action-button presses.
- Do NOT add CSS under `packages/features/shell`.
- Do NOT change `filterNotifications`, fixture data, item ordering, action callbacks, or drawer dimensions.
- Do NOT stagger the cards. All selected cards exit together and remain non-blocking.
- Do NOT call `onMarkAllRead` once per item; call it exactly once per user action.
- Do NOT add a dependency.
- If plan 001 is not implemented or `MotionPresence` differs from its specified API, STOP and report the dependency mismatch.

## Verification

- **Mechanical**:
  - `pnpm exec vitest run packages/features/shell/src/notification-drawer.test.tsx packages/frontend/design-system/src/presence.test.tsx` — expected all focused tests pass.
  - Run the narrowest Shell package typecheck with an explicit exit status. If the existing workspace `vitest`/CSS baseline blocks it, preserve the exact blocker.
  - `node tooling/check-ui-opacity.mjs` — expected no feature-owned opacity violations.
  - `pnpm design:lint` — expected no new design violations; report the known pnpm signature blocker if it prevents startup.
  - `git diff --check` — expected no whitespace errors.
- **Feel check**: only when Erick explicitly authorizes launching the app, run `pnpm dev:desktop`, open Notifications, select `Unread`, and trigger `Mark all as read`. Confirm:
  - all unread cards lift only 4px and fade together for 160ms;
  - the empty message enters after the cards finish, never underneath them;
  - buttons remain disabled only during the brief exit;
  - repeated clicking cannot double-fire the callback;
  - closing the drawer during exit still preserves the read action;
  - in DevTools at 10% playback, no card height, margin, padding, top, or left property animates;
  - with `prefers-reduced-motion: reduce`, cards do not move but retain the 120ms opacity cue.
- **Done when**: the unread bulk action has a deterministic exit-to-empty bridge, callbacks fire once, filter behavior is unchanged, and no feature-local opacity is introduced.
