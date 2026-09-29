# Deterministic import (B2)

Status: Shipped for MVP source families

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md).

## RPC

`ProjectModelService.ImportProjectSources(project_id)` reads `projects.root_path`, discovers MVP files, and upserts:

- PPM evidence (`kind = file`)
- PPM facts (`truth_class = observed`, `actor = system`, `source = import:<relpath>`)
- Knowledge items (`kind = source`) with `extra_json.path`

Response counts: `fact_count` (file facts only), `evidence_count` (file evidence plus Git rows upserted on this scan), `source_count`, `skipped_count`, plus an honest `message`. The message names the Git branch when metadata was read, and says Git metadata was skipped when the root is not a work tree.

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

## Git metadata

When `root_path` is a Git work tree with at least one commit, the same import upserts commit evidence. The read is local (`rev-parse`, `git log`); it does not fetch.

- Current HEAD: `ev-git-head-<sha12(project)>`, `kind = commit`, `source = git:HEAD`, `path` = full SHA, `summary` = branch (`HEAD` when detached). A new SHA replaces the previous one on this row.
- Recent commits: up to 20 first-parent commits, `ev-git-<sha12(project|sha)>`, `source = git:commit`, `path` = full SHA, `summary` = subject. Commits that leave the window stay in the database.
- Revision stamp: every file fact upserted by that scan is linked to the HEAD evidence row. There is no revision column.

A directory that is not a work tree, or a repository with no commits, adds no commit evidence and does not fail the import.

Extra checkouts live in `project_repos` (`polterware repo add --name <slug> --path <dir>`). Each one is read the same way, with ids `ev-git-head-<sha12(project|name)>` and `ev-git-<sha12(project|name|sha)>`, and sources `git:<name>:HEAD` / `git:<name>:commit`. Removing a checkout leaves the evidence rows. The project directory itself is not a row in `project_repos`; registering that path is rejected. File facts are stamped only with the project-root HEAD.

## Limitations

- No full HCL or OpenAPI schema validation — parsers extract a small deterministic subset.
- No commit-to-task links, no provider inventory, and no filesystem indexing beyond the MVP set. Merge commits are omitted (`--first-parent`).
- `CompileContext` includes imported facts with reasons and evidence (B3). See [`context-compiler.md`](context-compiler.md).
- CLI `polterware scan` is shipped (B8); Desktop also triggers import from Knowledge → Sources.
