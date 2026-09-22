# Stable MCP reads (B4)

Status: Shipped for `polterware-core mcp` stdio read contract

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

## Desktop vs MCP

- MCP stdio exposes the read tools above plus AgentRun audit tools.
- Desktop Agents → Compile still calls Connect RPC `ContextService.CompileContext` (same package shape; RPC name unchanged).
- Desktop Agents → History lists AgentRuns via `AgentRunService`.
- Agents skill markdown documents the stable MCP names.

## Out of scope

- Audited MCP writes (B7)
- CLI surface (B8)
- Desktop Drift UI
