import { describe, expect, it, vi } from "vitest";
import { createPolterwareBridge } from "./bridge.js";
import { IPC_CHANNELS } from "../shared/ipc.js";

describe("createPolterwareBridge", () => {
  it("routes local chrome calls through the expected IPC channels", async () => {
    const invoke = vi.fn().mockResolvedValue([]);
    const bridge = createPolterwareBridge({ invoke });

    await bridge.app.listIcons();
    await bridge.app.getIcon();
    await bridge.app.setIcon("black");
    await bridge.commander.hideOverlay();
    await bridge.commander.showMainWindow();

    expect(invoke).toHaveBeenNthCalledWith(1, IPC_CHANNELS.app.listIcons);
    expect(invoke).toHaveBeenNthCalledWith(2, IPC_CHANNELS.app.getIcon);
    expect(invoke).toHaveBeenNthCalledWith(3, IPC_CHANNELS.app.setIcon, {
      id: "black",
    });
    expect(invoke).toHaveBeenNthCalledWith(
      4,
      IPC_CHANNELS.commander.hideOverlay,
    );
    expect(invoke).toHaveBeenNthCalledWith(
      5,
      IPC_CHANNELS.commander.showMainWindow,
    );
  });

  it("routes core calls through namespaced IPC channels", async () => {
    const invoke = vi.fn().mockResolvedValue({});
    const bridge = createPolterwareBridge({ invoke });

    await bridge.core.projects.list();
    await bridge.core.health.getStoreHealth();

    expect(invoke).toHaveBeenCalledWith(IPC_CHANNELS.core.listProjects, undefined);
    expect(invoke).toHaveBeenCalledWith(IPC_CHANNELS.core.getStoreHealth, undefined);
  });

  it("subscribes to Commander focus events and returns an unsubscribe function", () => {
    const invoke = vi.fn().mockResolvedValue([]);
    const listeners = new Map<string, (...args: unknown[]) => void>();
    const on = vi.fn(
      (channel: string, listener: (...args: unknown[]) => void) => {
        listeners.set(channel, listener);
      },
    );
    const removeListener = vi.fn();
    const bridge = createPolterwareBridge({ invoke, on, removeListener });
    const callback = vi.fn();

    const unsubscribe = bridge.commander.onFocusSearch(callback);
    listeners.get("commander:focus-search")?.();
    unsubscribe();

    expect(callback).toHaveBeenCalledOnce();
    expect(removeListener).toHaveBeenCalledWith(
      "commander:focus-search",
      listeners.get("commander:focus-search"),
    );
  });
});
