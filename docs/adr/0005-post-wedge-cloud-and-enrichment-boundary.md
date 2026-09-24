# ADR 0005: Post-wedge Cloud and enrichment boundary

- Status: Accepted
- Date: 2026-09-24
- Relates to: [ADR 0004](0004-product-positioning-and-wedge.md), [ADR 0001](0001-open-source-cloud-boundary.md), [ADR 0002](0002-single-desktop-and-cloud-connection.md)

## Context

Wedge validation B1–B9 is complete for local use. A 2026-09-24 strategy discussion asked whether Polterware Cloud still justifies payment under the reduced category (Project Context & Change Intelligence for AI Coding), and whether “AI memory”, RAG-as-truth, Nodes/VPS orchestration, or a chat-first Cloud product should define the hosted offer.

ADR 0004 already rejects Mem0-style memory positioning, GraphRAG marketing, and Operations/Nodes as product thesis. This ADR records the post-wedge commercial and enrichment boundary so later specs do not reopen those paths.

Living backlog narrative: [`docs/architecture/post-wedge-direction.md`](../architecture/post-wedge-direction.md). Shipped behavior remains [`docs/spec.md`](../spec.md). Product direction authority remains [`docs/product-vision.md`](../product-vision.md) and this ADR within its scope.

## Decision

1. **Cloud monetization thesis.** Polterware Cloud is justified only by continuity and collaboration on the **same** Polterware Project Model (PPM): project sync across machines, managed backup/restore, organization and project membership, project-scoped hosted MCP, and shared Change History. Later: Teams/ACL, hosted Change Gates, and Web App review/admin. Cloud does **not** unlock local Desktop features. Local value without an account remains mandatory ([ADR 0002](0002-single-desktop-and-cloud-connection.md), [deployment-model.md](../architecture/deployment-model.md)).

2. **Explicit anti-features (do not build as product).** Do not ship: personal or shared “AI memory” as positioning; Nodes / VPS or AWS provisioning / remote execution / process control; an Operations or provider control plane; RAG or embeddings as the **source of truth**; chat as the primary Cloud product.

3. **Context for agents.** Agents receive budgeted PPM packages (`context.get` locally; hosted Context Oracle when Cloud exists). Exact entity resolution, declared relationships, facts, and evidence take precedence. Semantic retrieval may enrich Knowledge when the graph is insufficient; it never replaces PPM provenance or silently becomes project truth.

4. **AI enrichment (later, Cloud-first).** Architecture review, suggest, and summarize capabilities may emit `inferred` proposals with provenance. Optional chat may explain the model. Promotion into durable `declared` / `verified` truth requires a recorded human decision or Change Gate. TypeSafe Jev and any LLM calls run only behind `polterware/cloud` (API keys never in Desktop, renderer, or `polterware-core`). The local product must remain useful offline without those services. `pnpm jev` remains a development aid only.

5. **Git ingest policy.** Prefer Git **metadata and evidence** (HEAD, branch, recent commits, revision stamps on observed facts). Full source-tree indexing and codebase-wide RAG are out of scope. Code excerpts enter context only on demand when evidence is missing.

6. **Delivery sequence.** Deepen local PPM, deterministic import, context, and drift before Cloud. Ship Cloud sync and hosted MCP before Cloud AI enrichment.

## Consequences

- Roadmaps and specs after the wedge must cite this ADR when proposing Cloud, AI, Git ingest, or retrieval features.
- “Context Oracle” means budgeted hosted/local context over the PPM, not a parallel memory store.
- Provider Integrations remain an observe/evidence path ([ADR 0004](0004-product-positioning-and-wedge.md)); they must not grow into machine fleets or hosting.
- Pricing and packaging remain undecided ([product-vision.md](../product-vision.md) non-goals).
