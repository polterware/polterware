# Contract: Policies Detail UI

**Owner**: `packages/features/access`  
**Consumers**: `AccessView` (Access → Policies tab)  
**Host**: global `DetailPanel` via `MainSurface`  
**Data (mock stage)**: `@polterware/feature-shell` workspace mocks (`WorkspacePolicy`, filtered `WorkspaceAuditEvent`)

## Scope

Production-ready **policy** detail under Access → Policies. Mocked only.

Out of scope: live policy engine, real enable/delete APIs, Agents detail, Audit detail composition, Agent session detail, rewrite of Agents/Audit/Sessions collections.

## Activation

```ts
scopeId: "access:policies"
entityKind: "policy"
entityId: policy.id
```

## Props

```ts
type PolicyRelatedAuditEvent = {
  id: string;
  actor: string;
  actorKind: "person" | "agent";
  action: string;
  entity: string;
  result: "Success" | "Pending" | "Failed" | "Denied";
  at: string;
  environment?: string;
};

type PolicyDetailProps = {
  policy: WorkspacePolicy | null | undefined;
  relatedAuditEvents?: readonly PolicyRelatedAuditEvent[];
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onEdit?: () => void;
  onDuplicate?: () => void;
  onToggleEnabled?: () => void;
  onDelete?: () => void;
  onOpenAudit?: (eventId: string) => void;
};
```

## Layout (LAW)

- `DetailPanel.Content`
- `DetailPanel.Header` + identity `ShieldCheck` icon (16px) + `DetailPanel.Actions` / `DetailPanel.Action`
- `DetailPanel.Body layout="sections"`
- `DetailPanel.Grid` + `Section` / `Metric` / `Metadata` — no `Card`, no second scroller
- Status / risk / result / approval posture → `StatusChip` (filled)
- Collections → compound `DataTable` (no Actions column)
- Optional action-balance chart (`DitherSegmentedBar`) may render when total allowed + blocked actions > 0 (zero-count segments allowed for empty side)

## Sections

| Section | Content |
| --- | --- |
| Metrics | Status StatusChip, Scope, Enforcement, Requires approval |
| Description | Policy description text |
| Actors | Kind + name (person / agent / role); empty copy when none |
| Environments | Environment scope list or empty copy |
| Allowed actions | Mono action identifiers DataTable |
| Blocked actions | Mono action identifiers DataTable |
| Approval & risk | Risk posture StatusChip, dual approval, approval requirement summary |
| Update history | updated + updatedBy metadata; optional activity trail DataTable |
| Related audit | Read-only audit rows filtered by policy; activate → `onOpenAudit` |
| Notes | Optional free-text when present |

## Header actions

| Action | Tone | Mock behavior |
| --- | --- | --- |
| Edit | default | toast default/success labeled mock |
| Duplicate | default | toast success mock |
| Disable (when Active) / Enable (when Disabled) | default | toast success mock |
| Delete | danger | toast success mock |

## States

| State | UI |
| --- | --- |
| loading | Header “Loading policy” + `PageLoading` |
| error | Header “Failed to load” + `PageError` |
| notFound / null | Header “Policy not found” + `PageError` |
| ready | Full sections |

## Fixture requirements

`WorkspacePolicy` MUST include (mock stage):

- identity/list: `id`, `name`, `scope`, `status`, `requiresApproval`, `updated`
- posture: `description`, `enforcement`, `environments[]`, `actors[]`, `allowedActions[]`, `blockedActions[]`, `updatedBy`
- detail richness (primary `p1`): `actorEntries[]`, `riskPosture`, `approvalRequirement`, `activity[]` (optional but preferred non-empty for p1)

Related audit for primary `p1` MUST resolve from existing audit fixtures (do not invent a second audit store).

## Security honesty

- Policy tables and chips are **display only** — not authorization enforcement.
- Mock mutations MUST NOT claim live core enforcement.
- No secrets, tokens, or credential material in detail DTOs.
- Related audit listing MUST NOT claim write access to the audit log.

## Cross-navigation

- Related audit row activate → `onOpenAudit(eventId)` → Access audit activation when available.
- This contract does **not** define Audit detail body content.
