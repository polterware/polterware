# Data Model: Access Invitation Detail

**Feature**: `024-invitation-detail`  
**Stage**: Mock fixtures (shell workspace mocks)  
**Live persistence**: out of scope

---

## Overview

```text
Invitation
  ├── Invitee email (not yet a member)
  ├── Status (Pending | Expired | Cancelled)
  ├── Assignment (role, team, optional teamId, optional environments)
  ├── Provenance (invitedBy, optional invitedAt, expires)
  └── Optional message/notes
```

Domain roots (`docs/spec.md` §8.8 Access / §8.10 Organizations): invitations are pre-membership offers under project Access → People.

---

## Entities

### Invitation — `WorkspaceInvitation`

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable fixture id (`i1`…) |
| `email` | string | yes | Header title; table PrimaryCell |
| `role` | string | yes | Role granted on accept |
| `team` | string | yes | Display team name (`—` when none) |
| `teamId` | string | no | Resolvable TEAM_MOCKS id for open-team |
| `status` | `"Pending" \| "Expired" \| "Cancelled"` | yes | StatusChip |
| `expires` | string | yes | Humanized expiry label |
| `invitedBy` | string | yes | Provenance inviter display name |
| `invitedAt` | string | no | When invite was issued |
| `environments` | string[] | no | Optional environment scope on accept |
| `message` | string | no | Invite note shown when present |

**Validation (mock stage)**:
- `id`, `email` non-empty
- At least one Pending and one Expired fixture with complete assignment + provenance
- Pending fixtures used as completeness gates SHOULD include `invitedAt` and non-empty environments or explicit empty omission
- Cancelled optional; not required in People table filter

**State (invitation status)**:

| status | UI label | StatusChip tone | Header actions |
| --- | --- | --- | --- |
| `Pending` | Pending | warn | Resend, Cancel (danger) |
| `Expired` | Expired | danger | Re-invite |
| `Cancelled` | Cancelled | muted | none |

### Assignment

Role + team (+ optional environments) the invitee receives on accept. Display-only in mock stage.

### Provenance

Who invited (`invitedBy`), optional `invitedAt`, and expiry posture (`expires`).

### Lifecycle Intent

Resend / Cancel / Re-invite — mock toasts only; not email delivery or membership mutation.

---

## Relationships

| From | To | Cardinality | Notes |
| --- | --- | --- | --- |
| Invitation | Team | 0..1 | via `teamId` or name match |
| Invitation | Person (member) | 0 | no member until accept — do not invent |

---

## Out of model

- SMTP delivery receipts
- SSO / SCIM directory invites
- Member profile for unaccepted email
- Live IdentityService DTOs
