# Claude Code Instructions

Project rules live in [`AGENTS.md`](AGENTS.md). Follow them in every session.

## Codex delegation

Delegate to Codex through the `codex@openai-codex` plugin to save Claude usage.

- Large, well-specified tasks (long implementation, refactor, migration, fixing tests): use the `codex-sol` subagent (gpt-6.1-sol, medium effort, background).
- Mechanical tasks (lint fixes, renames, boilerplate, simple investigations): use the `codex-luna` subagent (gpt-6-luna, high effort).
- Do not use `codex:codex-rescue` directly; it leaves the model to the Codex default.
- Both subagents are read-only unless the task text includes `--write`.
- Before delegating, write the full task: goal, files involved, what must not change, and how to validate.
- After Codex finishes, review only `git diff --stat` and read the diff of critical files (contracts, schemas, security, migrations, public APIs). Do not read the full diff or redo the work.
- Keep in Claude: architecture, design decisions, final review.
- Do not delegate small tasks; the handoff costs more than doing them.
- Never enable the review gate (`/codex:setup --enable-review-gate`).
- Do not run Codex tasks with write access until the flow has been validated by Erick.

## Cheap search

For broad searches and code reading, use the `buscador` subagent (Haiku, read-only) instead of reading many files in the main thread.
