# W1-MCP: provide core and CLI integration points

- Owner: W1-CORE / integrator
- Blocked sub-task: core discovery/start, production `/mcp`, `sparky mcp`, Claude Code and Inspector against the dev core.
- Why: `runtimes/cli/main.go` only prints a stub for `mcp`; `platform/coreclient`, `platform/runstate`, and the core implementation do not exist at the starting HEAD. W1-MCP cannot edit those paths.
- Exact CLI change requested after the bridge is available: split `"mcp"` out of the `case "serve", "mcp", "status", "doctor"` stub; dispatch to the W1-MCP function in `mcp.go`, report its error on stderr, and exit nonzero on failure.
- Required core interface: expose the MCP endpoint and stable token to the CLI discovery/start client; register a stateful SDK handler at `/mcp` on an IPv4 loopback listener with the actual bound port. Wrap it with `providers/mcp.NewGuard(token, port, maxBodyBytes)` before serving. Supply the same domain services used by Connect, and honor `agents.mcpEnabled` and `agents.allowFileAttach` dynamically.
- Dependencies: W1-DOMAIN, W1-RECURRENCE, W1-CORE, W1-SEARCH, and W1-ATTACH implementations. This checkout currently has only ports and panic-based fake skeletons. Do not substitute a private store for the production domain services.
- Resume: implement and verify the bridge and real-client sessions after these interfaces and the SDK dependency land.
