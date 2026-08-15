# Feature Specification: Ark UI–Backed Interactive Primitives

**Feature Branch**: `009-ark-ui-primitives`

**Created**: 2026-08-03

**Updated**: 2026-08-03 (headless backend: **Ark UI** replaces Kobalte)

**Status**: Draft

**Input**: User description: "Complete scan of UI and design system; migrate interactive primitives so behavior comes from a headless library and styling stays Polter. Adopt `@ark-ui/solid` as the official interactive primitive layer, fully replacing Kobalte. Ark UI is behavior only — not visual direction. Preserve local design-system components and public APIs. Do not hand-roll behavior Ark already owns."

## Problem Statement

The Desktop operator UI mixes three layers of interactive chrome:

1. **Design-system wrappers already on a headless base** (product dialog and large anchored popover — currently Kobalte).
2. **Design-system presentational shells with hand-rolled behavior** (section tabs with custom arrow keys; compact menu panel/rows with only static ARIA roles; side drawer with document-level Escape; toast region with no focus/dismiss model).
3. **Feature-local modal/overlay stacks** (command palette, terminal side panel, some canvas pickers) that reimplement Escape, outside click, and `role="dialog"` without shared focus trap or presence.

Hand-rolled behavior drifts. Kobalte was a partial adoption with known import/namespace fragility and incomplete coverage. The project now standardizes on **Ark UI (`@ark-ui/solid`)** as the **sole behavior, state, and accessibility** layer for interactive primitives.

**Critical split (non-negotiable):**

> **Ark UI controls how the component works. DESIGN.md, Polter tokens, and project Tailwind control how it looks.**

Ark UI **must not** decide colors, borders, radius, spacing, typography, shadows, density, visual animations, state appearance, or Polter visual language. Local design-system components remain the **official application API**; they encapsulate Ark primitives and apply existing Polter styles.

**Intent:** Inventory design-system and feature hand-rolls; migrate interactive internals to Ark UI; **remove Kobalte** from maintained workspace code and manifests; preserve visual SSOT and public APIs where practical.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Overlays behave the same everywhere (Priority: P1)

As an operator, every modal, drawer, command palette, and large anchored menu follows the same interaction contract: open/close is predictable, Escape dismisses, focus moves into the surface and returns to the trigger on close, and outside dismiss works where the product intends non-modal or modal policy.

**Why this priority**: Broken focus and Escape are the highest-frequency reliability and accessibility failures in operator chrome.

**Independent Test**: Open dialog, drawer, command palette, and breadcrumb/org switcher popovers; verify Escape, outside dismiss (where applicable), and focus return on each without feature-specific hacks.

**Acceptance Scenarios**:

1. **Given** any product dialog (including catalog-style flush dialogs), **When** the operator presses Escape or activates the close control, **Then** the dialog closes and keyboard focus returns to a sensible trigger or prior control.
2. **Given** a side drawer or side panel used for temporary detail, **When** the operator presses Escape or clicks the scrim, **Then** the panel closes with the same focus-return expectation as dialogs.
3. **Given** the command palette is open, **When** the operator presses Escape or clicks outside the frame, **Then** the palette closes without leaving a focus trap on a removed node.
4. **Given** a large anchored menu (org/project/user/target switchers), **When** the operator dismisses it via Escape or outside click, **Then** the underlying page remains usable and focus returns to the chevron/trigger.
5. **Given** two different product surfaces that open overlays, **When** an operator compares keyboard dismiss and focus behavior, **Then** the contracts match (no per-feature Escape-only document listeners as the sole behavior source).

---

### User Story 2 - Compact menus and section tabs are keyboard-complete (Priority: P1)

As an operator navigating without a mouse, compact dropdown/context menus and section navigation tabs support standard keyboard patterns and expose correct roles/names to assistive technology.

**Why this priority**: Menus and tabs are used constantly; static `role="menu"` without real menu semantics is a false accessibility signal.

**Independent Test**: Open a compact menu and a section tablist with keyboard only; complete selection and navigation.

**Acceptance Scenarios**:

1. **Given** a compact dropdown or context menu built from design-system menu primitives, **When** the operator opens it from a trigger, **Then** arrow keys move among items, Enter/Space activate, and Escape closes the menu.
2. **Given** the same menu, **When** inspected for accessibility tree roles, **Then** menu/menuitem (or the correct listbox pattern for a given picker) is consistent—no bare buttons inside a decorative `role="menu"` without a real keyboard model.
3. **Given** section navigation tabs on a feature index, **When** the operator focuses the tablist and uses arrow keys / Home / End, **Then** selection moves among enabled tabs and panel association remains correct.
4. **Given** inline **view filters** (grid/table icon toggles), **When** the operator uses them, **Then** they remain **not** section tabs (pressed toggles / filter chips).

---

### User Story 3 - Features stop shipping private modal stacks (Priority: P1)

As a maintainer, feature packages do not implement private document-level Escape handlers, custom `aria-modal` shells, or ad-hoc focus traps when a design-system primitive exists.

**Why this priority**: Private stacks reintroduce regressions after the design system is fixed.

**Independent Test**: Inventory high-traffic overlays (commander, terminal panel, canvas relationship picker, shell switchers); confirm each uses design-system wrappers.

**Acceptance Scenarios**:

1. **Given** the command palette, **When** reviewed, **Then** it uses shared dialog chrome and only customizes layout/search content.
2. **Given** the terminal side panel, **When** reviewed, **Then** it uses the shared drawer/dialog sheet primitive.
3. **Given** canvas pickers that act as listboxes/menus, **When** reviewed, **Then** they compose design-system popover/menu/select primitives.
4. **Given** a new feature needing a dropdown after this work, **When** implemented, **Then** AGENTS/DESIGN point only to design-system wrappers.

---

### User Story 4 - Presentational atoms and visual SSOT stay intact (Priority: P1)

As a designer/operator, the **look** of Polter does not change because of the headless migration. Pure visual atoms stay lightweight. Headless defaults, docs samples, or third-party themes never leak into product chrome.

**Why this priority**: Visual regression would be a failed migration even if behavior improves. Ark must adapt to Polter, not the reverse.

**Independent Test**: Spot-check dialogs, popovers, tabs, cards, fields against DESIGN.md; Card/Badge/Input remain presentational; no Ark/Chakra sample styling.

**Acceptance Scenarios**:

1. **Given** nested section cards and elevated surfaces, **When** rendered after migration, **Then** materials, radius, and density match pre-migration Polter chrome.
2. **Given** interactive overlays, **When** open/hover/focus/selected/disabled, **Then** visual states still use project tokens and CSS—not library demo classes.
3. **Given** the migration inventory, **When** a primitive is classified presentational, **Then** it is not forced through a headless root without need.
4. **Given** DESIGN.md, **When** migration completes, **Then** DESIGN.md is **not** rewritten to match Ark; code maps to existing DESIGN.

---

### User Story 5 - Kobalte removed; Ark is the only interactive headless layer (Priority: P1)

As a maintainer, the workspace has **zero** maintained `@kobalte/core` usage; design-system interactive components that need headless behavior use Ark UI under local wrappers; dependency manifests drop Kobalte when code is clean.

**Why this priority**: Dual headless stacks violate single source of truth and reintroduce import fragility.

**Independent Test**: Workspace grep for `@kobalte/core` empty in maintained product paths; `package.json` / lockfile no longer depend on Kobalte for product UI; design-system depends on `@ark-ui/solid`.

**Acceptance Scenarios**:

1. **Given** design-system interactive wrappers (Dialog, Popover, Tabs, Drawer, menus, …), **When** inspected, **Then** internals use Ark UI (or remain presentational with no headless).
2. **Given** `packages/features` and product apps, **When** grepped, **Then** there are no direct `@kobalte/core` or `@ark-ui/solid` imports where a local wrapper exists (features use design-system only).
3. **Given** package manifests for design-system and consumers, **When** migration is done, **Then** Kobalte is removed and Ark is declared where needed.
4. **Given** public local APIs (`A11yDialog` open/onClose, `Tabs` items/value/onChange, …), **When** consumers are reviewed, **Then** APIs are preserved where practical—no forced rewrite of every call site.

---

### User Story 6 - Full inventory is the migration SSOT (Priority: P2)

As a team, every design-system export and known feature hand-roll has disposition **migrate / leave presentational / out of scope / deferred**, and dual paths are removed before done.

**Independent Test**: Disposition checklist complete; no dual Tabs/Drawer/Dialog implementations.

**Acceptance Scenarios**:

1. **Given** the baseline inventory, **When** plan/tasks execute, **Then** every design-system export has disposition.
2. **Given** known feature hand-rolls, **When** complete, **Then** each is migrated or deferred with rationale.
3. **Given** temporary dual APIs, **When** feature is done, **Then** dual paths are gone (XIV).

---

### Edge Cases

- Nested overlays: last-opened dismisses first; focus restores through the stack.
- Modal vs non-modal policy must not collapse incorrectly.
- Disabled tabs/menu items skipped by keyboard.
- Reduced motion: open/close state still correct if motion shortened.
- Controlled open state: headless open-change calls product `onClose` / `onOpenChange` without loops.
- Portal z-index uses existing tokens.
- Flush catalog dialogs keep accessible names when title is sr-only.
- MenuItem-style rows **inside** Popover: no double focus trap (composition rules in plan).
- View filters must not use section Tabs semantics.
- DOM structure differences between Kobalte and Ark: adjust selectors/CSS only as needed; do not restyle to look like Ark docs.
- Compiling alone is **not** done — keyboard, focus, and visual parity required.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Interactive chrome behavior (state, keyboard, focus trap/restore, ARIA, portals, overlays, dismiss, positioning, selection, pointer/touch where applicable, controlled/uncontrolled) MUST come from **Ark UI** via design-system wrappers—not hand-rolled duplicates of those capabilities.
- **FR-002**: Product dialogs MUST use the shared local dialog component (Ark-backed) for modal behavior, including command palette chrome unless a local palette wrapper uses the same modal contract.
- **FR-003**: Drawers / temporary side panels MUST use a shared local drawer/sheet (Ark-backed dialog or drawer primitive) with scrim, Escape, and focus management.
- **FR-004**: Large anchored menus MUST use the shared local popover (Ark-backed) for positioning and dismiss.
- **FR-005**: Compact dropdown and context menus MUST be local design-system components with real menu keyboard behavior (Ark Menu / similar)—not static `role="menu"` panels alone.
- **FR-006**: Section navigation tabs MUST be local `Tabs` with standard keyboard model (Ark Tabs); view filters remain separate (ghost / FilterChip).
- **FR-007**: Searchable/complex selects MUST use shared local combobox/select (Ark-backed) when introduced—not ad-hoc absolute lists for behavior.
- **FR-008**: Feature packages MUST NOT introduce document-level Escape handlers or custom `aria-modal` shells for surfaces covered by design-system primitives.
- **FR-009**: Migration MUST complete disposition for every design-system export and known feature hand-rolls in this spec.
- **FR-010**: Visual language MUST remain Polter tokens and existing materials (`control-field`, `surface-panel` / `surface-panel-float`, `menu-panel`, radius tokens, project motion CSS). Ark MUST NOT introduce sample themes, Chakra-like chrome, or a second visual system. **Do not change DESIGN.md to fit Ark.**
- **FR-011**: Public local component APIs SHOULD remain stable (e.g. `open`/`onClose`, `items`/`value`/`onChange`); Ark internals MUST NOT be required by domain/feature code when a local wrapper exists.
- **FR-012**: Tooltips with non-trivial interaction MUST use a shared local tooltip (Ark-backed) when first needed; native `title` MAY remain for trivial tips.
- **FR-013**: Toast MUST keep a defined a11y contract (polite live region at minimum). Headless toast optional if it improves dismiss/queue without visual rewrite.
- **FR-014**: AGENTS.md / DESIGN.md MUST state: **behavior = Ark (via design-system); look = DESIGN + tokens + Tailwind.** Creating local styled wrappers over Ark is required—not a violation. Hand-rolling behavior Ark already provides **is** a violation.
- **FR-015**: Pure presentational primitives (Card, Surface, Badge, Separator, static Status*, page layout) MUST remain without mandatory headless roots.
- **FR-016**: Temporary dual paths MUST have exit criteria and be removed before done (XIV).
- **FR-017**: Workspace MUST remove maintained `@kobalte/core` imports and drop the Kobalte dependency from product manifests when migration completes.
- **FR-018**: Features and domain code MUST NOT scatter direct `@ark-ui/solid` imports when a design-system wrapper exists (wrap once).
- **FR-019**: Migration of each component MUST: map Ark equivalent → swap internals → keep styles → keep public API when possible → fix inevitable DOM diffs → validate focus/keyboard/a11y → visual compare → delete old Kobalte path.
- **FR-020**: Completion MUST NOT be claimed from compile-only success; keyboard, focus, open/closed/disabled/selected states, and visual consistency with DESIGN.md are required.

### Baseline Inventory *(authoritative starting point for plan/tasks)*

#### Design-system exports — expected disposition

| Export / surface | Baseline state | Disposition |
| --- | --- | --- |
| `Dialog` / `A11yDialog` | Kobalte-backed | **Re-implement on Ark**; preserve public API + Polter dialog frame |
| `Popover` | Kobalte-backed | **Re-implement on Ark**; preserve API + surface-panel-float |
| `Tabs` / `TabPanel` | Hand-rolled keyboard | **Migrate** to Ark Tabs; preserve pill styling + `items`/`value`/`onChange` |
| `Drawer` | Hand-rolled Escape | **Migrate** to Ark dialog/drawer sheet |
| `MenuPanel` / `MenuItem` | Presentational roles only | **Split**: presentational rows for Popover lists; Ark Menu for true dropdown/context |
| `ToastRegion` / `pushToast` | Simple live region | **Harden**; optional Ark toast later |
| `Input` / `Textarea` | Styled native | **Leave presentational** |
| `Select` (native) | Styled native | **Leave** simple; **add** Ark Select/Combobox when complex needed |
| `Field` | Label shell | **Leave presentational** |
| `FilterChip` / `FilterChipGroup` | Pressed toggles | **Leave** (not section Tabs) |
| `Button` | Styled control | **Leave** (optional Ark for specialized later) |
| `Badge`, `Card`, `Surface`, `Separator` | Visual | **Leave presentational** |
| `StatusDot` / `StatusChip` | Visual | **Leave presentational** |
| `PageHeader` / `PageContent` / `PageToolbar` | Layout | **Leave presentational** |
| Feedback empty/error/… | Layout | **Leave presentational** |
| `DataTable` | Table layout | **Leave** |
| `ResizableSplit` | Panel resize | **Out of scope** (Corvu / existing policy) |

#### Known feature hand-rolls — must address

| Surface | Issue | Disposition |
| --- | --- | --- |
| Command palette | Local Escape + custom modal | **Migrate** to local Dialog |
| Terminal side panel | Local Escape + custom dialog | **Migrate** to local Drawer |
| Relationship type popover | Bare listbox | **Migrate** to local Popover + rows |
| Shell context switchers / pickers | Popover + MenuItem | **Verify** after Popover/Menu Ark migration |

#### Kobalte purge inventory (must close)

| Signal | Action |
| --- | --- |
| `import … from "@kobalte/core…"` | Replace or delete |
| Solid UI copies depending on Kobalte | Re-base on Ark + Polter styles or remove |
| CSS coupled to Kobalte DOM | Retarget to Ark DOM only as needed; keep Polter look |
| Tests asserting Kobalte markup/API | Retarget to local public API / behavior |
| `package.json` / lockfile Kobalte deps | Remove when code clean |

### Key Entities

- **Interactive primitive**: Design-system export whose job includes open state, focus, keyboard, or overlay dismiss.
- **Headless backend**: Ark UI machine/parts providing behavior only.
- **Local styled component**: Public Polter wrapper (Dialog, Popover, Tabs, …) — official app API.
- **Presentational atom**: Visual-only export without overlay/keyboard ownership.
- **Feature hand-roll**: Feature-local overlay/keyboard shell duplicating primitive duties.
- **Disposition**: migrate | leave-presentational | out-of-scope | deferred.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Reliable operator chrome; not a specialized-platform clone.
- **Runtime owner (III)**: Desktop **Renderer** + `@polterware/frontend-design-system` only.
- **Domain vs adapter (II, IV)**: UI presentation only.
- **Contracts (V)**: UI contracts under this feature; no protobuf churn.
- **Product loop impact**: All surfaces using shared chrome.
- **Evidence expectations**: a11y/keyboard + visual parity + Kobalte purge.
- **Mutation safety (VII)**: UI-only.
- **Security boundary (VI)**: Unprivileged renderer; dismiss always available.
- **Performance (IX)**: No Main blocking; portals while open.
- **Resilience (X)**: No stuck modals.
- **Test plan (XII)**: Unit/export tests; residual grep; visual/keyboard acceptance; typecheck. Full e2e only if Erick asks.
- **Structure (XIII)**: Extend design-system only; no scenic packages.
- **SSoT (XIV)**: One headless stack (Ark); no permanent Kobalte+Ark dual; no permanent dual local APIs.
- **Big-bang (XV)**: Authorized UI refactor—decisive headless migration.
- **Non-goals**: No visual redesign; no React product UI; no Corvu for general chrome; no Ark/Chakra theme adoption; no DESIGN.md rewrite for the library.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of inventoried modals, drawers/side panels, command palette, and large anchored switcher menus dismiss via Escape and restore keyboard usability (no stuck focus).
- **SC-002**: 100% of section navigation tablists support keyboard movement among enabled tabs.
- **SC-003**: 100% of compact menus shipped via design-system menu primitives support keyboard open → move → activate → dismiss.
- **SC-004**: Zero feature-local document Escape listeners as primary dismiss for covered surfaces (or written deferrals only).
- **SC-005**: Disposition covers every design-system public export and known hand-rolls.
- **SC-006**: Visual language matches DESIGN.md / approved screens—no Ark docs theme, no Chakra-like chrome, no token drift without justification.
- **SC-007**: New interactive chrome is implementable via local design-system components only.
- **SC-008**: Zero maintained `@kobalte/core` imports in product workspace paths; Kobalte removed from product manifests when code is clean.
- **SC-009**: Interactive applicable components use Ark under local wrappers; public APIs preserved where practical.
- **SC-010**: Open/closed/active/disabled/selected states work for keyboard and look correct under Polter styles (not compile-only).

## Assumptions

- Official interactive headless library is **`@ark-ui/solid`** (Zag-based). It **fully replaces** Kobalte for this product.
- Local components in `@polterware/frontend-design-system` remain the only API features should use for interactive chrome.
- Ark usage is **behavior-only**; styling is 100% project-owned (tokens, Tailwind, component CSS, DESIGN.md).
- Creating local wrappers over Ark is **required and correct**; hand-rolling Ark-covered behavior is forbidden.
- Searchable switchers stay Popover + presentational rows (not Menu-inside-Popover double root).
- Corvu remains only for true resizable panels.
- Native checkboxes/simple selects may remain where DESIGN allows.
- Presentation (Remotion), backend, CLI, MCP out of scope.
- Full formal WCAG certification out of scope; target consistent keyboard/focus and honest ARIA.
)
