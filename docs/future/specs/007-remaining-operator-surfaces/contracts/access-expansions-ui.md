# Contract: Access Expansions UI

**Owner**: `packages/features/access`  
**Extends**: 004 Access members/agents/policies/audit

## Tabs

`members` | **`teams`** | `agents` | `policies` | `audit` | **`invitations`**

## Teams

```ts
type TeamsIndexProps = {
  teams: Team[];
  onOpen: (teamId: string) => void;
  onCreate: () => void;
};

type TeamDetailProps = {
  team: Team;
  members: { id: string; name: string; role: string; presence: string }[];
  ownedSystems: { id: string; name: string; type: string }[];
  defaultEscalationName?: string;
  onEdit: () => void;
  onChangeEscalation: () => void;
};
```

## Invitations

```ts
type InvitationsProps = {
  items: Invitation[];
  onInvite: () => void;
  onResend: (ids: string[]) => void;
  onRevoke: (ids: string[]) => void;
};
```

## Roles matrix

```ts
type RolesMatrixProps = {
  roles: string[]; // Admin, Operator, Viewer, Agent, …
  permissions: string[];
  cells: Record<string, Record<string, "allow" | "deny" | "request-only">>;
  onCreateRole?: () => void;
  onEditCustom?: () => void;
};
```

Reachable from Policies (button or sub-route). Agent column supports `request-only`.

## Audit event detail

```ts
type AuditEventDetailProps = {
  event: AuditEventDetail;
  onExport?: () => void;
};
```

Must show actor, action, target, time, source, request id, changes, integrity signals when available.

## Agent session detail

```ts
type AgentSessionDetailProps = {
  session: AgentSessionDetail;
  onTerminate?: () => void;
  onExport?: () => void;
  onOpenApproval?: () => void;
};
```

Terminate requires confirmation when active.
