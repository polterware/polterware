# Polter format and agent protocol

Status: Proposed (2026-09-28 discussion). Nothing here is implemented.

Authority: none yet. Part of [`atlas-grounded-sdd-roadmap.md`](atlas-grounded-sdd-roadmap.md). Requires an ADR before it replaces existing MCP tools, because it changes the stable MCP surface ([`stable-mcp-reads.md`](stable-mcp-reads.md), [`audited-mcp-writes.md`](audited-mcp-writes.md)).

## Problem

Agents should be able to create and update project state (a roadmap, Work cards, tasks, open questions, handoffs) as easily as they write a markdown file, and cheaply in tokens.

Today, creating one Work feature over MCP takes about seven calls: `agent_run.start`, five `knowledge.upsert` (spec, plan, tasks, clarify, checklist, each with a full body and envelope), and `agent_run.complete`. A ten-feature roadmap is roughly seventy calls. It is slow, expensive, and easy to duplicate.

Token cost is driven by three things, in order:

1. Number of calls (each call repeats context and envelope).
2. Tool schemas loaded into every session (every MCP tool costs tokens even when unused, especially in Codex).
3. Payload format (JSON repeats keys; prose does not compress).

## Decision direction

A small markdown dialect, the **Polter format**, is the single language between agents and Polterware, in both directions. It works the way Mermaid does: a text convention the model writes, a tool that parses and validates it, and an app that renders it.

- **Valid markdown.** Renders on GitHub and in any editor; models already know most of the syntax. The extension is conventions on top of it, not new syntax.
- **Real parser, not regex.** A small line-oriented grammar in the Go core, deterministic, with line-numbered errors.
- **Versioned.** Frontmatter `polter: 1` so the grammar can evolve without breaking old files.
- **A language, not a storage format.** Polterware (local SQLite, and Cloud for teams) is always the source of truth. Polter text is how agents talk to it: written as input, returned as output. A `.pw.md` file is only an optional input or a convenience copy, never the truth.

### Why not TOON or JSON

- TOON saves tokens against JSON for uniform arrays of objects. It does not compress free text (goals, tasks, questions), which is most of this content.
- Models write markdown reliably. A less familiar format produces syntax errors; one retry costs more than the format saves.
- The biggest savings come from fewer calls and fewer tool schemas, not from payload encoding.
- List outputs (many items with the same fields) may use a TOON-like header table, which is the one place that shape pays off.

## Source of truth and storage

Decided 2026-09-28 (revised the same day; an earlier draft made repo files the truth).

- **Polterware is the truth, locally and in Cloud.** One project spans many repositories and services; the Atlas is the view above them, so the truth cannot live in any single repository.
- **Relational SQL, normalized.** Nodes, edges, tasks, decisions, questions, claims, facts, evidence, sessions, and handoffs are typed tables. Polter text is never stored as an opaque blob; prose bodies are text columns.
- **Append-only change log.** Every mutation is an operation with actor, `summary`, `rationale`, and timestamp (the existing change envelope). The log is the audit trail (Unified Change History) and the sync contract with Cloud.
- **Stable ids generated where the change happens** (ULID or similar), so offline creation works and ids never collide between machines. Slugs are human handles resolved to ids.
- **FTS5 index** over text blocks for search and duplicate hints (see below).

### Repositories: evidence in, projections out

```text
                 ┌──────────────────────────────┐
                 │  Polterware (source of truth)│
                 │  local: SQLite · team: Cloud │
                 └──┬─────────┬─────────┬───────┘
          evidence ▲│projection▼│        │
        ┌──────────┴┴┐ ┌───────┴────┐ ┌──┴─────────┐
        │ repo: api  │ │ repo: web  │ │ repo: infra│  + services
        └────────────┘ └────────────┘ └────────────┘
```

- **Evidence in:** commits, branches, imported files, provider observations. Each Atlas node records where it lives (`repo: org/api`, `path: src/auth`).
- **Projections out,** in formats engineers already recognize, so an agent without Polterware still reads the essentials:
  - ADRs as `docs/adr/NNNN-*.md` in the repository they govern (standard format, import and export).
  - A generated block in `AGENTS.md` with the decisions and invariants that apply to that repository.
- **Cross-repository features** are one card with tasks per repository: `- [ ] Callback endpoint [repo: api]`. Claim scopes include the repository: `api:src/auth/*`.
- **A `.polter/` folder is optional.** A team may keep a roadmap file next to the code as a convenience; applying it is an input like any other.

## Codec (encode and decode engine)

One Go package in the core, used by CLI, MCP, Desktop (through the core), and Cloud (same code):

```text
 write:  text ─► decode ─► AST ─► plan (diff vs current state) ─► validate ─► apply (one transaction + change ops)
 read:   query ─► rows ─► encode (zoom, budget) ─► text
```

- **Decode:** line-oriented parser to an AST with line numbers. Lint is decode + validate without apply.
- **Plan:** compare the desired state in the text with the store and produce operations (create card, add task, check task, open question). `polterware apply --dry-run` prints the plan, like `terraform plan`.
- **Apply:** one transaction per call, one change envelope, operations appended to the log.
- **Encode:** render rows back to Polter text at a zoom level and token budget (Oracle, `status`, lists, one card).
- **Round trip:** `encode(decode(x))` equals the normalized `x`; this is the codec's main property to test.
- **Partial by default:** a text only touches what it mentions. Removal is explicit (`status: removed` or a scoped `--prune`), never inferred from absence.
- The Desktop UI keeps typed APIs (proto); the codec is the agent-facing surface. Both converge on the same operation layer.

## Grammar sketch (v1)

```markdown
---
polter: 1
---

## git-metadata-import: Git metadata import
stage: plan · under: core/import · decisions: ADR-0007

Import HEAD, branch and recent commits as evidence.

- [ ] Commit evidence upsert in core  `backend/domains/ppm/*`
- [x] Stamp revision on observed facts
? Include merge commits? [blocking] (yes | no | first-parent)
! No network calls during scan
```

| Syntax | Meaning |
| --- | --- |
| `## slug: Title` | A card (feature node). The slug is its identity. |
| `key: value` lines (separated by `·` or new lines) right after the heading | Metadata: `stage`, `under` (parent node), `decisions`, `owner`, … |
| Paragraph | Intent / description |
| `- [ ]` / `- [x]` | Task; optional backticked path glob is the task write scope |
| `? Question [blocking] (a \| b \| c)` | Open question with optional blocking flag and options; becomes an Inbox Decision item |
| `! Statement` | Contract invariant; inherited by children |

The whole spec must fit in about 150 tokens so it can be taught cheaply. Keep few symbols, all close to things models already know (checkbox, frontmatter, `key: value`).

**Identity and idempotency.** Upsert by slug. Re-applying an edited text updates cards and tasks instead of duplicating them. Removal is explicit; history is never deleted.

## Lint

`polterware lint <file|dir|->` validates Polter text (files or stdin).

- **Deterministic.** No LLM. Same input, same result.
- **Silent on success.** Correct files cost zero tokens of feedback.
- **Short, actionable errors,** one line each, with the fix:

  ```text
  roadmap.pw.md:14 task outside a card: move under a "## slug: title"
  roadmap.pw.md:3  unknown stage "doing": use shape|plan|build|verify|delivered
  roadmap.pw.md:21 duplicate slug "drift-panel" (also line 8)
  ```

- **Tolerant.** Accepts harmless variations (`- [X]`, extra spaces, `-` vs `·` separators); only rejects what is ambiguous.
- **Two levels.**
  - *Syntax:* structure, known keys and values, duplicate slugs, version header.
  - *Reference:* `under:` points at an existing node, `decisions:` at an existing ADR, scopes are valid globs.
- **Run automatically.** A Claude Code `PostToolUse` hook runs lint when an agent writes a `.pw.md` file, so the agent sees only errors, right away. The error message itself teaches the format.
- **Fix mode later.** `--fix` normalizes safe variations (checkbox case, separators, key order).

## Transports

One format, three paths:

| Kind | Path | Why |
| --- | --- | --- |
| Durable state (cards, decisions, contracts, facts, handoff) | `polterware apply <file>` or `polter.apply(text)` into the store | One call per batch; the file, if any, is only input |
| Coordination (claim a task, session, verify) | CLI or MCP command | Needs an arbiter now: two agents must not claim the same scope, and files cannot lock |
| Queries (status, oracle, drift, lists) | CLI or MCP command | Answer is returned in Polter format |

### MCP: three generic tools

The MCP surface shrinks to about three tools that take and return Polter text:

| Tool | Purpose |
| --- | --- |
| `polter.apply(text)` | Apply writes and coordination commands (upsert cards, claim, handoff) |
| `polter.query(text)` | Status, oracle, lists, single card |
| `polter.format()` | Return the format spec (for agents without the skill) |

Example:

```text
→ polter.query("oracle core/import zoom:task budget:800")
← ## git-metadata-import: Git metadata import
  stage: build · claim: codex (import/git/*)
  ! No network calls during scan          (from core)
  ? Include merge commits? [blocking]
  - [ ] Stamp revision on observed facts
```

Tradeoff: tools lose MCP schema validation; the lint errors replace it, by line.

Existing tools (`context.get`, `entity.get`, `evidence.get`, `drift.check`, `fact.upsert`, `knowledge.upsert`, …) remain until an ADR approves the migration.

### Reads stay compact

- Lists are one line per item (or a TOON-like header table), never full bodies:

  ```text
  git-metadata-import  plan   2/5 tasks  1 open?  "Git metadata import"
  drift-panel          verify 3/3 tasks          "Drift panel"
  ```

- Full bodies only on request for one card (`polter.query("card git-metadata-import")`).

### Audit without ceremony

The session-start hook opens the AgentRun and stores its id for the session. Each write carries only a short `summary` and `rationale`. No explicit start/complete calls. Human and agent changes stay in Unified Change History.

## Channels

| Channel | Standing cost per session | For |
| --- | --- | --- |
| CLI + plugin skill (`polterware apply`, `lint`, `format`, `status`) | Near zero; only the skill description is in context until used | Claude Code |
| One line in `AGENTS.md` + CLI (`polterware format` prints the spec) | One line | Codex and others |
| MCP (`polter.apply`, `polter.query`, `polter.format`) | Three small schemas | Agents without a shell |

CLI is the primary channel; MCP mirrors it with the fewest tools possible.

## Teaching the model

Unlike Mermaid, no model has seen this format in training. So:

- The spec is tiny (~150 tokens), in a plugin skill loaded only when the agent touches the roadmap.
- `polterware format` prints the same spec for non-Claude agents.
- Lint errors include the correction.
- Later: editor extensions (highlight, preview), like Mermaid had.

## Similarity and matching (no embeddings first)

Decided 2026-09-28: start without embeddings, RAG, or vector search. Use deterministic tools that already exist, measure what they miss, and only then consider embeddings.

1. **Normalization.** Slugs and titles are compared after removing accents and case, reducing plurals, and ignoring word order and stop words (`"Git import"` ≈ `"Import Git metadata"`).
2. **SQLite FTS5.** A full-text index (BM25 ranking, built into SQLite, no model) over Polter blocks: card titles and intents, questions, invariants, decisions. Each block is one indexed row, re-indexed only when its hash changes.
3. **Explicit references by convention.** Commit messages and PR titles that cite a card slug and task (`git-metadata-import T2`, or `polter: git-metadata-import#T2`) link deterministically and exactly. Similarity is never needed when the reference is explicit.

Uses:

- **Duplicate warning on apply/lint.** When a new card, question, or decision closely matches an existing one, `polterware lint` / `polter.apply` report a one-line hint, never a hard error: `roadmap.pw.md:30 similar to "git-metadata-import" (card): same? reuse the slug`.
- **Placement hint.** A card without `under:` gets the best-matching node suggested.
- **Commit ↔ task linking** in Git import: explicit references first; FTS5 match only as a suggestion, never as automatic "Delivered".
- **Prose search** in Commander and Oracle.

Measure before adding embeddings: count duplicates and commit links that FTS5 + normalization miss on real use (Polterware's own development). If the miss rate is material, add local embeddings (`sqlite-vec` + a small local model, downloaded only when enabled) as an optional layer. Known costs: binary size, ONNX Runtime via cgo in a single cross-platform binary, and threshold tuning.

## Open points

- How existing Knowledge spec items (Spec/Plan/Tasks/Clarify/Checklist) map onto the new tables.
- Projection details: which decisions go to which repository's `AGENTS.md` block, and how ADR edits made in a repository are imported back.
- Cloud replication of the change log (see roadmap "Pending discussion").
- Whether `status` and `handoff` use the same card grammar or a small sibling block.
- Jev: still under discussion (see roadmap "Pending discussion").
