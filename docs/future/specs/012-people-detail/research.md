# Research: Access People Detail

**Feature**: `012-people-detail`  
**Date**: 2026-08-11  
**Status**: Complete — no open NEEDS CLARIFICATION

---

## R1 — Detail host and composition pattern

**Decision**: Use the single global `DetailPanel` with `Body layout="sections"` + `Grid` / `Section` / `Metric` / `Metadata`, matching Resources and Architecture operational details. Extract a dedicated `MemberDetail` component; do not keep a thin inline block in `AccessView`.

**Rationale**:
- AGENTS.md / DESIGN.md LAW forbid second inspectors, drawers, or entity detail pages for operational entities.
- ResourceDetail / ArchitectureNodeDetail already define the production operational chrome operators expect.
- Extraction keeps Access index maintainable and makes the composition testable.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Full-page member route | Violates DetailPanel LAW and Access IA |
| Document layout (`layout="document"`) | Reserved for Knowledge / Spec Kit writing surfaces |
| Keep inline thin detail | Fails FR-012 and SC-005 quality bar |

---

## R2 — Data source for mock stage

**Decision**: Enrich `WorkspacePerson` in `@polterware/feature-shell` workspace mocks (`PEOPLE_MOCKS`). Access consumes mocks; no ConnectRPC / polter-core membership API in this slice.

**Rationale**:
- People list already uses `PEOPLE_MOCKS`; detail must not invent a second person type.
- Shell mocks are the established SSOT for operator surface demos (integrations, audit, agents).
- Future live DTO can replace fixtures without changing DetailPanel section structure if field names stay stable.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Feature-local fixtures only in access | Dual person SSOT with list vs detail drift |
| Fake live RPC with empty backend | Dishonest “live” claim (Constitution III) |
| Derive permissions only in UI from role string | Hides fixture authoring; harder to show Viewer deny rows explicitly |

---

## R3 — Permissions model (display only)

**Decision**: Fixture-authored capability rows `{ id, capability, effect: allow | deny | request-only }`. Effects render with `StatusChip`. Table is **not** authorization.

**Rationale**:
- docs/spec.md + access-ui: members show permissions picture; role chips are not authz.
- `request-only` matches agent/human governance language already used in roles matrix sketches (007).
- Explicit deny rows for Viewer make least-privilege posture inspectable.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Live policy evaluation in renderer | Security boundary violation / fake enforcement |
| Binary allow list only | Cannot express production “request approval” posture |
| Roles matrix editor in this feature | Deferred to 007 expansions |

---

## R4 — Sessions and revoke interaction

**Decision**: Sessions DataTable with device, location, last active, current StatusChip. **Row activate on non-current session** calls mock `onRevokeSession`. Current session has no revoke action. No Actions column.

**Rationale**:
- DataTable LAW forbids Actions columns; product actions live in DetailPanel header or row activation patterns already used elsewhere.
- Distinguishing current session prevents accidental “revoke self” confusion in mock UX.
- Mock toast must not claim real session invalidation.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Per-row icon buttons | Actions-column smell / hand-rolled row chrome |
| Header-only “revoke all sessions” | Less inspectable; weaker demo of multi-session posture |
| Real kill API stub | Out of scope; dishonest without core |

---

## R5 — Teams resolution

**Decision**: AccessView filters `TEAM_MOCKS` by `member.teamIds` and passes `teams` into `MemberDetail`. Team row activate → existing team detail activation (`entityKind: "team"`).

**Rationale**:
- Teams already have a detail branch on People tab; reusing activation keeps one navigation model.
- Filtering at the view keeps `MemberDetail` presentational.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Embed full team objects on `WorkspacePerson` | Duplicates team SSOT |
| Navigate to separate Teams route | Leaves DetailPanel model |

---

## R6 — Governance header actions

**Decision**: Three header actions via `DetailPanel.Action`: Change role, Add to team, Revoke access (danger). All mock-toast only.

**Rationale**:
- 004 access-ui: detail supports edit/remove when permitted; mock stage uses toast honesty.
- Icon-only header actions match DetailPanel LAW (Tooltip + accessible label).

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Body “Controls” button row (agent style) | Inconsistent with polished Resource/Architecture header actions |
| Real dialogs for role/team | Nice later; not required for mock production chrome of **inspection** surface |

---

## R7 — Testing strategy

**Decision**:
1. Pure helper unit tests (presence/effect/result tones) in access package node vitest.
2. Source composition contract (sections, empty states, no PrimaryCell `description=`).
3. Desktop `detail-fixture-coverage` asserts Admin person completeness.
4. Keep `detail-targets` tests for member activation.

**Rationale**:
- Solid `renderToString` is unreliable under jsdom for portaled `DetailPanel.Content` without full host; operations and architecture already favor contract/fixture tests for similar surfaces.
- Fixture coverage is the monorepo gate for “19 detail compositions” completeness.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Full jsdom DetailPanel.Root mount only | Works (run-detail pattern) but heavier; helpers+source+fixture gate sufficient for mock stage |
| Skip tests | Violates Constitution XII for a governance surface |

---

## R8 — Numbering / feature directory

**Decision**: Own work under `specs/012-people-detail` regardless of parallel `012-*` directories on the branch. Operators set `.specify/feature.json` → `specs/012-people-detail` when planning/implementing this feature.

**Rationale**: Spec Kit feature directories are independent of git branch name; the repo currently carries multiple sequential collisions from parallel work.

**Alternatives considered**: Renumber to `015-*` — optional cleanup outside this plan; not required for correctness.

---

## Resolved clarifications

No remaining NEEDS CLARIFICATION markers from Technical Context. Defaults locked:

| Topic | Default |
| --- | --- |
| Live API | None (mock fixtures) |
| Role set | Admin / Operator / Viewer fixtures |
| Auth methods | SSO / Password strings in fixtures |
| Session revoke | Mock toast; non-current only |
| Agents | Out of scope |
