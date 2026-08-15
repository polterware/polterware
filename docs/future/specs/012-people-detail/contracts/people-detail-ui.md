# Contract: People (Member) Detail UI

**Owner**: `packages/features/access`  
**Consumers**: `AccessView` (Access → People tab)  
**Host**: global `DetailPanel` via `MainSurface`  
**Data (mock stage)**: `@polterware/feature-shell` workspace mocks (`WorkspacePerson`)

## Scope

Production-ready **human member** detail under Access → People. Mocked only.

Out of scope: live membership APIs, IdP, real session kill, Agents detail, Invitation/Team detail redesign, Roles matrix editor.

## Activation

```ts
scopeId: "access:people"
entityKind: "member"
entityId: person.id
```

## Props

```ts
type MemberDetailProps = {
  member: WorkspacePerson | null | undefined;
  teams?: readonly WorkspaceTeam[]; // resolved team rows for member.teamIds
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onChangeRole?: () => void;
  onAddToTeam?: () => void;
  onRevokeAccess?: () => void;
  onRevokeSession?: (sessionId: string) => void;
  onOpenTeam?: (teamId: string) => void;
};
```

## Layout (LAW)

- `DetailPanel.Content`
- `DetailPanel.Header` + identity `User` icon (16px) + `DetailPanel.Actions` / `DetailPanel.Action`
- `DetailPanel.Body layout="sections"`
- `DetailPanel.Grid` + `Section` / `Metric` / `Metadata` — no `Card`, no second scroller
- Status / presence / permission effect / session current → `StatusChip` (filled)
- Collections → compound `DataTable` (no Actions column)

## Sections

| Section | Content |
| --- | --- |
| Metrics | Presence label, Role, Team count, Environment count (optional access-scope visualization) |
| Overview | Email, role, presence, joined, invited by, auth method, MFA, last activity, last seen |
| Environments | Environment scope list or metadata |
| Permissions | Capability + effect (`allow` \| `deny` \| `request-only`) |
| Teams | Team name + optional on-call / owned; activate opens team detail when provided |
| Sessions | Device/client, location/origin, last active, current; mock revoke |
| Activity | Event title + when (+ optional result StatusChip) |
| Notes | Optional free-text when present |

## Header actions

| Action | Tone | Mock behavior |
| --- | --- | --- |
| Change role | default | toast success |
| Add to team | default | toast default |
| Revoke access | danger | toast success |

## States

| State | UI |
| --- | --- |
| loading | Header “Loading member” + `PageLoading` |
| error | Header “Failed to load” + `PageError` |
| notFound / null | Header “Member not found” + `PageError` |
| ready | Full sections |

## Fixture requirements

`WorkspacePerson` MUST include (mock stage):

- identity: `id`, `name`, `email`, `role`, `status`, `teamIds`, `teams`, `environments`, `lastActivity`, `lastSeen`
- posture: `joinedAt`, `invitedBy?`, `authMethod`, `mfaEnabled`
- collections: `permissions[]`, `sessions[]`, `activity[]`
- optional: `title?`, `notes?`

Primary Admin fixture MUST have non-empty `permissions`, `sessions`, and `activity`.

## Security honesty

- Permission table is **display only** — not authorization.
- Mock mutations MUST NOT claim live core enforcement.
- No secrets, tokens, or credential material in detail DTOs.
