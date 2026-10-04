import { For, Show, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import {
  Button, Dialog, PageContent, PageEmpty, PageError, PageHeader, PageLoading, PageOffline, PageToolbar, StatusChip,
} from "@polterware/frontend-design-system";
import { MapView } from "@polterware/frontend-map";
import { useSparkyShell } from "@polterware/feature-sparky-shell/feature";
import {
  OFFLINE_MESSAGE, OFFLINE_TITLE, PLACE_NOTICE, TILE_NOTICE, honestyLabel, honestyTone,
} from "./model";
import { mapStore, type MapStore } from "./store";
import "./map-page.css";

export function MapPage(props: { store?: MapStore }) {
  const store = props.store ?? mapStore;
  const shell = useSparkyShell();

  onMount(() => onCleanup(store.connect()));

  createEffect(() => {
    const id = store.selectedId();
    if (id) shell.openMemory(id);
  });

  return (
    <div data-map-page class="sparky-map-page">
      <PageHeader
        title="Map"
        subtitle={<StatusChip tone={honestyTone(store.loadState())}>{honestyLabel(store.loadState())}</StatusChip>}
      />
      <PageToolbar.Root>
        <PageToolbar.Leading>
          <StatusChip tone={store.tilesLive() ? "ok" : "muted"}>{store.tilesLive() ? "Tiles live" : "Tiles off"}</StatusChip>
        </PageToolbar.Leading>
        <PageToolbar.Actions>
          <Show when={!store.tilesLive()}>
            <Button variant="secondary" size="sm" onClick={() => void store.setOnline(true)}>Use online tiles</Button>
          </Show>
        </PageToolbar.Actions>
      </PageToolbar.Root>
      <PageContent class="sparky-map-page__body">
        <Show when={store.loadState() === "queued"}><PageLoading label="Loading places" /></Show>
        <Show when={store.loadState() === "unavailable"}>
          <PageOffline title="Map unavailable" message={store.loadError() || "The local core is unreachable."} onRetry={() => void store.reload()} />
        </Show>
        <Show when={store.loadState() === "failed"}>
          <PageError title="Map failed" message={store.loadError()} onRetry={() => void store.reload()} />
        </Show>
        <Show when={store.loadState() === "live"}>
          <div class="sparky-map-page__layout">
            <section class="sparky-map-page__canvas" aria-label="Map canvas">
              <Show when={store.tilesLive()} fallback={
                <PageEmpty title={OFFLINE_TITLE} description={`${OFFLINE_MESSAGE} ${PLACE_NOTICE}`}>
                  <p class="text-[13px] text-[var(--color-text-muted)]">{TILE_NOTICE}</p>
                </PageEmpty>
              }>
                <MapView
                  online
                  provider={store.provider()}
                  center={store.center()}
                  zoom={store.zoom()}
                  markers={store.markers()}
                  selectedId={store.selectedId() ?? undefined}
                  onSelect={id => store.select(id)}
                  onBoundsChange={(box, zoom) => store.setBounds(box, zoom)}
                />
              </Show>
            </section>
            <aside class="sparky-map-page__list" aria-label="Places">
              <p class="sparky-map-page__list-title">Places</p>
              <Show when={store.places().length} fallback={<p class="text-[13px] text-[var(--color-text-muted)]">No places yet. Pin a memory from its Place card.</p>}>
                <ul class="sparky-map-page__rows">
                  <For each={store.places()}>{place => (
                    <li>
                      <button
                        type="button"
                        class={`sparky-map-page__row ${store.selectedId() === place.memoryId ? "is-selected" : ""}`}
                        aria-current={store.selectedId() === place.memoryId ? "true" : undefined}
                        onClick={() => store.select(place.memoryId)}
                      >
                        <span class="sparky-map-page__row-title">{place.title}</span>
                        <span class="sparky-map-page__row-meta">{place.name || `${place.lat.toFixed(4)}, ${place.lng.toFixed(4)}`}</span>
                      </button>
                    </li>
                  )}</For>
                </ul>
              </Show>
            </aside>
          </div>
        </Show>
      </PageContent>
      <Dialog
        open={store.showDisclosure()}
        title="Map tiles and places"
        onClose={() => store.markDisclosureSeen()}
        footer={
          <>
            <Button variant="ghost" onClick={() => store.markDisclosureSeen()}>Keep tiles off</Button>
            <Button variant="primary" onClick={() => void store.setOnline(true)}>Use online tiles</Button>
          </>
        }
      >
        <p>{TILE_NOTICE}</p>
        <p>{PLACE_NOTICE}</p>
        <p>You can turn tiles off later in Settings. While they are off, Sparky makes no map request and still lists places.</p>
      </Dialog>
    </div>
  );
}
