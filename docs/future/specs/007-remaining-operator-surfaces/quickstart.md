# Quickstart Validation Guide

**Feature**: `007-remaining-operator-surfaces`  
**Scope**: Remaining operator surfaces — shell IA v2, chrome, observability, provisioning, access/settings expansions  
**Related**: [plan.md](./plan.md) · [spec.md](./spec.md) · [contracts/](./contracts/) · [data-model.md](./data-model.md)

> Visual language follows `DESIGN.md` / `003`–`004`.  
> Agent policy: prefer static checks; do not run long `pnpm dev` unless the user asks.

---

## Prerequisites

- Repo checkout with artifacts under `specs/007-remaining-operator-surfaces/`
- `004` shell/tabs/process drawer/palette foundation present (or in progress)
- Optional: polterware-core healthy; fixtures cover offline/unavailable paths

---

## 1. Spec & plan artifacts present

```bash
test -f specs/007-remaining-operator-surfaces/spec.md
test -f specs/007-remaining-operator-surfaces/plan.md
test -f specs/007-remaining-operator-surfaces/research.md
test -f specs/007-remaining-operator-surfaces/data-model.md
test -f specs/007-remaining-operator-surfaces/quickstart.md
test -f specs/007-remaining-operator-surfaces/contracts/overview.md
test -f specs/007-remaining-operator-surfaces/contracts/shell-ia.md
test -f specs/007-remaining-operator-surfaces/contracts/global-chrome.md
test -f specs/007-remaining-operator-surfaces/contracts/observability-ui.md
test -f specs/007-remaining-operator-surfaces/contracts/provisioning-ui.md
```

**Expect**: All files exist.

---

## 2. Navigation IA static checks (after implement)

```bash
rg -n "DEFAULT_NAV|INSIGHTS|observability" packages/features/shell/src
rg -n "overview|search|sync|onboarding" packages/features/shell/src/route.ts
rg -n "notifications|DrawerState" packages/features/shell/src/route.ts
```

**Expect**:

- Exactly **eight** nav destinations in `DEFAULT_NAV`
- Groups **CORE** / **INSIGHTS** / **CONFIGURATION**
- `observability` present; group label not `KNOWLEDGE`
- Chrome sections `overview`, `search`, `sync`, `onboarding` routable
- Drawer supports `process` and `notifications`

**Anti-expect**:

- Billing, Webhooks, Provisioning, Notifications as permanent sidebar items
- Dual seven-destination nav left enabled after cutover

---

## 3. Package presence

```bash
test -d packages/features/observability   # created when Observability ships
test -d packages/features/shell
test -d packages/features/resources
test -d packages/features/access
test -d packages/features/integrations
test -d packages/features/settings
test -d packages/features/operations
```

**Expect**: observability package exists once Phase D lands; others already present.

---

## 4. Route / unit checks (when tests exist)

```bash
# From package roots as implemented, e.g.:
# pnpm --filter @polterware/feature-shell test
# pnpm --filter @polterware/feature-observability test
```

**Expect** (targeted):

- `resolveSection("logs")` → observability + logs (007 mapping)
- `goSection` to overview clears entity detail
- Default tabs include provisioning, webhooks, teams, invitations, observability tabs
- History excludes notification drawer open state

> Prefer package-level vitest; do not run full monorepo suite unless asked.

---

## 5. Manual Desktop paths (when user runs UI)

Agent does not start `pnpm dev` unless requested. When validating interactively:

| # | Path | Expect |
| --- | --- | --- |
| 1 | Sidebar groups | CORE / INSIGHTS / CONFIGURATION; Observability visible |
| 2 | Click Polter logo | Project Overview with status / attention / activity or empty/offline |
| 3 | Project switcher chevron | Popover; search; current project check |
| 4 | Bell | Notification drawer; filters; mark all read; Escape closes |
| 5 | ⌘K → query → Show all results | Search page with category chips |
| 6 | Footer local core status | Sync center Status tab |
| 7 | Observability tabs | Overview → Logs → Metrics → Traces → Monitors |
| 8 | Open a trace | Waterfall detail; span selection |
| 9 | Resources → Provisioning | Catalog; open template configure → plan review |
| 10 | Provisioning → Drift | List + Accept/Restore/Ignore controls |
| 11 | Add node wizard | Install command + waiting heartbeat |
| 12 | Architecture import / compare / edit view | Wizards and editor reachable |
| 13 | Workflows → edit + schedules strip | Builder + schedules actions |
| 14 | Access Teams / Invitations | List + detail; invite resend/revoke |
| 15 | Integrations Webhooks + Connect wizard | List + multi-step connect |
| 16 | Settings Org/Developer/Billing/Security/Data | Reachable; Billing unavailable OK for local-only |
| 17 | Onboarding (no project) | Create org/project local-first |

---

## 6. Honesty & security smoke

```bash
rg -n "dataSource|Demo data|unavailable" packages/features/observability packages/features/settings packages/features/shell 2>/dev/null | head
rg -n "signingSecret|token|password" packages/features/integrations packages/features/resources packages/features/settings 2>/dev/null | head
```

**Expect**:

- Fixture/unavailable labels present for demo paths  
- No secret re-display fields after create (webhook secret, API tokens, credentials)  
- Install token only during active registration session  

---

## 7. Accessibility smoke (manual)

- Keyboard: open project switcher, notification drawer, one multi-step wizard end-to-end  
- Focus visible; Escape closes overlays  
- Status not color-only (text/icon)  

---

## Done criteria for implementers

- [ ] IA static checks pass (eight destinations, INSIGHTS)
- [ ] Contracts-covered surfaces reachable without new sidebar items beyond Observability
- [ ] Quickstart manual table paths pass or blocked only by explicit missing core with honest empty states
- [ ] No dual seven/eight nav left in tree
- [ ] DESIGN.md navigation model updated to match
