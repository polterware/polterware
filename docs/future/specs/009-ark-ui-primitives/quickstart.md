# Quickstart Validation: Ark UI–Backed Interactive Primitives

**Feature**: `009-ark-ui-primitives`  
**Date**: 2026-08-03  

Prefer static checks + focused tests; Desktop manual only if Erick asks (`pnpm dev` forbidden for agents by default).

---

## Prerequisites

- Spec: [spec.md](./spec.md) · Contracts: [contracts/](./contracts/) · Plan: [plan.md](./plan.md)

---

## 1. Static inventory (required)

```bash
# Kobalte must be gone from product code when feature is done
rg -n '@kobalte/core' packages apps/desktop --glob '!**/node_modules/**'

# Features must not import Ark directly when wrappers exist
rg -n '@ark-ui/solid' packages/features apps/desktop/src --glob '!**/node_modules/**'

# Design-system may use Ark
rg -n '@ark-ui/solid' packages/frontend/design-system/src

# Feature-local Escape / modal shells
rg -n 'document\.addEventListener\([\"'\'']keydown' packages/features packages/frontend/design-system/src
rg -n 'aria-modal="true"|role="dialog"' packages/features --glob '*.tsx'
```

**Expected (done)**:

| Check | Pass |
| --- | --- |
| Kobalte imports | empty in packages + desktop |
| Feature Ark imports | empty |
| DS Ark imports | present for Dialog/Popover/Tabs/Drawer/Menu as migrated |
| Commander/terminal | use design-system Dialog/Drawer; no own Escape |
| package.json DS | has `@ark-ui/solid`; no `@kobalte/core` |

---

## 2. Package tests / typecheck

```bash
pnpm --filter @polterware/frontend-design-system exec vitest run --passWithNoTests 2>/dev/null \
  || pnpm exec vitest run packages/frontend/design-system --passWithNoTests

pnpm --filter @polterware/frontend-design-system typecheck
pnpm --filter @polterware/feature-shell typecheck
pnpm --filter @polterware/feature-commander typecheck 2>/dev/null || true
```

**Expected**: tests pass; typecheck clean for changed packages.

---

## 3. Manual keyboard + visual (Desktop when available)

### Behavior

| Scenario | Pass |
| --- | --- |
| Product dialog Escape + focus return | ✓ |
| Command palette Escape + product frame size | ✓ |
| Terminal drawer Escape/scrim | ✓ |
| Org/project Popover Escape/outside | ✓ |
| Section Tabs arrows | ✓ |
| Compact DropdownMenu (if shipped) keyboard | ✓ |

### Visual (must match DESIGN / pre-migration Polter)

| Check | Pass |
| --- | --- |
| No Ark docs / Chakra-like chrome | ✓ |
| surface-panel-float / control-field / menu-panel unchanged family | ✓ |
| Dialog one-size frame | ✓ |
| Hover/focus/selected/disabled look project-owned | ✓ |

---

## 4. Acceptance mapping

| SC | Proof |
| --- | --- |
| SC-001–004 | Manual behavior + residual grep |
| SC-005 | disposition notes complete |
| SC-006 | visual §3 |
| SC-007 | AGENTS points to local wrappers |
| SC-008 | Kobalte rg empty + package.json |
| SC-009–010 | API stable + states work |

---

## 5. Done criteria checklist

- [ ] Applicable interactive locals use Ark under the hood  
- [ ] Zero maintained `@kobalte/core` imports; Kobalte dep removed  
- [ ] Public APIs preserved where practical  
- [ ] Visual consistent with DESIGN.md  
- [ ] Keyboard + focus restore validated  
- [ ] Dialogs/menus/popovers/tooltips (as shipped) work  
- [ ] Typecheck/tests for changed surfaces pass  
- [ ] Not “done” from compile-only  
