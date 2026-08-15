# Feature Specification: Operational Component Placement

**Feature Branch**: `011-operational-component-place`

**Created**: 2026-08-11

**Status**: Draft

**Input**: User description: "Fazer o fluxo de adicionar componente no Architecture canvas no padrão do produto e da documentação — profissional, operacional (Model → Observe → Link), não um sticker board. Environment do project, estado planned/unlinked honesto, caminho opcional de link a resource, sem fingir integração/target/health live no create."

**Related product docs**: `docs/spec.md` (§4.1 canvas operacional, §7 environments, §20 architecture, §21 resources, §110 architecture rules), `product.md` (loop Model → Observe → Link…), `specs/006-semantic-architecture-canvas` (typed create, FR-022 states), DESIGN (catalog flush + product dialog frame).

**Extends**: `005-architecture-canvas`, `006-semantic-architecture-canvas` — does **not** replace the canvas engine or invent a second architecture product surface.

---

## Summary

Operators must be able to place architecture components that feel like **real system model elements**, not decorative stickers.

Placing a component is the **Model** step of the product loop: typed identity in a project environment, honest **Planned / Unlinked** lifecycle when no resource is attached. Reality (inventory, integrations, targets, observed health) attaches through **Observe → Link**, not by faking live wiring at create time.

This feature defines the end-to-end **Add to architecture** experience at product quality: discover type → configure instance identity against **project environments** → place on the map → optional link to an existing resource (or honest defer) → detail reflects state without inventing evidence.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Place a planned component with real identity (Priority: P1)

As an operator modeling a system, I pick a catalog type, name the **instance**, assign a **project environment**, and place it on the System Map so the map records a real architectural intent (e.g. `payments-db` in Staging as PostgreSQL), not an anonymous card.

**Why this priority**: Without identity + project environment + honest planned state, the canvas remains a whiteboard and breaks the product loop’s Model step.

**Independent Test**: From Architecture Canvas, open Add to architecture, choose PostgreSQL, set name `payments-db`, choose Staging from the project’s environments, place; the card shows the name and planned/unlinked meaning; detail shows environment Staging and no fake health or linked resource.

**Acceptance Scenarios**:

1. **Given** the Architecture canvas is open for a project with at least one environment, **When** I open Add to architecture (`+`, `N`, empty double-click, context “Add component here”, empty-state CTA, or command palette), **Then** a searchable type catalog opens and **no** component is created yet.
2. **Given** the catalog is open, **When** I select a placeable component type, **Then** I enter an identity step (configure) that shows the chosen type context and asks for instance fields before the component is committed.
3. **Given** I am on the identity step, **When** I see Environment options, **Then** they are the **project’s environments** (stable names/kinds), not a free-floating unrelated list disconnected from the project.
4. **Given** name and environment are valid, **When** I confirm place, **Then** a component is committed on the map with that name, type, environment association, origin manual, and lifecycle **Planned** while unlinked.
5. **Given** I placed a component without linking a resource, **When** I view the card and detail, **Then** the product clearly communicates **Planned** and **not linked to a resource** — no healthy/live status is implied.
6. **Given** I only fill name and environment (fast path), **When** I confirm, **Then** place still succeeds without requiring purpose, role, owner, or resource link.
7. **Given** I cancel catalog or identity (Escape / Cancel) before commit, **Then** nothing is persisted for that create attempt.

---

### User Story 2 - Understand what I am adding before place (Priority: P1)

As an operator, during configure I understand the **catalog type** (class of thing) versus the **instance** I am creating, and that place creates a planned model element—not a provisioned cloud resource or registered machine.

**Why this priority**: Confusion between type blurb, instance purpose, resource, and target is the root of “sticker board” and fake control-plane UX.

**Independent Test**: Select API vs Database vs SaaS; configure header and copy differ by kind; status line states planned/unlinked; no control claims “deploy”, “register node”, or “connect AWS” as the primary place action.

**Acceptance Scenarios**:

1. **Given** I selected a type, **When** configure opens, **Then** I see type identity (icon, label, semantic kind, short type description) separate from instance fields (name, environment, optional purpose).
2. **Given** configure is open, **When** I read the primary action, **Then** it is place/commit to the map (e.g. “Place on canvas”), not provision, sync, or register target.
3. **Given** optional purpose is offered, **When** I fill it, **Then** it describes the instance’s role in **our** system and is stored as instance description — it MUST NOT silently copy the catalog type marketing blurb as the instance purpose if I left purpose empty.
4. **Given** optional more-context fields (role, owner) are available, **When** collapsed by default, **Then** they do not block the fast path; when set, they appear later in detail overview.

---

### User Story 3 - Optionally link an existing resource (Priority: P1)

As an operator, when inventory already knows a real resource, I can **link** it while placing or immediately after place, so the component becomes **Linked** with provenance—without inventing a resource or health.

**Why this priority**: Link is the product’s bridge from Model to reality; without an honest link path, architecture never becomes operational.

**Independent Test**: With mock inventory containing a Staging Postgres resource, place `payments-db` and link that resource; component shows Linked; detail references the resource; unlinked place still possible by skipping.

**Acceptance Scenarios**:

1. **Given** resources exist for the project (mock or live inventory), **When** I am on identity or a post-place link step, **Then** I can open “Link existing resource” filtered by compatibility (type family / environment when known).
2. **Given** I select a compatible resource and confirm, **When** place/link commits, **Then** the component is **Linked**, stores the resource association, and does **not** claim Observed unless observation evidence exists.
3. **Given** no matching resources, **When** I open link, **Then** I see an honest empty state (e.g. none for this type/env) with guidance to discover/import later — not a fake resource list.
4. **Given** I skip link, **When** place completes, **Then** component remains Planned / Unlinked and a clear path remains later from detail (“Link resource”).
5. **Given** a linked component, **When** I remove the visual node later (out of scope delete semantics already in architecture rules), **Then** this feature does not redefine resource deletion: link removal must not imply destroying the external resource.

---

### User Story 4 - Enter from Connected / import stubs honestly (Priority: P2)

As an operator, from the catalog’s Connected (or equivalent) actions I can start “link from resources” or see import-from-integration placeholders that are honest when not available.

**Why this priority**: Product catalog already groups Connected intents; stubs must not claim live provider import.

**Independent Test**: Open catalog Connected; choose “Link from Resources” → resource-first flow that ends in a placed/linked component or cancel; choose unavailable import → clear not-available message, no half-created node.

**Acceptance Scenarios**:

1. **Given** catalog Connected (or equivalent) shows link/import intents, **When** an action is not implemented, **Then** the product states it is not available yet and creates no component.
2. **Given** “Link from Resources” is available (mock), **When** I pick a resource first, **Then** I get a pre-filled component type/name/env suggestion when inferable, confirm place, and land on a Linked (or Planned if link deferred) component.

---

### User Story 5 - See lifecycle and environment on map and detail (Priority: P1)

As an operator, after place I can tell planned vs linked at a glance on the card and in the global detail panel, and environment filter behaves against the component’s project environment association.

**Why this priority**: Operational map without visible lifecycle is still a diagram.

**Independent Test**: Place unlinked and linked components; cards and detail differ; filter Staging de-emphasizes or filters non-Staging components per existing environment-filter product rules.

**Acceptance Scenarios**:

1. **Given** a Planned unlinked component, **When** selected, **Then** detail Overview shows environment, kind, optional role/owner/purpose (notes), and unlinked/planned meaning without inventing runtime health.
2. **Given** a Linked component, **When** selected, **Then** detail shows link to the resource identity (name/ref) and Linked state; health remains unknown/honest unless observed evidence exists (Observed is out of scope unless evidence pipeline is present).
3. **Given** environment filter is set to a project environment, **When** components are associated with environments, **Then** filter behavior remains consistent with architecture canvas rules (dim or hide non-matching — no fake multi-env compare in this feature).

---

### User Story 6 - Keep placement modes professional and cancel-safe (Priority: P2)

As an operator, place either near a known origin (double-click / context) or via placement mode when origin is unknown, without double-asking for name after configure and without orphan drafts on Escape.

**Why this priority**: Spec 006 already requires placement and cancel safety; this feature must not regress them while adding identity.

**Independent Test**: `N` without click origin → identity → placement click → commit once; Escape during placement discards; configure name is the source of truth (no forced second rename unless user opens rename).

**Acceptance Scenarios**:

1. **Given** create was triggered with a document point, **When** I finish identity, **Then** place occurs at/near that origin without requiring a second name step.
2. **Given** create was triggered without a point, **When** I finish identity, **Then** I enter placement (preview) and click to commit; Escape cancels with no node.
3. **Given** identity already set the name, **When** the node appears, **Then** I am not forced through inline rename to complete create (inline rename remains available as edit, not a mandatory second gate).

---

### Edge Cases

- Project has **zero environments**: identity step MUST block place with a clear message to create/configure an environment first (or offer a single honest default only if product already guarantees one environment on project create — see Assumptions).
- Catalog type is **not placeable** (action / unavailable connected import): no component; honest toast or empty action reason.
- **Boundary / note** types: if still not placeable in current product, same honest unavailable path; this feature does not require full boundary authoring redesign.
- Duplicate instance names: place is still allowed in v1 (map may have duplicates); search remains usable; uniqueness enforcement is out of scope unless already global policy.
- Resource list fails to load: link step shows error + retry / skip place as planned.
- Operator links a resource in a **different environment** than the component: MUST warn or prevent; default is prevent cross-env link unless resource has no env (then allow with explicit note).
- Switching type after configure started: not required; cancel and re-pick from catalog is acceptable.
- Offline / mock-only inventory: link uses project-scoped mock fixtures; never presents mock data as live cloud truth without labeling demo/fixture context when the shell is in fixture mode.

---

## Requirements *(mandatory)*

### Functional Requirements

#### Catalog (Discover)

- **FR-001**: System MUST NOT create an architecture component from Add entry points without an explicit type selection from the catalog (or an explicit resource-first link flow that implies type).
- **FR-002**: System MUST open the searchable **Add to architecture** catalog from: toolbar `+`, shortcut `N`, empty-canvas double-click, context “Add component here”, empty-state CTA, and command palette, consistent with existing architecture entry points.
- **FR-003**: Catalog MUST keep dense flush picker chrome (search → category tabs → favorites/recent → list) and MUST NOT present a multipage wizard as the primary create metaphor.
- **FR-004**: Catalog MUST continue grouping types into Applications, Data, Infrastructure, Development, External, Organization, and Connected (or product-equivalent labels), including Connected intents for link/import even when stubbed.
- **FR-005**: Non-placeable or unimplemented Connected actions MUST fail closed with an honest unavailable reason and MUST NOT leave provisional nodes in the document.

#### Identity (Configure)

- **FR-006**: After selecting a placeable type, system MUST present an identity step before permanent commit, capturing at minimum: **instance name** (required) and **project environment** (required).
- **FR-007**: Environment choices MUST be the **project’s environments** (id + display label/kind as available). System MUST NOT use a disconnected hardcoded list that ignores project environment definitions when project environments exist.
- **FR-008**: Identity step MUST show type context (icon, label, semantic kind, type-level description) and an explicit statement that place creates a **planned** model element **unlinked** until a resource is linked.
- **FR-009**: Identity step MAY offer optional **purpose** (instance description), **role**, and **owner**; these MUST NOT be required for place.
- **FR-010**: Primary confirm action MUST mean place/commit to the architecture map. It MUST NOT be labeled or behave as provision, deploy, register machine/target, or connect integration.
- **FR-011**: Cancel / Escape on identity MUST discard the in-progress create with no committed component.
- **FR-012**: Identity UI MUST use the product’s shared form and dialog language (same product dialog frame family as other focused creates; labeled fields; no ad-hoc parallel field chrome).

#### Place

- **FR-013**: On successful place without resource link, system MUST commit a component with: chosen type semantics, instance name, environment association, origin manual, lifecycle **Planned**, link state **Unlinked**.
- **FR-014**: System MUST NOT set Linked, Observed, healthy, or provider-live status solely because a type was chosen or a purpose was typed.
- **FR-015**: When create origin includes a map point, place MUST use that origin; when not, system MUST use placement mode (preview + click) or an equivalent explicit place gesture — not silent ambiguous commits without operator spatial intent when the product already supports placement mode.
- **FR-016**: Confirmed place MUST participate in existing canvas history/undo and autosave expectations for architecture documents.
- **FR-017**: After place, system SHOULD select the new component and surface detail in the global detail panel so the operator can enrich without a second entity page.

#### Resource link (optional, honest)

- **FR-018**: System MUST allow operators to **skip** resource link and still complete place as Planned/Unlinked.
- **FR-019**: System MUST offer a path to **link an existing project resource** either during identity (optional section) or immediately post-place from detail / CTA — at least one of these paths MUST exist in this feature.
- **FR-020**: Link candidate list MUST be scoped to the project and SHOULD filter by environment and type compatibility when metadata allows; empty and error states MUST be honest.
- **FR-021**: On successful link, component MUST become **Linked**, retain resource association with enough identity for detail navigation, and MUST NOT invent Observed or healthy without observation evidence.
- **FR-022**: Linking MUST NOT transfer ownership of the provider resource to Polter and MUST NOT delete external resources when the component or link is later removed (align architecture invariants).
- **FR-023**: System MUST NOT require an integration connection to place a planned component (Model-first remains valid offline / without providers).

#### Targets, integrations, provisioning (boundaries)

- **FR-024**: Place component MUST NOT register a **Target** (machine/node connector) as a side effect of adding a non-host architecture component.
- **FR-025**: Place component MUST NOT open a provisioning plan or create cloud resources as part of the default create path.
- **FR-026**: Import-from-integration actions, when not implemented, MUST remain stubs with honest unavailable messaging (Connected catalog).

#### Visibility & detail

- **FR-027**: Component cards MUST be able to reflect lifecycle/link meaning relevant after place (at least Planned vs Linked distinction when linked), without strong false health chrome.
- **FR-028**: Global detail for the component MUST show environment, kind, optional purpose/role/owner when set, and link state; purpose appears as operational description/notes when present.
- **FR-029**: Environment filter on the canvas MUST remain driven by project environment association on components (no separate fake env taxonomy for filter vs place).

#### Consistency & honesty

- **FR-030**: Component type and resource type remain distinct concepts; place creates an architecture **component**, never silently a resource row.
- **FR-031**: Canvas document remains reconcilable with Components/Resources model rules: no orphan “sticker-only” lifecycle that cannot represent Planned/Linked.
- **FR-032**: Fixture/demo inventory and environments MUST be labeled or mode-consistent with the product’s fixture honesty rules when the shell is not on live core data.
- **FR-033**: The Components-tab create entry, if it remains, MUST NOT permanently diverge into a second freeform create model; it SHOULD reuse the same catalog → identity → place/link semantics or be narrowed to open the same flow.

### Key Entities *(include if feature involves data)*

- **System (Project in UI)**: Container for architecture and environments.
- **Environment**: Project-owned named context (e.g. local, development, staging, production); place associates the component to one primary environment in v1.
- **Architecture Component**: Typed model element on the System Map (name, type/subtype, environment association, lifecycle state, optional resource link, optional purpose/role/owner).
- **Component Type (catalog definition)**: Preset class (PostgreSQL, API, …) — not an instance.
- **Resource**: Inventory entity from discovery/import/manual resource model; optional link target for a component.
- **Component–Resource Link**: Association with provenance; enables Linked state; does not imply Observed alone.
- **Target**: Execution/location identity for operations (machines/nodes); **out of default place path** for ordinary components.
- **Lifecycle state**: Planned (manual unlinked default) → Linked (resource associated) → Observed/Stale/Drifted/Unreachable only with evidence semantics defined elsewhere.
- **Identity draft**: Transient configure state (type + name + env + optional fields + optional resource pick) before commit.
- **Placement intent**: Origin point or placement-mode gesture completing spatial commit.

### Constitution Constraints *(mandatory)*

Map the feature to `.specify/memory/constitution.md` (v2.x). Analysis MUST block
implementation on critical unresolved conflicts.

- **Operational architecture goal (I)**: Strengthens **architecture understanding** and the bridge to **resource discovery/organization** via honest link; MUST NOT become a specialized-platform clone (no fake AWS console in place).
- **Runtime owner (III)**: Renderer/feature UI for authoring; persistence of architecture document via existing desktop/core layout path; no Node privilege in renderer.
- **Domain vs adapter (II, IV)**: Architecture domain owns component identity and link association; inventory/integrations supply resources through ports; UI MUST NOT invent provider truth.
- **Contracts (V)**: Architecture document component fields and optional resource link identity; no requirement to invent new cloud provider contracts for mock link.
- **Product loop impact**: Primary **Model**; secondary **Link**; does not implement Operate/Approve. System Map + Detail (Inspector successor) + eventual Resources.
- **Evidence expectations**: Planned/Unlinked without evidence; Linked only with explicit resource association; Observed/health **MUST NOT** appear without observation evidence (FR-014, FR-021).
- **Mutation safety (VII)**: Local architecture document mutation (create/link association) is operator map authoring; cloud resource mutation is **out of scope**. No privileged side effects.
- **Operation audit fields**: N/A for pure local map place in mock; if core audit exists for architecture mutations later, correlation should follow existing architecture save events — not invented here as fake operation runs.
- **Security boundary (VI)**: No secrets in architecture identity; resource link uses opaque resource ids/refs only; MCP/renderer cannot bypass policy for real integrations (stubs only).
- **Local vs cloud authority (VIII)**: Local architecture model authoritative for components; providers remain authoritative for resource-owned fields when linked later.
- **Performance (IX)**: Catalog and identity must stay interactive for normal catalog sizes; resource picker SHOULD remain usable for typical project inventory sizes (virtualize only if already standard for long lists).
- **Resilience (X)**: Cancel-safe drafts; save failure must not claim success; link load errors allow skip-as-planned.
- **Observability (XI)**: No new cross-runtime stream required for mock; reuse existing architecture autosave/status.
- **Test plan (XII)**: Unit for identity normalization and env/link rules; feature tests for catalog → identity → place → state; honest empty/unavailable paths; no e2e required to claim draft complete unless desktop harness already covers architecture create.
- **Structure created now (XIII)**: Prefer extending existing architecture feature surfaces; no scenic empty packages.
- **Single source of truth (XIV)**: One create flow semantics for canvas (and Components entry aligned); no permanent dual freeform vs catalog models.
- **Big-bang / refactor context (XV)**: Builds on post big-bang Solid architecture canvas; incremental product hardening, not engine rewrite.
- **Non-goals (I, XIII)**: No provisioning wizard; no target registration; no fake health; no multipage create wizard; no second detail inspector; no specialized cloud console.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a guided demo with project environments and catalog types available, an operator can complete typed place (name + project environment) in **under 20 seconds** on the happy path without linking a resource.
- **SC-002**: **100%** of components created through this flow that skip resource link show **Planned** and **Unlinked** (or equivalent honest copy) — **0%** show healthy/live solely due to create.
- **SC-003**: When project environments exist, **100%** of successful places associate the component to a **project environment** selected from that project’s list (not an orphaned label).
- **SC-004**: In a demo with at least one compatible mock resource, an operator can place **and** link in **under 45 seconds**, after which detail shows the resource association and Linked state.
- **SC-005**: At least **90%** of first-time demo operators (internal dogfood) correctly describe the new card as “planned model / not live resource” when asked after one unlinked place (qualitative check in review).
- **SC-006**: Cancel before commit (catalog or identity or placement) leaves **zero** new components in the document in scripted cancel scenarios.
- **SC-007**: Unavailable Connected/import actions produce **zero** half-created components and always show an understandable unavailable reason.

---

## Assumptions

- Projects in scope already have (or fixtures provide) **one or more environments**; if product guarantees a default environment on project create, empty-env is rare and still handled with a clear block/message.
- v1 place associates **one primary environment** per component; multi-environment membership remains a later enrich concern.
- Resource inventory for link MAY be **mock/fixture** in Desktop Community until discovery/integrations are wired; fixture mode stays honest.
- Observed / stale / drift / unreachable visualization beyond Linked vs Planned is largely owned by existing/future evidence features; this feature only forbids faking them at create.
- Target registration, SSH nodes, and provisioning remain under Resources / Operations surfaces, not this create path.
- Catalog type registry and Add to architecture flush pattern from prior architecture features remain the discover UI.
- Global `DetailPanel` remains the detail surface (no permanent lateral inspector; no second entity create page).
- Canvas history, autosave, and environment filter behaviors from `005`/`006` remain unless this feature explicitly tightens them.
- Purpose/role/owner are optional metadata for human readability; they are not substitutes for resource links or evidence.
- Language of UI strings remains English; this specification is written in English for product/engineering consistency.

---

## Out of Scope

- Full multi-environment comparison mode on the canvas.
- Creating, discovering, or provisioning resources as a side effect of Place.
- Registering Targets / install tokens / heartbeats (Resources “Add node” wizard is a different product flow).
- Live provider import (AWS/Vercel/Supabase/…) beyond honest stubs.
- Credentials, secrets, or integration OAuth inside the place flow.
- Relationship/connection authoring during create (connections remain post-place).
- Custom component type authoring UI.
- Boundary/note full redesign (only honest placeable gating).
- Replacing the canvas engine or DESIGN system primitives.
- Claiming Observed/healthy from static form fields.

---

## Dependencies

- Existing Architecture canvas document model and place/select/detail host.
- Project environment model (or fixtures that stand in for project environments).
- Resources inventory surface or fixture list for optional link.
- Design language for catalog dialogs, forms, and global detail panel.
- Product lifecycle meanings from semantic architecture canvas (Planned / Linked / Observed…).

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Operators expect Place to provision real infra | Copy + primary action wording + success criteria SC-005 |
| Environment list still feels fake | Bind to project environments only (FR-007) |
| Link path buried → map stays stickers | FR-019 CTA on detail + optional configure section |
| Over-building configure into second inspector | Optional fields only; detail remains enrich surface |
| Dual create paths (Components tab vs canvas) | FR-033 align or funnel |
