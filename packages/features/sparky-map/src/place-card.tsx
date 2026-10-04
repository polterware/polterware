import { Show, createEffect, createSignal, onCleanup, on, type JSX } from "solid-js";
import { useSparkyShell } from "@polterware/feature-sparky-shell/feature";
import {
  Button, Field, Input, PageError, PageLoading, SegmentedControl, Slider, StatusChip,
} from "@polterware/frontend-design-system";
import { DEFAULT_MAP_CENTER, MapView } from "@polterware/frontend-map";
import { browserBridge, type MapBridge } from "./bridge";
import {
  DEFAULT_RADIUS_M, MAX_RADIUS_M, MIN_RADIUS_M, PLACE_NOTICE, clampRadius, draftFromPlace, errorText, honestyLabel,
  honestyTone, parseCoordinate, placeFromDraft, revisionString, type Honesty, type MemoryJSON, type PlaceDraft, type PlaceJSON,
} from "./model";
import { mapStore } from "./store";

export function PlaceCard(): JSX.Element {
  const shell = useSparkyShell();
  const memoryId = () => shell.selectedMemoryId();
  return <Show when={memoryId()}>{id => <PlaceEditor memoryId={id()} />}</Show>;
}

function PlaceEditor(props: { memoryId: string }) {
  const [draft, setDraft] = createSignal<PlaceDraft>(draftFromPlace());
  const [saved, setSaved] = createSignal(false);
  const [rev, setRev] = createSignal("0");
  const [online, setOnline] = createSignal(false);
  const [pending, setPending] = createSignal(true);
  const [busy, setBusy] = createSignal("");
  const [saveState, setSaveState] = createSignal<Honesty>("unavailable");
  const [loadError, setLoadError] = createSignal("");
  const [formError, setFormError] = createSignal("");
  const [coreError, setCoreError] = createSignal("");
  const lat = () => parseCoordinate(draft().lat, "lat");
  const lng = () => parseCoordinate(draft().lng, "lng");
  const pinReady = () => lat() !== undefined && lng() !== undefined;

  function patch(partial: Partial<PlaceDraft>) {
    setDraft(current => ({ ...current, ...partial }));
    setSaveState("queued");
    setFormError("");
  }

  async function withBridge<T>(label: string, run: (bridge: MapBridge) => Promise<T>): Promise<T | undefined> {
    try {
      setBusy(label);
      const result = await run(browserBridge());
      setCoreError("");
      return result;
    } catch (error) {
      setCoreError(errorText(error));
      setSaveState("failed");
      return undefined;
    } finally {
      setBusy("");
    }
  }

  async function load(id: string) {
    setPending(true); setLoadError("");
    try {
      const bridge = browserBridge();
      const [memoryResponse, settingsResponse] = await Promise.all([
        bridge.memories.get({ id, include: { place: true } }) as Promise<{ memory?: MemoryJSON }>,
        bridge.settings.get({}).catch(() => ({ settings: { mapOnline: false } })) as Promise<{ settings?: { mapOnline?: boolean } }>,
      ]);
      const memory = memoryResponse.memory;
      if (!memory) throw new Error("Memory unavailable. It may have been deleted.");
      setRev(revisionString(memory.rev));
      setDraft(draftFromPlace(memory.place));
      setSaved(Boolean(memory.place && Number.isFinite(memory.place.lat)));
      setOnline(Boolean(settingsResponse.settings?.mapOnline) && mapStore.provider() !== "pmtiles");
      setSaveState("live");
    } catch (error) {
      setLoadError(errorText(error));
      setSaveState("unavailable");
    } finally {
      setPending(false);
    }
  }

  async function save() {
    const place = placeFromDraft(props.memoryId, draft());
    if (!place) { setFormError("Enter a valid latitude and longitude."); return; }
    const result = await withBridge("save", bridge => bridge.places.set({ expectedRev: rev(), place }) as Promise<{ place?: PlaceJSON }>);
    if (!result?.place) return;
    setDraft(draftFromPlace(result.place));
    setSaved(true);
    const memory = await withBridge("reload", bridge => bridge.memories.get({ id: props.memoryId, include: { place: true } }) as Promise<{ memory?: MemoryJSON }>);
    if (memory?.memory) setRev(revisionString(memory.memory.rev));
    setSaveState("live");
  }

  async function clearPlace() {
    const result = await withBridge("clear", bridge => bridge.places.clear({ memoryId: props.memoryId, expectedRev: rev() }));
    if (result === undefined && saveState() === "failed") return;
    setDraft(draftFromPlace());
    setSaved(false);
    const memory = await withBridge("reload", bridge => bridge.memories.get({ id: props.memoryId, include: { place: true } }) as Promise<{ memory?: MemoryJSON }>);
    if (memory?.memory) setRev(revisionString(memory.memory.rev));
    setSaveState("live");
  }

  function dropPin() {
    patch({
      lat: String(lat() ?? DEFAULT_MAP_CENTER.lat),
      lng: String(lng() ?? DEFAULT_MAP_CENTER.lng),
      radiusM: draft().radiusM || DEFAULT_RADIUS_M,
    });
  }

  createEffect(on(() => props.memoryId, id => {
    void load(id);
    let unsubscribe: (() => void) | undefined;
    try {
      unsubscribe = browserBridge().onChange(event => {
        if (saveState() === "queued") return;
        if (event.resync || event.changes.some(change => ["place", "places", "memory", "memories", "settings"].includes(change.entity))) {
          void load(id);
        }
      });
    } catch { /* Bridge missing until Desktop hosts the page. */ }
    onCleanup(() => unsubscribe?.());
  }));

  return (
    <section data-place-card class="flex flex-col gap-3">
      <div class="flex items-center justify-between gap-2">
        <p class="text-[13px] font-medium text-[var(--color-text-primary)]">Place</p>
        <StatusChip tone={honestyTone(saveState())}>{honestyLabel(saveState())}</StatusChip>
      </div>
      <p class="text-[12px] text-[var(--color-text-muted)]">{PLACE_NOTICE}</p>
      <Show when={pending()}><PageLoading label="Loading place" /></Show>
      <Show when={loadError()}><PageError title="Place unavailable" message={loadError()} onRetry={() => void load(props.memoryId)} /></Show>
      <Show when={!pending() && !loadError()}>
        <Show when={online() && pinReady()}>
          <div class="h-52 overflow-hidden rounded-[var(--radius-card)]">
            <MapView
              online
              pickEnabled
              center={{ lat: lat() ?? DEFAULT_MAP_CENTER.lat, lng: lng() ?? DEFAULT_MAP_CENTER.lng }}
              zoom={14}
              markers={[{ id: props.memoryId, lat: lat()!, lng: lng()!, radius: draft().radiusM }]}
              selectedId={props.memoryId}
              onPick={pick => patch({ lat: String(pick.lat), lng: String(pick.lng) })}
            />
          </div>
          <p class="text-[12px] text-[var(--color-text-muted)]">Drag the pin or Shift-click the map to move it.</p>
        </Show>
        <Show when={!online()}>
          <p class="text-[12px] text-[var(--color-text-muted)]">Tiles are off. Enter coordinates or drop a pin without loading a map.</p>
        </Show>
        <Field label="Name" for="place-name">
          <Input id="place-name" value={draft().name} onInput={(event: Event) => patch({ name: (event.currentTarget as HTMLInputElement).value })} placeholder="Optional place name" />
        </Field>
        <div class="grid grid-cols-2 gap-2">
          <Field label="Latitude" for="place-lat" error={draft().lat !== "" && lat() === undefined ? "Invalid latitude" : undefined}>
            <Input id="place-lat" inputMode="decimal" value={draft().lat} onInput={(event: Event) => patch({ lat: (event.currentTarget as HTMLInputElement).value })} placeholder="-23.5505" />
          </Field>
          <Field label="Longitude" for="place-lng" error={draft().lng !== "" && lng() === undefined ? "Invalid longitude" : undefined}>
            <Input id="place-lng" inputMode="decimal" value={draft().lng} onInput={(event: Event) => patch({ lng: (event.currentTarget as HTMLInputElement).value })} placeholder="-46.6333" />
          </Field>
        </div>
        <Slider
          label={`Radius ${draft().radiusM} m`}
          min={MIN_RADIUS_M}
          max={MAX_RADIUS_M}
          step={25}
          value={[draft().radiusM]}
          onValueChange={(details: { value: number[] }) => patch({ radiusM: clampRadius(details.value[0] ?? DEFAULT_RADIUS_M) })}
        />
        <div class="flex flex-col gap-1">
          <p class="text-[12px] text-[var(--color-text-muted)]">Enter / exit (stored for later; not armed)</p>
          <SegmentedControl
            value={draft().event}
            onValueChange={(details: { value: string | null }) => { if (details.value === "enter" || details.value === "exit") patch({ event: details.value }); }}
            options={[{ value: "enter", label: "Enter" }, { value: "exit", label: "Exit" }]}
          />
        </div>
        <Show when={formError()}><p role="alert" class="text-[13px] text-[var(--color-text-secondary)]">{formError()}</p></Show>
        <Show when={coreError()}><p role="alert" class="text-[13px] text-[var(--color-text-secondary)]">{coreError()}</p></Show>
        <div class="flex flex-wrap gap-2">
          <Button variant="secondary" disabled={Boolean(busy())} onClick={dropPin}>Drop pin</Button>
          <Button variant="primary" disabled={Boolean(busy()) || !pinReady()} onClick={() => void save()}>{busy() === "save" ? "Saving" : "Save place"}</Button>
          <Show when={saved()}>
            <Button variant="danger" disabled={Boolean(busy())} onClick={() => void clearPlace()}>{busy() === "clear" ? "Clearing" : "Clear place"}</Button>
          </Show>
        </div>
      </Show>
    </section>
  );
}
