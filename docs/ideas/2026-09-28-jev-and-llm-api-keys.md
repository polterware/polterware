# Jev and user LLM API keys

Status: Idea only — not shipped, not a specification

Date: 2026-09-28

Source: brainstorm against TypeSafe System One (Jev) and a bring-your-own LLM API key, constrained by the current wedge (Project Context & Change Intelligence for AI Coding).

## Boundary already accepted

[ADR 0005](../adr/0005-post-wedge-cloud-and-enrichment-boundary.md) and [`docs/architecture/post-wedge-direction.md`](../architecture/post-wedge-direction.md) already place TypeSafe Jev and vendor LLM calls behind `polterware/cloud`. API keys must not live in the Desktop, the renderer, or `polterware-core`. The local product must stay useful offline. `pnpm jev` remains a development aid only (triage, pick, review, copy).

Building any of the product ideas below on the Desktop, or storing a user LLM key in the local app, requires an explicit ADR change. Until then, treat Cloud-first enrichment as the compatible path.

Jev does not generate prose. It returns typed judgments (Choice, Score, Noul) with probabilities. Code owns the workflow. An LLM generates text. That split matches PPM truth classes: Jev classifies and ranks; an LLM explains; a person or Change Gate promotes the result into durable truth.

Anti-features from ADR 0004 and ADR 0005 still apply: AI memory as the product, RAG or embeddings as the source of truth, chat as the Cloud headline, and full source-tree indexing for an LLM.

## Jev ideas (System One)

Closed judgments over the PPM, Context Compiler, Drift, and AgentRun. Candidates stay in code; Jev selects among them.

### Context Compiler and Agents

| Idea | Judgment | Why it fits |
| --- | --- | --- |
| Context package rerank | Score query-to-candidate relevance (fact, ADR, spec, evidence) | Complements deterministic selection without becoming RAG-as-truth |
| Semantic budget packing | Choice of what still fits the token budget while preserving intent | Agents read less and know more |
| Skill or tool suggestion | Choice among MCP skills and write tools | Same shape as the TypeSafe skill-suggestion cookbook |
| `context.get` intent | Choice: architecture, knowledge, drift, or change | Different package shapes per intent |
| Freshness gate | Noul: does this fact still answer the question? | Drops stale context |

### Drift and change intelligence

| Idea | Judgment | Why it fits |
| --- | --- | --- |
| Semantic drift severity | Score blast radius against ADR or Constitution | Deterministic drift plus a graded reading |
| Drift type | Choice: architecture, dependency, spec, or policy | UI and gates can branch on the category |
| Evidence citation check | Choice: does this passage support the finding? | Same shape as the TypeSafe citation-check cookbook |
| Entity alignment | Score plus field-level Nouls | Decide whether two imported records are the same service |
| Finding inbox priority | Composite score of severity, confidence, and blast radius | Less noise, clearer next action |

### Knowledge (Specs, Library, Sources)

| Idea | Judgment | Why it fits |
| --- | --- | --- |
| Import assist (classification only) | Choice of kind, status, and severity; Noul for Constitution | Import stays deterministic; the label is `inferred` |
| Checklist or Clarify routing | Noul: does this item block mark-ready? | Helps only where the checkbox rule is ambiguous |
| Suggested Spec, feature, and ADR links | Choice among existing graph candidates | Does not invent nodes |
| Constitution semantic lint | One Noul per team rule | Same shape as TypeSafe semantic linting |

### Architecture and Resources

| Idea | Judgment | Why it fits |
| --- | --- | --- |
| `architecture.review` | Fan-out of Nouls and Scores over entities and ADRs | Already named in the post-wedge Cloud AI stack |
| `architecture.suggest` triage | Choice of hold, clear, or review, plus a risk Score | Accept or reject feeds a gate |
| Inventory hierarchy | Choice beam search over a fixed taxonomy | Normalize types without a long LLM call |
| Commander intent | Choice of a typed action and closed arguments | Function-calling shape, not free chat |

### AgentRun and audit

| Idea | Judgment | Why it fits |
| --- | --- | --- |
| AgentRun quality and policy | Nouls: are summary and rationale honest? is the tool call acceptable? | Cheap harness guardrails |
| Mutation routing | Choice: auto-apply, Change Gate, or human | Low confidence escalates |
| Run history classification | Choice of outcome or theme | Cheap map-reduce over traces |

### Development aid only

Extend `pnpm jev` (not the product): boundary checks, design-system checks, copy that might describe planned behavior as shipped, and pick among ADRs. This path does not conflict with ADR 0005.

## User LLM API key ideas

Role: generate and explain. Never the source of truth. A user-supplied key in the Desktop conflicts with ADR 0005 until that decision is revised. The same capabilities can run in Cloud with Polterware-held keys.

### Enrichment that stays `inferred` until promotion

| Idea | What the LLM writes | Safeguard |
| --- | --- | --- |
| Summarize a context package or finding | Prose over an existing package or drift finding | Ephemeral text, or stored as `inferred` |
| Draft Spec, Plan, Tasks, or Clarify | Markdown from the PPM | The person edits, then promotes |
| Explain a canvas relationship | Why an edge exists | Reads only entities already in the model |
| Suggest an ADR from a drift finding | ADR template | Accept promotes it to `declared` |
| Chat that explains the model | Q&A over a package | Secondary UX; not a memory store |

### Jev then LLM cascade

1. Jev filters, ranks, checks citations, and decides whether prose is needed.
2. The LLM runs only on the expensive branch (summary, draft, or chat).
3. Jev checks again: does the draft contradict the Constitution, or is a citation unsupported?

That is the TypeSafe extraction cascade plus guardrails, applied to the PPM.

### Bring-your-own-key surface (only if an ADR allows keys in the product)

| Idea | Detail |
| --- | --- |
| Settings for OpenAI, Anthropic, and similar providers | OS keychain, same posture as provider credentials; never SQLite |
| Default model and fallback | Jev can route difficulty toward a cheaper or a stronger model |
| Per-project quota and kill switch | Calls stop when offline or when no key is present |
| Bring-your-own TypeSafe key | Same pattern if local Jev is reopened by an ADR |
| Provenance on AgentRun | Record provider, model, `inferred`, and token use |

## Suggested order

This order follows the accepted post-wedge sequence. It is a preference, not a commitment.

1. Deepen the local PPM, Drift UI, and local Change Gates without AI.
2. Cloud sync and hosted MCP.
3. Cloud AI: Jev for review, rank, and route; a cheap LLM for summarize and chat; promotion through a human or a gate.
4. Revisit a Desktop bring-your-own-key only if offline AI is an explicit product decision.

Highest leverage inside the wedge, if Cloud AI is built: Context Compiler rerank and budget packing, Drift severity and citation checks, and AgentRun guardrails. Those are Jev judgments with little prose.

Highest leverage for an LLM key: Knowledge drafts and finding explanations, each with an explicit promote action and a Jev check against the Constitution and ADRs.
