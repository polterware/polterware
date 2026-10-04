import { describe, expect, it, vi } from "vitest";
const mock = vi.hoisted(() => ({ expose: vi.fn(), invoke: vi.fn(), on: vi.fn(), remove: vi.fn() }));
vi.mock("electron", () => ({ contextBridge: { exposeInMainWorld: mock.expose }, ipcRenderer: { invoke: mock.invoke, on: mock.on, removeListener: mock.remove } }));
import "./index.js";
import type { SparkyBridge } from "../shared/bridge.js";
const exposedName = mock.expose.mock.calls[0]![0];
const bridge = mock.expose.mock.calls[0]![1] as SparkyBridge;
describe("sandbox bridge", () => {
  it("exposes only the typed Sparky API", () => {
    expect(exposedName).toBe("sparky");
    expect(bridge).not.toHaveProperty("ipcRenderer");
    expect(bridge).not.toHaveProperty("invoke");
    expect(bridge).not.toHaveProperty("endpoint");
    expect(bridge).not.toHaveProperty("token");
    expect(bridge.app).not.toHaveProperty("require");
    expect(bridge.app).not.toHaveProperty("process");
  });
  it("uses fixed channels", async () => {
    await bridge.memories.list();
    expect(mock.invoke).toHaveBeenCalledWith("sparky:memories:list", {});
  });
  it("does not expose Electron events and unsubscribes exactly its listener", () => {
    const callback = vi.fn();
    const unsubscribe = bridge.onChange(callback);
    const [event, listener] = mock.on.mock.calls.at(-1)!;
    listener({ sender: "secret" }, { seq: "1", changes: [], resync: false });
    expect(callback).toHaveBeenCalledWith({ seq: "1", changes: [], resync: false });
    unsubscribe();
    expect(mock.remove).toHaveBeenCalledWith(event, listener);
  });
});
