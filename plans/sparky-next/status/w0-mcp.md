# w0-mcp status

- Result: done
- Branch / last commit: `sparky/w0-mcp` (see `git log -1` after the status commit)
- Acceptance criteria:
  - Comparison table (spec coverage, streamable HTTP, error mapping, `tools/list` size, lines of code, test ergonomics, maintenance) and a recommendation → **pass** (this file)
  - Both transports pass the MCP Inspector → **pass** (`npx @modelcontextprotocol/inspector --cli`, 2026-10-03)
  - Anti-DNS-rebinding `curl` (bad Host, bad Origin) → **pass** (403/403, no CORS header)
  - O2 decision with the exact module version → **pass** (below). The resolutions log in `00-decisions.md` was not edited; that file is outside Owns. Integrator: append the resolution line.
- Deviations from the plan (and why):
  - Did not append O2 to `plans/sparky-next/00-decisions.md`. AGENT-PROTOCOL allows only Owns (`spikes/mcp/**`) plus this status file. The line to append is in the decision section.
  - Demo listeners used `127.0.0.1:47610` (SDK) and `127.0.0.1:47611` (hand-rolled) so the spike would not take the product port `47600`. The guard checks the actual listen port.
  - `spikes/mcp` is not in `go.work` (shared file, W0-SCAFFOLD). Tests run with `GOWORK=off` via `spikes/mcp/verify.sh`.
  - Cursor was not launched. The config snippets in `spikes/mcp/testdata/` were parsed, and the same stdio command and HTTP URL were connected by the Inspector and by Claude Code.
- Requests filed (plans/sparky-next/requests/...): none
- Follow-ups for other WS:
  - Integrator: append the resolution line to `plans/sparky-next/00-decisions.md`.
  - **W1-MCP**: implement `providers/mcp` on `github.com/modelcontextprotocol/go-sdk` **v1.8.0**. Keep streamable HTTP stateful (`Stateless: false`, `JSONResponse: true`, `SessionTimeout` 30m). Put the bearer + Host:port + Origin wrapper in front; leave the SDK localhost check on. `sparky mcp` is a stdio proxy to `/mcp`, same shape as `spikes/mcp/proxy`.
  - **W1-MCP**: `tools/list` for these 3 tools was **2420 bytes** on the SDK (inferred input and output schemas, including `"type": ["null","array"]`) and **1079 bytes** hand-rolled. Eighteen tools at the SDK density will miss the 6 KB budget. Pass compact `InputSchema` values and omit output schemas (`Out` = `any`) unless a golden test proves the page still fits. Inspector reported 0 schema errors and 4 portability warnings on the nullable slices.
  - **W1-MCP**: do not set `Stateless: true` until clients require protocol `2026-07-28`. Per-session `expectedRev` needs the session id, and current clients negotiated `2025-11-25`.

## Decision (O2)

Use the official SDK. Do not hand-roll the JSON-RPC server.

**Resolution line:**

`2026-10-03 O2: Use github.com/modelcontextprotocol/go-sdk v1.8.0, stateful streamable HTTP, with a Sparky bearer + Host:port + Origin wrapper. (W0-MCP, plans/sparky-next/status/w0-mcp.md)`

Module (pinned in `spikes/mcp/go.mod`): `github.com/modelcontextprotocol/go-sdk v1.8.0` (published 2026-09-04, protocol set unchanged from v1.7.0: newest is `2026-07-28`, and `2025-11-25` / `2025-06-18` / `2025-03-26` / `2024-11-05` still negotiate).

R3 does not fire. The SDK has streamable HTTP, structured output, and progress. The hand-rolled server stays in this spike as the comparison, not as the production design.

## Comparison

| | Official SDK v1.8.0 | Hand-rolled (Polterware-style JSON-RPC) |
|---|---|---|
| Tools | Yes. `mcp.AddTool` infers and validates schemas. | Yes. Schemas written by hand. |
| Resources | Yes. `sparky://today` listed by the official client. | Yes. Same resource, manual `resources/list` and `resources/read`. |
| Prompts | Yes. `capture`. | Yes. Same prompt, manual `prompts/list` and `prompts/get`. |
| Structured output | Yes. `structuredContent` plus inferred `outputSchema`. Inspector showed both on `get_current_time`. | Yes. `structuredContent` set by hand. No output schema (smaller payload, no SDK validation). |
| Progress | Yes. `NotifyProgress` reached the official client over streamable HTTP (`TestSDKAndHandRoll/sdk`). | Yes. `notifications/progress` on the GET SSE stream reached the same client (`TestSDKAndHandRoll/hand`). |
| Streamable HTTP + sessions | Stateful. `Mcp-Session-Id`, POST JSON (`JSONResponse`), GET SSE, DELETE, idle timeout. `2026-07-28` requires `Stateless: true`, which drops session ids. Inspector and Claude negotiated `2025-11-25`. | Same POST/GET/DELETE session model, implemented here (~203 lines in `handserver/http.go`). No `2026-07-28` path. |
| Error mapping | Wrapper: missing token **401**, bad Host **403**, bad Origin **403**, no CORS. Tool handler errors become `isError` results; a `*jsonrpc.Error` stays a protocol error. Schema failures never reach the handler. | Same HTTP wrapper. Parse **-32700**, invalid request **-32600**, method **-32601**, params **-32602**. Unknown tool is `isError` with one sentence, not a protocol error. |
| `tools/list` payload | **2420 bytes** (3 tools, compact JSON, envelope included). | **1079 bytes**. |
| Lines of code (non-test) | **201** (`sdkserver/server.go`). HTTP transport, negotiation, and SSE are in the module. | **565** (`handserver/dispatch.go` 362 + `http.go` 203). |
| Shared either way | `proxy/proxy.go` 130 (stdio bridge). `internal/guard/guard.go` 124 (bearer, Host, Origin). | same |
| Test ergonomics | In-process `mcp.Client` plus `httptest`. Typed tool handlers. | The official client also connected, so one test drives both. Dispatch is `map[string]any`. |
| Maintenance | Pin v1.8.0. v1.7 rewrote the wire protocol; v1.8 hardened body limits, session leaks, and protocol-version narrowing. We do not carry that. | We would track every spec revision. Polterware's server is still `2024-11-05` and POST-only. |

## Evidence

Commands were run in `spikes/mcp` on 2026-10-03. `GOWORK=off go test -race ./...` and `bash verify.sh` passed.

Inspector (`npx -y @modelcontextprotocol/inspector --cli`), all exited 0:

- SDK HTTP `http://127.0.0.1:47610/mcp` `--transport http` with `Authorization: Bearer spike-token`: `tools/list`, `tools/call get_current_time` (structured content included).
- SDK stdio `/tmp/sparky-w0-mcp/sdk-stdio`: `tools/list`, `tools/call get_current_time`.
- Stdio proxy `/tmp/sparky-w0-mcp/proxy-sdk` → SDK HTTP: `tools/list`, `tools/call get_current_time`.
- Hand-rolled HTTP `http://127.0.0.1:47611/mcp`: `tools/list`.
- Hand-rolled stdio: `tools/call create_memories` returned `created 1`, actor `mcp:inspector-cli`.

The Inspector does not forward environment variables into the stdio child. The proxy wrapper passed `-url` and `-token` on the command line. W1's `sparky mcp` should take flags or read `core.json`, not rely on the parent environment alone.

Claude Code 2.1.288, scope `local` (only `~/.claude.json`, removed afterward):

- `claude mcp add --scope local sparky-w0-spike -- /tmp/sparky-w0-mcp/proxy-sdk` then `claude mcp get sparky-w0-spike` → **Connected** (stdio).
- `claude mcp add --scope local --transport http sparky-w0-http http://127.0.0.1:47610/mcp --header "Authorization: Bearer spike-token"` then `claude mcp get sparky-w0-http` → **Connected** (http).

`curl` against both listeners:

- `Host: evil.example` → **403** `forbidden host`
- `Origin: https://evil.example` → **403** `forbidden origin`
- missing `Authorization` → **401**
- `Origin: http://127.0.0.1:47610` → **200**
- no `Access-Control-Allow-Origin` on the response

The SDK also rejects a non-loopback `Host` when the listener is loopback (`DisableLocalhostProtection` left false). Our wrapper is stricter: the port must match, and any non-loopback `Origin` is rejected. The SDK does not check the bearer token.

Cursor snippets (parsed with `python3 -m json.tool`): `spikes/mcp/testdata/cursor-stdio.json`, `spikes/mcp/testdata/cursor-http.json`.
