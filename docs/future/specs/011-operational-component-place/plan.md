# Implementation Plan: Operational Component Placement

**Branch**: `011-operational-component-place` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/011-operational-component-place/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `005-architecture-canvas`, `006-semantic-architecture-canvas` (catalog + host + document v1); DESIGN.md product dialog/catalog; Resources fixtures for mock link; project environment context (shell/project mocks)

**Risk control**: No canvas engine rewrite. No provisioning. No Target registration. No fake Observed/healthy on place. Incremental host + domain + card/detail honesty only.

---

## Summary

Make **Add to architecture** a professional **Model → optional Link** flow: catalog type → identity against **project environments** → place as **Planned / Unlinked** → optional link to an existing resource (mock inventory) → **Linked** without inventing health.

**Technical approach** (see research):

1. **ProjectEnvironmentPort** — configure reads project envs (fixture/shell), not hardcoded `PLACE_ENVIRONMENTS` alone.
2. **PlaceConfig v2** — name, environmentId, optional purpose/role/owner, optional `resourceId`.
3. **Configure dialog** — identity + progressive disclosure + optional “Link existing resource…”.
4. **placeConfigured** — `docAddNode` with planned/unlinked or linked+resourceId; history immediately.
5. **Card + detail honesty** — Planned ≠ Healthy; Linked ≠ Observed/healthy without evidence.
6. **Detail CTA** — “Link resource” when unlinked.
7. **Connected catalog** — resource-first mock path + honest stubs for import.
8. **CreateNodeDialog** — funnel into the same flow (FR-033).

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**: `@polterware/feature-architecture`, `@polterware/frontend-canvas`, `@polterware/frontend-design-system`, `@polterware/frontend-state` (DetailPanel), existing Resources fixtures / workspace mocks

**Storage**: Architecture document in existing layout body (autosave); mock environments + inventory in fixtures for Community/demo. No new SQLite tables required for mock phase.

**Testing**: Vitest unit (place-config, env normalize, link rules, card labels); feature host tests where cheap; no full `pnpm dev` unless operator asks

**Target Platform**: Desktop renderer (Electron) Architecture Canvas; browser-safe packages only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**: Catalog + identity interactive &lt; 100ms open; place commit feels immediate; resource picker OK for fixture lists (&lt; ~200 rows without mandatory virtualization)

**Constraints**:
- Renderer: `window.polterware` only; no secrets in place form
- Honest lifecycle (Constitution I evidence rules)
- DESIGN: Field/Input/Select/Textarea/A11yDialog; one product dialog frame; no multipage wizard
- Agents.md: no `pnpm dev`/build unless asked; English UI strings

**Scale/Scope**: One authoring flow surface (canvas primary); Components tab aligned; mock link inventory; no live provider import

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Model + optional Link; not sticker; not cloud console clone |
| **II. Domain Before Framework** | PASS | normalize/link rules in feature domain pure modules; UI binds only |
| **III. Runtime Separation** | PASS | Renderer feature; persist via existing layout path; no Main domain |
| **IV. Monorepo deps** | PASS | architecture → canvas/design-system/state; resources fixtures as data port, no reverse domain import cycle |
| **V. Contracts-First** | PASS | UI/document contracts in `contracts/`; no new dual protobuf for mock |
| **VI. Security** | PASS | No secrets; resource ids only; no privileged provision |
| **VII. Safe execution** | PASS | Map mutation only; no silent cloud destroy |
| **VIII. Local-first** | PASS | Place works offline without integrations |
| **IX. Performance** | PASS | No Main work; small dialogs |
| **X. Resilience** | PASS | Cancel-safe; link load fail → skip planned |
| **XI. Observability** | PASS | Reuse autosave/status; no fake ops |
| **XII. Testability** | PASS | Pure normalize/link tests without Electron |
| **XIII. Real code** | PASS | Extend existing packages only |
| **XIV. SSOT** | PASS | One create semantics; align CreateNodeDialog |
| **XV. Big-bang** | PASS | Incremental on refactor branch; not dual canvas |
| **XVI. Docs** | PASS | Spec + plan + contracts |

**Hard stops**: No security/dependency/SSOT violations.  
**Known honesty bug to fix**: `nodeStatusLabel` maps `linked` → “Healthy” — violates evidence rules; fix in this feature (research R3).

**Post-Phase 1 re-check**: PASS — design keeps domain pure, mock ports, honest states, single create flow.

---

## Project Structure

### Documentation (this feature)

```text
specs/011-operational-component-place/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── authoring-flow.md
│   ├── place-config.md
│   └── resource-link-port.md
├── checklists/requirements.md
└── tasks.md             # /speckit-tasks — not this command
```

### Source Code (touch list)

```text
packages/features/architecture/src/
├── domain/
│   ├── place-config.ts          # expand env as id from port; resourceId optional
│   ├── place-hints.ts           # keep type-aware copy
│   ├── place-link.ts            # NEW: filter/link validation pure helpers
│   └── project-environments.ts  # NEW: types + fixture defaults for env port
├── canvas/
│   ├── add-to-architecture-dialog.tsx   # Connected link-from-resources entry
│   ├── configure-component-dialog.tsx   # project envs + optional link UI
│   ├── architecture-canvas-host.tsx     # placeConfigured + placement mode align
│   ├── architecture-node-card.tsx       # honest status labels; link indicator
│   └── resource-link-picker.tsx         # NEW: compact mock resource list (or inline)
├── create-node-dialog.tsx               # funnel to same configure/place path
├── index.tsx                            # env list injection; detail CTA + role/link
└── node-detail.tsx                      # Link resource action when unlinked

packages/features/resources/src/fixtures.ts   # ensure linkable rows (types/env) — consume only
packages/frontend/canvas/src/document/mutations.ts  # already has owner/linkStatus — verify complete
```

**Structure Decision**: No new packages. Runtime owner = Desktop Renderer feature-architecture. Domain helpers pure under `domain/`. Resource inventory via injected port/fixtures (adapter at ArchitectureView composition). Contracts are UI/document behavioral contracts, not new protobuf.

---

## 0. Preserve / Modify / Replace

### Preserve

| Area | Why |
| --- | --- |
| Catalog flush `AddToArchitectureDialog` | DESIGN reference catalog |
| `docAddNode` / history / autosave | Confirmed commits only |
| DetailPanel global host | LAW — no second inspector |
| Placement origin / center place | Extend, do not rewrite pointer engine |
| Registry `ComponentTypeDefinition` | Type discover SSOT |
| Security boundary | No Node in renderer |

### Modify

| Area | Change |
| --- | --- |
| `place-config` | Project env ids; optional `resourceId`; normalize rules |
| Configure dialog | Project envs; purpose/role/owner; optional link |
| `placeConfigured` | description, owner, metadata.role, linkStatus, resourceId, state planned\|linked |
| Node card labels | Planned / Linked / … honesty (no Healthy for linked) |
| detailNode mapping | env labels, role, link CTA, resource ref |
| CreateNodeDialog | Same flow |
| Connected catalog | Link-from-resources mock + stubs |

### Replace

| Area | With |
| --- | --- |
| Hardcoded-only `PLACE_ENVIRONMENTS` as sole env source | Project environment list (fixtures may match same ids) |
| Linked → “Healthy” card copy | Linked / Observed / Planned honest labels |

### Out of bounds

- Provisioning plans, Target install wizards, live provider import, multipage create wizard, permanent dual create models, fake Observed from form fields

---

## Implementation phases (for `/speckit-tasks`)

| Phase | Deliverable | Spec refs |
| --- | --- | --- |
| **A** | ProjectEnvironmentPort + place-config env normalize + configure Select | FR-006, FR-007 |
| **B** | placeConfigured fields + planned/unlinked honesty + card label fix | FR-013–014, FR-027 |
| **C** | Optional resource link in configure + pure link filters | FR-018–022 |
| **D** | Detail Link resource CTA + detail mapping | FR-019, FR-028 |
| **E** | Connected “Link from Resources” mock + import stubs | FR-004–005, US4 |
| **F** | Placement mode when origin null (if not already) + cancel safety | FR-015–016, US6 |
| **G** | CreateNodeDialog funnel + tests + quickstart validation | FR-033, XII |

Suggested ship order: **A → B → C → D → G**, then **E/F** as polish.

---

## Complexity Tracking

> No constitution violations requiring justification.

| Note | Handling |
| --- | --- |
| Mock inventory vs live resources | Fixture port + `dataSource: "fixture"` honesty |
| Env id casing inconsistency (`production` vs `Production`) | Normalize to stable lowercase ids in document; display labels from port |

---

## Phase 0 / Phase 1 outputs

- [x] [research.md](./research.md) — decisions R1–R8
- [x] [data-model.md](./data-model.md)
- [x] [contracts/](./contracts/)
- [x] [quickstart.md](./quickstart.md)
- [ ] `tasks.md` — via `/speckit-tasks`

**Agent context script**: no `update-agent-context` script present in this repo’s `.specify/scripts`; skipped. Feature recorded in `.specify/feature.json`.
