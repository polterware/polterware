# Stable MCP reads (B4)

Status: Shipped for `polterware mcp` stdio read contract

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`docs/product-vision.md`](../product-vision.md) v3, [`context-compiler.md`](context-compiler.md), [`narrow-drift-checker.md`](narrow-drift-checker.md), [`minimal-agent-run-audit.md`](minimal-agent-run-audit.md).

## Tools

| Tool | Input | Output |
|------|-------|--------|
| `context.get` | `project_id`, optional `query`, `entity_id`, `token_budget`, `agent_run_id` | Context Compiler package (reasons + evidence refs); optional context step on open AgentRun |
| `entity.get` | `id` | One PPM entity |
| `evidence.get` | `id` | One PPM evidence record |
| `drift.check` | `project_id`, optional `entity_id`, `agent_run_id` | Narrow Drift Checker result (`checker: "narrow"`); optional drift step on open AgentRun |

Former experimental names (`search`, `compile_context`, `get_entity`) are removed. Retrieval uses `context.get` (`query` / FTS inside the compiler).

AgentRun lifecycle tools (`agent_run.start` / `append` / `complete` / `get` / `list`) are documented in [`minimal-agent-run-audit.md`](minimal-agent-run-audit.md).

## `drift.check` shape

```json
{
  "project_id": "...",
  "entity_id": "...",
  "status": "ok",
  "checker": "narrow",
  "findings": [
    {
      "type": "dependency_added|dependency_removed|edge_decision_conflict|declared_observed_conflict",
      "summary": "...",
      "severity": "info|warning|conflict",
      "subject_ids": [],
      "evidence_ids": [],
      "details": {}
    }
  ]
}
```

Empty `findings` with `checker: "narrow"` means the checker ran and found no disagreements under its rules — not “no checker yet”. Rules and limits: [`narrow-drift-checker.md`](narrow-drift-checker.md).

## Desktop vs MCP vs CLI

- MCP stdio exposes the read tools above plus AgentRun audit tools and audited write tools ([`audited-mcp-writes.md`](audited-mcp-writes.md)).
- Desktop Agents → Context (Compile section) calls Connect RPC `ContextService.CompileContext` (same package shape as `context.get`; RPC name unchanged).
- Desktop Agents → Context (Drift section) calls Connect RPC `ContextService.CheckDrift` (same narrow result shape as MCP `drift.check`; Desktop preview does not pass `agent_run_id`).
- Desktop Agents → History lists justified changes via `AgentRunService.ListChanges` / `GetAgentRun`.
- Agents skill markdown documents the stable MCP names.
- CLI `polterware context` / `polterware drift` share wedge composition with MCP ([`cli.md`](cli.md)).

## Out of scope

- Dedicated Drift product surface outside Agents
- Connect HTTP / session-file CLI mode
