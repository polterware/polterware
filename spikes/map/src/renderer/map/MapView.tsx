import { createEffect, onCleanup } from "solid-js";
import type { MapProviderHandle, MapViewProps } from "./types";
import { createLeafletProvider } from "./providers/leaflet";
import { createMapLibreProvider } from "./providers/maplibre";
import { createPmtilesProvider } from "./providers/pmtiles";

/**
 * Framework-agnostic MapView prototype for W2-MAP.
 * Props match plans/sparky-next/05-search-attachments-map.md.
 */
export function MapView(props: MapViewProps) {
  let containerRef: HTMLDivElement | undefined;
  let handle: MapProviderHandle | undefined;

  const createProvider = () => {
    handle?.destroy();
    handle = undefined;
    if (!containerRef) return;

    const factory =
      props.provider === "leaflet"
        ? createLeafletProvider
        : props.provider === "pmtiles"
          ? createPmtilesProvider
          : createMapLibreProvider;

    handle = factory(containerRef, {
      center: props.center,
      zoom: props.zoom,
      markers: props.markers,
      selectedId: props.selectedId,
      online: props.online,
      onSelect: props.onSelect,
      onPick: props.onPick,
    });
  };

  createEffect(() => {
    createProvider();
  });

  createEffect(() => {
    handle?.setCenter(props.center, props.zoom);
  });

  createEffect(() => {
    handle?.setMarkers(props.markers, props.selectedId);
  });

  createEffect(() => {
    handle?.setOnline(props.online);
  });

  onCleanup(() => {
    handle?.destroy();
    handle = undefined;
  });

  return (
    <div
      ref={containerRef}
      class={props.class ?? "h-full w-full"}
      role="application"
      aria-label="Map"
    />
  );
}
