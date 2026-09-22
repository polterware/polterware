# Deterministic import (B2)

Status: Shipped for MVP source families

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md).

## RPC

`ProjectModelService.ImportProjectSources(project_id)` reads `projects.root_path`, discovers MVP files, and upserts:

- PPM evidence (`kind = file`)
- PPM facts (`truth_class = observed`, `actor = system`, `source = import:<relpath>`)
- Knowledge items (`kind = source`) with `extra_json.path`

Response counts: `fact_count`, `evidence_count`, `source_count`, `skipped_count`, plus an honest `message`.

## MVP discovery

Project root exact names:

- `go.mod`, `package.json`
- `AGENTS.md`, `CLAUDE.md`, `README.md`
- `Dockerfile`, `docker-compose.yml`, `docker-compose.yaml`

OpenAPI / Swagger (root and `docs/`, `api/`, `openapi/`, `swagger/`):

- `openapi.yaml|yml|json`, `swagger.yaml|yml|json`

Terraform (root and `terraform/`, `infra/`):

- `*.tf` only (no recursive deep walk)

## Idempotency

Stable IDs:

- Evidence: `ev-import-<sha12(project|path)>`
- Facts: `fact-import-<sha12(project|path|key)>`
- Knowledge sources: `src-import-<sha12(project|path)>`

Re-running import upserts the same rows; it does not duplicate.

## Limitations

- No full HCL or OpenAPI schema validation — parsers extract a small deterministic subset.
- No Git history, provider inventory, or filesystem indexing beyond the MVP set.
- `CompileContext` includes imported facts with reasons and evidence (B3). See [`context-compiler.md`](context-compiler.md).
- CLI `polterware scan` is B8; Desktop triggers import from Knowledge → Sources.
