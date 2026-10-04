import { createEffect, createSignal, onCleanup, untrack } from "solid-js";
import { createMapEngine, type MapEngineHandle } from "./engine";
import { shouldLoadMapTiles } from "./network";
import type { MapViewProps } from "./types";
import "./map-view.css";

/**
 * Shared map primitive. Online tiles load only when `online` is true and the
 * provider is MapLibre + OpenFreeMap. Otherwise this component makes no map request.
 */
export function MapView(props: MapViewProps) {
  let container: HTMLDivElement | undefined;
  const [engine, setEngine] = createSignal<MapEngineHandle | null>(null);
  let generation = 0;

  const live = () => shouldLoadMapTiles(props.online, props.provider ?? "maplibre");

  createEffect(() => {
    const online = live();
    const provider = props.provider ?? "maplibre";
    const pickEnabled = Boolean(props.pickEnabled);
    const onSelect = props.onSelect;
    const onPick = props.onPick;
    const onBoundsChange = props.onBoundsChange;
    generation += 1;
    const current = generation;
    let local: MapEngineHandle | null = null;
    setEngine(null);
    if (!online || !container) return;
    void createMapEngine(container, {
      provider,
      pickEnabled,
      onSelect,
      onPick,
      onBoundsChange,
      center: untrack(() => props.center),
      zoom: untrack(() => props.zoom),
      markers: untrack(() => props.markers),
      selectedId: untrack(() => props.selectedId),
    }).then(next => {
      if (current !== generation) {
        next?.destroy();
        return;
      }
      local = next;
      setEngine(next);
    }).catch(() => {
      if (current === generation) setEngine(null);
    });
    onCleanup(() => {
      generation += 1;
      local?.destroy();
      setEngine(null);
    });
  });

  createEffect(() => {
    engine()?.setCenter(props.center, props.zoom);
  });

  createEffect(() => {
    engine()?.setMarkers(props.markers, props.selectedId);
  });

  createEffect(() => {
    if (!container || typeof ResizeObserver !== "function") return;
    const observer = new ResizeObserver(() => engine()?.resize());
    observer.observe(container);
    onCleanup(() => observer.disconnect());
  });

  return (
    <div
      ref={element => { container = element; }}
      class={`pw-map-view ${props.class ?? ""}`}
      role="application"
      aria-label="Map"
      data-map-online={live() ? "true" : "false"}
    />
  );
}
