# Research: Operator Product Surfaces

**Feature**: `004-operator-product-surfaces`  
**Date**: 2026-08-03  
**Status**: Complete — planning unknowns resolved against repo evidence + `spec.md` + `003` artifacts

---

## R1. Primary navigation collapse (many items → seven)

**Decision**: Replace current `DEFAULT_NAV` (Architecture, Resources, Operations, Processes, Logs, Knowledge, Approvals, Workflows, Commander, Agents, Integrations, Settings) with exactly:

| Group | Destinations |
| --- | --- |
| CORE | Architecture, Resources, Operations |
| KNOWLEDGE | Knowledge, Access |
| CONFIGURATION | Integrations, Settings |

Relocations:

| Former top-level | New home |
| --- | --- |
| Processes | Resources → tab **Processes** |
| Approvals | Operations → tab **Approvals** |
| Workflows | Operations → tab **Workflows** |
| Agents | Access → tab **Agents** |
| Commander | **Command palette overlay** (not a destination) |
| Logs | Hand-off from detail/ops quick actions; optional secondary open—not sidebar |

Legacy section ids in deep links or history MUST redirect to the new section+tab.

**Rationale**: Spec FR-001–FR-003; wireframe IA; Constitution XIV single nav SSoT.

**Alternatives considered**: Keep Processes as CORE sibling (rejected — wireframe and “seven destinations”); nested sidebar trees (rejected — subdivisions are tabs).

---

## R2. ClientRoute extension (tabs, entity kinds, drawer)

**Decision**: Extend typed client route (no `@solidjs/router` required):

```ts
type ClientRoute = {
  section: SectionId; // 7 ids
  tab: string;        // section-defaulted, e.g. architecture: "canvas"
  page: "index" | "detail";
  entityId?: string;
  entityKind?: EntityKind; // resource | node | arch-node | run | workflow | approval | document | member | agent | policy | integration
  environmentId?: string;
  search?: string;
  filter?: string;
  dialog?: DialogState | null;
  drawer?: DrawerState | null; // process only in this feature
  selectedNodeId?: string | null; // canvas selection without page change
};
```

History snapshots include `section`, `tab`, `page`, `entityId`, `entityKind`, `environmentId`, `search`, `filter`—exclude transient dialog/drawer unless product later requires drawer in history (default: drawer not in history).

**Rationale**: 003 R4 worked; tabs are first-class in spec; multi-entity detail needs `entityKind` to avoid id collisions across catalogs.

**Alternatives considered**: Separate signal for tab only (weaker deep-link/history); full solid-router (optional later).

---

## R3. Process drawer exclusivity

**Decision**: `DrawerState = { type: "process"; processId: string } | null` only. All other durable entities use full pages. Design-system Drawer is generic; product policy forbids using it as default detail for Resources inventory, Architecture nodes, Runs, etc.

**Rationale**: Spec FR-010; processes are ephemeral.

**Alternatives considered**: Process detail page (rejected — wireframe + ephemeral nature); permanent inspector revival (forbidden by 003/004).

---

## R4. Data strategy: core adapters + fixtures

**Decision**:

| Surface | Prefer core when healthy | Fixture when offline/empty |
| --- | --- | --- |
| Resources inventory | `listResources` + view map | fixture catalog (003 pattern) |
| Architecture | existing layout/map APIs + fixtures | expand fixtures for components/deps/views/changes |
| Operations runs | `OperationService` | fixture runs with steps/output |
| Approvals | `ApprovalService` | fixture pending/approved |
| Workflows | `WorkflowService` if available | fixture workflows |
| Processes | `ProcessService` | fixture process rows |
| Knowledge / Access / Integrations / Settings | wire where protos exist (`knowledge`, `identity`, `policy`, `audit`, `integration`, `secret`, `project`) | fixtures labeled |

Always expose `dataSource: "core" | "fixture" | "empty" | "cached"` to UI for banners. Never present fixture credentials as live secrets.

**Rationale**: Core protos already cover much of the domain (schemas under `local/v1`); UI must ship without blocking on full backend depth; Constitution honesty.

**Alternatives considered**: Block UI until every RPC is perfect (rejected); invent parallel localStorage domain (rejected — dual authority).

---

## R5. Package ownership map

**Decision**:

| Destination | Package |
| --- | --- |
| Shell / route | `packages/features/shell` |
| Architecture | `packages/features/architecture` |
| Resources | `packages/features/resources` (+ process drawer; may import process helpers from `feature-processes` or absorb) |
| Operations | `packages/features/operations` (compose workflows/approvals/terminal views) |
| Knowledge | `packages/features/knowledge` |
| Access | **`packages/features/access` (new)** — Members, Agents, Policies, Audit; re-export or wrap former `feature-agents` |
| Integrations | `packages/features/integrations` |
| Settings | `packages/features/settings` |
| Command palette | `packages/features/commander` as overlay only |
| Composition | `apps/desktop/src/renderer/App.tsx` |

`feature-logs` remains available for log panes opened from actions—not a primary section.

**Rationale**: Constitution XIII real consumers; Access has no home today; avoid scenic empty packages beyond the one justified Access package.

**Alternatives considered**: Stuff Access into Settings (rejected — separate destination); keep Agents as sidebar item (rejected — IA).

---

## R6. Design-system primitives needed

**Decision**: Add/complete in `@polterware/frontend-design-system`:

1. **Tabs** — keyboardable tablist/tabpanel for page subdivisions  
2. **Drawer** — right-side temporary panel (process) with focus trap + Escape  
3. **Dialog** — ensure create/edit/run-command/confirm  
4. **Permission / Offline page states** — extend `feedback.tsx`  
5. Dense **table** utility classes or thin `DataTable` shell (presentation only)

Prefer native + tokens; Kobalte only if a11y gaps block Tabs/Dialog.

**Rationale**: Multiple destinations need identical tab/drawer grammar; avoid per-feature one-offs.

**Alternatives considered**: Copy-paste tab markup per feature (rejected — drift); full data-grid library (overkill).

---

## R7. Architecture multi-tab scope vs canvas

**Decision**: Keep canvas as default Architecture tab. Components/Dependencies are dense tables. Views are card grid of saved views. Changes is chronological feed. Canvas selection stays neutral; Enter/open → detail page. No permanent inspector.

Layout save and view switcher remain canvas-scoped controls.

**Rationale**: Spec US2 + 003 canvas decisions.

**Alternatives considered**: Separate Architecture destinations (rejected — tabs).

---

## R8. Operations mutation UX

**Decision**: UI never “applies” high-risk production commands directly without:

1. Dialog collect intent (run command / run workflow)  
2. Call core **request** path  
3. If approval required → show pending approval / navigate Approvals  
4. Logs/output on Run detail  
5. Audit visible under Access → Audit when events exist  

If core path missing, fixture mode simulates states but labels them as demo—not live success in production claims.

**Rationale**: Constitution VII; spec FR-031/SC-011.

**Alternatives considered**: Instant local shell exec from renderer (forbidden).

---

## R9. Command palette vs Commander page

**Decision**: Remove Commander from sidebar. Reuse `feature-commander` as **CommandPalette** overlay (⌘K / Ctrl+K) listing Navigation (7), Actions (create node, run command, connect integration), Recent entities from route history/fixtures.

**Rationale**: Spec US9; wireframe 8.3.

**Alternatives considered**: Keep Commander page (rejected — not in seven-destination map).

---

## R10. Settings “Project” vs domain System

**Decision**: Settings UI label may say **Project** as transitional alias; view model and comments map to domain **System**. Project ID field is system id display. Danger zone delete is high-friction confirm only; implement real wipe only if core supports it—otherwise disable with honest message.

**Rationale**: Constitution I domain language; Agents.md alias rule.

---

## R11. Secrets and credentials UI

**Decision**: Credentials list shows name, provider, scope, status, rotated time only. Create credential dialog accepts secret once; after submit, value is never bound back into list/detail state. Rotate invalidates old reference via core SecretService when available.

**Rationale**: Spec FR-044; Constitution VI.

---

## R12. DESIGN.md update scope

**Decision**: Do **not** rewrite tokens/identity (owned by 003). Update sections: Navigation model (seven destinations + tabs), Screen blueprints (new pages), Drawer rule, Command palette. Implementation map paths for new/expanded features.

**Rationale**: XIV single design language; avoid dual design work.

---

## Resolved clarifications

No remaining NEEDS CLARIFICATION markers for planning. Open product choices deferred to implement defaults:

- Exact keyboard shortcuts may follow platform conventions if wireframe chords conflict with Electron/macOS.  
- Integration catalog cards may be “available” vs “coming soon” per connector maturity without blocking Access/Operations.
