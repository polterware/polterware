import { describe, expect, it, vi } from "vitest";
import { testMcpConnection } from "./mcp-test";

describe("MCP test connection", () => {
  it("reports failure honestly when MCP is off without calling the network", async () => {
    const fetchImpl = vi.fn();
    const result = await testMcpConnection({ enabled: false, port: 47600, token: "live-token", fetchImpl });
    expect(result).toEqual({ ok: false, detail: "MCP is off. Enable it in Agents > Access, then test again." });
    expect(fetchImpl).not.toHaveBeenCalled();
  });

  it("treats HTTP 403 disabled copy as off", async () => {
    const fetchImpl = vi.fn(async () => ({ ok: false, status: 403, text: async () => "MCP is disabled. Enable it in Settings > Agents" }));
    const result = await testMcpConnection({ enabled: true, port: 47600, token: "live-token", fetchImpl });
    expect(result.ok).toBe(false);
    expect(result.detail).toContain("MCP is off");
  });

  it("treats a rejected token as a failure", async () => {
    const fetchImpl = vi.fn(async () => ({ ok: false, status: 401, text: async () => "unauthorized" }));
    const result = await testMcpConnection({ enabled: true, port: 47600, token: "old-token", fetchImpl });
    expect(result).toEqual({ ok: false, detail: "The MCP token was rejected. Rotate the token and try again." });
  });

  it("reports initialize plus tools/list success", async () => {
    const fetchImpl = vi.fn(async (_url: string, init?: { body?: string }) => {
      if (init?.body?.includes("initialize") && !init.body.includes("notifications/initialized")) {
        return { ok: true, status: 200, text: async () => JSON.stringify({ jsonrpc: "2.0", id: 1, result: { protocolVersion: "2025-11-25" } }) };
      }
      if (init?.body?.includes("tools/list")) {
        return { ok: true, status: 200, text: async () => JSON.stringify({ jsonrpc: "2.0", id: 2, result: { tools: [{ name: "list_memories" }, { name: "get_memory" }] } }) };
      }
      return { ok: true, status: 200, text: async () => "" };
    });
    const result = await testMcpConnection({ enabled: true, port: 47600, token: "live-token", fetchImpl });
    expect(result).toEqual({ ok: true, detail: "Connected. 2 tools listed." });
  });
});
