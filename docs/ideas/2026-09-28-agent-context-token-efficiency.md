# Agent context files and token efficiency

Status: Idea only — not shipped, not a specification

Date: 2026-09-28

Source: brainstorm on context files (`AGENTS.md`, `DESIGN.md`, and similar), a hybrid project folder, TOON as a token-efficient encoding, and whether agents should work only through MCP. Constrained by the current wedge (Project Context & Change Intelligence for AI Coding).

## Boundary already accepted

[`docs/product-vision.md`](../product-vision.md) and [ADR 0004](../adr/0004-product-positioning-and-wedge.md) already reject a proprietary context DSL. Architecture as Code starts as Markdown, JSON, or YAML. PCTX is **KILL**.

Shipped pieces this idea must not replace:

- `ImportProjectSources` discovers a fixed MVP set (including `AGENTS.md` and `CLAUDE.md`) and upserts observed facts plus file evidence. Generic filesystem indexing is not shipped. See [`deterministic-import.md`](../architecture/deterministic-import.md).
- `CompileContext` / MCP `context.get` returns a bounded package of excerpts, reasons, and evidence refs under a token budget. It does not return full knowledge bodies. See [`context-compiler.md`](../architecture/context-compiler.md).
- MCP reads are `context.get`, `entity.get`, `evidence.get`, and `drift.check`. There is no `knowledge.get` that dumps markdown. See [`stable-mcp-reads.md`](../architecture/stable-mcp-reads.md).

Nothing in this note authorizes a new product surface, a `.polterware/` knowledge store, or a change to the MCP contract. Promote only through an explicit spec or ADR update.

## The bar

Polterware must not be worse, in tokens or round-trips, than markdown already in the repo.

If the agent already knows the path, `Read docs/adr/0004-….md` wins. Replicating those files in SQLite and forcing `get` after `get` of the full body is a regression: tool schemas are a fixed context cost, each response breaks prompt cache, and the agent must decide to call the tool.

The gain is not a cheaper encoding of the same prose. The gain is delivering **less, and more correct**, than the agent would find by searching the filesystem — or delivering something the filesystem does not have.

| Situation | Who wins |
| --- | --- |
| The path of the ADR is already known | Repo (`Read`) |
| The agent does not know what to pull, or the budget is tight | Polterware (Context Compiler: excerpts, reasons, evidence) |
| A doc disagrees with the repo | Polterware (Narrow Drift Checker) |
| An agent changed the project model and must justify it | Polterware (AgentRun audit) |

Analogy: the repo is the library. Polterware is the librarian that hands back a few relevant pages, warns that a chapter is stale, and records who changed the model. A librarian that makes you carry the library again is useless.

## Where the product actually wins

### Selection

Without Polterware the agent guesses, opens several ADRs and specs, and spends tokens on prose that does not answer the task.

`context.get` with a query, an entity id, and a token budget returns a small set of excerpts with machine reasons. The long body stays on disk until the agent chooses `Read` on the path.

### Drift and evidence

`AGENTS.md` can say `npm test` while the repo uses `pnpm`. Reading the markdown does not catch that. Declared-versus-observed facts and `drift.check` do. That is the differentiator; import already stores file evidence for the MVP set.

### Audited change

Git blame covers committed files. Knowledge and PPM mutations that live only in SQLite need AgentRun (`summary`, `rationale`, actor). That history is not a substitute for git, and git is not a substitute for it.

## Context files (import, drift, export)

Agents already load conventions from the repo. Known files as of this note (the set moves quickly):

| File | Who reads it |
| --- | --- |
| `AGENTS.md` (root and nested) | Open convention: Codex, Cursor, Copilot, Jules, Amp, Zed, and others |
| `CLAUDE.md` | Claude Code |
| `GEMINI.md` | Gemini CLI |
| `.cursor/rules/*.mdc` | Cursor |
| `.github/copilot-instructions.md` and `.github/instructions/*.instructions.md` | GitHub Copilot |
| `.windsurf/rules/`, `.clinerules/`, `.kiro/steering/`, `.junie/guidelines.md` | Windsurf, Cline, Kiro, Junie |
| `DESIGN.md` | Design-system brief for agents |
| `SKILL.md` under `.claude/skills/` or `.agents/skills/` | Skills loaded on demand |
| Spec Kit constitution and specs | Spec-driven repos |
| `llms.txt` | LLM-oriented docs, more common on sites |

A context-files capability, if built, should treat these as **declared facts with file evidence**, not as an in-app editor for `AGENTS.md`.

Candidates, in wedge order:

1. **Wider import.** Recognize the formats above. Content becomes `declared` or `observed` facts with `file` evidence. Stay inside `ImportProjectSources` and Knowledge → Sources. Do not add a product surface.
2. **Context-file drift.** A declared instruction that disagrees with an observed fact (test runner, missing path, ADR without evidence) is a Narrow Drift Checker finding. This is the part other tools do not do well.
3. **Export / sync.** Generate a short `AGENTS.md` from the PPM and point adapters at it (`CLAUDE.md`, `.cursor/rules`) so one source does not fork into five files. Export is a projection, not a second source of truth.

Stable instructions stay in a short always-loaded file (on the order of 150 lines) that tells the agent to use Polterware MCP or `polterware context` for entity, drift, and history. Skills stay on demand. MCP stays for task-dependent reads and audited writes. The CLI remains the path for agents that have a shell and should not pay tool-schema cost.

## Hybrid layout

A proprietary knowledge folder is the wrong optimization.

- **Human source of truth** stays in conventions agents already know: `AGENTS.md`, `DESIGN.md`, `docs/adr/`, runbooks, Spec Kit layout.
- **Typed index** stays in SQLite (entities, facts, evidence, knowledge) with stable ids.
- **Agent wire** stays the Compiler package. Full markdown is fetched by path, not by dumping the library through MCP.

Optional, and only if a file in git is required: a **generated manifest** (YAML or JSON) of keys and paths, regenerated by `scan`. It is a catalog, not the truth. Do not put knowledge in `.polterware/`. A minimal `.polterware/project.json` (project id, MCP config) is the most that belongs there, and only if a later spec asks for it.

Inventing `.polterware/knowledge/` or a Polterware-only doc format makes the product invisible to any agent that does not call the MCP server. That is the opposite of the wedge.

## TOON

TOON (Token-Oriented Object Notation) is a compact, lossless encoding of the JSON data model for **LLM input**. Official benchmarks show on the order of 40% fewer tokens than formatted JSON, with the gain concentrated on uniform arrays of objects. Nested or irregular prose saves less. Asking the model to **emit** TOON can cost more than it saves, because the format has to be explained.

Use it, if at all, as an **optional codec for the Compiler package** (facts, entities, evidence refs) after a measurement on Polterware's own item shape. Do not store project docs as TOON. Do not ask agents to write TOON back. Markdown and YAML remain the human formats. A TOON knowledge tree on disk is PCTX under another name and collides with the KILL.

Token cost, in practice, is dominated by selection, excerpt length, and progressive disclosure. Encoding is a smaller lever, and only on the tabular slice of a package.

## MCP contract shape this idea implies

Keep three layers. Do not add a fourth that returns every markdown body.

1. **Catalog / package** — `context.get` under a budget: excerpts, reasons, evidence refs, paths.
2. **Record** — `entity.get` and `evidence.get`: one short record.
3. **Full body** — rare, explicit, and preferably `Read` on the path already returned. A full-body tool, if it ever exists, must be opt-in by id and must not be the default path.

Stable facts the agent needs every session ("test command", "local auth is a bearer token") belong in the PPM as short typed facts, not as repeated prose. Long markdown stays for when the agent chooses to go deeper.

## Anti-ideas

- An `AGENTS.md` editor as a product surface.
- MCP-only agents, with no short file and no CLI, as the recommended path.
- A Polterware folder or DSL as the source of project knowledge.
- TOON (or any proprietary encoding) as the documentation standard in the repo.
- Indexing whose only output is the same full markdown the repo already has.
- Treating this note as permission to widen import, drift rules, or MCP tools without a spec change.

## Suggested next step

If this is promoted, the spec change is: wider context-file import, drift findings for context files, and an export projection — still Knowledge → Sources and Agents → Drift. Measure TOON only as an optional serialization of `context.get`, after the package shape is the thing being optimized.
