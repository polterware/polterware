# Minimal AgentRun audit (B6)

Status: Shipped for `polterware mcp` AgentRun tools + Desktop Agents → History

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`stable-mcp-reads.md`](stable-mcp-reads.md), [`docs/product-vision.md`](../product-vision.md) v3.

## Purpose

Reconstruct **context → task → change → evidence → drift** for one agent task. Enterprise audit dashboards are out of scope. Audited PPM/Knowledge mutations are documented in [`audited-mcp-writes.md`](audited-mcp-writes.md).

## Persistence

- `agent_runs` — header: `project_id`, `task`, `agent_label`, `status` (`open` \| `completed`), timestamps
- `agent_run_steps` — ordered steps with `kind` in `context` \| `change` \| `evidence` \| `drift`, `summary`, `payload_json`
- `agent_tool_calls.agent_run_id` — optional correlation for the flat tool-call log

Task lives on the run header, not as a step.

### Step payloads

| Kind | Payload |
|------|---------|
| `context` | `{ query, entity_id, item_ids, evidence_ids }` (light snapshot) |
| `change` | `{ summary, rationale, paths, tool?, subject_ids? }` (declared envelope via `agent_run.append`; audited writes also set `tool` + `subject_ids`) |
| `evidence` | `{ evidence_ids }` |
| `drift` | `{ findings: [{ type, severity, summary, subject_ids }] }` |

## MCP tools

| Tool | Role |
|------|------|
| `agent_run.start` | Create open run (`project_id`, `task`, optional `agent_label`) |
| `agent_run.append` | Append `change` or `evidence` (`change` requires `summary` + `rationale`) |
| `agent_run.complete` | Mark completed |
| `agent_run.get` | Return run + ordered steps |
| `agent_run.list` | List runs for a project |

`context.get` and `drift.check` accept optional `agent_run_id`. When the run exists and is `open`, they append the matching step automatically. Context and drift steps are not appended via `agent_run.append`.

## Desktop

Agents → History lists AgentRuns for the current project and shows the audit chain for the selected run. Connect RPCs: `AgentRunService.ListAgentRuns`, `AgentRunService.GetAgentRun`.

## Limits

- `agent_run.append` change steps remain declared-only; real PPM/Knowledge mutations use audited write tools (B7).
- History is agent-run inspection only; unified human + agent Change History is B9 ([`unified-change-history.md`](unified-change-history.md)).
- No enterprise audit dashboard.
- CLI `polterware` surface is B8 ([`cli.md`](cli.md)).

## Code

- Domain: `backend/domains/agentrun`
- Migration: `backend/platform/db/sqlite/migrations/0012_agent_runs.sql`
- Store: `backend/providers/sqlite/agent_run.go`
- MCP: `backend/providers/mcp/agent_run.go`
- Proto: `schemas/polterware/local/v1/agentrun.proto`
