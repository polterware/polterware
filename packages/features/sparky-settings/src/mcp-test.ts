export type McpTestResult = { ok: boolean; detail: string };

type FetchLike = (input: string, init?: { method?: string; headers?: Record<string, string>; body?: string }) => Promise<{
  ok: boolean;
  status: number;
  text(): Promise<string>;
}>;

function rpc(id: number, method: string, params?: Record<string, unknown>): string {
  return JSON.stringify({ jsonrpc: "2.0", id, method, params });
}

function parseRpc(body: string): { error?: { message?: string }; result?: { tools?: unknown[] } } {
  try {
    return JSON.parse(body) as { error?: { message?: string }; result?: { tools?: unknown[] } };
  } catch {
    return { error: { message: body.trim() || "The MCP server returned unreadable JSON." } };
  }
}

/** initialize + tools/list against the loopback MCP HTTP endpoint. Disabled MCP never claims success. */
export async function testMcpConnection(input: {
  enabled: boolean;
  port: number;
  token: string;
  fetchImpl?: FetchLike;
}): Promise<McpTestResult> {
  if (!input.enabled) {
    return { ok: false, detail: "MCP is off. Enable it in Agents > Access, then test again." };
  }
  const port = Number.isInteger(input.port) && input.port > 0 && input.port <= 65535 ? input.port : 0;
  if (!port) return { ok: false, detail: "MCP port is invalid." };
  if (!input.token.trim()) return { ok: false, detail: "MCP token is missing. Rotate the token, then test again." };
  const fetchImpl = input.fetchImpl ?? fetch;
  const endpoint = `http://127.0.0.1:${port}/mcp`;
  const headers = {
    "Content-Type": "application/json",
    Accept: "application/json, text/event-stream",
    Authorization: `Bearer ${input.token}`,
  };
  try {
    const init = await fetchImpl(endpoint, {
      method: "POST",
      headers,
      body: rpc(1, "initialize", {
        protocolVersion: "2025-11-25",
        capabilities: {},
        clientInfo: { name: "sparky-settings", version: "0" },
      }),
    });
    const initBody = await init.text();
    if (init.status === 403 || /MCP is disabled/i.test(initBody)) {
      return { ok: false, detail: "MCP is off. Enable it in Agents > Access, then test again." };
    }
    if (init.status === 401) return { ok: false, detail: "The MCP token was rejected. Rotate the token and try again." };
    if (!init.ok) return { ok: false, detail: initBody.trim() || `MCP initialize failed (${init.status}).` };
    const initialized = parseRpc(initBody);
    if (initialized.error?.message) return { ok: false, detail: initialized.error.message };
    await fetchImpl(endpoint, { method: "POST", headers, body: JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }) }).catch(() => undefined);
    const listed = await fetchImpl(endpoint, { method: "POST", headers, body: rpc(2, "tools/list") });
    const listedBody = await listed.text();
    if (!listed.ok) return { ok: false, detail: listedBody.trim() || `MCP tools/list failed (${listed.status}).` };
    const parsed = parseRpc(listedBody);
    if (parsed.error?.message) return { ok: false, detail: parsed.error.message };
    const count = parsed.result?.tools?.length ?? 0;
    return { ok: true, detail: `Connected. ${count} tools listed.` };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { ok: false, detail: message || "MCP connection failed." };
  }
}
