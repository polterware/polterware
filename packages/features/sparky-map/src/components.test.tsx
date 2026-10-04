import { render } from "solid-js/web";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import shellFeature, { SparkyShell, createShellBridge, useSparkyShell, type ShellWireBridge, type SparkyFeature, type DetailSectionDef } from "@polterware/feature-sparky-shell";
import { Button } from "@polterware/frontend-design-system";
import mapFeature from "./index";
import { MapPage } from "./map-page";
import { fakeMapBridge } from "./fake-bridge";
import { OFFLINE_TITLE, PLACE_NOTICE, TILE_NOTICE } from "./model";
import { createMapStore } from "./store";
import { createMapBridge } from "./bridge";

let dispose: (() => void) | undefined;
beforeEach(() => {
  localStorage.clear();
  localStorage.setItem("sparky.shell.onboarding", "complete");
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockImplementation(function (this: HTMLElement) {
    return this.isConnected && !this.closest("[hidden]")
      ? ([new DOMRect(0, 0, 100, 32)] as unknown as DOMRectList)
      : ([] as unknown as DOMRectList);
  });
});
afterEach(() => {
  dispose?.();
  dispose = undefined;
  Reflect.deleteProperty(globalThis, "sparky");
  document.body.replaceChildren();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function OpenMemory() {
  const shell = useSparkyShell();
  return <Button onClick={() => shell.openMemory("memory-1")}>Open memory</Button>;
}

function namedButton(host: HTMLElement, label: string) {
  const found = [...host.querySelectorAll("button")].find(button => button.textContent?.replace(/\s+/g, " ").trim() === label);
  if (!found) throw new Error(`Button ${label} missing: ${host.textContent}`);
  return found;
}

function pointerClick(element: HTMLElement) {
  const opts = { bubbles: true, cancelable: true, pointerId: 1, pointerType: "mouse", button: 0, buttons: 1 } as const;
  element.dispatchEvent(new PointerEvent("pointerdown", opts));
  element.dispatchEvent(new MouseEvent("mousedown", opts));
  element.dispatchEvent(new PointerEvent("pointerup", { ...opts, buttons: 0 }));
  element.dispatchEvent(new MouseEvent("mouseup", { ...opts, buttons: 0 }));
  element.click();
}

function mount(path: "map" | "today") {
  const fake = fakeMapBridge();
  Reflect.set(globalThis, "sparky", fake.wire);
  const store = createMapStore(() => createMapBridge(fake.wire));
  const opener: SparkyFeature = {
    id: "opener",
    routes: [{ path: "/today", title: "Today", component: OpenMemory }],
    memoryDetail: () => {
      const shell = useSparkyShell();
      return <>{shell.detailSections().map((section: DetailSectionDef) => {
        const Content = section.component;
        return <Content />;
      })}</>;
    },
  };
  const feature: SparkyFeature = {
    ...mapFeature,
    routes: [{ path: "/map", title: "Map", component: () => <MapPage store={store} /> }],
  };
  const host = document.createElement("div");
  document.body.append(host);
  dispose = render(() => <SparkyShell features={[shellFeature, opener, feature]} bridge={createShellBridge(fake.wire as unknown as ShellWireBridge)} />, host);
  return { host, fake, store, go: () => pointerClick(namedButton(host, path === "map" ? "Map" : "Open memory")) };
}

describe("map route and place card", () => {
  it("shows an honest offline empty state with the place list and first-run copy", async () => {
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    const { host, go } = mount("map");
    await vi.waitFor(() => namedButton(host, "Map"));
    go();
    await vi.waitFor(() => expect(host.textContent).toContain(OFFLINE_TITLE));
    expect(host.textContent).toContain(PLACE_NOTICE);
    expect(host.textContent).toContain(TILE_NOTICE);
    expect(host.textContent).toContain("Buy milk");
    expect(host.textContent).toContain("Home");
    expect(host.querySelector("[data-map-online='true']")).toBeNull();
    expect(fetch.mock.calls.filter(call => String(call[0]).includes("openfreemap"))).toEqual([]);
  });

  it("saves and clears a place with the memory expectedRev", async () => {
    const { host, fake, go } = mount("today");
    await vi.waitFor(() => namedButton(host, "Open memory"));
    go();
    await vi.waitFor(() => expect(host.querySelector("#place-name")).not.toBeNull());
    expect(host.textContent).toContain(PLACE_NOTICE);
    const name = host.querySelector<HTMLInputElement>("#place-name")!;
    name.value = "Studio";
    name.dispatchEvent(new Event("input", { bubbles: true }));
    pointerClick(namedButton(host, "Save place"));
    await vi.waitFor(() => expect(fake.wire.places.set).toHaveBeenCalled());
    const sent = (fake.wire.places.set as ReturnType<typeof vi.fn>).mock.calls[0]?.[0] as { expectedRev?: string; place?: { name?: string } };
    expect(sent.expectedRev).toBe("3");
    expect(typeof sent.expectedRev).toBe("string");
    expect(sent.place?.name).toBe("Studio");
    await vi.waitFor(() => {
      const clear = namedButton(host, "Clear place");
      if (clear.hasAttribute("disabled") || clear.getAttribute("aria-disabled") === "true") throw new Error("Clear still busy");
      return clear;
    });
    pointerClick(namedButton(host, "Clear place"));
    await vi.waitFor(() => expect(fake.wire.places.clear).toHaveBeenCalled());
    const cleared = (fake.wire.places.clear as ReturnType<typeof vi.fn>).mock.calls[0]?.[0] as { expectedRev?: string; memoryId?: string };
    expect(cleared).toMatchObject({ memoryId: "memory-1", expectedRev: "4" });
  });
});
