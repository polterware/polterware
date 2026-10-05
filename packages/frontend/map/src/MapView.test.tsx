import { render } from "solid-js/web";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MapView } from "./MapView";
import { OPENFREEMAP_DARK_STYLE } from "./style";
import { isMapNetworkUrl } from "./network";

let dispose: (() => void) | undefined;
afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.replaceChildren();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("MapView network isolation", () => {
  it("does not fetch the OpenFreeMap style or tiles while online is false", async () => {
    const fetch = vi.fn(async (input: RequestInfo | URL) => {
      throw new Error(`unexpected fetch ${String(input)}`);
    });
    vi.stubGlobal("fetch", fetch);
    const host = document.createElement("div");
    document.body.append(host);
    dispose = render(() => (
      <MapView
        online={false}
        center={{ lat: -23.55, lng: -46.63 }}
        zoom={11}
        markers={[{ id: "home", lat: -23.55, lng: -46.63, radius: 200 }]}
      />
    ), host);
    await new Promise(resolve => setTimeout(resolve, 50));
    const mapCalls = fetch.mock.calls.filter(call => isMapNetworkUrl(String(call.at(0) ?? "")));
    expect(mapCalls).toEqual([]);
    expect(fetch).not.toHaveBeenCalled();
    expect(host.querySelector("[data-map-online='false']")).not.toBeNull();
    expect(OPENFREEMAP_DARK_STYLE).toContain("openfreemap.org");
  });

  it("does not load tiles for the pmtiles provider without a local file", async () => {
    const fetch = vi.fn(async () => new Response("{}"));
    vi.stubGlobal("fetch", fetch);
    const host = document.createElement("div");
    document.body.append(host);
    dispose = render(() => (
      <MapView
        online
        provider="pmtiles"
        center={{ lat: 43.77, lng: 11.25 }}
        zoom={12}
        markers={[]}
      />
    ), host);
    await new Promise(resolve => setTimeout(resolve, 50));
    expect(fetch.mock.calls.filter(call => isMapNetworkUrl(String(call.at(0) ?? "")))).toEqual([]);
    expect(host.querySelector("[data-map-online='false']")).not.toBeNull();
  });
});
