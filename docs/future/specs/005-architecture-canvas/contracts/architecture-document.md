# Contract: Architecture Document JSON

**Owner**: `@polterware/frontend-canvas` (generic parse/migrate helpers) + `@polterware/feature-architecture` (domain defaults)  
**Transport**: `Layout.body_json` string on `ArchitectureService`  
**Version**: `formatVersion: 1`

## Wire API (unchanged)

```text
GetLayout(system_id) → Layout { id, system_id, revision, body_json }
UpdateLayout(system_id, expected_revision, body_json) → Layout
```

Concurrency: optimistic on `revision`. Body schema evolves independently via `formatVersion`.

## Body schema (normative summary)

See [data-model.md](../data-model.md) for full field tables. Minimal valid v1:

```json
{
  "formatVersion": 1,
  "id": "layout-sys-local",
  "systemId": "sys-local",
  "nodes": [],
  "relationships": [],
  "boundaries": [],
  "preferences": {},
  "viewport": { "x": 0, "y": 0, "zoom": 1 }
}
```

## Operations

| Op | Contract |
| --- | --- |
| `parseAndMigrate(json)` | Returns document v1 or error; never throws raw on corrupt — structured error |
| `serialize(doc)` | JSON string; no selection/history/interaction |
| `validate(doc)` | Structural + referential integrity warnings/errors |
| `migrateLegacy(json)` | format 0 → 1 |
| `exportFragment(selection)` | clipboard/export subset |
| `importDocument(json, { mode })` | validate; preview; replace only on confirm |

## Security

- Treat all imports as untrusted data.  
- No `eval`, no HTML injection into DOM without sanitization.  
- Reject unknown top-level executable keys if ever introduced.  
- Max payload size enforced client-side before send.

## Compatibility

| Input | Behavior |
| --- | --- |
| Legacy `{nodes,edges}` | Migrate to v1 |
| v1 | Load |
| Future v2+ unknown | Error; do not replace open doc |
| Empty `{}` / `""` | Empty v1 document |

## Forbidden

- Storing secrets in body  
- Using body-embedded revision as sole concurrency token (RPC revision remains authority)  
- Silent rewrite by discovery  
