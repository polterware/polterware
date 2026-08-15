# Research: Access Teams Detail

**Feature**: `015-teams-detail`  
**Date**: 2026-08-12  
**Status**: Complete — no open NEEDS CLARIFICATION

---

## R1 — Detail host and composition pattern

**Decision**: Use the single global `DetailPanel` with `Body layout="sections"` + `Grid` / `Section` / `Metric` / `Metadata`, matching People member and Resources operational details. Extract a dedicated `TeamDetail` component; do not keep a thin inline block in `AccessView`.

**Rationale**:
- AGENTS.md / DESIGN.md LAW forbid second inspectors, drawers, or entity detail pages for operational entities.
- MemberDetail already defines the production Access detail chrome operators expect.
- Extraction keeps Access index maintainable and makes the composition testable.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Full-page team route | Violates DetailPanel LAW and Access IA |
| Document layout (`layout="document"`) | Reserved for Knowledge / Spec Kit writing surfaces |
| Keep inline thin detail | Fails FR-012 and SC-005 quality bar |

---

## R2 — Data source for mock stage

**Decision**: Enrich `WorkspaceTeam` in `@polterware/feature-shell` workspace mocks (`TEAM_MOCKS`). Access consumes mocks; no ConnectRPC / polter-core team membership API in this slice.

**Rationale**:
- Teams list already uses `TEAM_MOCKS`; detail must not invent a second team type.
- Shell mocks are the established SSOT for operator surface demos (people, integrations, audit).
- Future live DTO can replace fixtures without changing DetailPanel section structure if field names stay stable.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Feature-local fixtures only in access | Dual team SSOT with list vs detail drift |
| Fake live RPC with empty backend | Dishonest “live” claim (Constitution III) |
| Derive roster only by filtering PEOPLE_MOCKS | Cannot show multi-member Platform Team without expanding people fixtures; roster on team keeps ownership-centric inspection honest |

---

## R3 — Roster vs person SSOT

**Decision**: Team carries `memberRoster[]` summaries for display. When `member.id` matches a People fixture, row activate opens member detail via existing Access activation (`entityKind: "member"`). Person → team remains `member.teamIds` + filtered `TEAM_MOCKS` (People detail path).

**Rationale**:
- docs/spec.md: teams are ownership/collaboration groups; roster is the primary team inspection surface.
- Full Person objects on every team would duplicate people SSOT and bloat fixtures.
- Summary rows keep table lean while preserving deep-link when id is known.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Only filter PEOPLE_MOCKS by teamIds | List counts (4/3/2 historically) did not match three people fixtures; roster authoring is clearer |
| Embed full WorkspacePerson on team | Dual person SSOT |
| Separate Teams product page | Leaves DetailPanel model and Access IA |

---

## R4 — Owned resources and policies (display only)

**Decision**: Fixture-authored `ownedResources[]` and `policies[]` tables. Not live inventory sync or policy evaluation. StatusChip for policy status; resource kind maps to DitherIconTone only.

**Rationale**:
- Operators need ownership posture next to on-call without opening Resources/Policies first.
- Display-only keeps security boundary: renderer is not authz or inventory authority.
- Identity Team fixture may use empty policies to prove empty-state honesty.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Live resource query by owner team | Out of scope; dishonest without core |
| Policy evaluation in UI | Security boundary violation / fake enforcement |
| Omit resources/policies until live | Fails production mock inspection quality for Platform Team |

---

## R5 — Governance header actions

**Decision**: Three header actions via `DetailPanel.Action`: Edit team, Add member, Archive team (danger). All mock-toast only.

**Rationale**:
- Matches People detail governance adjacency pattern.
- Icon-only header actions match DetailPanel LAW (Tooltip + accessible label).
- Archive is the team-level destructive analogue of revoke membership; mock only.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Body “Controls” button row | Inconsistent with polished Resource/People header actions |
| Real dialogs for edit/add | Nice later; not required for mock inspection surface |
| Remove archive from mock stage | Operators still expect a danger path; toast honesty is enough |

---

## R6 — On-call / escalation posture

**Decision**: Surface on-call, escalation, optional schedule, and lead in Overview/metrics. Status is team lifecycle (`active` \| `archived`), not on-call health.

**Rationale**:
- docs/spec.md organizations/teams + operational ownership; on-call is core team ops metadata.
- Avoid conflating team archive status with person presence or resource health.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| StatusChip for on-call “duty” states | No fixture domain for paging duty cycles in this slice |
| Only list column onCall without escalation | Incomplete ops posture for production mock |

---

## R7 — Testing strategy

**Decision**:
1. Pure helper unit tests (team status, presence, policy status, result, resource kind tones) in access package node vitest.
2. Source composition contract (sections, empty states, header actions, no PrimaryCell `description=`).
3. Desktop `detail-fixture-coverage` asserts Platform Team completeness + `members`/`owned` count consistency.
4. Keep `detail-targets` tests for team activation.

**Rationale**:
- Solid `renderToString` is unreliable under jsdom for portaled `DetailPanel.Content` without full host; people/operations already favor contract/fixture tests.
- Fixture coverage is the monorepo gate for detail composition completeness.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Full jsdom DetailPanel.Root mount only | Heavier; helpers+source+fixture gate sufficient for mock stage |
| Skip tests | Violates Constitution XII for a governance surface |

---

## R8 — Feature directory / active feature pointer

**Decision**: Own work under `specs/015-teams-detail`. Operators set `.specify/feature.json` → `specs/015-teams-detail` when planning/implementing this feature. Git branch name may remain `refactor/new`.

**Rationale**: Spec Kit feature directories are independent of git branch name; sequential numbering continues past `014-nodes-detail`.

**Alternatives considered**: Reuse `007` expansions only — insufficient; team detail is a full production composition worthy of its own feature SSOT.

---

## Resolved clarifications

No remaining NEEDS CLARIFICATION markers from Technical Context. Defaults locked:

| Topic | Default |
| --- | --- |
| Live API | None (mock fixtures) |
| Team status set | `active` / `archived` |
| Roster deep-link | Activate when person id exists in PEOPLE_MOCKS |
| Owned resources | Fixture ownership links only |
| Policies | Display links only |
| Mutations | Mock toast only |
| Agents / invitations / roles matrix | Out of scope |
