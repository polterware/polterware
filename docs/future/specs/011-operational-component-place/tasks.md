# Tasks: Operational Component Placement

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — unit for `place-config`, `place-link`, env normalize, card honesty labels; keep existing architecture tests green. Do not mark a task complete while required typecheck/tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US6) for independent implementation and testing.

**Constitution / guardrails**:

- Preserve canvas engine — no rewrite of `packages/frontend/canvas` core beyond existing `docAddNode` field pass-through
- Dependency direction IV: `feature-architecture` → canvas / design-system / state; resources fixtures via port adapter only
- SSoT XIV: single catalog → identity → place/link flow; funnel `CreateNodeDialog`
- Security VI: no secrets; no provision; no Target registration
- Honesty: Planned/Unlinked vs Linked; never map Linked → Healthy
- DESIGN: Field/Input/Select/Textarea/A11yDialog; product dialog frame; no multipage wizard

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US6 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Architecture feature: `packages/features/architecture/src/`
- Domain: `packages/features/architecture/src/domain/`
- Host: `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- Configure: `packages/features/architecture/src/canvas/configure-component-dialog.tsx`
- Catalog: `packages/features/architecture/src/canvas/add-to-architecture-dialog.tsx`
- View shell: `packages/features/architecture/src/index.tsx`
- Canvas document: `packages/frontend/canvas/src/document/`
- Resources fixtures: `packages/features/resources/src/fixtures.ts`
- Contracts: `specs/011-operational-component-place/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm touch list and inventory; no product behavior change

- [x] T001 Re-read preserve/modify/replace and touch list in `specs/011-operational-component-place/plan.md`; confirm files exist under `packages/features/architecture/src/domain/place-config.ts`, `place-hints.ts`, `canvas/configure-component-dialog.tsx`, `canvas/architecture-canvas-host.tsx`, `canvas/architecture-node-card.tsx`, `index.tsx`, `create-node-dialog.tsx`, `packages/frontend/canvas/src/document/mutations.ts` — no new packages
- [x] T002 [P] Confirm fixture env/resource sources usable for mock ports: `packages/features/resources/src/fixtures.ts`, `packages/features/shell/src/workspace-mocks/data.ts` (`RESOURCE_MOCKS`), and current hardcoded envs in `packages/features/architecture/src/domain/place-config.ts`

**Checkpoint**: Inventory confirmed; engine rewrite not planned

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Project env port, PlaceConfig v2 (`environmentId` + optional `resourceId`), pure link helpers, document field pass-through — **MUST complete before story UI**

**⚠️ CRITICAL**: No US1–US6 chrome polish until normalize + ports compile and unit tests for pure rules exist

### Project environments + PlaceConfig

- [x] T003 Create `ProjectEnvironment` type + `ProjectEnvironmentPort` + `DEFAULT_PROJECT_ENVIRONMENTS` fixture list (stable lowercase ids + labels) in `packages/features/architecture/src/domain/project-environments.ts` per [contracts/place-config.md](./contracts/place-config.md) and [data-model.md](./data-model.md)
- [x] T004 [P] Add `normalizeEnvironmentId` / label helpers (e.g. `Production` → `production`) in `packages/features/architecture/src/domain/project-environments.ts`
- [x] T005 Evolve `PlaceConfig` to use `environmentId` (not hardcoded-only allowlist), optional `resourceId`, and `normalizePlaceConfig(input, envs)` that rejects unknown env and empty env list in `packages/features/architecture/src/domain/place-config.ts` per [contracts/place-config.md](./contracts/place-config.md)
- [x] T006 [P] Keep `PLACE_ROLES`, `metadataFromPlaceConfig`, `environmentIdsFromConfig` working with new shape in `packages/features/architecture/src/domain/place-config.ts`
- [x] T007 Update unit tests for name/env/empty-list/role/resourceId optional omit in `packages/features/architecture/src/domain/place-config.test.ts`

### Resource link pure domain

- [x] T008 [P] Implement `LinkableResource`, type-family map, `listCandidates` pure filter helpers, and `validateLink(componentEnvId, resource)` in `packages/features/architecture/src/domain/place-link.ts` per [contracts/resource-link-port.md](./contracts/resource-link-port.md)
- [x] T009 [P] Unit tests: env mismatch reject, type family match, empty candidates, normalize env labels in `packages/features/architecture/src/domain/place-link.test.ts`

### Document mutations readiness

- [x] T010 Verify `docAddNode` / `addNode` persists `owner`, `linkStatus`, `resourceId`, `description`, `metadata` in `packages/frontend/canvas/src/document/mutations.ts`; add any missing field pass-through only (no engine rewrite)
- [x] T011 [P] Export any new pure helpers needed by feature from package barrels only if already the public pattern (`packages/features/architecture` main is `src/index.tsx` — prefer internal imports for domain helpers)

### Fixture resource adapter (data only)

- [x] T012 Implement `createFixtureResourceLinkPort()` mapping `RESOURCE_FIXTURES` (and/or shell mocks) → `LinkableResource` with `dataSource: "fixture"` in `packages/features/architecture/src/domain/fixture-resource-link-port.ts` (or `canvas/fixture-resource-link-port.ts`) without importing Electron

**Checkpoint**: Domain normalize + link validate tested; ports defined; stories may implement UI

---

## Phase 3: User Story 1 — Place planned component with real identity (Priority: P1) 🎯 MVP

**Goal**: Catalog → identity (name + **project environment**) → place as Planned/Unlinked

**Independent Test**: Add PostgreSQL, name `payments-db`, Staging from project envs, place; card shows name; detail shows Staging; Planned/unlinked; cancel creates nothing

### Tests for User Story 1

- [x] T013 [P] [US1] Extend/confirm `place-config.test.ts` covers project env list injection (not hardcoded-only success path) in `packages/features/architecture/src/domain/place-config.test.ts`
- [x] T014 [P] [US1] Unit test helper that builds `docAddNode` partial from `PlaceConfig` (planned/unlinked when no resourceId) in `packages/features/architecture/src/domain/place-commit.test.ts` (new) or extend existing create-node tests

### Implementation for User Story 1

- [x] T015 [US1] Wire `ProjectEnvironmentPort` into configure: pass `environments` prop (from parent or default fixture) into `packages/features/architecture/src/canvas/configure-component-dialog.tsx`
- [x] T016 [US1] Replace environment `Select` options with project environments; default production-like or first env; validate via `normalizePlaceConfig(input, envs)` in `packages/features/architecture/src/canvas/configure-component-dialog.tsx`
- [x] T017 [US1] Ensure Name + Environment required fast path (purpose/role/owner optional) and Enter submits place in `packages/features/architecture/src/canvas/configure-component-dialog.tsx`
- [x] T018 [US1] Update `placeConfigured` in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` to write `environmentIds`, `description`, `owner`, `metadata`, `state: "planned"`, `linkStatus: "unlinked"`, `resourceId: null` when no link; history `immediately`
- [x] T019 [US1] Provide environments from `ArchitectureView` (fixture port default) into host/configure props in `packages/features/architecture/src/index.tsx`
- [x] T020 [US1] Cancel/Escape on configure closes without `docAddNode` (verify host state reset) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`

**Checkpoint**: MVP — project-env place as Planned/Unlinked works

---

## Phase 4: User Story 2 — Understand type vs instance before place (Priority: P1)

**Goal**: Configure header/copy separates catalog type from instance; primary action is place, not provision

**Independent Test**: API vs Database vs SaaS show different kind/placeholder; status line planned/unlinked; button “Place on canvas”

### Tests for User Story 2

- [x] T021 [P] [US2] Unit tests for `placeHints` kindLabel/purposePlaceholder/statusLine in `packages/features/architecture/src/domain/place-hints.test.ts`

### Implementation for User Story 2

- [x] T022 [US2] Ensure configure header uses `placeHints(def)` (kind · type blurb · planned/unlinked status line) and does not prefill Purpose with catalog description in `packages/features/architecture/src/canvas/configure-component-dialog.tsx`
- [x] T023 [US2] Primary footer action labeled **Place on canvas** (or equivalent place/commit); no Deploy/Provision/Register target copy in `packages/features/architecture/src/canvas/configure-component-dialog.tsx`
- [x] T024 [US2] Keep **More context** (role, owner) collapsed by default with Field/Select/Input design-system controls in `packages/features/architecture/src/canvas/configure-component-dialog.tsx`
- [x] T025 [P] [US2] Extend purpose placeholders by category if gaps remain in `packages/features/architecture/src/domain/place-hints.ts`

**Checkpoint**: Identity step feels operational, not sticker or provision wizard

---

## Phase 5: User Story 3 — Optionally link existing resource (Priority: P1)

**Goal**: Optional link during configure and/or commit with resourceId → Linked without inventing Observed

**Independent Test**: Link fixture postgres on place → Linked + resource ref; skip → Planned; empty list honest

### Tests for User Story 3

- [x] T026 [P] [US3] Unit tests for place-commit with `resourceId` → `state: "linked"`, `linkStatus: "linked"` in `packages/features/architecture/src/domain/place-commit.test.ts`
- [x] T027 [P] [US3] Unit tests for fixture port returns candidates and filters by env/type in `packages/features/architecture/src/domain/fixture-resource-link-port.test.ts` (or place-link tests)

### Implementation for User Story 3

- [x] T028 [US3] Add optional collapsed **Link existing resource** section + search/list UI in configure (use new `packages/features/architecture/src/canvas/resource-link-picker.tsx` or inline) consuming `ResourceLinkPort`
- [x] T029 [US3] Pass `resourceLinkPort` from `packages/features/architecture/src/index.tsx` into host → configure; default fixture port
- [x] T030 [US3] On confirm with `resourceId`, `placeConfigured` sets `state: "linked"`, `linkStatus: "linked"`, `resourceId` and does **not** set observed/healthy in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T031 [US3] Cross-env selection blocked with inline error via `validateLink` in configure path `packages/features/architecture/src/canvas/configure-component-dialog.tsx`
- [x] T032 [US3] Empty candidates show honest empty state (no fake rows) in `packages/features/architecture/src/canvas/resource-link-picker.tsx`

**Checkpoint**: Model-first skip and optional link both work

---

## Phase 6: User Story 5 — Lifecycle & environment on map and detail (Priority: P1)

**Goal**: Card and DetailPanel show Planned vs Linked honestly; env labels; purpose/role/owner; post-place Link CTA

**Independent Test**: Unlinked vs linked cards differ; detail shows env/role/resource; Linked ≠ Healthy

### Tests for User Story 5

- [x] T033 [P] [US5] Update `nodeStatusLabel` tests: `planned` → Planned, `linked` → Linked (not Healthy) in `packages/features/architecture/src/canvas/architecture-node-card.test.tsx`
- [x] T034 [P] [US5] Unit/assert detail mapping for role/env labels if pure helper extracted; otherwise cover via small mapper test next to `packages/features/architecture/src/index.tsx` logic if extracted to `packages/features/architecture/src/domain/node-detail-map.ts`

### Implementation for User Story 5

- [x] T035 [US5] Fix `nodeStatusLabel` honesty mapping in `packages/features/architecture/src/canvas/architecture-node-card.tsx` (linked ≠ Healthy; planned = Planned) per research R3
- [x] T036 [US5] Show link indicator when `linkStatus === "linked"` or `resourceId` set on card props pipeline from host render in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` + `architecture-node-card.tsx` (minimal chrome, DESIGN tokens)
- [x] T037 [US5] Map detail: environment label from project envs, role label, purpose → notes, link state, resource id/name in `packages/features/architecture/src/index.tsx` `detailNode` memo
- [x] T038 [US5] Add DetailPanel action **Link resource** when unlinked in `packages/features/architecture/src/node-detail.tsx` + wire open resource picker / updateNode link in `packages/features/architecture/src/index.tsx`
- [x] T039 [US5] Ensure canvas environment filter continues to use `environmentIds` (normalized ids) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` / parent `envId` without separate fake taxonomy

**Checkpoint**: Map + detail communicate lifecycle and env; honesty regression fixed

---

## Phase 7: User Story 4 — Connected catalog / import stubs (Priority: P2)

**Goal**: Link-from-Resources mock path; import stubs honest; no half-created nodes

**Independent Test**: Connected stub toast no node; Link from Resources → prefilled identity → linked place

### Tests for User Story 4

- [x] T040 [P] [US4] Unit test resource→suggested type/name/env mapping helper in `packages/features/architecture/src/domain/place-link.ts` (+ tests)

### Implementation for User Story 4

- [x] T041 [US4] Add Connected catalog action **Link from Resources** (placeable action row or dedicated control) in `packages/features/architecture/src/canvas/add-to-architecture-dialog.tsx` / registry if needed without scenic types
- [x] T042 [US4] Host: resource-first open → pick resource → open configure prefilled (type infer, name, env, resourceId) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T043 [US4] Unimplemented import-from-integration actions remain toast “Not available yet” with zero nodes in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` / catalog `onUnavailableAction`
- [x] T044 [US4] Cancel resource-first leaves document unchanged in host state machine

**Checkpoint**: Connected path honest; resource-first works in mock

---

## Phase 8: User Story 6 — Placement modes & cancel-safe (Priority: P2)

**Goal**: Origin known → place after configure; origin null → placement preview; name SSOT; Escape discards

**Independent Test**: N → configure → placement click commits once; Escape drops; no forced second rename

### Tests for User Story 6

- [x] T045 [P] [US6] Extend escape/placement unit coverage if modes exist in `packages/features/architecture/src/canvas/escape-layers.test.ts` or host test for “configure cancel / placement escape leaves no node”

### Implementation for User Story 6

- [x] T046 [US6] After configure confirm: if `placeOrigin` set, commit immediately; if null, enter placement mode with draft config (name/type visible) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` per [contracts/authoring-flow.md](./contracts/authoring-flow.md)
- [x] T047 [US6] Escape in placement discards draft without `docAddNode`; Escape stack order preserved in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T048 [US6] Do not force inline rename after configure-named place; rename remains optional action only in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`

**Checkpoint**: Spatial place professional; cancel-safe

---

## Phase 9: Cross-story SSOT — CreateNodeDialog funnel (supports FR-033)

**Purpose**: Components tab uses same catalog → identity → place semantics

- [x] T049 Align `packages/features/architecture/src/create-node-dialog.tsx` to open shared catalog (or host-equivalent flow) instead of freeform draft that skips identity
- [x] T050 Wire Components tab `+` through same place pipeline (switch to canvas tab on commit if needed) in `packages/features/architecture/src/index.tsx`

**Checkpoint**: No permanent dual freeform create

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Convergence, quickstart, residual honesty

- [x] T051 [P] Run domain unit tests for place-config, place-link, place-hints, node card labels; fix failures under `packages/features/architecture/src/domain/*.test.ts` and `canvas/architecture-node-card.test.tsx`
- [x] T052 [P] `pnpm design:lint` (or opacity lint) if UI chrome changed; fix any new violations in architecture feature files only
- [x] T053 Remove leftover hardcoded-only env assumptions in call sites (`environment` vs `environmentId`) across `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` and tests
- [x] T054 [P] Confirm no Deploy/Provision/Healthy-for-linked strings remain in configure/card (grep + fix)
- [x] T055 Manual walkthrough of [quickstart.md](./quickstart.md) scenarios A–E when operator requests visual validation (document results in PR notes; agents skip `pnpm dev` unless asked)
- [x] T056 Convergence check: code ↔ spec FR-001–033 ↔ plan phases ↔ this tasks list; update tasks checkboxes only when done

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 Setup**: immediate
- **Phase 2 Foundational**: after Setup — **BLOCKS all user stories**
- **US1 (Phase 3)**: after Foundational — **MVP**
- **US2 (Phase 4)**: after Foundational; naturally after US1 configure exists (same dialog files — sequential preferred)
- **US3 (Phase 5)**: after Foundational + benefits from US1 place path
- **US5 (Phase 6)**: after US1 place (and ideally US3 for linked samples)
- **US4 (Phase 7)**: after US3 link port + configure prefill
- **US6 (Phase 8)**: after US1 placeConfigured
- **Phase 9 SSOT**: after US1 configure/place stable
- **Phase 10 Polish**: after desired stories

### User Story Dependencies

| Story | Depends on | Independently testable? |
| --- | --- | --- |
| US1 | Foundational | Yes — MVP |
| US2 | US1 dialog shell | Yes — copy/header checks |
| US3 | Foundational + US1 commit path | Yes — link skip/with resource |
| US5 | US1 (+ US3 for linked) | Yes — card/detail honesty |
| US4 | US3 port + US1 identity | Yes — connected stubs |
| US6 | US1 placeConfigured | Yes — placement/escape |

### Parallel Opportunities

- T002 ∥ T001 (setup)
- T004 ∥ T003 after types started; T008–T009 ∥ T005–T007 once env types exist
- T013–T014 ∥ after Phase 2
- T021 ∥ US2 tests; T026–T027 ∥ US3 tests
- T033–T034 ∥ US5 tests
- T051–T052–T054 polish in parallel

---

## Parallel Example: Foundational + US1

```bash
# After T003 types land:
Task: "T008 place-link pure helpers"
Task: "T009 place-link tests"
Task: "T005–T007 place-config evolve + tests"

# After Phase 2 checkpoint:
Task: "T015–T017 configure project envs"
Task: "T018 placeConfigured planned/unlinked"
```

---

## Parallel Example: US3 + US5 honesty

```bash
Task: "T035 nodeStatusLabel honesty (US5)"
Task: "T028–T030 link in configure + place (US3)"
```

---

## Implementation Strategy

### MVP First (US1 only)

1. Phase 1 Setup  
2. Phase 2 Foundational  
3. Phase 3 US1  
4. **STOP** — validate quickstart scenario A (model-first place)  
5. Demo planned component with project environment  

### Incremental delivery

1. US1 MVP → project-env Planned place  
2. US2 → professional identity copy  
3. US3 → optional link  
4. US5 → card/detail honesty + Link CTA  
5. US4 → Connected resource-first  
6. US6 → placement mode polish  
7. Phase 9–10 → SSOT + polish  

### Suggested single-agent order

`T001→T012` → `T013→T020` (MVP) → `T021→T025` → `T026→T032` → `T033→T039` → `T040→T044` → `T045→T048` → `T049→T050` → `T051→T056`

---

## Notes

- [P] = different files, no incomplete deps  
- Every task has checkbox + ID + path  
- Prefer extending existing configure/host over new packages  
- Fixture ports only until core inventory/env RPC exists  
- Do not implement provisioning, Target install, or live provider import  
- Implementation ends only after code ↔ spec ↔ plan ↔ tasks convergence  
