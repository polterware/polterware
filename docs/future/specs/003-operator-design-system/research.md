# Research: Operator Design System & Primary Surfaces

**Feature**: `003-operator-design-system`  
**Date**: 2026-08-02  
**Status**: Complete — planning unknowns resolved against repo evidence + spec

---

## R1. Design document path and dual DESIGN.md content

**Decision**: Treat repository-root **`DESIGN.md`** as the single visual source of truth. On case-insensitive filesystems it is the same inode as `design.md`. Rewrite it in place to the quiet dark operator system (sections required by the product brief). Do **not** maintain a second parallel design language file.

**Rationale**: Existing scripts already reference `design:lint` against `DESIGN.md`. Spec FR-001/FR-026 and Constitution XIV forbid permanent dual systems. Current content documents a light/zinc alpha language that conflicts with `/prototype` and the operator brief.

**Alternatives considered**: Keep light DESIGN.md + new dark `design.md` (rejected — dual SSoT); put SSOT only under `specs/003-...` (rejected — product brief and agents expect root design doc).

---

## R2. Token implementation home

**Decision**: Authoritative CSS tokens live in `packages/configuration/tailwind/base.css` (`@theme` + `:root` semantic aliases). Desktop `apps/desktop/src/renderer/styles.css` continues to import that file only. Component packages consume tokens via Tailwind utilities and CSS variables—no ad-hoc hex in feature views for in-scope surfaces.

**Rationale**: Already the monorepo token shell; Tailwind v4 `@theme` is the established pattern post big-bang.

**Alternatives considered**: Tokens only in DESIGN.md without CSS (not executable); per-feature CSS Modules (forbidden by brief and monorepo practice).

---

## R3. Shell anatomy vs current Shell

**Decision**: Replace provisional `packages/features/shell` top bar + bordered side nav with:

```text
AppWindow
├── WindowChrome (drag region / traffic-light inset awareness)
├── AppSidebar (integrated, same luminance family as app background)
└── MainSurface (single large rounded surface)
    ├── PageHeader
    ├── PageToolbar (optional)
    └── PageContent
```

Active nav = neutral raised surface. Brand green never fills nav active state.

**Rationale**: Spec US1; `/prototype` composition; current Shell is a temporary layout that fails FR-008–FR-010.

**Alternatives considered**: Keep header-top IA (rejected — fights prototype grammar); permanent dual workbench dock + inspector as primary chrome (out of scope for this feature’s Resources/Architecture pages; dock/terminal may remain product capability elsewhere but not as permanent right inspector on these pages).

---

## R4. Navigation / routing model

**Decision**: Introduce a **typed client route model** in Desktop composition (`apps/desktop` + shell), not a full browser multi-app router product. Shape:

- `route.section`: primary nav id (`architecture` | `resources` | …)
- `route.page`: `index` | `detail`
- `route.entityId?: string`
- optional query: search, environment filter, dialog open state

Deep context (Resource detail, Architecture node detail) = **page** within MainSurface. Create/edit = **dialog** state. Prefer pure Solid signals/stores first. Add `@solidjs/router` only if hash/history deep-linking becomes a hard requirement during tasks—not a prerequisite.

**Rationale**: No router exists today; panel signal in `App.tsx` already owns IA. Spec needs page depth without forcing a new dependency. Constitution keeps composition in app shell.

**Alternatives considered**: `@solidjs/router` immediately (acceptable later; extra dep without proven need); URL-less modal-only details (rejected — FR-011).

---

## R5. Permanent inspector removal on Architecture

**Decision**: Remove `ResizableSplit` permanent right Inspector from Architecture index/canvas. Canvas page is full-width MainSurface content. Node depth navigates to Architecture node detail page. Knowledge/ops/logs snippets move to detail page sections or quick actions (deep-link/hand-off), not a fixed inspector column.

Corvu/resizable remains available in design-system for true workbench splits elsewhere; not default Architecture chrome.

**Rationale**: Spec FR-012/FR-013/US3; current `ArchitectureView` violates page-based depth.

**Alternatives considered**: Collapse inspector into drawer (still permanent risk); keep split with hide toggle (fails “no permanent intrusive inspector” default).

---

## R6. Design system primitives strategy

**Decision**:

| Need | Approach |
| --- | --- |
| Simple controls | Native HTML + Tailwind + tokens |
| Button | Restyle existing `Button` to quiet compact variants; remove scale-on-active as default motion gimmick; drop or gate heavy texture material if it fights quiet system |
| Dialog | Evolve `A11yDialog` with focus trap/Escape; adopt Kobalte Dialog only if native path is insufficient |
| Menu / Select / Tooltip / Popover | Add selectively via **@kobalte/core** copy-adapted components under `packages/frontend/design-system` |
| Icons | Add **@hugeicons/core-free-icons** with **per-icon imports** (single icon library) |
| Virtualization | Existing `@tanstack/solid-virtual` for long Resources tables / logs |
| Canvas layout heavy work | Existing `packages/frontend/canvas/worker` |
| Toast | Lightweight design-system toast region (no Sonner/React) |
| Command palette | Optional: thin overlay composing existing Commander patterns; not blocking P1 |

Do **not** run `solidui-cli init` overwriting Tailwind/tokens. Copy-adapt one component at a time.

**Rationale**: Brief + 002 research R5; deps already partial (virtual, canvas worker); Kobalte/Hugeicons not yet in desktop package.json—add only when first consumer lands.

**Alternatives considered**: Full Solid UI kit install (rejected — overwrite risk); multiple icon packs (rejected); React Flow (forbidden).

---

## R7. Resources feature depth

**Decision**: Expand `packages/features/resources` from stub list to:

1. **Index page** — header, environment chip/select, search, filter chips, dense table (name, type, environment, provider, status, last sync), row open → detail
2. **Detail page** — breadcrumb, identity, status chips, overview facts, linked architecture, activity, notes, quick actions
3. **Create/Edit dialog** — focused form; validation errors inline

Data: prefer `createCoreClient().listResources` + related calls when core healthy; **fixture catalog** when empty/offline so visual acceptance and empty/error states are independently demonstrable. Never claim fixture rows are live provider sync without labeling.

**Rationale**: Spec US2; current feature is a minimal stub; core already exposes listResources.

**Alternatives considered**: Backend-only with empty UI when core down (fails SC visual acceptance); invent new Connect services for UI-only fields (rejected — use view-model enrichment client-side).

---

## R8. Architecture canvas depth

**Decision**: Evolve `packages/features/architecture` + `packages/frontend/canvas`:

- Layered/grouped presentation matching prototype grammar (Cloud / Applications / Data bands) as **view layout**, without claiming false evidence on edges
- Neutral selection styling
- Pan/zoom controls; keep shell interactive (no multi-second main-thread layout on prototype-scale graphs)
- Persist positions via existing `getLayout` / `updateLayout`
- Node open → detail page (not inspector)

Evidence rules from 002 remain: dashed/non-evidence edges must not look “live wired.”

**Rationale**: Spec US3; canvas package already owns model serialize/select; worker exists for layout.

**Alternatives considered**: Third-party graph library (rejected unless pure Solid/DOM and justified); keep inspector (rejected).

---

## R9. Brand green and status color mapping

**Decision**:

- `--color-brand: #01DD96` and muted brand wash for rare emphasis only
- Status: semantic `danger` / `warning` / `info` / success-as-status (may use brand or a restrained success token for Healthy **dot + label**, not green chrome)
- Active/selected surfaces: neutral scale only

**Rationale**: Spec FR-004; prototype uses small green status dots, not green nav.

**Alternatives considered**: Green accent on all interactive elements (rejected).

---

## R10. Window chrome / Electron

**Decision**: Renderer shell accounts for macOS traffic lights via CSS padding / `-webkit-app-region` drag on non-interactive chrome strips. Main process window config remains in `apps/desktop/src/main/window.ts`—only adjust if current frame conflicts with integrated sidebar (no redesign of privilege boundary).

**Rationale**: Spec application anatomy; keep Main thin (Constitution III).

---

## R11. Testing and verification gates

**Decision**: Risk-proportional:

- Unit: design-system primitives (Button variants, dialog open/close), canvas select/serialize, resources filter pure functions
- Component/feature tests where vitest+jsdom already used
- Manual/visual: quickstart checklist vs `/prototype` composition + DESIGN.md
- Run typecheck / lint / test / build on changed packages when claiming complete (Agents.md: full suites when verification required for completion claims)

**Rationale**: Constitution XII; feature is presentation-heavy with low new domain mutation surface.

---

## R12. Scope of other nav destinations

**Decision**: Shell may list Overview, Operations, Knowledge, Integrations, Policies, Settings, etc. Non-P1 destinations may remain honest stubs or existing lightweight views restyled to shell tokens—**full product depth not required** this feature except Resources + Architecture.

**Rationale**: Spec scope boundaries; avoid scenic fake dashboards.

---

## R13. Agent context update script

**Decision**: **Skip** — repository has no Spec Kit `update-agent-context` script under `.specify/scripts`. Feature plan + DESIGN.md rewrite are the durable agent-facing artifacts; `AGENTS.md` already points to DESIGN.md.

**Rationale**: Skill step is best-effort when tooling exists; inventing a script is out of scope.

---

## R14. Relationship to 002 big-bang

**Decision**: This feature is a **presentation/IA completion** on the Solid Desktop path established by 002. It does not reopen backend language, contracts pipeline, or dual-renderer debates. It **does** remove UI patterns that regressed toward permanent inspectors and provisional chrome.

**Rationale**: Constitution XV/XIV; stay on authorized architecture.
)
