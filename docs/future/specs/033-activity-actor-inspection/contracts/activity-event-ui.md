# UI Contract: Activity Event Feed + Detail

## Surface

- **Page**: Project → GENERAL → Activity
- **Collection scope**: `activity:index`
- **Entity kind**: `activity-event`
- **Host**: Global DetailPanel via MainSurface

## Collection columns

| Column | Content |
| --- | --- |
| Event | PrimaryCell + ActivityEventIcon(type) + title |
| Actor | actor display name |
| Actor kind | StatusChip (Person / Agent / System) |
| Environment | environment label |
| When | relative timestamp |

Forbidden: Actions column, checkbox selection, batch toolbar.

## Filters

| Control | Values |
| --- | --- |
| Type | architecture, operation, person, agent, sync, access, node, settings |
| Actor | person, agent, system |
| Environment | Production, Staging, Development (+ All if fixtures use it) |
| Search | title, description, actor, target |

Empty: honest DataTable empty message referencing filters/search.

## Detail sections

1. **Header**: event icon + title; optional Open target action
2. **Metrics / Overview**: type, actor, actor kind StatusChip, environment, when
3. **Target**: target name + target kind when present
4. **Description**: event description text
5. **Honesty**: mock-stage read-only note; not-found / loading states

## Non-goals

- No compliance result Success/Denied trail (that is Audit)
- No policy/approval related controls section (Audit owns those)
- No merge with Access Audit page or `audit-event` entity kind

## Copy differentiation

| Activity | Audit |
| --- | --- |
| Operational activity / project feed | Audit trail / governance event |
| Actor kind Person · Agent · System | Person · Agent (compliance actor) |
| Empty: no activity matches filters | Empty: no audit events match filters |
