# 004 — Bridge integration credential method swaps

- **Status**: TODO
- **Commit**: 497766a
- **Severity**: LOW
- **Category**: State indication; missed opportunities
- **Estimated scope**: 2 files, approximately 70–110 lines including tests

## Problem

The integration setup wizard replaces the Assume Role field with two Access Key fields directly from a ternary:

```tsx
// packages/features/integrations/src/setup-wizard.tsx:48 — current
{method() === "assume-role" ? (
  <label class="block text-[12px] text-[var(--color-text-secondary)]">
    Role ARN
    <Input
      size="sm"
      class="mt-1 w-full font-mono"
      value={roleArn()}
      onInput={(e) => setRoleArn(e.currentTarget.value)}
    />
  </label>
) : (
  <>
    <label class="block text-[12px] text-[var(--color-text-secondary)]">
      Access key ID
      <Input size="sm" class="mt-1 w-full" value={accessKey()} ... />
    </label>
    <label class="block text-[12px] text-[var(--color-text-secondary)]">
      Secret access key
      <Input size="sm" class="mt-1 w-full" type="password" value={...} ... />
    </label>
  </>
)}
```

This is a rare setup flow, and the one-field/two-field replacement changes both meaning and layout with no bridge. A short overlapping crossfade will indicate the selected authentication method without turning the wizard into a page transition.

## Target

**Dependency**: execute plan `001-shared-presence-selection-toolbar.md` first. This plan consumes `MotionPresence` and adds no feature CSS.

Render both method branches in the same one-cell CSS grid and let Ark Presence keep the outgoing branch mounted only through its exit:

```tsx
<div class="grid">
  <MotionPresence
    present={method() === "assume-role"}
    motion="swap"
    class="col-start-1 row-start-1 min-w-0"
  >
    {/* Role ARN label and Input */}
  </MotionPresence>
  <MotionPresence
    present={method() === "access-key"}
    motion="swap"
    class="col-start-1 row-start-1 min-w-0 space-y-4"
  >
    {/* Access key ID and Secret access key labels */}
  </MotionPresence>
</div>
```

The exact `swap` recipe is:

- outgoing: settled → `opacity: 0; transform: translateY(-4px)` over `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))`;
- incoming: `opacity: 0; transform: translateY(4px)` → settled over `160ms var(--ease-enter, cubic-bezier(0.16, 1, 0.3, 1))`;
- both run concurrently, so the total interaction remains 160ms rather than 280ms;
- reduced motion: no translation, opacity only over `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))`.

`MotionPresence` must make the outgoing branch inert and `aria-hidden` immediately, so Tab never enters fields for the method that is no longer selected. Signal values must remain preserved when a branch unmounts and later remounts.

## Repo conventions to follow

- Plan 001's `MotionPresence` owns Ark Presence, the exact keyframes, centralized opacity, inert exit state, and reduced motion.
- `packages/features/shell/src/onboarding.tsx:61` is the product exemplar for directional state changes that preserve focus and skip movement under reduced motion. Do not copy its feature-local CSS; use the shared primitive.
- `packages/configuration/tailwind/base.css:78` is the source of the settled repository motion tokens.
- Keep the wizard's existing English strings, signals, secret-clearing behavior, toast behavior, and callbacks.

## Steps

1. Consolidate the two current design-system imports in `packages/features/integrations/src/setup-wizard.tsx` and add `MotionPresence` to the same import.
2. Replace the ternary with a single `grid` host and two `MotionPresence motion="swap"` branches at `col-start-1 row-start-1`. Use a real wrapper element for each branch; do not use `asChild` around the two-element access-key fragment.
3. Preserve `roleArn`, `accessKey`, `secretKey`, and `secretSaved` signals exactly. Do not clear values when changing radio methods.
4. Ensure only the selected method branch is interactive. Rely on `MotionPresence` inert/`aria-hidden` during exit; do not add manual `tabindex` mutations to every Input.
5. Add `packages/features/integrations/src/setup-wizard.test.tsx`. Toggle both radio choices repeatedly; verify correct fields, outgoing inert state, values preserved across remounts, Test connection behavior unchanged, secret values cleared only by existing submission logic, and no duplicate toast/callback.

## Boundaries

- Do NOT animate PageHeader, step text, permissions, footer actions, toasts, or the overall route.
- Do NOT change the setup wizard's incomplete fixture/runtime semantics or add new steps.
- Do NOT migrate native radios or restructure the form beyond the credential-method host.
- Do NOT add feature CSS, a dependency, blur, height animation, or stagger.
- Do NOT animate layout properties. The temporary grid overlap may change host height when the outgoing branch unmounts; do not tween that height.
- If plan 001 is not implemented or `MotionPresence` differs from its specified API, STOP and report the dependency mismatch.

## Verification

- **Mechanical**:
  - `pnpm exec vitest run packages/features/integrations/src/setup-wizard.test.tsx packages/frontend/design-system/src/presence.test.tsx` — expected all focused tests pass.
  - Run the narrowest Integrations package typecheck with an explicit exit status and preserve any existing workspace blocker.
  - `node tooling/check-ui-opacity.mjs` — expected no feature-owned opacity violations.
  - `pnpm design:lint` — expected no new design violations; preserve the exact pnpm signature blocker if it prevents startup.
  - `git diff --check` — expected no whitespace errors.
- **Feel check**: only when Erick explicitly authorizes launching the app, run `pnpm dev:desktop`, open Connect AWS, and alternate the two methods. Confirm:
  - the old fields lift 4px while the new fields rise 4px, concurrently;
  - the interaction settles within 160ms and never feels like navigation;
  - rapid toggling does not leave both branches focusable or a stale branch mounted;
  - typed values survive a round trip between methods;
  - in DevTools at 10% playback, only transform and opacity animate;
  - with `prefers-reduced-motion: reduce`, movement disappears and the 120ms opacity cue remains.
- **Done when**: credential-method changes have a restrained, accessible crossfade, preserve all form state, and introduce no feature-local motion recipe.
