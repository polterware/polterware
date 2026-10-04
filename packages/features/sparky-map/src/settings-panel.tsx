import { Show, createSignal, onCleanup, onMount } from "solid-js";
import { Button, StatusChip, SwitchRow } from "@polterware/frontend-design-system";
import { PLACE_NOTICE, TILE_NOTICE, honestyLabel, honestyTone } from "./model";
import { mapStore } from "./store";

export function MapSettingsPanel() {
  const [busy, setBusy] = createSignal(false);
  onMount(() => onCleanup(mapStore.connect()));

  async function toggle(online: boolean) {
    if (busy()) return;
    setBusy(true);
    try { await mapStore.setOnline(online); }
    finally { setBusy(false); }
  }

  return (
    <div data-map-settings class="flex flex-col gap-4">
      <div class="flex items-center justify-between gap-2">
        <p class="text-[13px] font-medium text-[var(--color-text-primary)]">Map</p>
        <StatusChip tone={honestyTone(mapStore.saveState() === "live" ? mapStore.loadState() : mapStore.saveState())}>
          {honestyLabel(mapStore.saveState() === "failed" ? "failed" : mapStore.loadState())}
        </StatusChip>
      </div>
      <SwitchRow
        label="Download map tiles"
        checked={mapStore.mapOnline()}
        disabled={busy()}
        onCheckedChange={checked => void toggle(checked)}
      />
      <p class="text-[13px] text-[var(--color-text-secondary)]">{TILE_NOTICE}</p>
      <p class="text-[13px] text-[var(--color-text-secondary)]">{PLACE_NOTICE}</p>
      <p class="text-[13px] text-[var(--color-text-muted)]">Address search is controlled in Settings → Map. It sends typed address text to Nominatim only when enabled. Changing it reloads Sparky.</p>
      <Show when={mapStore.saveError()}><p role="alert" class="text-[13px] text-[var(--color-text-secondary)]">{mapStore.saveError()}</p></Show>
      <Show when={!mapStore.mapOnline()}>
        <Button variant="secondary" size="sm" disabled={busy()} onClick={() => void toggle(true)}>Use online tiles</Button>
      </Show>
    </div>
  );
}
