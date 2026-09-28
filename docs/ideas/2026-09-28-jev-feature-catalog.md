# Jev feature catalog

Status: Idea only — not shipped, not a specification

Date: 2026-09-28

Context: [`../architecture/atlas-grounded-sdd-roadmap.md`](../architecture/atlas-grounded-sdd-roadmap.md), [`../architecture/cloud-architecture.md`](../architecture/cloud-architecture.md), [`../architecture/polter-format.md`](../architecture/polter-format.md). Supersedes the Jev half of [`2026-09-28-jev-and-llm-api-keys.md`](2026-09-28-jev-and-llm-api-keys.md) for the Atlas direction. The local `pnpm jev` development tooling was removed on 2026-09-28; Jev is now a product candidate only.

## Why Jev fits Polterware

TypeSafe Jev returns typed judgments with probabilities, not prose:

| Primitive | Answers | Example in Polterware |
| --- | --- | --- |
| **Choice** | One of a defined set | Which kind of decision is this: dependency, persistence, API, pattern, none? |
| **Noul** | Probability that a condition holds | Does this diff violate the invariant "no network calls during scan"? |
| **Score** | Position on ordered, described levels | How large is the blast radius of this drift finding? |

Two properties change what the product can do:

- **Cheap and fast enough to run on every event** (every push, PR, handoff, and `polter.apply`), where an LLM call per event would be too expensive and slow. This makes an always-on reviewer possible.
- **Probabilities, not text.** Code keeps the workflow and policy; thresholds decide between acting, suggesting, and asking a person. The same input with the same threshold gives the same outcome, which fits Grounded SDD's "bounded, predictable results".

## Ground rules

1. **Code finds candidates, Jev selects, a threshold decides.** Git, FTS5, and the graph produce candidates; Jev never invents nodes, ids, or values.
2. **Jev output is `inferred`.** It becomes `declared` or `verified` only through a person or a gate. Nothing is silently overwritten.
3. **Every judgment is stored** (question version, probabilities, threshold, outcome, human accept/reject) in `judgments`. That is the audit trail and the calibration dataset. Thresholds can change without re-running inference.
4. **Batch questions per event.** Ask all independent questions about one diff or one handoff in a single request (speculative fan-out); code consumes the relevant answers.
5. **Cache by content hash.** The same diff, block, or handoff with the same question version is never judged twice.
6. **Runs in Cloud only** (ADR 0005): keys stay in the Cloud service. Local use would need a new ADR (for example, a user-supplied key). The local product stays fully useful without Jev.
7. **Opt-in per repository for diffs.** Sending code diffs to TypeSafe needs explicit consent and a retention policy.
8. **Measure before trusting.** Treat cookbook thresholds as starting points; calibrate on Polterware's own history and per team.

## Catalog

Priority: **P1** strongest and first to build, **P2** strong follow-up, **P3** later or exploratory. "Pattern" names the TypeSafe cookbook or pattern closest to the design.

### Change and delivery (Git, PRs)

| # | Feature | Judgments | Trigger → outcome | Pattern | Priority |
| --- | --- | --- | --- | --- | --- |
| 1 | **Unrecorded decision detector** | Noul: diff contains an architectural decision not in the graph? Choice: kind (dependency, persistence, API contract, error strategy, pattern, config, none) | Push/PR → Inbox Decision item + PR Check | Parallel questions, confidence-gated routing | **P1** |
| 2 | **Invariant guard** | One Noul per `!` invariant of the touched nodes: violated by this diff? | Push/PR → Check annotation on the lines, Inbox Drift item | Guardrails, parallel questions | **P1** |
| 3 | **Commit ↔ task linking** | Choice among FTS candidate tasks plus "none" | Commit without explicit reference → suggested link | Pre-parsed value extraction, re-ranking | **P1** |
| 4 | **Delivered confidence** | Noul: do the linked commits plausibly complete this task's acceptance? | Task linked → Delivered, or "needs verify" | Citation check | P2 |
| 5 | **Scope creep** | Noul: does the diff touch areas outside the claimed task scope for reasons unrelated to the task? | PR → hint on the card and Check | Guardrails | P2 |
| 6 | **PR ↔ card summary fit** | Noul: does the PR description match the card's intent? | PR opened → hint | Citation check | P3 |

### Continuity (sessions, claims, handoffs)

| # | Feature | Judgments | Trigger → outcome | Pattern | Priority |
| --- | --- | --- | --- | --- | --- |
| 7 | **Handoff verification** | One Noul per claim in the handoff ("done: X"): supported by the diff? | Session end → declared vs observed per line; unsupported lines flagged | Double-checking citations | **P1** |
| 8 | **Handoff quality** | Score: could the next agent continue from this handoff alone? Noul: are open questions stated? | Session end → ask the agent to fill gaps before closing | Composite scoring | P2 |
| 9 | **Silent-default detector** | Noul: did the agent choose a default for an open question instead of asking? | Session end → Inbox item "confirm default" | Guardrails | P2 |
| 10 | **Next-task suggestion** | Score each free task for fit with the finishing session's context | Session end → suggested next task in `status` | Re-ranking | P3 |

### Decisions and Inbox

| # | Feature | Judgments | Trigger → outcome | Pattern | Priority |
| --- | --- | --- | --- | --- | --- |
| 11 | **Duplicate decision/question** | Noul on FTS candidates: same decision? Choice: duplicate, contradicts, related, unrelated | `polter.apply` creates a decision or question → hint to reuse or link | KG entity alignment | **P1** |
| 12 | **Already answered** | Noul: is this new `?` question answered by an active ADR or invariant? | Question created → auto-link answer, close as "answered by ADR-X" after confirm | Line-by-line search | P2 |
| 13 | **Automatic challenge** | Noul: does this new fact or change meet the ADR's review trigger? | New fact/commit → ADR moves to `challenged` (proposed, human confirms) | Guardrails | P2 |
| 14 | **Blocking classification** | Noul: does this question block the task's acceptance? | Question without `[blocking]` → suggested flag | Classification using confidence | P2 |
| 15 | **Owner routing** | Choice among owners/domains (from `CODEOWNERS` and node owners) | Inbox item → routed owner | Intent routing, hierarchical classification | P2 (Cloud teams) |
| 16 | **Inbox priority** | Scores: severity, blast radius, urgency; combined in code with team weights | Inbox list order | Composite scoring | P3 |

### Atlas, Oracle, and context

| # | Feature | Judgments | Trigger → outcome | Pattern | Priority |
| --- | --- | --- | --- | --- | --- |
| 17 | **Oracle packing** | Score relevance of candidate blocks to the task/question | `oracle` / `status` → best blocks within the token budget | Re-ranking, classifying RAG passages | P2 |
| 18 | **Placement suggestion** | Beam search over the Atlas tree: which node should this card live under? | Card without `under:` → suggestion | Hierarchical classification | P2 |
| 19 | **Import classification** | Choice: kind of imported file/section (ADR, runbook, spec, context file, none) | Import → suggested kind, `inferred` | Classification using confidence | P3 |
| 20 | **Entity alignment across repos** | Score + field Nouls: are these two components the same service? | Multi-repo ingest → merge suggestion | KG entity alignment | P3 |
| 21 | **Context file drift** | One Noul per claim in `AGENTS.md`/`CLAUDE.md`: still true given facts? | Import/scan → Drift item | Double-checking citations | P2 |

### Authoring (Polter format, specs)

| # | Feature | Judgments | Trigger → outcome | Pattern | Priority |
| --- | --- | --- | --- | --- | --- |
| 22 | **Semantic lint** | Nouls: task belongs to this card? acceptance criteria testable? invariant contradicts a parent contract? | `polterware lint --semantic` → one-line hints | Guardrails, parallel questions | P2 |
| 23 | **Constitution check** | One Noul per Constitution rule against a new spec/plan | Feature moves to Plan → hints | Guardrails | P3 |
| 24 | **Ambiguity finder** | Noul per sentence: does this requirement admit two incompatible readings? | Shape stage → suggested `?` questions | Line-by-line search | P3 |

### Product UX

| # | Feature | Judgments | Trigger → outcome | Pattern | Priority |
| --- | --- | --- | --- | --- | --- |
| 25 | **Commander intent** | Choice of typed action + closed arguments from a natural-language command ("move git import to verify") | Commander input → typed action, confirm if low confidence | Function calling | P3 |
| 26 | **Skill/tool suggestion for agents** | Choice among Polter commands for an agent's stated goal | Agent asks "how do I…" → exact command | Skill suggestion | P3 |

### Research and calibration

| # | Feature | Judgments | Trigger → outcome | Pattern | Priority |
| --- | --- | --- | --- | --- | --- |
| 27 | **Variance metric (BR)** | Judgments over repeated agent runs on the same task: same decisions taken? | Benchmark runs → evidence that bounded context reduces variance | Autoresearch feature discovery | P3 |
| 28 | **Threshold calibration** | Reuse stored judgments + human accept/reject | Periodic → per-team thresholds | Self-consistency (nouls, choices) | P2 |

## P1 designs

### 1 + 2. Decision detector and invariant guard (one request per diff)

State:

```json
{
  "diff": { "files": ["..."], "hunks": "..." },
  "nodes": [{ "id": "core/import", "invariants": ["No network calls during scan"] }],
  "active_decisions": [{ "id": "ADR-0007", "title": "Deterministic import only" }],
  "task": { "slug": "git-metadata-import", "title": "...", "scope": "api:import/git/*" }
}
```

Questions in one batch:

- Noul: "Does `diff` introduce an architectural decision (a new dependency, storage, API contract, error strategy, or pattern) that is not covered by `active_decisions`?"
- Choice: kind of decision (dependency, persistence, API contract, error strategy, pattern, config, none).
- For each invariant: Noul "Does `diff` violate the invariant `nodes[i].invariants[j]`?"

Policy in code:

- Decision Noul ≥ high threshold → Inbox Decision item (non-blocking) + Check annotation.
- Between thresholds → Check note only ("possible decision").
- Invariant Noul ≥ threshold → Drift item + Check annotation on the hunk; a team may make it a required check.

### 3. Commit ↔ task linking

1. Explicit reference in the commit message → link, done (no Jev).
2. Otherwise FTS5 returns up to five open tasks.
3. Choice: which task does this commit advance? Options: the candidates + "none".
4. High confidence → suggested link shown on the card; never automatic Delivered.

### 7. Handoff verification

1. Split the handoff "Done" lines into claims (code, not Jev).
2. One Noul per claim: "Is `claim` supported by `diff`?"
3. Supported → marked observed; unsupported → flagged in `status` for the next agent and on the card.

### 11. Duplicate decision or question

1. FTS5 + normalization return candidates for a new decision/question.
2. Choice per candidate: duplicate, contradicts, related, unrelated.
3. Duplicate → `polter.apply` returns a one-line hint to reuse the slug; contradicts → Inbox item.

## Cost control

- One batched request per event (diff, handoff, apply), not one per question.
- Skip Jev when deterministic rules decide (explicit references, exact slugs, lint errors).
- Content-hash cache per question version.
- Per-organization quota and a kill switch; Guard plan covers the always-on checks.
- Track cost per judgment kind in `judgments` to prune checks that never change outcomes.

## Suggested order

1. **Dogfood on Polterware's own repository** once Cloud C0/C3 exists: decision detector (1), invariant guard (2), handoff verification (7).
2. Add linking (3) and duplicates (11) with Git ingest and the Polter codec.
3. Calibrate (28) from accept/reject data; then expand to P2 items.
4. P3 items only after P1/P2 show measurable value (caught decisions, fewer duplicates, fewer re-explanations).

## Open questions

- Which P1 check is the Guard plan headline: decision detector or handoff verification?
- Diff size limits: judge whole diffs, per-file hunks, or only files under governed nodes?
- Should a required Check ever block merge on a Jev judgment alone, or only after a human confirms?
- Local Jev with a user-supplied key: worth an ADR for solo users, or Cloud-only forever?
