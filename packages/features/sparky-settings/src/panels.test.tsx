import { afterEach, describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import type { JSX } from "solid-js";
import { DesignSystemProvider } from "@polterware/frontend-design-system";
import feature, { AgentsAccessPanel, DataPanel, GeneralPanel, NotificationsPanel, SearchPanel } from "./index";
import { fakeSettingsBridge } from "./fake-bridge";
import { settingsStore } from "./store";

const cleanups: (() => void)[] = [];
afterEach(() => {
  cleanups.splice(0).reverse().forEach(cleanup => cleanup());
  settingsStore.reset();
  Reflect.deleteProperty(globalThis, "sparky");
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

function mount(component: () => JSX.Element, seed?: Parameters<typeof fakeSettingsBridge>[0]) {
  const fake = fakeSettingsBridge(seed);
  Reflect.set(globalThis, "sparky", fake.bridge);
  const host = document.createElement("div");
  document.body.append(host);
  const dispose = render(() => <DesignSystemProvider storagePrefix="sparky-settings-test">{component()}</DesignSystemProvider>, host);
  cleanups.push(() => { dispose(); host.remove(); });
  return { ...fake, host };
}

function button(label: string) {
  return [...document.querySelectorAll<HTMLButtonElement>("button")].find(element => element.textContent?.trim() === label || element.getAttribute("aria-label") === label);
}

function pointerClick(element: HTMLElement) {
  const opts = { bubbles: true, cancelable: true, composed: true, clientX: 8, clientY: 8, buttons: 1 };
  element.dispatchEvent(new PointerEvent("pointerdown", opts));
  element.dispatchEvent(new MouseEvent("mousedown", opts));
  element.dispatchEvent(new PointerEvent("pointerup", { ...opts, buttons: 0 }));
  element.dispatchEvent(new MouseEvent("mouseup", { ...opts, buttons: 0 }));
  element.click();
}

describe("settings panels", () => {
  it("registers every settings panel on the feature", () => {
    expect(feature.settingsPanels?.map(panel => panel.id)).toEqual([
      "general", "notifications", "focus", "calendar", "map", "search", "agents-access", "agents-activity", "data", "shortcuts", "advanced",
    ]);
  });

  it("saves keep running in background through SettingsService.Set", async () => {
    const { host, bridge } = mount(() => <GeneralPanel />);
    await vi.waitFor(() => expect(host.textContent).toContain("Sparky 0.1.0"));
    const toggle = host.querySelector<HTMLElement>(".pw-switch-row");
    expect(toggle).not.toBeNull();
    pointerClick(toggle!);
    await vi.waitFor(() => expect(bridge.settings.set).toHaveBeenCalledWith({ keys: ["app.keepRunningInBackground"], settings: { appKeepRunningInBackground: false } }));
  });

  it("asks for notification permission when it is not determined", async () => {
    const { host, bridge } = mount(() => <NotificationsPanel />);
    await vi.waitFor(() => expect(button("Enable notifications")).toBeTruthy());
    button("Enable notifications")!.click();
    await vi.waitFor(() => expect(bridge.app.requestNotificationAuthorization).toHaveBeenCalled());
  });

  it("tells the user to open macOS System Settings when notifications are denied", async () => {
    const fake = fakeSettingsBridge();
    fake.setPermission("denied");
    Reflect.set(globalThis, "sparky", fake.bridge);
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(() => <DesignSystemProvider storagePrefix="sparky-settings-test"><NotificationsPanel /></DesignSystemProvider>, host);
    cleanups.push(() => { dispose(); host.remove(); });
    await vi.waitFor(() => expect(host.textContent).toContain("Open System Settings"));
    expect(button("Enable notifications")).toBeFalsy();
  });

  it("reindexes and shows progress then the new document count", async () => {
    const { host } = mount(() => <SearchPanel />);
    await vi.waitFor(() => expect(host.textContent).toContain("12 documents"));
    button("Reindex")!.click();
    await vi.waitFor(() => expect(host.textContent).toContain("24 documents"));
  });

  it("rotates the token and marks the previous value invalid", async () => {
    const { host } = mount(() => <AgentsAccessPanel />);
    await vi.waitFor(() => expect(button("Rotate")).toBeTruthy());
    const previous = settingsStore.agent()?.token;
    button("Rotate")!.click();
    await vi.waitFor(() => expect(host.textContent).toContain("Previous token is no longer valid"));
    expect(settingsStore.agent()?.token).not.toBe(previous);
    expect(settingsStore.previousToken()).toBe(previous);
  });

  it("reports Test connection failure when MCP is off", async () => {
    const { host } = mount(() => <AgentsAccessPanel />, { agentsMcpEnabled: false });
    await vi.waitFor(() => expect(button("Test connection")).toBeTruthy());
    button("Test connection")!.click();
    await vi.waitFor(() => expect(host.textContent).toContain("MCP is off"));
  });

  it("requires typing RESET and does not delete data", async () => {
    const { host, bridge } = mount(() => <DataPanel />);
    await vi.waitFor(() => expect(host.textContent).toContain("Type RESET"));
    const reset = [...document.querySelectorAll<HTMLButtonElement>("button")].find(element => element.textContent?.trim() === "Reset local data")!;
    expect(reset.disabled).toBe(true);
    const input = document.querySelector<HTMLInputElement>('input[aria-label="Type RESET to confirm"]')!;
    input.value = "RESET";
    input.dispatchEvent(new InputEvent("input", { bubbles: true }));
    await vi.waitFor(() => expect(reset.disabled).toBe(false));
    reset.click();
    expect(host.textContent).toContain("No data was changed");
    expect(bridge.maintenance.export).not.toHaveBeenCalled();
  });
});
