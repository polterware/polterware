# Feature Specification: Operator Design System & Primary Surfaces

**Feature Branch**: `003-operator-design-system`

**Created**: 2026-08-02

**Status**: Draft

**Input**: User description: "Define the Polter design system formally and implement the main operator interfaces (shell, Resources, Architecture, detail pages, focused dialogs) with a quiet dark desktop visual language inspired by prototype composition—not a disposable demo."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Consistent operator shell and visual language (Priority: P1)

As an operator opening Polter Desktop, I land in a calm, dark, technically precise shell: integrated navigation on the left, a single large rounded content surface for the active page, compact typography, and restrained brand accent. The product feels like a mature control plane, not a generic admin dashboard, crypto panel, gamer UI, or AI-generated card collage.

**Why this priority**: Without a coherent shell and visual system, every subsequent surface fragments. This is the foundation operators experience on every session and the contract future screens must follow.

**Independent Test**: Launch Desktop (or the product shell surface), navigate between primary destinations, and verify shell anatomy, surface hierarchy, active navigation treatment, brand-color restraint, and absence of permanent intrusive inspector panels. Compare composition density against `/prototype` as a guide, not pixel-perfect source.

**Acceptance Scenarios**:

1. **Given** the application window is open, **When** the operator views any primary page, **Then** they see an integrated sidebar that blends with the app background and a single large rounded main content surface for page content.
2. **Given** a navigation item is active, **When** the operator inspects selection styling, **Then** the active item uses a neutral raised surface—not brand green fill, glow, or high-chroma highlight.
3. **Given** the brand green accent exists, **When** the operator scans the interface, **Then** green appears only in small strategic signals (for example status dots or rare primary emphasis), never as large surface fills, gradients, or default selection color.
4. **Given** the design system document is the visual source of truth, **When** an engineer or designer reviews tokens and components, **Then** colors, typography, spacing, radius, borders, motion, and shell dimensions are documented and used consistently—no ad-hoc repeated hex values in product surfaces covered by this feature.
5. **Given** visual anti-patterns are forbidden, **When** screens are reviewed, **Then** there is no card-in-card nesting cascade, glassmorphism/heavy blur, strong noise, blue-tinted backgrounds, decorative chart dashboards, or Halloween/cyberpunk aesthetics.

---

### User Story 2 - Browse and open Resources as pages (Priority: P1)

As an operator, I browse Resources in a dense index (filters, search, table or list), open a resource into its own detail page (not a permanent right inspector), and return via clear page navigation. Create and edit use focused dialogs, not full-page noise for short tasks.

**Why this priority**: Resources are a primary operational inventory. Page-based depth and progressive disclosure are core to the product navigation model described for Polter.

**Independent Test**: From Resources index, search/filter, open a resource detail page, use header actions (open provider, view logs where available), open create/edit dialog, and observe loading/empty/error states without a fixed right-hand inspector.

**Acceptance Scenarios**:

1. **Given** Resources is selected in navigation, **When** the index loads with data, **Then** the operator sees a page header, optional environment context, search/filter controls, and a dense resource list or table with essential columns (name, type, environment, provider, status, recency as applicable).
2. **Given** a resource row or item is activated, **When** navigation completes, **Then** Resource detail opens as a full page within the main surface (breadcrumb or clear back path), not as a permanent side inspector.
3. **Given** Resource detail is open, **When** the operator reviews content, **Then** they see identity, status chips, overview facts, linked architecture, activity or notes, and quick actions that deep-link or hand off to specialized tools where appropriate.
4. **Given** the operator starts create or edit for a resource, **When** the flow opens, **Then** it appears as a focused dialog (or equivalent focused overlay), not a permanent split-pane editor.
5. **Given** no resources match filters or the catalog is empty, **When** the index renders, **Then** an empty state explains the situation and offers a clear next action when available.
6. **Given** data fails to load, **When** the index or detail renders, **Then** an error state is shown with recovery guidance without leaving the shell broken.

---

### User Story 3 - Explore Architecture canvas and node detail pages (Priority: P1)

As an operator, I explore the System architecture on a canvas (layers/groups, nodes, relationships), select nodes with quiet neutral emphasis, and open a node’s detail as its own page. The canvas remains responsive while I pan, zoom, and select.

**Why this priority**: Architecture is the product’s structural entry point. The canvas must feel operational and calm, and detail depth must follow the same page model as Resources.

**Independent Test**: Open Architecture, pan/zoom, select a node (neutral selection), open node detail page, use relationship/linked lists to navigate related nodes, confirm no permanent right inspector on the canvas page.

**Acceptance Scenarios**:

1. **Given** Architecture is selected, **When** the canvas loads, **Then** the operator sees environment context, search/filter or view controls, a layered or grouped architecture map of services and dependencies, and canvas controls (zoom, fit, lock as applicable).
2. **Given** a node is selected on the canvas, **When** selection is shown, **Then** emphasis is neutral (subtle border/surface change)—not brand-green glow or thick decorative outlines.
3. **Given** the operator opens a node for detail, **When** navigation completes, **Then** Architecture node detail is a full page (overview, connected architecture, relationship mini-view, activity, notes, quick actions, linked docs as available).
4. **Given** the canvas has many nodes, **When** the operator pans and zooms, **Then** the rest of the application shell remains interactive and the view does not feel frozen for ordinary map sizes used in demos/prototypes.
5. **Given** Architecture has no model yet, **When** the page loads, **Then** an empty state explains how to begin without decorative filler charts.

---

### User Story 4 - Navigate with page depth and focused overlays (Priority: P2)

As an operator, I understand when the product takes me to a new page versus a dialog, command palette, dropdown, or toast. Important context lives on pages; short tasks use dialogs; global jump uses command palette; transient feedback uses toasts.

**Why this priority**: Navigation model prevents inspector sprawl and teaches consistent interaction grammar across features.

**Independent Test**: From shell, use primary nav, open detail pages, open a create dialog, invoke command palette if present, trigger a toast, and confirm no permanent dual-inspector layouts on Resources/Architecture.

**Acceptance Scenarios**:

1. **Given** the operator needs deep context (resource or architecture node), **When** they open it, **Then** the product uses page navigation within the main surface.
2. **Given** the operator performs a short focused task (create/edit, confirm), **When** the task starts, **Then** a dialog is used and dismisses cleanly without redesigning the page layout.
3. **Given** the operator needs a global action or jump, **When** they invoke the command palette (if shipped in this feature), **Then** it overlays without permanently consuming layout.
4. **Given** an asynchronous action completes or fails, **When** feedback is needed, **Then** a toast or inline status appears without blocking the entire shell unless confirmation is required.
5. **Given** Resources or Architecture index/canvas is open, **When** the operator inspects layout, **Then** there is no permanent right-hand detail inspector stealing space from the primary surface.

---

### User Story 5 - Accessible, dense desktop operation (Priority: P2)

As an operator using keyboard and assistive tech on a desktop window, I can move focus visibly, activate controls, read meaningful labels, and rely on non-color-only status. Motion respects reduced-motion preferences. Layout remains usable from ~1280×720 upward with graceful narrowing.

**Why this priority**: Professional control-plane software must be operable without mouse-only paths and without sacrificing density for decoration.

**Independent Test**: Keyboard-tab through shell and a primary page; verify focus-visible rings; check status not color-only; set reduced motion and confirm no large motion dependence; resize across target desktop widths.

**Acceptance Scenarios**:

1. **Given** keyboard-only use, **When** the operator tabs and activates controls on shell + Resources + Architecture, **Then** focus order is logical and focus-visible indicators are present.
2. **Given** a status such as Healthy/Degraded, **When** presented, **Then** meaning is available via text and/or icon—not color alone.
3. **Given** reduced-motion preference is enabled, **When** UI transitions run, **Then** motion is minimized or removed while remaining understandable.
4. **Given** window sizes approximately 1280×720, 1440×900, 1512×982, and 1728×1117, **When** primary screens are viewed, **Then** shell proportions remain usable (sidebar readable, main surface dominant, controls still targetable).
5. **Given** a very narrow window, **When** space collapses, **Then** the product degrades gracefully (for example compact sidebar or overflow) rather than overlapping or clipping critical controls unusably.

---

### User Story 6 - Honest loading, empty, error, and offline states (Priority: P3)

As an operator under imperfect conditions, I always understand whether content is loading, empty, failed, or offline—without fake “live” data claims or scenic placeholders.

**Why this priority**: Trust is part of the brand; silent wrong states break operational confidence.

**Independent Test**: Force or simulate loading, empty catalog, failed fetch, and offline/core-disconnected indicators on Resources and Architecture.

**Acceptance Scenarios**:

1. **Given** data is loading, **When** a page is shown, **Then** a quiet loading treatment appears without layout thrash that looks like a finished empty product.
2. **Given** the local core or connectivity is offline/degraded, **When** the shell is visible, **Then** status in the shell communicates connectivity honestly.
3. **Given** an error occurs, **When** the page renders, **Then** the operator sees a clear message and, when possible, a retry or alternative path.

---

### Edge Cases

- What happens when Resources list is very large? Index remains usable (scroll performance acceptable; virtualization allowed for long lists/logs) without freezing the shell.
- What happens when Architecture graph is large? Pan/zoom and selection remain usable; heavy layout work must not freeze the whole application chrome.
- What if a resource has no provider deep-link? Actions that require an external provider are hidden or disabled with explanation—not dead buttons that claim live integrations.
- What if create/edit validation fails? Dialog stays open with field-level or form-level errors; no silent discard.
- What if the user opens detail for a deleted or missing entity? Error/not-found state with path back to index.
- What if prototype images disagree with product rules? Product rules and the design-system document win; prototypes guide composition only.
- What about light theme from prior design docs? This feature establishes the dark quiet operator language as the product default for Desktop surfaces in scope; any dual-theme support is out of scope unless already required elsewhere.
- How are temporary/provisional UI shells handled? Provisional layouts that violate the one-primary-surface / no-permanent-inspector model are replaced for the screens in scope.

## Requirements *(mandatory)*

### Functional Requirements

#### Design system source of truth

- **FR-001**: Product MUST maintain a formal design-system document at the repository root design source (`design.md` / established design doc path used by the project) that is the visual source of truth for identity, principles, anatomy, tokens, typography, components, navigation model, screen blueprints, responsiveness, accessibility, performance expectations, implementation map, and acceptance checklist.
- **FR-002**: The design system MUST express product identity as a technical control plane: quiet control, architectural clarity, operational precision, trust, visual silence, and sophistication—explicitly rejecting generic admin, crypto, gamer, cyberpunk, Halloween, literal Linear clone, and AI-card collage aesthetics.
- **FR-003**: Design tokens MUST define surfaces (app/shell/sidebar/main/raised/hover/active/selected), text levels, borders, brand accent, semantic status colors, radius scale, spacing scale, control sizes, motion durations/easing, z-index layers, and shell dimensions.
- **FR-004**: Brand accent green MUST be extremely restricted (small signals only); active navigation and selection MUST default to neutral surfaces.
- **FR-005**: Surfaces MUST avoid pure black as the only depth strategy; depth MUST come from small luminance steps among dark neutrals.
- **FR-006**: Reusable UI primitives used by in-scope screens MUST share the same tokens and interaction rules (button, input, dialog, menu, tooltip, tabs, checkbox, popover, separator, toast, context menu, and shell pieces as needed).
- **FR-007**: Component specifications in the design document MUST cover purpose, structure, sizes, states, variants, accessibility, correct and incorrect usage for each reusable control introduced or restyled by this feature.

#### Application anatomy and navigation

- **FR-008**: Desktop product chrome MUST include: window-aware shell, integrated sidebar, main content surface, page header, optional toolbar, page content region, dialog layer, optional command palette, toast region, and standard loading/empty/error presentations.
- **FR-009**: Sidebar MUST appear integrated with the application background (not a floating independent panel with heavy separation from the window).
- **FR-010**: Page content MUST live primarily inside one large rounded main surface.
- **FR-011**: Important detail contexts (Resource detail, Architecture node detail) MUST be full pages, not permanent side inspectors.
- **FR-012**: Create and edit flows for in-scope entities MUST use focused dialogs (or equivalent focused overlays), not permanent split editors.
- **FR-013**: Navigation model MUST define when to use page navigation, dialog, drawer (if any), popover, dropdown, and command palette; Resources and Architecture MUST NOT ship with permanent intrusive right inspectors.
- **FR-014**: Primary navigation MUST cover at least the destinations needed to reach Resources and Architecture (and may include existing product destinations already in the shell) without inventing specialized-platform clones.

#### Resources surfaces

- **FR-015**: Product MUST provide a Resources index with search, filtering (or equivalent scoping), and a dense list/table of resources with essential operational attributes.
- **FR-016**: Product MUST provide a Resource detail page with identity, status, overview metadata, linked architecture, activity or notes, and quick actions that prefer deep-link/hand-off over reimplementing external consoles.
- **FR-017**: Resources index and detail MUST present loading, empty, and error states.

#### Architecture surfaces

- **FR-018**: Product MUST provide an Architecture canvas page showing system structure (nodes and relationships), environment context, and canvas interaction (select, pan, zoom as applicable).
- **FR-019**: Product MUST provide an Architecture node detail page with overview, connections, relationship context, activity/notes, and quick actions consistent with Resource detail patterns.
- **FR-020**: Canvas node selection MUST use neutral emphasis consistent with the design system.
- **FR-021**: Architecture canvas MUST keep the application shell responsive during ordinary pan/zoom/select interactions for prototype-scale graphs.

#### Quality, accessibility, performance (operator-facing)

- **FR-022**: In-scope surfaces MUST support keyboard operation with visible focus indication.
- **FR-023**: Status and critical state MUST NOT rely on color alone.
- **FR-024**: Motion MUST respect reduced-motion preferences; heavy decorative animation (scale-on-hover gimmicks, continuous glow) MUST NOT be used.
- **FR-025**: Long lists and logs in scope MUST remain usable at high row counts (virtualization allowed where needed).
- **FR-026**: Design and implementation artifacts for this feature MUST remain consistent with a single visual system—no parallel permanent design languages for the same Desktop operator surfaces.
- **FR-027**: Prototype images under `/prototype` MUST be treated as composition references only; product MUST NOT copy third-party branding, entity names from reference apps, or AI-mockup defects.
- **FR-028**: In-scope UI MUST remain honest about live vs placeholder capabilities—no claim that unimplemented runtime integrations are live.
- **FR-029**: Screen blueprints for Resources index, Resource detail, Architecture canvas, Architecture node detail, create/edit dialog, loading, empty, error, and offline MUST be documented in the design system document.
- **FR-030**: Responsiveness expectations for desktop window sizes (~1280×720 through ~1728×1117 and narrow windows) MUST be documented and met for primary screens.

### Key Entities

- **Design System Document**: Authoritative description of visual identity, principles, tokens, components, navigation, blueprints, a11y, and acceptance for Polter operator UI.
- **App Shell**: Desktop window chrome + integrated sidebar + main surface framing all pages.
- **Main Surface**: Large rounded content host for page header, toolbar, and page body.
- **Page**: Navigable destination for deep context (indexes and details).
- **Dialog**: Focused overlay for short create/edit/confirm tasks.
- **Command Palette**: Optional global jump/action overlay (if included in delivery).
- **Toast**: Transient feedback region.
- **Resource**: Operational inventory entity (service, database, queue, app, node, etc.) with provider, environment, status, and links.
- **Architecture Node**: Visual/operational entity on the system map; may link to a Resource; Card remains visual-only, not a storage root.
- **Relationship**: Connection between architecture nodes (declared/linked/observed as product maturity allows).
- **Environment**: Scope filter/context (for example Production) for Resources and Architecture views.
- **Status Signal**: Compact operational state (healthy, warning, danger, muted) with text/icon affordance.
- **Quick Action**: Short operator action or deep-link to external specialized tool.

### Constitution Constraints *(mandatory)*

Map the feature to `.specify/memory/constitution.md` (v2.x). Analysis MUST block
implementation on critical unresolved conflicts.

- **Operational architecture goal (I)**: Strengthens architecture understanding and resource discovery/organization via calm, navigable Resources + Architecture surfaces; deep-links support external specialized tools rather than cloning them.
- **Runtime owner (III)**: Desktop Renderer owns presentation; Main/Preload remain thin for window/chrome only; no privileged execution logic in shared UI primitives; domain authority remains outside the design system.
- **Domain vs adapter (II, IV)**: Design system and presentational primitives are adapter/UI layer only—MUST NOT embed domain business rules, provider SDKs, or mutation authority. Resource/Architecture data access goes through existing product ports/contracts.
- **Contracts (V)**: Prefer existing generated contracts and feature modules; this feature MUST NOT invent parallel resource/architecture protocol entities for UI convenience.
- **Product loop impact**: System Map / Resources inventory / Inspector-as-page (detail pages) / Knowledge & external tool hand-off surfaces; Approvals/Audit not required as new mutation UIs in this feature unless already present.
- **Evidence expectations**: Visual edges and statuses MUST NOT claim observed/live wiring without honest state; placeholders labeled as such when demo data is used.
- **Mutation safety (VII)**: Create/edit dialogs in this feature are UI affordances; any real mutations MUST continue plan→review→approve→apply→logs→audit when they touch governed operations. Read-mostly catalog UI is acceptable for initial screens.
- **Operation audit fields**: Not newly required for pure navigation/presentation; if operations are exposed from detail quick actions, existing audit correlation rules apply.
- **Security boundary (VI)**: Renderer remains unprivileged (`window.polterware` bridge only); design system MUST NOT introduce unrestricted Node/OS access; secrets never rendered as permanent plain text in lists.
- **Local vs cloud authority (VIII)**: Shell status should reflect local core connectivity when available; offline/degraded honesty required; no fake cloud authority.
- **Performance (IX)**: Canvas and long lists MUST NOT block the shell; virtualization and non-blocking interaction expected for large data.
- **Resilience (X)**: Loading/empty/error/offline states required; retries where applicable.
- **Observability (XI)**: UI errors should be diagnosable; correlation IDs when surfacing operation failures if operations are invoked.
- **Test plan (XII)**: Risk-proportional tests for design-system primitives and critical navigation surfaces; visual acceptance against design document + prototype composition review; type/lint/test/build gates as project practice for the changed surface.
- **Structure created now (XIII)**: Extend existing frontend design-system and feature packages/apps—no scenic empty packages; no React product renderer; no resurrection of removed UI stacks.
- **Single source of truth (XIV)**: One operator visual language for in-scope Desktop surfaces; retire conflicting provisional shells on those screens; design document and tokens must not diverge permanently.
- **Big-bang / refactor context (XV)**: Aligns with post big-bang Solid desktop direction on branch work; replaces provisional UI patterns that contradict the quiet shell model.
- **Non-goals (I, XIII)**:
  - Not a full reimplementation of AWS/Vercel/Supabase/Grafana/Datadog consoles
  - Not pixel-perfect Linear clone or Linear branding
  - Not marketing/Remotion presentation work
  - Not light-theme dual system unless already mandated
  - Not permanent dual design systems
  - Not React Flow / React ecosystem UI
  - Not shipping every product nav destination with full backend depth—Resources and Architecture primary surfaces are the MVP depth; other nav items may remain stubs only if already present and honestly labeled

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a moderated review against the design-system document and `/prototype` composition guides, reviewers score shell anatomy (integrated sidebar + single main surface + page-based detail) as compliant on Resources and Architecture primary flows (≥ 90% of checklist items pass).
- **SC-002**: Brand accent appears on ≤ 5% of painted UI area on primary screens in default states (spot audit of Resources index, Resource detail, Architecture canvas, node detail); active nav and selection remain neutral.
- **SC-003**: Operators can open Resources index → resource detail → return, and Architecture canvas → node detail → return, each full path in under 30 seconds in a populated demo catalog without confusion about where they are.
- **SC-004**: Create/edit entry points for in-scope entities open as focused dialogs in under 2 seconds of user intent (click/shortcut) and dismiss without leaving residual permanent panels.
- **SC-005**: Keyboard-only users can reach and activate primary navigation, search, first list/canvas item, and dialog primary actions on in-scope screens without pointer use (task completion rate 100% in internal a11y smoke).
- **SC-006**: On a prototype-scale architecture map, pan/zoom/select keeps the shell interactive (no multi-second full-app freezes during ordinary interaction in manual performance smoke).
- **SC-007**: Resources lists with hundreds of rows remain scrollable and selectable without shell lockup in manual performance smoke (virtualization acceptable).
- **SC-008**: Loading, empty, and error states are present and distinguishable for Resources and Architecture primary pages (checklist 100%).
- **SC-009**: Visual anti-pattern audit finds zero instances of: permanent right inspector on Resources/Architecture indexes, brand-green active nav fill, card-in-card cascades, heavy glow/glass blur, or pure-black-only flat UI without surface hierarchy—on in-scope screens.
- **SC-010**: Design-system document exists at the agreed path and matches the implemented visual system for tokens and anatomy used by in-scope screens (no major undocumented divergences).

## Assumptions

- Desktop is the primary delivery surface for this feature; Web may later reuse browser-safe primitives but is not required to ship full Resources/Architecture parity in this feature.
- Existing monorepo structure (`apps/desktop`, `packages/frontend/*`, `packages/features/*`) is the home for implementation; paths equivalent to any suggested layout in the brief should be reused, not duplicated.
- Domain data for Resources and Architecture may use realistic fixtures or existing backend contracts; full multi-provider live sync is not required to accept the UI system if honesty about data source is preserved.
- Command palette may be included if already planned/present; if deferred, navigation via sidebar + pages still satisfies P1 stories.
- Prior light-oriented token docs may be superseded for Desktop operator chrome by this dark quiet system; migration of every legacy surface outside Resources/Architecture may be staged but in-scope screens must comply.
- Prototype folder contains composition references (Resources index/detail, Architecture canvas/node detail) and must be analyzed before implementation work in later phases.
- Icons: use a single existing project icon approach; if none, one Solid-compatible set with individual imports.
- No React, CSS Modules, styled-components/Emotion, full dashboard kits, or closed component libraries that block token restyling.
- Accessibility targets follow desktop operator norms (keyboard, focus-visible, labels, reduced motion, non-color-only status)—not a separate legal compliance certification scope unless later specified.
- `/speckit-plan` and `/speckit-tasks` will decompose design.md writing and implementation sequencing; this specification defines the product outcomes those phases must satisfy.

## Scope Boundaries

### In scope

- Formal design-system documentation as visual SSOT
- Desktop shell anatomy (integrated sidebar + main surface)
- Design tokens and shared primitives needed by primary screens
- Resources index + Resource detail page
- Architecture canvas + Architecture node detail page
- Create/edit dialogs for in-scope entities
- Loading / empty / error / offline presentations for those screens
- Navigation model and accessibility/performance expectations above

### Out of scope

- Full specialized-provider consoles and advanced observability product clones
- Complete implementation of every sidebar destination (Overview, Operations, Knowledge, etc.) beyond navigation presence/honest stubs
- Marketing site, presentation decks, or brand-illustration systems beyond small product mark usage
- Mobile-first responsive redesign
- Forced multi-theme marketplace
- Pixel-perfect replication of Linear or of AI mockup defects
)
