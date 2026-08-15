# Nav inventory (007 T001–T003)

**Date**: 2026-08-03

## Baseline (004)

| Item | Location | State |
| --- | --- | --- |
| DEFAULT_NAV | `packages/features/shell/src/index.tsx` | 7 items: CORE / KNOWLEDGE / CONFIGURATION |
| SectionId | `packages/features/shell/src/route.ts` | architecture, resources, ops, knowledge, access, integrations, settings |
| App Switch | `apps/desktop/src/renderer/App.tsx` | Match per 7 sections only |
| DrawerState | route.ts | process only |

## Target (007)

| Item | Target |
| --- | --- |
| DEFAULT_NAV | 8 items; group **INSIGHTS** (Observability, Knowledge, Access) |
| Nav sections | + observability |
| Chrome sections | overview, search, sync, onboarding (routable, not in nav) |
| DrawerState | process \| notifications |
| New package | `packages/features/observability` only |
| Overview / switcher / notifications | shell |
| Sync center page | integrations |
| Search results | commander |

## Feature depth snapshot (T002)

| Package | Present | 007 gaps |
| --- | --- | --- |
| architecture | tabs canvas…changes, canvas, node detail | import, compare, view editor |
| resources | inventory, nodes, processes, drawer | provisioning, node wizard |
| operations | runs, workflows, approvals, terminal | builder, schedules strip |
| access | members, agents, policies, audit | teams, invitations, roles, audit/session detail |
| integrations | connected, catalog, credentials, sync | webhooks, setup wizard, conflict resolve, sync center |
| settings | project, env, notifications, core, account | org, developer, billing, security, data |
| commander | palette | search results page |
| shell | 7-nav | 8-nav, overview, switcher, notifications, onboarding |

## Package policy (T003)

Only new package: `@polterware/feature-observability`. No scenic `features/sync` or `features/overview`.
