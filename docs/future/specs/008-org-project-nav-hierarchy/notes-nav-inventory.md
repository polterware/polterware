# Nav inventory (008)

## Baseline (before implement)

| Path | Behavior |
| --- | --- |
| Org breadcrumb | Single button name+chevron → switcher menu |
| `onManageOrganizations` | `settings` + tab `organization` |
| Select org in menu | Auto-enter first project |
| Sidebar | Always shown |
| Org catalog | Nested under project Settings |

## Target (after implement)

| Path | Behavior |
| --- | --- |
| Org name | → `organization` home |
| Org chevron | Menu only |
| All organizations | `organizations` root |
| Select org in menu | → org home (no auto project) |
| Hierarchy sections | No project sidebar |
| Settings → Organization | Admin config only + link to root |

## Package map

- Hierarchy pages: `packages/features/shell` only (no scenic package)
