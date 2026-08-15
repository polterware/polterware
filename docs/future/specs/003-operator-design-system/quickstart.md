# Quickstart Validation Guide

**Feature**: `003-operator-design-system`  
**Scope**: Desktop operator shell, design system, Resources, Architecture  
**Related**: [plan.md](./plan.md) · [spec.md](./spec.md) · [contracts/](./contracts/) · [data-model.md](./data-model.md)

> Visual comparison uses `/prototype` for **composition**, not pixel-perfect equality.  
> Agent policy: do not run long `pnpm dev` unless the user asks; static checks first.

---

## Prerequisites

- Repo root checkout with feature artifacts under `specs/003-operator-design-system/`
- pnpm workspace install already possible
- Optional: polterware-core healthy for live data; fixtures cover offline path

---

## 1. Design SSOT present

```bash
test -f DESIGN.md && wc -l DESIGN.md
```

**Expect**: `DESIGN.md` contains sections: Product identity, Design principles, Application anatomy, Design tokens, Typography, Component specifications, Navigation model, Screen blueprints, Responsiveness, Accessibility, Performance, Implementation map, Acceptance checklist.

---

## 2. Tokens compile into Desktop CSS path

```bash
rg -n "color-brand|surface-selected|app-background|radius-surface" packages/configuration/tailwind/base.css
rg -n "base.css" apps/desktop/src/renderer/styles.css
```

**Expect**: Quiet dark tokens present; desktop imports shared base.

---

## 3. Static quality (changed packages)

From repo root (adjust filters if scripts differ):

```bash
pnpm typecheck
pnpm lint
pnpm test --filter=@polterware/frontend-design-system --filter=@polterware/feature-resources --filter=@polterware/feature-architecture --filter=@polterware/feature-shell --filter=@polterware/frontend-canvas --filter=@polterware/desktop
pnpm build
```

**Expect**: No new failures on in-scope packages; build of desktop succeeds.

> If turbo filters are unavailable, run package-local `typecheck`/`test` scripts under each package path.

---

## 4. Unit smoke (primitives & pure logic)

```bash
pnpm exec vitest run packages/frontend/design-system packages/frontend/canvas packages/features/architecture --passWithNoTests
```

**Expect**: Button/status/dialog/canvas select tests pass when present.

---

## 5. Interactive validation (user-run Desktop)

Start Desktop only when you want a visual pass:

```bash
pnpm dev:desktop
```

### 5.1 Shell anatomy

| Check | Pass criteria |
| --- | --- |
| Sidebar | Integrated with app background; not a floating bordered island |
| Main surface | Single large rounded content host |
| Active nav | Neutral raised surface—not brand green fill |
| Brand green | Only small signals (e.g. status dots) |
| Core status | Honest online/offline in shell |

### 5.2 Resources

| Check | Pass criteria |
| --- | --- |
| Index | Search, filters/scopes, dense table columns |
| Open row | Full **page** detail (breadcrumb/back)—no permanent right inspector |
| Create/Edit | Dialog overlay; cancel leaves index intact |
| Empty/Error/Loading | Distinct states |
| Fixture honesty | If fixtures used, not labeled as live provider sync |

### 5.3 Architecture

| Check | Pass criteria |
| --- | --- |
| Canvas | Full-width in MainSurface; layers/groups if implemented |
| Selection | Neutral emphasis |
| Open node | **Page** detail |
| No permanent inspector | Right column inspector gone from canvas page |
| Pan/zoom | Shell still interactive on prototype-scale map |
| Edges | Non-evidence edges do not look fully “live” |

### 5.4 Accessibility smoke

| Check | Pass criteria |
| --- | --- |
| Keyboard | Tab through nav, search, first row/node, dialog actions |
| Focus-visible | Visible ring/outline on focus |
| Status | Not color-only |
| Reduced motion | OS reduce-motion does not rely on large motion to understand UI |

### 5.5 Composition vs prototype

Compare at ~1280×800, 1440×900, 1512×982, 1728×1117:

- Density, spacing, hierarchy, sidebar vs MainSurface proportion  
- Control size compactness  
- Excess cards / excess green / misaligned columns  

**Not** pixel-equal to Linear or AI mockups.

---

## 6. Acceptance checklist rollup

Map to spec SC-001–SC-010 and DESIGN.md acceptance checklist. Feature is not complete until:

- [ ] DESIGN.md rewritten and matches implemented tokens/anatomy  
- [ ] Shell integrated sidebar + MainSurface  
- [ ] Resources index + detail page + dialog  
- [ ] Architecture canvas + node detail page  
- [ ] No permanent inspector on those screens  
- [ ] Loading/empty/error present  
- [ ] typecheck / lint / test / build evidence captured  
- [ ] Anti-pattern audit clean for in-scope screens  

---

## 7. Rollback notes

If UI regresses operational data wiring: keep `createCoreClient` paths; disable fixtures via flag; shell can fall back to section index without detail routes.
)
