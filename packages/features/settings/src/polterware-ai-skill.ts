/** English skill body that teaches an AI how to use Polterware MCP tools. */
export const POLTERWARE_AI_SKILL_MARKDOWN = `# Polterware local project context

Use this skill when the operator asks about a Polterware project’s architecture, resources, knowledge, or bounded context. Polterware keeps a local SQLite project model (PPM) and knowledge store. Reach them through the experimental Polterware MCP tools — not by inventing cloud APIs.

## When to use

- Explain components, resources, environments, or linked library items (docs, ADRs, runbooks) for the active local project.
- Gather a bounded context package before proposing a change.
- Look up a PPM entity by id after a search hit.

## Tools (experimental — names may change)

| Tool | Purpose |
| --- | --- |
| \`search\` | Full-text / exact knowledge search within a \`project_id\`. Optional \`kind\`, \`limit\`. |
| \`compile_context\` | Bounded context package for optional \`query\` / \`entity_id\` under \`token_budget\`. Items include provenance (\`truth_class\`, \`source\`, \`path\`), machine \`reasons\` (why included), and \`evidence\` refs for facts. |
| \`get_entity\` | Fetch one PPM entity by \`id\`. |

Always pass the operator’s current \`project_id\` when the tool accepts it.

## Rules

1. Prefer \`compile_context\` for “what should I know before changing X?”.
2. Prefer \`search\` for named library docs, ADRs, runbooks, or specs.
3. Treat tool output as local truth with provenance — do not claim hosted or shared workspace data.
4. Do not invent Spec Kit CLI, \`.specify/\`, embeddings, GraphRAG, or fixed PCTX serialization.
5. Credentials and provider tokens are never in SQLite; do not ask Polterware MCP for secrets.

## Cursor MCP config

Configure a local stdio MCP server that runs \`polterware-core mcp\` against the same Desktop database path shown in Agents. Copy the JSON snippet from that screen.
`;

export const POLTERWARE_AI_SKILL_FILENAME = "polterware-ai-skill.md";
