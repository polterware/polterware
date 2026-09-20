# Polterware Product Vision

Status: Active product direction

Version: 2.0

Last revised: 2026-09-19

## Authority and scope

This document defines the intended direction and the product principles that future specifications must preserve. It does not claim that every capability described here exists today.

The documentation hierarchy is:

1. `docs/spec.md` is authoritative for currently implemented product behavior.
2. Accepted ADRs are authoritative for architecture decisions within their stated scope.
3. This document is authoritative for product direction and the invariants that new specifications must respect.
4. `docs/future/specs/` is a historical archive and is not normative.

When current behavior and product direction differ, documentation and UI copy must label that difference explicitly. Planned behavior must never be presented as shipped behavior.

## Product thesis

Polterware is the local-first source of truth for software projects, designed for people and AI agents.

Software knowledge is usually divided among specifications, diagrams, source code, infrastructure definitions, provider state, runbooks, operations, and chat history. The result is fast implementation without durable context: people lose the reason behind a change, and agents repeatedly reconstruct an incomplete version of the project.

Polterware brings those sources into one structured model. The canvas, documentation, search, spec-driven workflows, command-line tools, and agent interfaces are different ways to inspect or change that model; none of them is a separate source of truth.

The product is not a generic documentation editor with AI added to it. Its central value is the relationship between intent, architecture, implementation, infrastructure, runtime evidence, and the actors who changed them.

## Product principles

### One project model

Every product surface reads from or contributes to the same Polterware Project Model. The product must not create parallel architecture, documentation, agent, or operations models that silently diverge.

### Structured before inferred

Polterware should import deterministic structure directly from source systems whenever possible. Package manifests, OpenAPI documents, Terraform, Docker, Git, and provider APIs already express facts that do not need an LLM to rediscover them.

AI inference may enrich the model, but it must remain distinguishable from declared and observed facts.

### Provenance before confidence

A statement is not useful as project truth unless Polterware can explain where it came from, when it was observed, what revision or environment it describes, and whether a person approved it.

### Local value without Cloud

The Desktop must remain useful without an account or hosted service. Cloud adds managed continuity, collaboration, synchronization, administration, and operated infrastructure; it does not unlock local ownership of project data.

### People and agents use the same model

Agents must not receive a hidden or privileged project representation. Human and agent interfaces use the same domain contracts, provenance rules, policies, and audit semantics, with each mutation attributed to the correct actor.

### Deliberate friction for durable decisions

AI can accelerate production, but decisions that become project truth require explicit intent. Polterware may introduce small, configurable understanding and approval gates before durable specifications or architecture changes are accepted.

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
        |                     |                     |
        +---------- documentation and operations --+
```

### Core entities

The model is expected to represent entities such as:

- projects, environments, repositories, branches, and revisions;
- components, services, APIs, databases, queues, and infrastructure resources;
- specifications, requirements, plans, tasks, decisions, and ADRs;
- documents, runbooks, operations, deployments, incidents, and evidence;
- commits, code symbols, tests, build results, and provider observations;
- people, agents, agent runs, approvals, and overrides.

This list describes the conceptual model. Each entity requires an accepted feature specification and contract before implementation.

### Relationships

Relationships give the model its operational meaning. Examples include:

```text
Component DEPENDS_ON Component
Service USES Database
Component RUNS_ON InfrastructureResource
Spec DEFINES Requirement
Requirement SATISFIED_BY CodeSymbol
Task IMPLEMENTS Requirement
Test VERIFIES Requirement
ADR EXPLAINS Decision
Runbook OPERATES Service
Deployment DEPLOYS Commit
AgentRun PRODUCED Evidence
Human APPROVED Decision
```

Relationships must be typed, queryable, and attributable to their source. The graph is a product model, not an artifact generated after semantic search.

### Provenance

Entities, relationships, and claims should carry enough provenance to answer:

- Which source declared or observed this fact?
- Which branch, revision, and environment does it describe?
- When was it observed or last confirmed?
- Was it imported, entered by a person, produced by an agent, or inferred?
- What evidence supports it?
- Has a person approved or overridden it?

The exact storage schema is deferred to dedicated specifications. Provenance is not optional even when the initial persistence is simple.

## Classes of truth

Polterware distinguishes four classes of project knowledge.

### Declared Truth

Intentional statements owned by the project: specifications, requirements, architecture decisions, ownership, policies, and human-approved architecture.

### Observed Truth

Facts imported from authoritative sources: code, Git, manifests, infrastructure definitions, provider inventories, and environment-specific state.

### Operational Truth

Evidence produced while the system is built or operated: deployments, runs, logs, incidents, verification results, and runbook execution.

### Inferred Knowledge

Summaries, semantic relationships, classifications, and other conclusions produced by AI or heuristic analysis.

Inferred Knowledge must never silently replace Declared Truth or Observed Truth. Promotion of an inference into project truth requires a recorded human decision, including the actor, reason, affected claim, and supporting evidence.

## Import, export, and Architecture as Code

Polterware should build the PPM through deterministic importers before relying on semantic inference.

Initial source families include:

- Git and filesystem structure;
- package and build manifests;
- Docker and container definitions;
- Terraform and other supported infrastructure declarations;
- OpenAPI and supported interface definitions;
- authenticated provider CLI or API observations;
- Polterware-authored specifications, decisions, and canvas changes.

Architecture as Code provides a versionable, human-readable representation that can create or update relevant parts of the PPM. The canvas is another editor for the same data, not a disconnected diagram.

The exact file name, serialization format, merge semantics, and schema versioning are intentionally not fixed in this vision. They require a dedicated contract specification with round-trip and conflict behavior.

Importers must report their source, scope, revision, environment, and limitations. A bounded inventory import must not be described as a complete cloud graph.

## Context Compiler

The Context Compiler produces the smallest sufficient project context for a person or agent under an explicit token budget.

Its target pipeline is:

```text
request
  -> intent and entity resolution
  -> exact and full-text retrieval
  -> graph expansion
  -> optional semantic retrieval
  -> provenance-aware ranking
  -> deduplication and compression
  -> token-budgeted context package
```

Exact identifiers and declared relationships take precedence over semantic similarity when they directly answer the request. Semantic retrieval is an enrichment path, not the authority for project structure.

The local product may perform indexing, graph traversal, retrieval, ranking, and context assembly without running a local generative model. Agents can consume the compiled context through supported interfaces while project data remains local unless the user chooses otherwise.

No embedding model, vector extension, reranker, or GraphRAG implementation is selected by this document. Those choices must be justified by measured project needs and replaceable behind domain contracts.

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

Markdown import and export may remain available for interoperability, but the PPM retains relationships, versions, provenance, approvals, and evidence that flat files cannot express reliably.

Changes to a specification must preserve history. A newer requirement may supersede an earlier one, but it must not erase which implementation, test, deployment, or decision was associated with the earlier version.

### Understanding Gates

An Understanding Gate introduces optional human review before an artifact becomes durable project truth. Gates may address:

- clarification of ambiguous intent;
- understanding of specified behavior;
- architectural trade-offs and conflicts;
- failure scenarios and operational consequences;
- security and data-integrity implications;
- approval, override, and implementation evidence.

The gate is not a quiz whose only result is right or wrong. When an answer conflicts with a requirement or decision, the person may revise the answer, revise the source artifact, request an explanation, or record an explicit override.

### Anti-Vibe Mode

Anti-Vibe Mode is the product experience built from configurable Understanding Gates. Its purpose is to prevent unexamined approval, not to prevent AI-assisted development.

The mode may be applied selectively to constitution, spec, plan, task, architecture, or verification stages. Exact levels, question counts, blocking rules, and UI controls require a separate specification.

Every accepted override becomes part of the PPM with its actor, reason, timestamp, affected artifact, and prior decision.

## Agent access

Desktop, CLI, and MCP are interfaces over the same domain behavior. MCP is the intended programmatic interface for external agents, but this vision does not define final tool names or wire formats.

Interactive hosts may support native multi-round input. Polterware must also provide a universal stateful fallback so an agent can begin a session, present questions to the person, submit answers, inspect status, and finalize without relying on host-specific interactive capabilities.

Agent access must preserve:

- explicit project and environment scope;
- minimum necessary context;
- actor attribution and delegated authority;
- the same policy and approval path used for equivalent human actions;
- secret filtering and bounded outputs;
- durable evidence for relevant reads, proposals, mutations, and overrides.

## Drift and verification

The PPM makes disagreement visible across project truth:

- specification versus implementation;
- declared architecture versus code relationships;
- infrastructure declarations versus provider observations;
- canvas state versus imported structure;
- requirements versus tests or other evidence;
- documented operations versus runtime outcomes.

Drift is a finding, not an automatic overwrite. Polterware must show the conflicting sources, provenance, environment, revision, and confidence before a person accepts a reconciliation.

## Product surfaces

### Desktop

The Desktop is the primary local product. Its current implementation is defined by `docs/spec.md`. Future PPM capabilities should extend the existing local architecture rather than create a second desktop product.

### Polterware Website

The Website is the public, unauthenticated product surface. It explains the product without implying that planned capabilities are already available.

### Polterware Web App

The Web App is a future authenticated surface for hosted projects, collaboration, continuity, and administration. It will live in the `polterware-web` repository alongside the Website and will depend on the private Cloud control plane for hosted authority.

### Cloud

Polterware Cloud is the managed service. It owns identity, tenants, permissions, synchronization authority, billing, metering, managed infrastructure, and private operational material.

### Self-hosted

Self-hosted deployments expose only capabilities implemented by their public backend. Self-hosting does not imply parity with the managed Cloud service.

## Target technical direction

The current local foundation remains Electron, Solid, a Go core, and SQLite. The renderer remains unprivileged and accesses local authority only through the typed preload bridge and Electron Main.

The PPM should begin with embedded storage and ordinary relational representations of entities and relationships. A separate graph database, vector service, or Python sidecar is not a prerequisite. More specialized storage is justified only by measured scale or capability requirements.

Provider credentials remain in provider CLI or operating-system credential stores. Project knowledge stores references and safe metadata, not reusable secrets.

## Delivery sequence

1. **Local foundation** — maintain the current SQLite-backed Desktop, execution authority, and honest provider boundaries.
2. **Project Model** — define canonical entities, relationships, provenance, truth classes, and migrations.
3. **Discovery** — import Git, filesystem, manifests, containers, infrastructure, and supported provider observations.
4. **Knowledge and search** — connect documents, ADRs, and runbooks to the PPM with exact and full-text retrieval.
5. **Context Compiler and MCP** — expose bounded, provenance-aware context to external agents.
6. **Structured SDD** — model specifications, requirements, plans, tasks, decisions, and evidence.
7. **Understanding Gates** — add Anti-Vibe workflows, conflict handling, approvals, and overrides.
8. **Drift and verification** — compare intent, architecture, code, infrastructure, operations, and evidence.
9. **Web App and Cloud** — add hosted continuity, collaboration, administration, and synchronization after the local model is stable.

Each phase requires its own specification and acceptance criteria. Later phases must not be represented as shipped merely because their concepts are documented here.

## Non-goals

Polterware does not aim to replace source control, cloud providers, Terraform, Kubernetes, CI/CD systems, observability platforms, database administration tools, or general-purpose documentation editors.

It connects their relevant project facts, preserves intent and provenance, compiles context, and makes disagreement inspectable.

The following are explicitly not decided by this vision:

- a final Architecture as Code file format;
- a required embedding or generative model;
- a required vector or graph database;
- final MCP tool names and transport details;
- Cloud pricing, packaging, or free-tier policy;
- automatic promotion of inferred knowledge into project truth.

## Canonical short definition

> Polterware is a local-first source of truth that connects software intent, architecture, implementation, infrastructure, operations, and evidence in one structured project model for people and AI agents.
