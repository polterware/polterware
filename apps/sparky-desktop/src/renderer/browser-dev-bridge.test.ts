import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { RPC_METHODS } from "../shared/channels";
import { installBrowserDevBridge } from "./browser-dev-bridge";

const g = globalThis as unknown as { window: { sparky?: unknown } };
beforeEach(() => { g.window = {}; });
afterEach(() => { delete (g as { window?: unknown }).window; });

describe("browser dev bridge", () => {
  it("implements every allowlisted RPC method", async () => {
    expect(installBrowserDevBridge()).toBe(true);
    const bridge = g.window.sparky as unknown as Record<string, Record<string, () => Promise<unknown>>>;
    for (const [namespace, methods] of Object.entries(RPC_METHODS)) {
      for (const method of methods) expect(typeof bridge[namespace]![method], `${namespace}.${method}`).toBe("function");
    }
    expect(await (g.window.sparky as Window["sparky"]).memories.list({})).toMatchObject({ memories: [] });
  });
  it("keeps an existing bridge", () => {
    const existing = {} as Window["sparky"];
    g.window.sparky = existing;
    expect(installBrowserDevBridge()).toBe(false);
    expect(g.window.sparky).toBe(existing);
  });
});
