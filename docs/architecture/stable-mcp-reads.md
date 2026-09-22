# Stable MCP reads (B4)

Status: Shipped for `polterware-core mcp` stdio read contract

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`docs/product-vision.md`](../product-vision.md) v3, [`context-compiler.md`](context-compiler.md).

## Tools

| Tool | Input | Output |
|------|-------|--------|
| `context.get` | `project_id`, optional `query`, `entity_id`, `token_budget` | Context Compiler package (reasons + evidence refs) |
| `entity.get` | `id` | One PPM entity |
| `evidence.get` | `id` | One PPM evidence record |
| `drift.check` | `project_id`, optional `entity_id` | Drift result stub (see below) |

Former experimental names (`search`, `compile_context`, `get_entity`) are removed. Retrieval uses `context.get` (`query` / FTS inside the compiler).

## `drift.check` stub

Until B5 (Narrow Drift Checker):

```json
{
  "project_id": "...",
  "entity_id": "...",
  "status": "ok",
  "checker": "none",
  "findings": []
}
```

Empty `findings` with `checker: "none"` means no checker yet, not a permanent “no drift” claim. B5 fills findings without renaming the tool.

## Desktop vs MCP

- MCP stdio exposes the four stable tool names above.
- Desktop Agents → Compile still calls Connect RPC `ContextService.CompileContext` (same package shape; RPC name unchanged).
- Agents skill markdown documents the stable MCP names.

## Out of scope

- Narrow Drift Checker logic (B5)
- AgentRun audit (B6)
- Audited MCP writes (B7)
- CLI surface (B8)
