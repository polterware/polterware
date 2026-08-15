# Feature Specification: Shared UI Package

**Feature Branch**: `001-shared-ui-package`

**Created**: 2026-07-17

**Status**: Draft

**Input**: User description: "gostaria de fazer a modularizacao dos componentes de ui que temos para um pacote compartilhado de ui que pode ser usado tanto em web, quanto em desktop"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Desktop keeps working on a shared design system (Priority: P1)

As a product engineer shipping Polterware Desktop, I can build and evolve operator surfaces using a single shared design-system package instead of app-local UI primitives, without changing how operators use the workbench.

**Why this priority**: Desktop is the live surface today. Modularization only has value if Desktop continues to look and behave correctly while consuming the shared package. This is the MVP slice.

**Independent Test**: Open the main Desktop workbench flows that already use UI primitives (dialogs, buttons, tabs, cards, forms, side panels). Confirm visual and interaction parity with the pre-extraction experience, with imports coming from the shared package rather than a desktop-only component tree.

**Acceptance Scenarios**:

1. **Given** Desktop currently renders operator UI with local design primitives, **When** those primitives move into the shared UI package and Desktop consumes them, **Then** existing screens keep the same visual language, spacing, radius, materials, and interaction patterns defined by the product design system.
2. **Given** an operator opens common surfaces (commander-style overlays, settings, process/command boards, repository side panels, system views), **When** they interact with buttons, dialogs, tabs, inputs, menus, and empty states, **Then** behavior remains functional and no privileged desktop capability is required from the shared package.
3. **Given** a designer or engineer updates a shared primitive or design token in the shared package, **When** Desktop rebuilds against that package, **Then** the change is available to Desktop without copy-pasting component code between apps.

---

### User Story 2 - Web can consume the same browser-safe UI package (Priority: P2)

As a product engineer preparing Polterware Web (self-hosted or Cloud client), I can depend on the same shared UI package used by Desktop for design primitives and presentational building blocks, without inheriting Electron, Node, filesystem, or local execution concerns.

**Why this priority**: The constitutional reuse rule is “share design system + domain feature views; specialize shells and authority adapters.” Web is not the live runtime yet, but the package must be ready so Web does not reimplement the design system later.

**Independent Test**: A non-desktop browser-safe consumer (Web app scaffold or isolated consumer) can import and render shared primitives with the same tokens and public API surface as Desktop, without needing desktop adapters.

**Acceptance Scenarios**:

1. **Given** the shared UI package is published in the monorepo workspace, **When** a Web surface depends on it, **Then** it can render core primitives (button, input, dialog/sheet patterns, tabs, card/surface treatments, feedback states) without importing any desktop-only module.
2. **Given** a shared primitive is intended for both Desktop and Web, **When** package boundaries are reviewed, **Then** the primitive contains no privileged APIs, raw terminal authority, secret handling, local discovery, or Electron lifecycle hooks.
3. **Given** Desktop and Web both depend on the shared package, **When** operators move between product surfaces in the future, **Then** core controls and materials feel like one product family rather than two separate UI kits.

---

### User Story 3 - Clear share vs shell boundary for future reuse (Priority: P3)

As a product engineer, I know which UI pieces belong in the shared package versus which stay app-local (Desktop shell, windowing, dock chrome, authority adapters), so modularization does not become a forced super-package.

**Why this priority**: Prevents over-sharing. Constitution forbids force-sharing full page shells, Electron lifecycle, privileged adapters, and desktop-only workbench mechanics.

**Independent Test**: Review the package inventory and ownership map. Shared package contains only browser-safe design-system and presentational building blocks; shell/chrome/adapters remain in app code.

**Acceptance Scenarios**:

1. **Given** the current Desktop UI tree, **When** modularization is planned and applied, **Then** design primitives and reusable presentational components are candidates for the shared package, while Desktop shell chrome, window controls, dock mechanics, and privileged adapters remain Desktop-owned.
2. **Given** a component needs Desktop-only behavior (for example panel width persistence in local storage, Electron drag regions, or terminal session authority), **When** extraction is considered, **Then** either the privileged part stays in Desktop or the shared part is limited to a pure presentational surface with app-provided adapters.
3. **Given** domain feature views (map widgets, approval lists, audit timelines) are not yet multi-consumer, **When** this feature ships, **Then** they may remain in Desktop features until a second real consumer exists, without blocking the shared design-system package.

---

### Edge Cases

- What happens when a Desktop component mixes pure presentation with privileged behavior? Presentation may be shared; privilege stays app-local behind an adapter.
- How does the system handle design-token or theme updates? Tokens and surface utilities live with the shared design system so Desktop and future Web do not diverge.
- What if Web is not scaffolded yet? The shared package must still be consumable in principle; a live Web app is not required to validate Desktop consumption.
- What if a primitive is unused or desktop-experiment-only? Prefer not extracting speculative one-off experiments until a second consumer or clear design-system role exists.
- How are visual regressions caught? Desktop acceptance is visual/interaction parity on retained workbench surfaces; shared package changes must not silently break commander overlays, settings, process boards, or repository panels.
- What about presentation/marketing surfaces (for example Remotion decks)? Out of scope unless they explicitly need product operator UI primitives.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Product MUST provide a single shared UI package that owns the reusable design-system primitives currently scattered under the Desktop renderer UI tree.
- **FR-002**: Desktop MUST consume the shared UI package for extracted primitives instead of maintaining a parallel local copy of those primitives.
- **FR-003**: The shared UI package MUST be browser-safe and usable by both Desktop renderer surfaces and future Web surfaces.
- **FR-004**: The shared UI package MUST NOT include Electron APIs, Node privileged APIs, filesystem access, process execution, PTY/terminal session authority, secret stores, or desktop window lifecycle logic.
- **FR-005**: Shared package ownership MUST include design primitives and presentational building blocks needed for consistent operator UI (for example buttons, inputs, dialogs/sheets, tabs, menus, cards/surfaces, badges, feedback/empty states, and related form/layout primitives already used by Desktop).
- **FR-006**: Design tokens, radius, materials, and shared surface treatments defined by the product design system MUST remain the single source of truth consumed by the shared UI package and its app consumers.
- **FR-007**: Desktop shell/chrome, workbench docking mechanics, navigation IA shells, and authority adapters MUST remain app-owned specializations, not forced into the shared package.
- **FR-008**: Extraction MUST preserve operator-facing behavior and visual quality on retained Desktop workbench surfaces; modularization is not allowed to regress the professional operator interface bar.
- **FR-009**: The monorepo MUST expose the shared UI package as a first-class workspace package so Desktop and future Web can depend on it without path hacks or duplicated trees.
- **FR-010**: Package public surface MUST be intentional: only stable, reusable UI exports are public; internal helpers that are not part of the design system contract stay private or app-local.
- **FR-011**: When a current Desktop component is only partially shareable, the product MUST split pure presentation (shareable) from privileged/app-specific behavior (not shareable) rather than polluting the shared package.
- **FR-012**: Domain feature modules (process boards, repositories, system map views, etc.) MAY continue to live under Desktop features in this feature’s scope; full feature-view extraction into shared domain packages is optional follow-on once a second consumer exists.
- **FR-013**: Documentation of the share boundary MUST make clear what belongs in the shared UI package versus Desktop/Web shells so future contributors do not reintroduce duplicated primitives.
- **FR-014**: Existing product design rules for overlays, frosted materials, texture buttons, radius, and dense calm workbench aesthetics MUST continue to apply after modularization.

### Key Entities

- **Shared UI Package**: Browser-safe package that owns reusable design-system primitives and presentational building blocks for Polterware product surfaces.
- **Design Primitive**: Low-level reusable control or surface element (button, input, dialog, tabs, card, etc.) with no product-domain privilege.
- **Design Token / Material Contract**: Shared visual source of truth for color, radius, typography, glass/frosted surfaces, and related treatments.
- **App Shell**: Surface-specific chrome and navigation structure (Desktop workbench frame, future Web app frame) that composes shared primitives but is not itself shared wholesale.
- **Authority Adapter**: App-owned bridge that connects UI to privileged capabilities (Desktop local control plane, future Web governance client). Never part of the shared UI package.
- **Domain Feature View**: Higher-level operator view composed from primitives (approvals, map widgets, process boards). Share later when a second consumer exists; not required for this feature’s MVP.
- **Consumer Surface**: Product surface that depends on the shared package (Desktop renderer now; Web later).

### Constitution Constraints *(mandatory)*

- **Authority surface**: Shared browser-safe package; consumers are Desktop renderer now and Web later. No Cloud/Node authority inside the package.
- **Product loop impact**: Enables consistent System Map / Inspector / Evidence dock / Operations / Workspace panel chrome and controls without changing the control-plane loop itself.
- **Evidence expectations**: None for domain evidence lifecycle. Success is visual/interaction parity and boundary honesty, not new observed/managed resource states.
- **Mutation safety**: Read-only relative to system mutations. This feature reorganizes UI ownership only; it does not introduce plan→apply operations.
- **Execution owner**: Renderer-safe UI only. Shared package is unprivileged. Desktop Main/preload/Rust remain outside scope except that they must not be imported by the package.
- **Security boundary**: Shared package stays unprivileged and browser-safe. Secrets, exec, PTY, FS, and Electron stay out. Web must not inherit desktop privilege through UI reuse.
- **Performance notes**: Extraction must not degrade dense workbench interactions (lists, boards, overlays). Shared package should not force heavier runtime assumptions than current Desktop primitives.
- **UI quality bar**: DESIGN.md is hard constraint. Dense, calm, premium operator workbench. Overlay/dialog materials, radius, frosted surfaces, and texture-button rules remain mandatory.
- **Structure created now**: Add a shared UI workspace package under `packages/` (design-system oriented, e.g. UI package). Extend Desktop to consume it. Do not create Web app, services, or crates solely for this feature. Domain `feature-*` packages are later, not required now.
- **Roadmap phase fit**: Phase 0 Architecture Runway (shared UI packages if justified) with direct enablement for Phase 4 collaborative Web.
- **UI reuse**: Shared package required for design system primitives; shells/adapters remain specialized; domain feature views are shared-package candidates later, not mandatory in this feature.
- **Non-goals**:
  - Building Polterware Web runtime in this feature
  - Force-sharing Desktop shell, dock, windowing, or Electron lifecycle
  - Moving privileged process/terminal/session logic into the shared package
  - Extracting every domain feature view into packages before a second consumer exists
  - Rewriting the design language or replacing the existing operator aesthetic
  - Using modularization as an excuse to delete retained workbench features

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of extracted design primitives used by Desktop are imported from the shared UI package, with no surviving parallel copy of those primitives in the Desktop-local UI tree.
- **SC-002**: Operators can complete the same primary Desktop workbench tasks as before modularization with no intentional UX regressions on commander overlays, settings, process/command boards, repository panels, and system views.
- **SC-003**: A second browser-safe consumer can render the shared package’s core primitives without any desktop-only dependency, proving Web readiness even if Web is still scaffold-level.
- **SC-004**: Package boundary review finds zero privileged imports (Electron/Node exec/FS/PTY/secrets/window lifecycle) inside the shared UI package.
- **SC-005**: Visual consistency checks confirm shared tokens/materials (radius, surfaces, overlay treatment, control density) match the product design system on Desktop after extraction.
- **SC-006**: Contributors can identify in under 5 minutes, from project docs/ownership notes, whether a new component belongs in the shared UI package or in an app shell/feature.
- **SC-007**: Modularization does not require creating unrelated apps, services, or privileged runtimes; only the justified shared UI package and consumer wiring are introduced.

## Assumptions

- Desktop is the only live product consumer at feature start; Web is a future consumer and does not need full product implementation in this feature.
- The extraction source of truth is the existing Desktop renderer design primitives and DESIGN.md materials/tokens, not a greenfield redesign.
- MVP scope is the design-system package (primitives + tokens/materials contract + Desktop consumption), not full domain-feature package extraction.
- `apps/presentation` (motion/deck surfaces) is out of scope unless it later needs operator UI primitives.
- Partial shareability is preferred over over-sharing: pure presentation can move; privileged or shell-specific behavior stays in Desktop.
- Existing monorepo workspace layout already supports new packages under `packages/*`.
- Public package naming and exact export taxonomy can be decided in planning, as long as the package is first-class, browser-safe, and clearly owned.
- Domain feature views remain under Desktop features until a second real consumer justifies further extraction.
- No change to product mutation model, IPC catalog, or Rust authority boundaries is required for this feature.
- Quality bar remains “professional operator interface”; modularization is structural reuse, not a visual redesign project.
