# W1-MCP: add the pinned SDK dependency

- Owner: W0-SCAFFOLD / integrator
- Blocked sub-task: official SDK server, tool handlers, and transport smoke.
- Why: `sparky/backend/go.mod` and `go.sum` are outside W1-MCP Owns. At the starting HEAD they have no MCP SDK dependency. O2 requires the official SDK, not a replacement JSON-RPC server.
- Required change: add `github.com/modelcontextprotocol/go-sdk v1.8.0` to `sparky/backend/go.mod`, including its transitive requirements and checksums in `go.sum`. Use `cd sparky/backend && go get github.com/modelcontextprotocol/go-sdk@v1.8.0` and retain the pinned version when tidying after the server lands.
- Resume: W1-MCP implements the stateful SDK server and all 18 tools after this dependency is available. Independent guard, catalog, and session primitives can proceed without it.
