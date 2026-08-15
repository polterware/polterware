# Contract: Integration Detail UI

**Owner**: `packages/features/integrations`  
**Consumers**: `IntegrationsView` via global `DetailPanel` host in Desktop shell  
**Related**: `specs/004-operator-product-surfaces/contracts/integrations-ui.md` (list tabs); this contract is the detail SSOT.

## Hosting

- Entity detail **only** in global `DetailPanel` (`DetailPanel.Content` contributed by the feature).
- **Forbidden**: dedicated integration detail route/page, second inspector, modal-as-detail, drawer-as-detail.

## Page vs detail sections

| Level | Tabs / sections |
| --- | --- |
| Integrations **page** | `connected` · `catalog` |
| Integration **detail** (local) | `overview` · `resources` · `credentials` · `sync` · `permissions` · `webhooks` · `advanced` |

Detail sections are **not** page tabs and MUST NOT appear in the project sidebar or Integrations PageToolbar as primary destinations.

## Layout

- `DetailPanel.Body layout="sections"`.
- Shared design-system `Tabs` rail **may** sit directly under the header before `DetailPanel.Grid` (DESIGN Integration exception).
- Sectional content uses `DetailPanel.Grid` / `Section` / `Metric` / `Metadata` — no feature `Card` chrome for operational sections.
- Status / health / result → `StatusChip` only.

## Overview

MUST show:

- Provider and account
- Health/status (`StatusChip`)
- Environment(s) affected
- Last successful sync (or Never)
- Resource summary / counts
- Scopes or capabilities summary
- Error/stale reason when present

## Resources

- DataTable of imported resources for the active integration.
- Columns: name (PrimaryCell + kind icon), kind, environment, status (StatusChip).
- Empty state when none.
- Optional row activate → resource hand-off callback.

## Credentials

- DataTable or metadata list of **references**: name, scope, status, rotated.
- **Never** render secret material after create.
- Create dialog: write-only secret; clear on submit/cancel.

## Sync

- Last sync, interval, imported counts.
- Optional runs table: result StatusChip (success / partial / failed / running).
- Optional rate-limit / backoff notice.
- Sync now action (header and/or section) — mock-honest when fixture-backed.

## Permissions

- Capability rows: name + Allowed/Denied (StatusChip or clear text + chip).
- Optional coverage bar only when total > 0.

## Webhooks

- Per-integration endpoints: name, destination host, status, last call.
- Signing secret: Configured | Missing only.
- Failed deliveries use non-healthy tone.
- Empty state when none.

## Advanced + header actions

| Action | Notes |
| --- | --- |
| Sync now | Mock or real later; never silent |
| Reconfigure / Reconnect | Especially for error/auth failure |
| Enable / Disable | Local mock OK this delivery |
| Open provider console | External hand-off only |
| Disconnect | Copy MUST state external resources are not destroyed |

## Mock honesty

- Fixture / mock dataSource is allowed.
- MUST NOT claim live OAuth, live secret vault, or live provider health polling when using mocks.
- Partial/failed sync and webhook failure MUST NOT render as full healthy.

## A11y

- Detail section Tabs have `aria-label="Integration detail sections"`.
- Tables have descriptive `aria-label`s.
- Header actions are icon-only `DetailPanel.Action` with accessible labels.

## Module surface (feature package)

| Export | Role |
| --- | --- |
| `IntegrationsView` | Page: Connected/Catalog + collection register + credential dialog + wires detail |
| `IntegrationDetail` | `DetailPanel.Content` composition for one integration |
| `SetupWizard` | Connect/reconfigure mock flow (full-page within Integrations) |
| `WebhooksTab` | Secondary webhook list (non-entity-detail); entity webhooks live in `IntegrationDetail` |

### `IntegrationDetail` props (normative)

| Prop | Required | Notes |
| --- | --- | --- |
| `integration` | yes* | DTO or null when notFound |
| `status` | no | Effective status after mock overlays |
| `loading` / `error` / `notFound` | no | Honest panel states |
| `disabled` | no | Enable/Disable affordance |
| `onSync` | no | Mock/live sync trigger |
| `onReconfigure` | no | Opens setup / reconnect |
| `onToggleEnabled` | no | Disable/enable |
| `onDisconnect` | no | Non-destructive disconnect |
| `onOpenCredentialDialog` | no | Write-only create |
| `onOpenResource` | no | Inventory hand-off id |
| `onHandoff` | no | Console hand-off when URL missing |

\* When not loading and not notFound, integration MUST be present for full section body.

## Collection registration

- Scope id: `integrations:connected` when Connected tab is active and setup wizard is closed.
- Targets: one per connected fixture row; `entityKind: "integration"`.
- Catalog tab registers empty targets (no fake detail selection from catalog alone).
- Opening a catalog provider with `availability: "connected"` MUST `requestTarget` the matching integration and switch to Connected.

## Supersedes

For entity detail, this contract supersedes the “Integration detail page” wording in `specs/004-operator-product-surfaces/contracts/integrations-ui.md`. List tabs Connected/Catalog remain; Credentials/Sync as **page** tabs are obsolete (moved into detail sections).
