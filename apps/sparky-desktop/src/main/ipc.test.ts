import { describe, expect, it, vi } from "vitest";
import { create } from "@bufbuild/protobuf";
import { ListMemoriesResponseSchema } from "@polterware/sparky-contracts";
import { services, type CoreClients } from "./core/client.js";
import { assertTrustedSender, validateAppInput, channel, createRpcHandlers, handleRequestNotificationAuthorization } from "./ipc.js";
import { RPC_METHODS } from "../shared/channels.js";
import { APP_METHODS } from "../shared/bridge.js";
const list = vi.fn(async () => create(ListMemoriesResponseSchema, {}));
const options = () => ({ getClients: () => ({ memories: { list } }) as unknown as CoreClients, authorizePath: () => false });
describe("IPC allowlist and schemas", () => {
  it("matches all frozen public unary services and no hidden core channels", () => {
    const handlers = createRpcHandlers(options());
    for (const [namespace, service] of Object.entries(services)) {
      expect([...RPC_METHODS[namespace as keyof typeof RPC_METHODS]].sort()).toEqual(service.methods.map(method => method.localName).sort());
    }
    expect(handlers["sparky:health:check"]).toBeUndefined();
    expect(handlers["sparky:core:endpoint"]).toBeUndefined();
  });
  it("validates unknown fields and scalar types before reaching the core", async () => {
    list.mockClear();
    const handler = createRpcHandlers(options())[channel("memories", "list")]!;
    await expect(handler({ endpoint: "http://evil" })).rejects.toThrow();
    await expect(handler({ limit: "not a number" })).rejects.toThrow();
    await expect(handler([])).rejects.toThrow();
    expect(list).not.toHaveBeenCalled();
    await expect(handler({ limit: 50 })).resolves.toMatchObject({ memories: [] });
    expect(list).toHaveBeenCalledOnce();
  });
  it("requires native dialog grants for import and maintenance writes", async () => {
    const handlers = createRpcHandlers(options());
    await expect(handlers[channel("attachments", "importFromPath")]!({ sourcePath: "/etc/passwd" })).rejects.toThrow("file dialog");
    await expect(handlers[channel("maintenance", "export")]!({ zipPath: "/etc/secret" })).rejects.toThrow("save dialog");
    await expect(handlers[channel("maintenance", "backup")]!({ destPath: "/etc/secret" })).rejects.toThrow("save dialog");
  });
  it("validates local app method inputs", () => {
    expect(() => validateAppInput("pickFiles", { path: "/etc/passwd" })).toThrow();
    expect(() => validateAppInput("startMicrophone", "camera")).toThrow();
    expect(() => validateAppInput("openDeepLink", 123)).toThrow();
    expect(() => validateAppInput("getInfo", undefined)).not.toThrow();
    expect(() => validateAppInput("requestNotificationAuthorization", undefined)).not.toThrow();
    expect(() => validateAppInput("requestNotificationAuthorization", true)).toThrow();
  });
  it("forwards notification authorization to the reminders controller", async () => {
    expect(APP_METHODS).toContain("requestNotificationAuthorization");
    const requestAuthorization = vi.fn(async () => true);
    await expect(handleRequestNotificationAuthorization({ requestAuthorization })).resolves.toBe(true);
    expect(requestAuthorization).toHaveBeenCalledOnce();
  });
  it("rejects iframes, foreign windows and absent senders", () => {
    const contents = { mainFrame: {} };
    expect(() => assertTrustedSender({ sender: contents, senderFrame: contents.mainFrame }, contents)).not.toThrow();
    expect(() => assertTrustedSender({ sender: {}, senderFrame: contents.mainFrame }, contents)).toThrow();
    expect(() => assertTrustedSender({ sender: contents, senderFrame: {} }, contents)).toThrow();
  });
});
