# Research: Access Agents Detail

**Feature**: `017-agents-detail`  
**Date**: 2026-08-12  
**Status**: Complete — no open NEEDS CLARIFICATION

---

## R1 — Detail host and composition pattern

**Decision**: Use the single global `DetailPanel` with `Body layout="sections"` + `Grid` / `Section` / `Metric` / `Metadata`, matching MemberDetail / TeamDetail / operational resource details. Extract a dedicated `AgentDetail` component; replace the thin inline agent block in `AccessView`.

**Rationale**:
- AGENTS.md / DESIGN.md LAW forbid second inspectors, drawers, or entity detail pages for operational entities.
- MemberDetail / TeamDetail already define the production Access detail chrome operators expect.
- Extraction keeps Access index maintainable and makes the composition testable under parallel sibling work.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Full-page agent route | Violates DetailPanel LAW and Access IA |
| Document layout (`layout="document"`) | Reserved for Knowledge / Spec Kit writing surfaces |
| Keep inline thin detail | Fails FR-014 and SC-006 quality bar |

---

## R2 — Data source for mock stage

**Decision**: Enrich `WorkspaceAgent` in `@polterware/feature-shell` workspace mocks (`AGENT_MOCKS`). Access consumes mocks; no ConnectRPC / IdentityService / agent control API in this slice. Sessions remain in `AGENT_SESSION_MOCKS` and are filtered by `agentId` at the host.

**Rationale**:
- Agents list already uses `AGENT_MOCKS`; detail must not invent a second agent type.
- Shell mocks are the established SSOT for operator surface demos.
- Future live DTO can replace fixtures without changing DetailPanel section structure if field names stay stable.
- Reading `AGENT_SESSION_MOCKS` for links is allowed; mutating session/policy/audit fixtures is not (sibling ownership).

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Feature-local fixtures only in access | Dual agent SSOT with list vs detail drift |
| Fake live RPC with empty backend | Dishonest “live” claim (Constitution III) |
| Embed full session objects on agent | Dual session SSOT; collides with sibling session detail |

---

## R3 — Structured capabilities and tools

**Decision**: Replace free-form `string[]` for capabilities and tools with structured fixture rows (`id`, `name`, effect/kind). Display via DataTable. Keep list columns on Agents index independent (name/type/autonomy/status).

**Rationale**:
- Spec requires structured collections, not only join of strings.
- Member permissions already model `effect` StatusChip patterns operators understand.
- Tool `kind` (`mcp` | `builtin` | `integration`) clarifies MCP surfaces without claiming live MCP invocation.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Keep string[] + split in UI | Weak fixture gate; no per-row status/kind |
| Live capability evaluation in UI | Security boundary violation / fake enforcement |

---

## R4 — Session and operation hand-offs

**Decision**: Agent detail receives `sessions` (pre-filtered or filterable by agent id) and `recentOperationIds` from the agent model. Activation calls `onOpenSession` / `onOpenOperation` only. Session detail UI is sibling-owned (`020-agent-session-detail`).

**Rationale**:
- Parallel agents implement session/policy/audit details; rewriting those files causes merge collisions.
- Callbacks preserve Access activation already used for `entityKind: "agent-session"`.
- Operation open reuses existing Access `onOpenOperation` host prop when present.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Inline session steps in agent detail | Collides with session detail ownership; dual UI |
| Navigate only via Agents tab session table | Weak from agent context; header deep-link is expected |

---

## R5 — Parallel file boundaries

**Decision**: Touch only agent detail files, agent fixture fields, Access agent mount wiring, agent asserts in fixture coverage, and `specs/017-agents-detail/**`. Do not edit `policy-detail*`, `audit-detail*`, `agent-session-detail*`, `member-detail*`, `team-detail*`, or non-agent mock arrays except read-only session filtering.

**Rationale**: Three sibling agents work simultaneously on Access details; narrow file ownership is the merge strategy.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Shared “access detail kit” refactor | Cross-cutting collision risk mid-parallel |
| Rewrite session mocks for agent completeness | Sibling ownership of session model |

---

## R6 — Status and autonomy mapping

**Decision**: Pure helpers map agent status → StatusChip tone/label and autonomy → human-readable label; session status tones for session table rows; capability effect tones reuse member-like allow/deny/request-only mapping.

**Rationale**: Matches MemberDetail / TeamDetail pure-helper + composition-contract test pattern without Electron.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Inline tone maps in index only | Unusable for unit tests; drifts from composition |
| Ad-hoc pills | DESIGN LAW requires StatusChip |
