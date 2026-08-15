# Contract: Teams Detail UI

**Owner**: `packages/features/access`  
**Consumers**: `AccessView` (Access → People tab, Teams table)  
**Host**: global `DetailPanel` via `MainSurface`  
**Data (mock stage)**: `@polterware/feature-shell` workspace mocks (`WorkspaceTeam`)

## Scope

Production-ready **team** detail under Access → People (Teams collection). Mocked only.

Out of scope: live membership APIs, org-wide directory product, real archive/mutation APIs, Agents detail, Invitation redesign, Roles matrix editor.

## Activation

```ts
scopeId: "access:people"
entityKind: "team"
entityId: team.id
```

## Props

```ts
type TeamDetailProps = {
  team: WorkspaceTeam | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onEditTeam?: () => void;
  onAddMember?: () => void;
  onArchiveTeam?: () => void;
  onOpenMember?: (memberId: string) => void;
};
```

## Layout (LAW)

- `DetailPanel.Content`
- `DetailPanel.Header` + identity `AddTeam` icon (16px) + `DetailPanel.Actions` / `DetailPanel.Action`
- `DetailPanel.Body layout="sections"`
- `DetailPanel.Grid` + `Section` / `Metric` / `Metadata` — no `Card`, no second scroller
- Status / presence / policy status / activity result → `StatusChip` (filled)
- Collections → compound `DataTable` (no Actions column)

## Sections

| Section | Content |
| --- | --- |
| Metrics | Status, Members count, Owned count, On-call |
| Overview | Description, lead, on-call, escalation, schedule, created, updated |
| Environments | Environment scope list or empty copy |
| Members | Person name + role + presence; activate opens member detail when id known |
| Owned resources | Resource name + kind + environment |
| Policies | Policy name + enforcement + status |
| Activity | Event title + when (+ optional result StatusChip) |
| Notes | Optional free-text when present |

## Header actions

| Action | Tone | Mock behavior |
| --- | --- | --- |
| Edit team | default | toast success |
| Add member | default | toast default |
| Archive team | danger | toast success |

## States

| State | UI |
| --- | --- |
| loading | Header “Loading team” + `PageLoading` |
| error | Header “Failed to load” + `PageError` |
| notFound / null | Header “Team not found” + `PageError` |
| ready | Full sections |

## Fixture requirements

`WorkspaceTeam` MUST include (mock stage):

- identity/list: `id`, `name`, `members` (count), `owned` (count), `onCall`, `updated`
- posture: `description`, `status` (`active` \| `archived`), `leadName`, `leadId?`, `environments[]`, `escalation`, `schedule?`, `createdAt`
- collections: `memberRoster[]`, `ownedResources[]`, `policies[]`, `activity[]`
- optional: `purpose?`, `notes?`

Primary Platform Team fixture (`t1`) MUST have non-empty `memberRoster`, `ownedResources`, `policies`, and `activity`.

List counts MUST match collection lengths for primary fixtures:

- `members === memberRoster.length`
- `owned === ownedResources.length`

## Security honesty

- Policy table and membership UI are **display only** — not authorization.
- Mock mutations MUST NOT claim live core enforcement.
- No secrets, tokens, or credential material in detail DTOs.

## Cross-navigation

- Team member row activate → `onOpenMember(memberId)` → Access people member detail when person fixture exists.
- People member Teams table activate → existing `onOpenTeam(teamId)` path remains.
