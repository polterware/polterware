/** Local experimental MCP helper status exposed to the renderer. */
export type McpServerStatus = {
  running: boolean;
  pid: number | null;
  dbPath: string;
  command: string;
  cursorConfig: string;
};

/** Where an MCP client stands with the polterware entry (`polterware setup mcp list --json`). */
export type McpClientState = "not_installed" | "not_configured" | "configured" | "configured_other" | "unknown";

export type McpClientStatus = {
  id: "claude" | "codex" | "cursor";
  name: string;
  status: McpClientState;
  error?: string;
};

export const MCP_CLIENT_IDS = ["claude", "codex", "cursor"] as const;
