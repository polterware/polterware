# 002 — Move canvas menus onto the shared anchored dropdown

- **Status**: TODO
- **Commit**: 497766a
- **Severity**: MEDIUM
- **Category**: Physicality and origin; missed opportunities
- **Estimated scope**: 5 files, approximately 180–260 lines including tests

## Problem

The canvas bottom bar and contextual toolbars contain eight native `<details>` menus. Their panels mount immediately at feature-owned absolute coordinates and have no trigger-relative presence motion:

```tsx
// packages/features/architecture/src/canvas/canvas-bottom-bar.tsx:97 — current
<details class="relative">
  <summary class="cursor-pointer list-none rounded-[var(--radius-item)] px-1.5 py-0.5 text-[11px] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-raised)]">
    More
  </summary>
  <div class="absolute bottom-full left-0 z-30 mb-1 min-w-[10rem] menu-panel py-1 text-[12px]">
    <MenuBtn label="Change type" onClick={() => props.onChangeType?.()} />
    <MenuBtn label="Copy" onClick={() => props.onCopy?.()} />
    <MenuBtn label="Cut" onClick={() => props.onCut?.()} />
    <MenuBtn label="Highlight dependencies" onClick={() => props.onHighlight?.()} />
    <MenuBtn label="Delete" onClick={() => props.onDelete?.()} />
  </div>
</details>
```

```tsx
// packages/features/architecture/src/canvas/contextual-toolbar.tsx:35 — current
<details class="relative">
  <summary class="cursor-pointer list-none rounded-[var(--radius-item)] px-1.5 py-0.5 text-[11px] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-raised)]">
    More
  </summary>
  <div class="absolute right-0 top-full z-30 mt-1 min-w-[10rem] menu-panel py-1 text-[12px]">
    <MenuBtn label="Change type" onClick={props.onChangeType} />
    {/* more actions */}
  </div>
</details>
```

These are compact action menus, so repository law requires the Ark-backed `DropdownMenu` wrapper. Reusing it provides Escape, focus, outside dismissal, portal positioning, and the existing 160ms/120ms motion without introducing feature-local animation.

The current shared animation always starts at `translateY(-4px)`, including menus placed above a trigger:

```css
/* packages/frontend/design-system/src/dropdown-menu.css:3 — current */
@keyframes pw-dropdown-panel-in {
  from {
    opacity: 0;
    transform: translateY(-4px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
```

For `top-*` placements this moves the menu away from the trigger. The shared CSS must derive the offset sign from placement so every menu grows from its anchor.

## Target

All eight canvas toolbar menus use the exported design-system components:

```tsx
<DropdownMenu positioning={{ placement: "top-start", gutter: 4 }}>
  <DropdownMenuTrigger class="rounded-[var(--radius-item)] px-1.5 py-0.5 text-[11px] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-raised)]">
    More
  </DropdownMenuTrigger>
  <DropdownMenuContent class="min-w-[10rem]">
    <DropdownMenuItem value="change-type" onSelect={() => props.onChangeType?.()}>
      Change type
    </DropdownMenuItem>
    {/* remaining items */}
    <DropdownMenuItem value="delete" danger onSelect={() => props.onDelete?.()}>
      Delete
    </DropdownMenuItem>
  </DropdownMenuContent>
</DropdownMenu>
```

Extend the local wrapper without exposing Ark to features:

```tsx
// packages/frontend/design-system/src/dropdown-menu.tsx — target contract
import { Menu as ArkMenu, type MenuRootProps } from "@ark-ui/solid/menu";

export type DropdownMenuProps = {
  children: JSX.Element;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  defaultOpen?: boolean;
  positioning?: MenuRootProps["positioning"];
};

<ArkMenu.Root
  open={props.open}
  defaultOpen={props.defaultOpen}
  positioning={props.positioning}
  onOpenChange={(details) => props.onOpenChange?.(details.open)}
  lazyMount
  unmountOnExit
>
```

The motion remains inside `dropdown-menu.css` with the current exact timings:

- enter: `160ms var(--ease-enter, cubic-bezier(0.16, 1, 0.3, 1))`;
- exit: `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))`;
- bottom/side placement offset: `translateY(-4px) scale(0.98)`;
- top placement offset: `translateY(4px) scale(0.98)`;
- settled: `translateY(0) scale(1)`;
- reduced motion: no translation or scale, opacity only for `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))`.

Implement the directional offset in the shared stylesheet with a same-element custom property, not a parent-driven variable:

```css
.pw-dropdown-panel {
  --pw-dropdown-enter-y: -4px;
}

.pw-dropdown-panel[data-placement^="top"] {
  --pw-dropdown-enter-y: 4px;
}

@keyframes pw-dropdown-panel-in {
  from {
    opacity: 0;
    transform: translateY(var(--pw-dropdown-enter-y)) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
```

Apply the same variable to the closing keyframe. Keep `transform-origin: var(--transform-origin, top left)` so Zag placement remains authoritative.

Placement requirements:

| Consumer | Placement | Gutter |
| --- | --- | --- |
| Every menu in `CanvasBottomBar` | `top-start` | `4` |
| Node contextual `More` | `bottom-end` | `4` |
| Relationship type menu | `bottom-start` | `4` |
| Multi-selection `Align` / `Distribute` | `bottom-start` | `4` |

## Repo conventions to follow

- `packages/frontend/design-system/src/dropdown-menu.tsx:25` is the only public compact-menu behavior API; feature packages must not import Ark.
- `packages/frontend/design-system/src/dropdown-menu.css:47` already owns menu keyframes, tokens, transform origin, and reduced motion.
- `packages/frontend/design-system/src/select.tsx:55` is the consumer exemplar for composing `DropdownMenu`, `Trigger`, `Content`, `Item`, and `Separator`.
- `DESIGN.md` and `AGENTS.md` require compact dropdowns/context menus to use the Ark-backed design-system wrapper.
- Use `DropdownMenuItem danger` for Delete. Do not create a feature-local menu row style.

## Steps

1. Update `packages/frontend/design-system/src/dropdown-menu.tsx` to import `MenuRootProps`, add `positioning?: MenuRootProps["positioning"]`, and forward it to `ArkMenu.Root`. Do not expose any other Ark type or component to consumers.
2. Update `packages/frontend/design-system/src/dropdown-menu.css` so top placements use a positive 4px Y offset and all other placements retain negative 4px. Use the variable in both open and closed keyframes; keep the current 0.98 scale, timings, easing tokens, and reduced-motion static keyframes.
3. Extend `packages/frontend/design-system/src/dropdown-menu.test.tsx` beyond export-shape coverage. Render a menu with `positioning={{ placement: "top-start", gutter: 4 }}`, open it through the trigger, verify the portal content and menu roles, select an item, verify the callback and dismissal, and verify Escape dismissal. Add a static assertion that a top-placed content node receives `data-placement` after positioning.
4. In `packages/features/architecture/src/canvas/canvas-bottom-bar.tsx`, import the five local DropdownMenu components. Replace all four `<details>` blocks (`More`, relationship type, `Align`, `Distribute`) with `top-start` dropdowns. Preserve labels, callbacks, item order, optional callback behavior, widths, and the relationship list's `max-h-48 overflow-y-auto` class. Mark destructive items with `danger`.
5. In `packages/features/architecture/src/canvas/contextual-toolbar.tsx`, replace all four `<details>` blocks with the placements in the Target table. Preserve trigger typography and every callback. Remove `MenuBtn` only after confirming it has no remaining callers in that file.
6. Remove `MenuBtn` from `canvas-bottom-bar.tsx` only if no remaining callers exist. Keep `BarTextBtn`, `BarIconBtn`, and `TinyBtn` untouched.
7. Add focused render tests for both canvas toolbar files. Assert there are no `<details>` elements, triggers expose menu semantics, each item calls the same prop callback, Delete uses danger styling, and the relationship type menu still renders all registry values.

## Boundaries

- Do NOT animate the canvas, nodes, relationships, contextual toolbar host, or bottom bar host.
- Do NOT change menu labels, ordering, callback behavior, or selection semantics.
- Do NOT migrate the separate pointer-positioned right-click menu in `architecture-canvas-host.tsx`; it requires a dedicated ContextMenu wrapper and is outside this plan.
- Do NOT add a motion library or feature-owned CSS.
- Do NOT change the current 160ms/120ms timings, 0.98 scale, or easing tokens.
- Do NOT import `@ark-ui/solid` from Architecture.
- If Ark 5.38.0 no longer exposes `MenuRootProps["positioning"]`, STOP and report dependency drift instead of weakening the type to `any`.

## Verification

- **Mechanical**:
  - `pnpm exec vitest run packages/frontend/design-system/src/dropdown-menu.test.tsx packages/features/architecture/src/canvas/canvas-bottom-bar.test.tsx packages/features/architecture/src/canvas/contextual-toolbar.test.tsx` — expected all focused tests pass.
  - `pnpm exec tsc --noEmit -p packages/frontend/design-system/tsconfig.json` — expected exit 0.
  - Run the narrowest Architecture package typecheck available and capture an explicit exit status; do not convert known workspace baseline failures into success.
  - `pnpm design:lint` — expected no new design violations. Preserve the exact pnpm signature blocker if it prevents startup.
  - `git diff --check` — expected no whitespace errors.
- **Feel check**: only when Erick explicitly authorizes launching the app, run `pnpm dev:desktop`, open each menu from the canvas bottom bar and contextual toolbars, and confirm:
  - menus above the bottom bar rise from the trigger rather than moving away from it;
  - menus below contextual controls descend from the trigger;
  - Escape and outside click close every menu and focus returns to its trigger;
  - repeated open/close never leaves a stale portal or two panels;
  - in DevTools at 10% playback, the offset is exactly 4px and scale starts at 0.98;
  - with `prefers-reduced-motion: reduce`, movement and scale disappear but 120ms opacity feedback remains.
- **Done when**: all eight `<details>` menus are gone, all use the shared Ark wrapper, placement determines the correct motion direction, tests cover behavior instead of exports only, and canvas behavior is otherwise unchanged.
