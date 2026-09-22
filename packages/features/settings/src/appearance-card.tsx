import { CheckboxRow } from "@polterware/frontend-design-system";
import { createSignal, onMount } from "solid-js";
import {
  hydrateAppearancePreferences,
  persistReduceTransparency,
  readShellChromeMode,
} from "./appearance-persist";

/**
 * App-level window chrome: frosted (macOS vibrancy) vs solid fill.
 */
export function AppearanceCard() {
  const [reduceTransparency, setReduceTransparency] = createSignal(
    readShellChromeMode() === "solid",
  );
  const [ready, setReady] = createSignal(false);

  onMount(() => {
    void hydrateAppearancePreferences().then((preferences) => {
      setReduceTransparency(preferences.reduceTransparency);
      setReady(true);
    });
  });

  return (
    <div class="space-y-2">
      <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">Appearance</p>
      <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
        On macOS, the title bar and sidebar can show a frosted glass view of windows behind
        Polterware. Reduce transparency uses a solid fill instead.
      </p>
      <CheckboxRow
        checked={reduceTransparency()}
        disabled={!ready()}
        label="Reduce transparency"
        onCheckedChange={(checked) => {
          setReduceTransparency(checked);
          void persistReduceTransparency(checked).then((preferences) => {
            setReduceTransparency(preferences.reduceTransparency);
          });
        }}
      />
    </div>
  );
}
