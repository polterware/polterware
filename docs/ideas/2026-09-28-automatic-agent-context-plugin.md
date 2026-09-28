# Automatic agent context through a plugin

Status: Idea only — not shipped, not a specification

Date: 2026-09-28

Source: discussion about making Polterware useful automatically at the start of an AI coding session, with a plugin, a session hook, a skill, or both.

## Existing pieces

Polterware already exposes local MCP reads (`context.get`, `entity.get`, `evidence.get`, `drift.check`), a CLI (`polterware context` and `polterware drift`), and an AI skill markdown export in Desktop Agents. The current export is a `.md` file; packaging a conventional `SKILL.md` for an agent plugin would be separate work. See [`stable-mcp-reads.md`](../architecture/stable-mcp-reads.md), [`cli.md`](../architecture/cli.md), and [`agent-context-token-efficiency.md`](2026-09-28-agent-context-token-efficiency.md).

## Proposed experience

Package a Polterware skill, the local MCP connection, and an optional `SessionStart` hook in one agent plugin.

1. The hook runs when a supported local agent session starts. It resolves the current repository to a registered Polterware project and exits quietly when there is no match.
2. For a match, it fetches a small, read-only context package with a fixed token budget and injects a short summary, project id, provenance, and relevant evidence paths into the agent session. It can also include a bounded `drift.check` result when useful.
3. The skill tells the agent when to request task-specific `context.get`, inspect an entity or evidence record, check drift after relevant changes, and use AgentRun for audited model writes.
4. The agent reads full repository documents by path only when needed. Startup should not dump full knowledge bodies or repeat `AGENTS.md`.

The hook supplies an initial snapshot; the skill guides later decisions. A startup hook does not by itself guarantee that the agent will keep context fresh or run every task-specific check.

## Constraints to resolve before implementation

- Scope the hook to the active repository and registered project. Do not inject another project's context into an unrelated session.
- Keep startup fast, bounded, and read-only. Do not run `scan`, change the PPM, start an AgentRun, or write files merely because a session opened.
- Treat `drift.check` findings as advisory. Empty findings only mean no disagreement under the narrow shipped rules.
- Handle an unavailable binary, MCP server, or database with a short diagnostic or quiet fallback; do not block unrelated coding work.
- Avoid secrets and large source excerpts in hook output. Show provenance so the agent can verify the selected context.
- Account for host-specific hook support, installation, and trust. A plugin hook must be reviewed and trusted before it runs in Codex; other agents may need their own adapter.
- Measure startup time, injected tokens, project-match accuracy, and whether the context reduces unnecessary file reads. Keep the hook optional if its overhead outweighs the benefit.

## Suggested next decision

Prototype the smallest Codex-local package: conventional `SKILL.md`, local MCP configuration, and one `SessionStart` hook that injects a bounded context package for a matching project. Compare it with the skill-only flow before adding automatic drift or adapters for other agents. This note does not authorize implementation or change the shipped MCP contract.
