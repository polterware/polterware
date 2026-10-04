export type McpSnippetInput = {
  port: number;
  token: string;
  command?: string;
};

export type McpSnippets = {
  command: string;
  token: string;
  httpUrl: string;
  claudeStdio: string;
  claudeHttp: string;
  cursorStdioJson: string;
  cursorHttpJson: string;
  cursorConfig: string;
  claudeDesktopStdio: string;
  claudeDesktopHttp: string;
};

function compact(value: unknown): string {
  return JSON.stringify(value);
}

function pretty(value: unknown): string {
  return JSON.stringify(value, null, 2);
}

/** Copy-paste MCP client configs matching 04-mcp.md and the sparky runtime GetInfo shapes. */
export function mcpSnippets(input: McpSnippetInput): McpSnippets {
  const command = input.command?.trim() || "sparky";
  const token = input.token.trim() || "<mcp-token>";
  const port = Number.isInteger(input.port) && input.port > 0 ? input.port : 47600;
  const httpUrl = `http://127.0.0.1:${port}/mcp`;
  const stdio = { command, args: ["mcp"] };
  const http = { url: httpUrl, headers: { Authorization: `Bearer ${token}` } };
  const wrappedStdio = { mcpServers: { sparky: stdio } };
  const wrappedHttp = { mcpServers: { sparky: http } };
  return {
    command,
    token,
    httpUrl,
    claudeStdio: `claude mcp add sparky -- ${command} mcp`,
    claudeHttp: `claude mcp add --transport http sparky ${httpUrl} --header "Authorization: Bearer ${token}"`,
    cursorStdioJson: compact(stdio),
    cursorHttpJson: compact(http),
    cursorConfig: pretty(wrappedStdio),
    claudeDesktopStdio: pretty(wrappedStdio),
    claudeDesktopHttp: pretty(wrappedHttp),
  };
}
