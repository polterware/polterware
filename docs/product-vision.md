# Polterware Product Vision

Status: Active product direction

Version: 3.0

Last revised: 2026-09-22

## Authority and scope

This document defines the intended direction and the product principles that future specifications must preserve. It does not claim that every capability described here exists today.

The documentation hierarchy is:

1. `docs/spec.md` is authoritative for currently implemented product behavior.
2. Accepted ADRs are authoritative for architecture decisions within their stated scope ([ADR 0004](adr/0004-product-positioning-and-wedge.md) records this positioning).
3. This document is authoritative for product direction and the invariants that new specifications must respect.
4. `docs/future/specs/` is a historical archive and is not normative.

When current behavior and product direction differ, documentation and UI copy must label that difference explicitly. Planned behavior must never be presented as shipped behavior. Condemned surfaces still listed in `docs/spec.md` remain shipped until a future kill branch removes them; they must not be expanded.

## Category and positioning

**Category:** Project Context & Change Intelligence for AI Coding.

**One-liner:** Polterware gives every coding agent the same evidence-backed understanding of your project — and flags when their changes drift from your architecture, decisions or specs.

**Headline:** Stop re-explaining your project to every coding agent.

**Supporting line:** Agents read less and know more.

Polterware is not positioned as a generic “source of truth” headline, an Internal Developer Platform, a software catalog, Mem0 for developers, a documentation editor, a GraphRAG product, an architecture diagrammer, or an Operations/control plane.

## Product thesis

Coding agents reconstruct different pictures of the same project. Shared memory and retrieval alone are commoditizing. The durable thesis is:

> Maintain a local, verifiable model of declared intent and observed software state; give each coding agent only the context it needs, with provenance; detect when resulting changes diverge from that intent.

The central value is the relationship between intent (ADRs, specs, decisions), architecture and dependencies, observed implementation, evidence, and the actors (people and agents) who changed them.

## Product principles

### One project model

Every product surface reads from or contributes to the same Polterware Project Model. The product must not create parallel architecture, documentation, or agent models that silently diverge.

### Structured before inferred

Polterware should import deterministic structure directly from source systems whenever possible. Package manifests, OpenAPI documents, Terraform, Docker, Git, and provider APIs already express facts that do not need an LLM to rediscover them.

AI inference may enrich the model, but it must remain distinguishable as `inferred`.

### Provenance before confidence

A statement is not useful as project context unless Polterware can explain where it came from, when it was observed, what revision or environment it describes, and whether a person verified or approved it.

### Local value without Cloud

The Desktop must remain useful without an account or hosted service. Cloud adds managed continuity, collaboration, synchronization, administration, and operated infrastructure; it does not unlock local ownership of project data.

### People and agents use the same model

Agents must not receive a hidden or privileged project representation. Human and agent interfaces use the same domain contracts, provenance rules, and audit semantics, with each mutation attributed to the correct actor.

### Verification over ceremony

Drift findings and optional Change Gates introduce friction only where blast radius or confidence require it. Drift is a finding, never an automatic overwrite.

### Observe providers; do not operate them

Integrations and importers may read provider and infrastructure state as evidence. The product thesis does not include operating those systems as a control plane.

## Polterware Project Model

The Polterware Project Model, or PPM, is the canonical representation of a software project.

```text
Sources
  Git, code, manifests, infrastructure, providers, people, agents
                              |
                              v
                 Polterware Project Model
                              |
        +---------------------+---------------------+
        |                     |                     |
        v                     v                     v
    Desktop/UI          Context Compiler       CLI and MCP
```

The graph is internal structure that improves context, provenance, and verification. It is not the visible product.

### Core entities

The model is expected to represent entities such as:

- projects, environments, repositories, branches, and revisions;
- components, services, APIs, databases, queues, and infrastructure resources;
- specifications, requirements, plans, tasks, decisions, and ADRs;
- documents, runbooks, and evidence;
- commits, code symbols, tests, build results, and provider observations;
- facts, dependencies, people, agents, agent runs, approvals, and overrides.

This list describes the conceptual model. Each entity requires an accepted feature specification and contract before implementation.

### Relationships

Relationships give the model its meaning. Examples include:

```text
Component DEPENDS_ON Component
Service USES Database
Component GOVERNED_BY Decision
Spec DEFINES Requirement
Requirement SATISFIED_BY CodeSymbol
Task IMPLEMENTS Requirement
Test VERIFIES Requirement
ADR EXPLAINS Decision
Fact OBSERVED_IN File
Evidence SUPPORTS Fact
AgentRun PRODUCED Commit
Human APPROVED Decision
Decision SUPERSEDES Decision
```

Relationships must be typed, queryable, attributable, and evidence-backed. The graph is a product model, not an artifact generated after semantic search.

### Provenance

Entities, relationships, and claims should carry enough provenance to answer:

- Which source declared or observed this fact?
- Which branch, revision, and environment does it describe?
- When was it observed or last confirmed?
- Was it imported, entered by a person, produced by an agent, or inferred?
- What evidence supports it?
- Has a person verified, approved, or overridden it?

## Classes of truth

Polterware distinguishes four status classes for project knowledge:

### Declared

Intentional statements owned by the project: specifications, requirements, architecture decisions, ownership, policies, and human-approved architecture.

### Observed

Facts imported from authoritative sources: code, Git, manifests, infrastructure definitions, provider inventories, and environment-specific state.

### Inferred

Summaries, semantic relationships, classifications, and other conclusions produced by AI or heuristic analysis. Inferred knowledge must never silently replace declared or observed facts.

### Verified

A declared or observed fact sustained by sufficient evidence and, when required, human acknowledgement.

Runtime outcomes (deployments, runs, logs, incidents, verification results) are stored as **evidence** that supports facts — not as a parallel “truth class” that competes with the four statuses above.

Promotion of an inference into project truth requires a recorded human decision, including the actor, reason, affected claim, and supporting evidence.

## Import, export, and Architecture as Code

Polterware should build the PPM through deterministic importers before relying on semantic inference.

Initial source families include:

- Git and filesystem structure;
- package and build manifests;
- Docker and container definitions;
- `AGENTS.md`, `CLAUDE.md`, ADRs, and existing Markdown;
- Terraform and other supported infrastructure declarations;
- OpenAPI and supported interface definitions;
- authenticated provider CLI or API observations (read path);
- Polterware-authored specifications, decisions, and canvas edits that update the same model.

Architecture as Code provides a versionable, human-readable projection of relevant parts of the PPM. Markdown, JSON, or YAML are sufficient initially. Proprietary DSLs (including PCTX) are not part of the wedge.

Importers must report their source, scope, revision, environment, and limitations.

## Context Compiler

The Context Compiler produces the smallest sufficient project context for a person or agent under an explicit token budget.

Target shape:

```text
context.get(target, intent, budget)
  -> entity resolution
  -> exact and full-text retrieval
  -> graph expansion
  -> provenance-aware ranking
  -> deduplication and compression
  -> token-budgeted context package with reasons and evidence
```

Exact identifiers and declared relationships take precedence over semantic similarity. Semantic retrieval is an enrichment path only when measured need justifies it.

No embedding model, vector extension, reranker, or GraphRAG implementation is selected by this document. GraphRAG is an implementation detail, not a marketing feature.

## Spec-driven development

Specifications become structured, versioned entities instead of isolated Markdown files.

```text
Feature
  -> Spec
     -> Requirements
        -> Plan
           -> Tasks
              -> Evidence
```

Markdown import and export may remain available for interoperability. A full spec-authoring suite is not the wedge; structured specs linked to the PPM are a secondary capability after context and drift prove value.

### Change Gates (Assurance Gates)

A Change Gate introduces optional human review before an artifact becomes durable project truth. Gates may address ambiguous intent, architectural conflicts, high blast radius, security implications, and explicit overrides.

Do not use “Anti-Vibe” as public branding. Exact levels and UI require a separate later specification.

## Agent access

Desktop, CLI, and MCP are interfaces over the same domain behavior. MCP is the primary programmatic interface for external coding agents.

The wedge MCP **read** contract targets:

- `context.get`
- `entity.get`
- `evidence.get`
- `drift.check`

After AgentRun exists, the wedge also targets **audited MCP writes**: narrow semantic mutations over PPM and Knowledge (not a mirror of every Desktop surface). Every mutation requires `summary`, `rationale`, and `agent_run_id` and is rejected if any are missing. Mutations are versioned and attributable through AgentRun.

Agent access must preserve explicit project scope, minimum necessary context, actor attribution, secret filtering, bounded outputs, and durable evidence for relevant reads and resulting changes.

## Drift and verification

The PPM makes disagreement visible:

- specification versus implementation;
- declared architecture versus code relationships;
- infrastructure declarations versus provider observations;
- requirements versus tests or other evidence;
- dependency edges versus active decisions.

The first Drift Checker is narrow: new or removed dependency edges, edges incompatible with an active decision, and simple declared-versus-observed conflicts. Drift is a finding, not an automatic overwrite.

## AgentRun

Polterware must reconstruct: context delivered → agent/task → change → evidence → drift findings. A minimal AgentRun is part of the wedge. After audited writes, **Unified Change History** (Agents → History) lists justified project-model changes for both human and agent actors under the same envelope. Enterprise audit dashboards remain later and are not the wedge UI.

## Product surfaces

### Desktop

The Desktop (Electron + Solid) is the primary local human product and remains the shipped shell. Future PPM, context, and drift capabilities extend this architecture rather than creating a second desktop product. Architecture Canvas may act as a visual inspector of the model; it is not the category or the wedge.

### CLI

A local Go CLI (`init`, `scan`, `context`, `drift`) is part of the wedge validation path and shares `polterware-core`.

### MCP

MCP is the primary agent interface over the same core.

### Integrations (KEEP)

Integrations are sources of context and evidence: detect CLIs, Connect with intermediate UI, probe status, and later import observations. They are not provider consoles and not an operations control plane. PTY use is limited to Connect flows.

### Polterware Website / Web App / Cloud / Self-hosted

Unchanged in intent from prior vision: Website is public messaging; Web App and Cloud are later hosted continuity; self-hosted exposes only implemented public capabilities. None of these redefine the wedge.

## Condemned / scheduled for removal

The following are **outside the product vision**. Ops / Terminal-as-product / Nodes removal **landed** on the kill branch ([`architecture/kill-ops-terminal-nodes.md`](architecture/kill-ops-terminal-nodes.md)): backend domains, IPC channels, schemas, and SQLite tables are gone; residual `terminals.*` exists only for Integrations Connect auth PTY. Do not reintroduce them.

| Area | Destiny |
| --- | --- |
| Operations (Runs, Workflows, Schedules as product) | **REMOVED** |
| Terminal as a product surface | **REMOVED** (residual PTY kept for Connect) |
| Nodes / remote execution / process control as product | **REMOVED** |
| Operations-as-thesis / control plane | **REMOVED** |
| Personal memory as positioning | **KILL** |
| PCTX / proprietary context DSL | **KILL** |
| GraphRAG as a marketed feature | **KILL** |
| “Anti-Vibe” public branding | **KILL** |

## Target technical direction

The local foundation remains Electron, Solid, Go `polterware-core`, and SQLite. The renderer remains unprivileged and accesses local authority only through the typed preload bridge and Electron Main.

The PPM begins with embedded storage and ordinary relational representations. A separate graph database, vector service, or Python sidecar is not a prerequisite.

Provider credentials remain in provider CLI or operating-system credential stores.

## Delivery sequence

1. **Local foundation** — keep the SQLite-backed Desktop and Go core; Ops/Terminal/Nodes kill branch landed for backend/IPC/schemas.
2. **PPM Facts / Evidence / provenance** — expand the thin registry into facts, evidence, and `declared` / `observed` / `inferred` / `verified`.
3. **Deterministic import** — Git, filesystem, manifests, Docker/Compose, `AGENTS.md`, `CLAUDE.md`, ADRs, Markdown.
4. **Context Compiler and stable MCP reads** — `context.get` and related tools with reasons and evidence.
5. **Drift Checker (narrow) and AgentRun** — dependency and decision conflicts; context → change → evidence.
6. **Audited MCP writes** — narrow semantic mutations with required `summary`, `rationale`, and `agent_run_id`.
7. **CLI** — `polterware init | scan | context | drift` sharing the core.
8. **Unified Change History** — Agents → History as one feed of justified human + agent model changes (filters, envelope detail). See [`architecture/unified-change-history.md`](architecture/unified-change-history.md).
9. **Structured SDD and Change Gates** — later, after the wedge validates.
10. **Web App and Cloud** — after the local model is stable.

Each phase requires its own specification and acceptance criteria. Later phases must not be represented as shipped merely because their concepts are documented here.

## Non-goals

Polterware does not aim to replace source control, cloud providers, Terraform, Kubernetes, CI/CD systems, observability platforms, database administration tools, general-purpose documentation editors, coding agent IDEs, or enterprise ops platforms (Port, Harness, and similar).

It connects relevant project facts, preserves intent and provenance, compiles context, and makes disagreement inspectable.

The following are explicitly not decided by this vision:

- a final Architecture as Code file format;
- a required embedding or generative model;
- a required vector or graph database;
- final MCP wire formats beyond the named tool intents;
- Cloud pricing, packaging, or free-tier policy;
- automatic promotion of inferred knowledge into project truth;
- residual UI package cleanup for Ops/Terminal/Nodes after the kill branch backend/IPC/schemas landing.

## Canonical short definition

> Polterware is local-first project context and change intelligence for AI coding: a verifiable model of intent and observed software, a budgeted context compiler for any MCP-compatible agent, and drift checks when changes diverge from decisions and specs.
