# Idea inbox

This directory collects product ideas that are not yet specifications.

Nothing here is normative. It does not define shipped behavior, product direction, or an accepted architecture decision.

Current authority lives outside this directory:

- [`../spec.md`](../spec.md) defines implemented behavior.
- [`../product-vision.md`](../product-vision.md) defines product direction.
- [`../adr/`](../adr/) records accepted architecture decisions.
- [`../architecture/`](../architecture/) records living architecture notes that are still not a substitute for `spec.md`.

Add one file per idea or theme. Date the filename when the note is a snapshot of a discussion (`YYYY-MM-DD-topic.md`). Promote an idea into a specification or ADR only through an explicit decision; do not treat presence in this folder as approval to build.

Notes:

- Consolidated 2026-09-28 brainstorm (Atlas + Oracle, Grounded SDD, continuity, Inbox) and build order: [`../architecture/atlas-grounded-sdd-roadmap.md`](../architecture/atlas-grounded-sdd-roadmap.md).
- [2026-09-28-engineering-control-plane-overview.md](2026-09-28-engineering-control-plane-overview.md) — visão da discussão, fronteiras com a direção vigente e arquitetura conceitual.
- [2026-09-28-behavioral-runtime.md](2026-09-28-behavioral-runtime.md) — reprodutibilidade comportamental, ações tipadas, contratos, estados e limites de enforcement.
- [2026-09-28-engineering-knowledge.md](2026-09-28-engineering-knowledge.md) — biblioteca de evidências, packs, proveniência, lacunas e cápsulas de contexto.
- [2026-09-28-decision-ownership.md](2026-09-28-decision-ownership.md) — autoridade humana, ADRs, DDD/TDD, estratégia de testes e orçamento de delegação.
- [2026-09-28-research-and-product-validation.md](2026-09-28-research-and-product-validation.md) — hipóteses, métricas, experimento e validação de produto.
- [2026-09-24-post-wedge-strategy.md](2026-09-24-post-wedge-strategy.md) — Cloud, Context Oracle, Git, and enrichment discussion (boundary already in ADR 0005).
- [2026-09-28-agent-context-token-efficiency.md](2026-09-28-agent-context-token-efficiency.md) — context files, hybrid layout, and TOON.
- [2026-09-28-automatic-agent-context-plugin.md](2026-09-28-automatic-agent-context-plugin.md) — plugin, skill, and session hook for bounded automatic agent context.
- [2026-09-28-agent-tooling-sidebar.md](2026-09-28-agent-tooling-sidebar.md) — sidebar routes for agent skills, plugins, hooks, and configuration inventory.
- [2026-09-28-jev-feature-catalog.md](2026-09-28-jev-feature-catalog.md) — 28 Jev features for the Atlas direction (change, continuity, decisions, Oracle, authoring), P1 designs, cost control.
- [2026-09-28-jev-and-llm-api-keys.md](2026-09-28-jev-and-llm-api-keys.md) — Jev judgments and bring-your-own LLM keys.
