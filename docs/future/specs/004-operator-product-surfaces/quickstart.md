# Quickstart Validation Guide

**Feature**: `004-operator-product-surfaces`  
**Scope**: Seven-destination operator IA and all tabbed product surfaces  
**Related**: [plan.md](./plan.md) · [spec.md](./spec.md) · [contracts/](./contracts/) · [data-model.md](./data-model.md)

> Visual language follows `DESIGN.md` / `003`. Prototypes guide composition only.  
> Agent policy: prefer static checks; do not run long `pnpm dev` unless the user asks.

---

## Prerequisites

- Repo checkout with artifacts under `specs/004-operator-product-surfaces/`
- `003` shell/tokens/Resources/Architecture foundation present
- Optional: polterware-core healthy; fixtures cover offline paths

---

## 1. Spec & plan artifacts present

```bash
test -f specs/004-operator-product-surfaces/spec.md
test -f specs/004-operator-product-surfaces/plan.md
test -f specs/004-operator-product-surfaces/research.md
test -f specs/004-operator-product-surfaces/data-model.md
test -f specs/004-operator-product-surfaces/contracts/overview.md
test -f specs/004-operator-product-surfaces/quickstart.md
```

**Expect**: All files exist.

---

## 2. Navigation IA static checks (after implement)

```bash
rg -n "DEFAULT_NAV|group: \"CORE\"|group: \"KNOWLEDGE\"|group: \"CONFIGURATION\"" packages/features/shell/src
rg -n "tab:|entityKind|DrawerState|arch-node-create|run-command" packages/features/shell/src/route.ts
```

**Expect**:

- Exactly seven nav destinations in DEFAULT_NAV  
- Groups CORE / KNOWLEDGE / CONFIGURATION  
- Route supports tab, entityKind, drawer  

**Anti-expect**: Sidebar still listing Processes, Logs, Approvals, Workflows, Commander, Agents as primary items after cutover.

---

## 3. Package presence

```bash
test -d packages/features/access   # created when Access ships
test -d packages/features/operations
test -d packages/features/resources
test -d packages/features/architecture
```

**Expect**: Access package exists once Phase F lands; other destinations expanded in place.

---

## 4. Static quality (changed packages)

From repo root (adjust filters if scripts differ):

```bash
pnpm typecheck
pnpm test --filter=@polterware/feature-shell --filter=@polterware/feature-architecture --filter=@polterware/feature-resources --filter=@polterware/feature-operations --filter=@polterware/feature-knowledge --filter=@polterware/feature-access --filter=@polterware/feature-integrations --filter=@polterware/feature-settings --filter=@polterware/frontend-design-system --filter=@polterware/desktop
```

**Expect**: No new failures on in-scope packages.

> Full `pnpm build` / `pnpm dev:desktop` only when the user requests completion claims that need them.

---

## 5. Interactive validation (user-run Desktop)

```bash
pnpm dev:desktop
```

### 5.1 Shell IA

| Check | Pass |
| --- | --- |
| Sidebar groups | CORE, KNOWLEDGE, CONFIGURATION |
| Destination count | Exactly 7 |
| History | Back/forward across section + detail |
| Status | Core online/offline honest; node count when available |
| Active nav | Neutral raised, not brand-green fill |

### 5.2 Architecture

| Check | Pass |
| --- | --- |
| Tabs | Canvas, Components, Dependencies, Views, Changes |
| Canvas select | Neutral border only |
| Open node | Full detail page |
| Create node | Focused dialog |

### 5.3 Resources

| Check | Pass |
| --- | --- |
| Tabs | Inventory, Nodes, Processes |
| Resource/node | Full detail pages |
| Process | Temporary drawer only; dismiss clean |

### 5.4 Operations

| Check | Pass |
| --- | --- |
| Tabs | Runs, Workflows, Approvals, Terminal |
| Run detail | Steps + output |
| Run command | Dialog; approval messaging when required |
| Terminal | Multi-session chrome |

### 5.5 Knowledge / Access / Integrations / Settings

| Check | Pass |
| --- | --- |
| Each destination | Documented tabs present |
| Detail pages | Open for primary entities |
| Credentials | No secret values after create |
| Core settings | Offline honesty |

### 5.6 Overlays & states

| Check | Pass |
| --- | --- |
| Command palette | Opens overlay; jumps to all 7 destinations |
| Empty / loading / offline / forbidden | Distinguishable on at least one surface per group |

---

## 6. Security smoke

- Open Integrations → Credentials: no raw secret column/value  
- Attempt forbidden action (if role fixture exists): permission state, not silent apply  
- Approval detail: Approve/Reject visible only when permitted in UI model  

---

## 7. Acceptance mapping (spec SC)

| SC | How to verify |
| --- | --- |
| SC-001 | Sidebar audit = 7 destinations |
| SC-002 | Tab switch per destination |
| SC-003 | Arch/Resources detail round-trip < 30s demo |
| SC-004 | Ops run + approval path < 45s demo |
| SC-005 | Process = drawer only |
| SC-006 | System states per group |
| SC-007 | Credential secret smoke |
| SC-008 | Keyboard a11y smoke CORE |
| SC-009 | List scroll shell interactive |
| SC-010 | DESIGN consistency review |
| SC-011 | High-risk mutation gates |
| SC-012 | Palette → 7 destinations |

---

## Done for implementers

When quickstart sections 1–2 pass statically and 5.x pass interactively for the claimed slice, that slice may be marked complete in `tasks.md`. Full feature complete only when all destinations meet contracts and SC checklist.
