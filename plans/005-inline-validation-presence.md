# 005 — Give inline validation errors restrained presence

- **Status**: TODO
- **Commit**: 497766a
- **Severity**: LOW
- **Category**: Feedback; missed opportunities
- **Estimated scope**: 2 files, approximately 60–100 lines including tests

## Problem

The Configure Component dialog mounts its validation alert immediately after a failed submit and removes it immediately on any relevant edit:

```tsx
// packages/features/architecture/src/canvas/configure-component-dialog.tsx:29 — current
function submit() {
  const result = normalizePlaceConfig({
    name: name(),
    environment: environment(),
  });
  if ("error" in result) {
    setError(result.error);
    return;
  }
  props.onConfirm(result);
}
```

```tsx
// packages/features/architecture/src/canvas/configure-component-dialog.tsx:97 — current
<Show when={error()}>
  <p class="m-0 text-[12px] text-[var(--color-danger)]" role="alert">
    {error()}
  </p>
</Show>
```

The message appears between the fields and dialog footer with no feedback bridge. This is an occasional, user-triggered state change; a 2px/120ms alert entrance directs attention without animating the surrounding form.

## Target

**Dependency**: execute plan `001-shared-presence-selection-toolbar.md` first. This plan consumes `MotionPresence` and adds no feature CSS.

Use separate message and visibility state so the outgoing alert keeps its text during the closing animation:

```tsx
const [error, setError] = createSignal<string | null>(null);
const [errorPresent, setErrorPresent] = createSignal(false);

function showError(message: string) {
  setError(message);
  setErrorPresent(true);
}

function clearError() {
  setErrorPresent(false);
}
```

Render the semantic paragraph through the shared primitive:

```tsx
<MotionPresence
  present={errorPresent()}
  motion="alert"
  asChild
  onExitComplete={() => {
    if (!errorPresent()) setError(null);
  }}
>
  <p class="m-0 text-[12px] text-[var(--color-danger)]" role="alert">
    {error()}
  </p>
</MotionPresence>
```

The exact `alert` recipe is:

- enter: `opacity: 0; transform: translateY(-2px)` → settled over `120ms var(--ease-enter, cubic-bezier(0.16, 1, 0.3, 1))`;
- exit: settled → `opacity: 0; transform: translateY(-2px)` over `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))`;
- reduced motion: no translation, opacity only over `120ms var(--ease-standard, cubic-bezier(0.2, 0, 0, 1))`.

`MotionPresence` sets `aria-hidden` and inert during exit. The alert must retain `role="alert"` on entrance so assistive technology announces the validation result.

## Repo conventions to follow

- Plan 001's `MotionPresence` owns Ark Presence, centralized opacity, exact keyframes, exit unmount, and reduced motion.
- `packages/frontend/design-system/src/field.tsx:22` shows the shared error typography. Keep the current 12px dialog-specific typography in this narrow plan; do not broaden into a Field migration.
- `packages/features/architecture/src/canvas/configure-component-dialog.tsx:21` already resets dialog state when a new definition opens; reset both error signals there.
- Preserve `normalizePlaceConfig`, dialog semantics, English strings, focus behavior, and confirmation behavior.

## Steps

1. Add `MotionPresence` to the design-system import in `packages/features/architecture/src/canvas/configure-component-dialog.tsx`.
2. Add `errorPresent`, `showError`, and `clearError` as specified. Keep the previous error message until `onExitComplete`; do not set the text to null at the start of exit.
3. Replace `setError(result.error)` in `submit()` with `showError(result.error)`.
4. Replace both current `setError(null)` edit handlers with `clearError()`. In the dialog-open reset effect, set both `error` to null and `errorPresent` to false immediately so stale errors never animate into a newly selected component.
5. Replace the alert `<Show>` with `MotionPresence motion="alert" asChild`. On exit completion, clear the stored text only if no new error reopened the alert.
6. Add `packages/features/architecture/src/canvas/configure-component-dialog.test.tsx`. Cover invalid submit announcement, stable text during exit, unmount after animation end, clearing from Name and Environment edits, a new invalid submit interrupting exit, dialog reopen reset, and unchanged valid confirmation.

## Boundaries

- Do NOT animate labels, Inputs, Select, dialog chrome, footer buttons, or form layout.
- Do NOT migrate the current labels to `Field` in this plan; that is separate design-system cleanup.
- Do NOT add feature CSS, height animation, shake, bounce, color cycling, or a dependency.
- Do NOT change validation rules, error copy, submit-on-Enter behavior, autofocus, or confirmation payload.
- If plan 001 is not implemented or `MotionPresence` differs from its specified API, STOP and report the dependency mismatch.

## Verification

- **Mechanical**:
  - `pnpm exec vitest run packages/features/architecture/src/canvas/configure-component-dialog.test.tsx packages/frontend/design-system/src/presence.test.tsx` — expected all focused tests pass.
  - Run the narrowest Architecture package typecheck with an explicit exit status; preserve any existing workspace blocker.
  - `node tooling/check-ui-opacity.mjs` — expected no feature-owned opacity violations.
  - `pnpm design:lint` — expected no new design violations; preserve the exact pnpm signature blocker if it prevents startup.
  - `git diff --check` — expected no whitespace errors.
- **Feel check**: only when Erick explicitly authorizes launching the app, run `pnpm dev:desktop`, open Configure component, submit an invalid name, and confirm:
  - the alert settles from only 2px above within 120ms;
  - editing a field removes it cleanly without blank-text flicker;
  - submitting invalid data again during exit restores the alert without a stale unmount;
  - focus remains in the field and the dialog does not move as a whole;
  - in DevTools at 10% playback, only transform and opacity animate;
  - with `prefers-reduced-motion: reduce`, movement disappears and the 120ms opacity cue remains.
- **Done when**: the validation alert has accessible, interruptible shared presence, its text remains stable through exit, and validation/dialog behavior is unchanged.
