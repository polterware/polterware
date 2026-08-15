# 010 — Group DetailPanel action tooltips

- **Status**: DONE
- **Commit**: 715655e
- **Severity**: MEDIUM
- **Category**: Purpose and frequency
- **Estimated scope**: 4 files, small

## Problem

Every DetailPanel action creates an isolated tooltip root:

```tsx
// packages/frontend/design-system/src/detail-panel.tsx:295 — current
return (
  <Tooltip.Root disabled={buttonProps.disabled}>
    <Tooltip.Trigger ... />
    <Tooltip.Content>{local.label}</Tooltip.Content>
  </Tooltip.Root>
);
```

The shared Tooltip defaults every root to a 400ms opening delay:

```tsx
// packages/frontend/design-system/src/tooltip.tsx:42 — current
<ArkTooltip.Root
  lazyMount
  unmountOnExit
  openDelay={props.openDelay ?? 400}
  closeDelay={props.closeDelay ?? 100}
```

Moving across adjacent header controls therefore repeats the full delay and entrance animation for every button. A dense toolbar should delay only the first tooltip and make subsequent tooltips immediate while the pointer remains in the group.

## Target

Add a design-system `Tooltip.Group` that has no DOM wrapper and coordinates warm state through Solid context.

Exact behavior:

- First tooltip in a cold group keeps the current `400ms` open delay.
- When a grouped tooltip opens, the group becomes warm.
- A tooltip opened while the group is warm uses `0ms` open delay.
- When the last grouped tooltip closes, keep the group warm for the existing `100ms` close-delay window, then return to cold.
- Opening another grouped tooltip during that 100ms window cancels the cooldown.
- Explicit `Tooltip.Root openDelay` and `closeDelay` props continue to override defaults.
- Ungrouped tooltips remain exactly unchanged.
- Tooltip entrance/exit keyframes and durations remain `160ms` enter and `120ms` exit; reduced-motion behavior remains unchanged.

Suggested internal contract:

```tsx
type TooltipGroupContextValue = {
  isWarm: Accessor<boolean>;
  markOpen: () => void;
  markClosed: (cooldown: number) => void;
};

function TooltipGroup(props: { children: JSX.Element }) {
  // Solid context provider only; no rendered wrapper.
}
```

`TooltipRoot` resolves its delays as:

```tsx
const resolvedOpenDelay = () =>
  props.openDelay ?? (group?.isWarm() ? 0 : 400);
const resolvedCloseDelay = () => props.closeDelay ?? 100;
```

Its Ark `onOpenChange` handler must update group state and still call the consumer callback.

## Repo conventions to follow

- Ark UI remains the behavior owner. The group only coordinates the existing Ark root props; do not replace focus, hover, Escape, presence, positioning, or portal behavior.
- `Tooltip` is the local public API in `packages/frontend/design-system/src/tooltip.tsx`; feature code must not import Ark directly.
- Existing tooltip motion stays in `packages/frontend/design-system/src/tooltip.css`.
- Use Solid `createContext`, `createSignal`, `onCleanup`, and `useContext`; do not add a dependency.

## Steps

1. In `packages/frontend/design-system/src/tooltip.tsx`, add the internal group context and `TooltipGroup` provider with one cooldown timer. Clean the timer in `onCleanup`.
2. Make `TooltipRoot` consume optional group context, resolve cold/warm delays exactly as specified, and update group state from Ark's `onOpenChange` without changing the public callback contract.
3. Export the provider as `Tooltip.Group` and add its prop type if the package convention exports compound prop types.
4. In `packages/frontend/design-system/src/detail-panel.tsx`, wrap feature actions and global actions in one `Tooltip.Group` inside the header. The provider must render no layout element, so button adjacency and order remain unchanged.
5. In `packages/frontend/design-system/src/tooltip.test.tsx`, cover cold 400ms, warm 0ms, 100ms cooldown, cooldown cancellation, explicit delay overrides, ungrouped behavior, cleanup, and callback forwarding. Update `detail-panel.test.tsx` to assert both action groups share the provider without adding a DOM wrapper.

## Boundaries

- Do NOT change Tooltip CSS, transform origin, keyframes, visual styling, labels, button order, DetailPanel geometry, or action API.
- Do NOT remove Ark UI, add a global singleton, persist warm state across unrelated groups, or add dependencies.
- Do NOT run tests or open a browser under the current authorization.
- This plan is independent of plan 008 and 009 but should execute last to avoid overlapping DetailPanel header edits.

## Verification

- **Mechanical**: update coverage but do not run it; run `pnpm design:lint`, `pnpm typecheck`, and `git diff --check`.
- **Feel check**: with later browser authorization, hover one DetailPanel action until its tooltip opens, then move across every adjacent action. Only the first tooltip waits 400ms; subsequent labels appear immediately while retaining their 160ms entrance. Leave the header for more than 100ms and confirm the next tooltip is cold again.
- **Reduced motion**: group timing remains correct while tooltip keyframes are disabled by the existing media rule.
- **Done when**: adjacent DetailPanel tooltips share warm timing without changing ungrouped Tooltip behavior or header layout.
