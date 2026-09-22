/** English skill body that teaches an AI how to use Polterware MCP tools. */
export const POLTERWARE_AI_SKILL_MARKDOWN = `# Polterware local project context

Use this skill when the operator asks about a Polterware project’s architecture, resources, knowledge, or bounded context. Polterware keeps a local SQLite project model (PPM) and knowledge store. Reach them through the stable Polterware MCP tools — not by inventing cloud APIs.

## When to use

- Explain components, resources, environments, or linked library items (docs, ADRs, runbooks) for the active local project.
- Gather a bounded context package before proposing a change.
- Look up a PPM entity or evidence record by id after a context hit.
- Check drift findings for the project (import dependency diffs, edges under accepted ADRs without evidence, declared-versus-observed conflicts).
- Record a minimal AgentRun audit trail: context → task → change → evidence → drift.

## Tools

### Reads

| Tool | Purpose |
| --- | --- |
| \`context.get\` | Bounded context package for optional \`query\` / \`entity_id\` under \`token_budget\`. Items include provenance (\`truth_class\`, \`source\`, \`path\`), machine \`reasons\` (why included), and \`evidence\` refs for facts. Pass optional \`agent_run_id\` to append a context step. |
| \`entity.get\` | Fetch one PPM entity by \`id\`. |
| \`evidence.get\` | Fetch one PPM evidence record by \`id\`. |
| \`drift.check\` | Narrow Drift Checker for a \`project_id\` (optional \`entity_id\`). Returns \`checker: "narrow"\` and typed \`findings\`. Pass optional \`agent_run_id\` to append a drift step. |

### AgentRun audit

| Tool | Purpose |
| --- | --- |
| \`agent_run.start\` | Start an open run with \`project_id\` + \`task\` (optional \`agent_label\`). |
| \`agent_run.append\` | Append \`kind: "change"\` (\`summary\` + \`rationale\` required, optional \`paths\`) or \`kind: "evidence"\` (\`evidence_ids\`). |
| \`agent_run.complete\` | Mark the run completed. |
| \`agent_run.get\` | Reconstruct the ordered audit chain for one run. |
| \`agent_run.list\` | List runs for a project. |

Always pass the operator’s current \`project_id\` when the tool accepts it.

## Suggested audit flow

1. \`agent_run.start\` with a clear task.
2. \`context.get\` with the returned \`agent_run_id\`.
3. \`agent_run.append\` change envelope (\`summary\` + \`rationale\`).
4. \`agent_run.append\` evidence ids you relied on.
5. \`drift.check\` with the same \`agent_run_id\`.
6. \`agent_run.complete\`, then \`agent_run.get\` to verify the chain.

## Rules

1. Prefer \`context.get\` for “what should I know before changing X?”.
2. Use \`entity.get\` / \`evidence.get\` to drill into ids returned by \`context.get\`.
3. Call \`drift.check\` when asking whether the model disagrees with itself; treat empty findings as “no drift under the narrow rules”, not “no checker”.
4. Treat tool output as local truth with provenance — do not claim hosted or shared workspace data.
5. Do not invent Spec Kit CLI, \`.specify/\`, embeddings, GraphRAG, or fixed PCTX serialization.
6. Credentials and provider tokens are never in SQLite; do not ask Polterware MCP for secrets.
7. Drift findings are advisory — never overwrite project truth from a finding alone.
8. Change steps are declared envelopes only in this slice; they do not mutate the PPM.

## Cursor MCP config

Configure a local stdio MCP server that runs \`polterware-core mcp\` against the same Desktop database path shown in Agents. Copy the JSON snippet from that screen.
`;

export const POLTERWARE_AI_SKILL_FILENAME = "polterware-ai-skill.md";
