/** English skill body that teaches an AI how to use Polterware MCP tools. */
export const POLTERWARE_AI_SKILL_MARKDOWN = `# Polterware local project context

Use this skill when the operator asks about a Polterware project’s architecture, resources, knowledge, or bounded context. Polterware keeps a local SQLite project model (PPM) and knowledge store. Reach them through the stable Polterware MCP read tools — not by inventing cloud APIs.

## When to use

- Explain components, resources, environments, or linked library items (docs, ADRs, runbooks) for the active local project.
- Gather a bounded context package before proposing a change.
- Look up a PPM entity or evidence record by id after a context hit.
- Check drift findings for the project (empty until the narrow Drift Checker ships).

## Tools (stable read contract)

| Tool | Purpose |
| --- | --- |
| \`context.get\` | Bounded context package for optional \`query\` / \`entity_id\` under \`token_budget\`. Items include provenance (\`truth_class\`, \`source\`, \`path\`), machine \`reasons\` (why included), and \`evidence\` refs for facts. |
| \`entity.get\` | Fetch one PPM entity by \`id\`. |
| \`evidence.get\` | Fetch one PPM evidence record by \`id\`. |
| \`drift.check\` | Drift findings for a \`project_id\` (optional \`entity_id\`). Returns empty \`findings\` until the narrow Drift Checker is available. |

Always pass the operator’s current \`project_id\` when the tool accepts it.

## Rules

1. Prefer \`context.get\` for “what should I know before changing X?”.
2. Use \`entity.get\` / \`evidence.get\` to drill into ids returned by \`context.get\`.
3. Call \`drift.check\` when asking whether the model disagrees with itself; treat empty findings as “no checker yet”, not “no drift forever”.
4. Treat tool output as local truth with provenance — do not claim hosted or shared workspace data.
5. Do not invent Spec Kit CLI, \`.specify/\`, embeddings, GraphRAG, or fixed PCTX serialization.
6. Credentials and provider tokens are never in SQLite; do not ask Polterware MCP for secrets.

## Cursor MCP config

Configure a local stdio MCP server that runs \`polterware-core mcp\` against the same Desktop database path shown in Agents. Copy the JSON snippet from that screen.
`;

export const POLTERWARE_AI_SKILL_FILENAME = "polterware-ai-skill.md";
