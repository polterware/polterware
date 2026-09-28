# Context Compiler (B3)

Status: Shipped for `CompileContext` package shape

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`docs/product-vision.md`](../product-vision.md) v3.

## RPC

`ContextService.CompileContext(project_id, query?, entity_id?, token_budget?)` assembles a bounded package from:

1. Entity resolution (`entity_id`) plus one-hop relationships and knowledge links
2. Exact / FTS knowledge hits and entity title/ref matches (`query`)
3. PPM facts (entity-scoped, query statement match, or project sample) with linked evidence
4. Provenance-aware ranking and token-budget packing

This package shape is exposed to agents as MCP `context.get` (B4). The Connect RPC remains `CompileContext` for Desktop Agents → Context. See [`stable-mcp-reads.md`](stable-mcp-reads.md).

## Item shape

Each `ContextItem` includes:

- Provenance: `kind`, `id`, `title`, `excerpt`, `truth_class`, `source`, `path`
- `reasons` — stable machine codes explaining inclusion
- `evidence` — compact refs (`id`, `kind`, `summary`, `path`, `source`) when the item is a fact (or otherwise carries support)

### Reason codes

| Code | Meaning |
|------|---------|
| `exact_entity` | Matched `entity_id` |
| `knowledge_link` | Knowledge linked to the resolved entity |
| `relationship_hop` | One-hop related entity |
| `fts_hit` | Knowledge full-text / exact search hit |
| `title_match` | Entity title or ref contains the query |
| `project_sample` | Fallback sample when query and entity are empty |
| `fact_for_entity` | Fact linked to the resolved entity |
| `fact_query_match` | Fact statement contains the query |
| `imported_fact` | Observed / import-sourced fact in the empty sample |

## Ranking (summary)

Truth class: `verified` > `declared` > `observed` > `operational` > `inferred`.  
Kind boost: `entity` > `fact` > ADR / docs / runbooks / specs.

No embedding model, vector store, reranker, or GraphRAG.

## Desktop

Agents → Context previews the package (optional query, optional entity id, budget). Empty query compiles the project sample (entities + imported facts).

## Limitations

- Semantic / embedding retrieval is out of scope.
- Evidence on non-fact items is not required in this slice.
- Narrow Drift Checker findings for MCP `drift.check` are implemented; see [`narrow-drift-checker.md`](narrow-drift-checker.md).
