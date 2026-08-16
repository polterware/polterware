import { describe, expect, it, vi } from "vitest";
import { createPolterwareIpcHandlers } from "./ipc.js";
import { IPC_CHANNELS } from "../shared/ipc.js";

describe("createPolterwareIpcHandlers", () => {
  it("registers the public bridge surface", () => {
    const handlers = createPolterwareIpcHandlers();

    expect(handlers[IPC_CHANNELS.app.getInfo]).toBeTypeOf("function");
    expect(handlers[IPC_CHANNELS.app.listIcons]).toBeTypeOf("function");
    expect(handlers[IPC_CHANNELS.app.getIcon]).toBeTypeOf("function");
    expect(handlers[IPC_CHANNELS.app.setIcon]).toBeTypeOf("function");
    expect(handlers[IPC_CHANNELS.commander.hideOverlay]).toBeTypeOf("function");
    expect(handlers[IPC_CHANNELS.commander.showMainWindow]).toBeTypeOf("function");
  });

  it("wires local app information when provided", async () => {
    const handlers = createPolterwareIpcHandlers({
      appInfo: { getInfo: () => ({ name: "Polterware", version: "0.1.0" }) },
    });

    await expect(handlers[IPC_CHANNELS.app.getInfo]()).resolves.toEqual({
      name: "Polterware",
      version: "0.1.0",
    });
  });

  it("wires Commander window controls when controllers are provided", async () => {
    const commander = {
      hideOverlay: vi.fn(),
      showMainWindow: vi.fn(),
    };
    const handlers = createPolterwareIpcHandlers({ commander });

    await handlers[IPC_CHANNELS.commander.hideOverlay]();
    await handlers[IPC_CHANNELS.commander.showMainWindow]();

    expect(commander.hideOverlay).toHaveBeenCalledOnce();
    expect(commander.showMainWindow).toHaveBeenCalledOnce();
  });

  it("wires app icon controls when controllers are provided", async () => {
    const appIcons = {
      getIcon: vi.fn().mockReturnValue({
        defaultId: "black",
        icon: {
          id: "black",
          label: "Black",
          previewDataUrl: "data:image/png;base64,YmxhY2s=",
        },
        selectedId: "black",
      }),
      listIcons: vi.fn().mockReturnValue([
        {
          id: "black",
          label: "Black",
          previewDataUrl: "data:image/png;base64,YmxhY2s=",
        },
      ]),
      setIcon: vi.fn().mockReturnValue({
        defaultId: "black",
        icon: {
          id: "white",
          label: "White",
          previewDataUrl: "data:image/png;base64,d2hpdGU=",
        },
        selectedId: "white",
      }),
    };
    const handlers = createPolterwareIpcHandlers({ appIcons });

    await handlers[IPC_CHANNELS.app.listIcons]();
    await handlers[IPC_CHANNELS.app.getIcon]();
    await handlers[IPC_CHANNELS.app.setIcon](undefined, { id: "white" });

    expect(appIcons.listIcons).toHaveBeenCalledOnce();
    expect(appIcons.getIcon).toHaveBeenCalledOnce();
    expect(appIcons.setIcon).toHaveBeenCalledWith("white");
  });
});
