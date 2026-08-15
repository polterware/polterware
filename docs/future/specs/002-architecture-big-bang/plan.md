# Implementation Plan: Architecture Big Bang Refactor

**Branch**: `002-architecture-big-bang` | **Date**: 2026-08-02 | **Spec**: [spec.md](./spec.md) (amended post-analyze)

**Input**: Spec completion set — local-first, single-operator; Desktop + polterware-core + CLI + MCP only.

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/) · [migration-map.md](./migration-map.md)

---

## Summary

Replace mock-first Electron/React desktop and disconnected TypeScript `packages/core` with a **modular local-first control plane**:

| Layer | Choice |
| --- | --- |
| Desktop | Electron + electron-vite + **SolidJS** + Tailwind v4; thin Main; secure Preload; **one** product renderer |
| Local authority | Go **polterware-core** (SQLite, ops, policy, roles, audit FR-027a/b, knowledge light) |
| Transport | ConnectRPC loopback + ephemeral session (FR-067) |
| Contracts | Protobuf + Buf → generated Go/TS |
| Clients | **CLI** + **MCP** same domain rules |
| Data | **Greenfield DISCARD** for listed legacy stores; capabilities preserved |
| Out of build | Web, polterware-server, polterware-node, PostgreSQL, cloud orgs — **no scenic packages** |

**Must also deliver:** Main auto-updater (FR-039a); single renderer cutover (FR-039b); roles Owner/Operator/Viewer/Agent; skills preview/setup **removed** from product path.

---

## Technical Context

**Language/Version**: TypeScript (strict) frontend; Go 1.22+ backend workspace

**Primary Dependencies**:

- Desktop: Electron, electron-vite, SolidJS, Tailwind CSS v4, Kobalte (on demand), Corvu (panels on demand), Hugeicons Free Stroke Rounded, TanStack Virtual, electron-updater (Main only)
- Contracts: Protocol Buffers, Buf, ConnectRPC
- Core: Go, ConnectRPC, SQLite (prefer pure-Go driver), structured logging + correlation IDs
- Observability: structured logs + correlation; **OpenTelemetry optional later** — not required for acceptance (structured correlation satisfies FR-035/NFR-012)
- CLI/MCP: Go clients of local contracts

**Storage**: SQLite only (completion). PostgreSQL/pgx/sqlc **not implemented**.

**Testing**: Domain unit; contract tests per service group; SQLite integration; session security; role×capability; audit field completeness; desktop e2e; CLI/MCP parity; perf logs; legacy absence

**Target Platform**: Desktop macOS primary packaging; Go core portable; CLI/MCP same host

**Project Type**: Monorepo — desktop + local backend + CLI + MCP

**Performance Goals**: SC-010 (logs UI); NFR-001–005; Main off data path; bounded streams (see research limits)

**Constraints**: Spec completion set; constitution v2; no React in final product; no dual renderer product path; no dual-format conf readers; no empty web/server/node

**Scale/Scope**: Single-operator local workspace; not multi-tenant SaaS

---

## Constitution Check

*GATE: Pass for planning against constitution v2 + amended spec.*

| Principle | Status | Evidence |
| --- | --- | --- |
| I Operational architecture | Pass | Canvas + real ops; handoff not clone |
| II Domain before framework | Pass | Go domains/ports; Solid/Electron adapters only |
| III Runtime separation | Pass | Main/Preload/Renderer/core/CLI/MCP; web/server/node not live |
| IV Directed deps | Pass | apps→features→frontend→contracts; runtimes→compositions→domains→ports |
| V Contracts-first | Pass | Buf/Connect; generated only |
| VI Security | Pass | Sandbox, FR-067 session, roles, MCP no bypass |
| VII Safe auditable exec | Pass | FR-027a/b, approvals, FR-064 |
| VIII Local-first | Pass | No cloud required; tenant N/A |
| IX Performance | Pass | Workers, virtualization, bounds |
| X Resilience | Pass | Timeouts, cancel, FR-065 |
| XI Observability | Pass | Correlation; not APM product |
| XII Testability | Pass | Domain without Electron; SC suite |
| XIII Real code | Pass | No scenic web/server/node |
| XIV SSoT | Pass | Single renderer final; greenfield; remove dual |
| XV Big bang | Pass | Branch cutover; remove legacy |
| XVI Docs | Pass | Update at end |

**Hard stops**: VI, IV, XIV — no convenience exceptions.

### Post-design re-check

Aligned with amended spec: local roles, no org MUST, DISCARD stores, session file, updater Main, knowledge light, skills REMOVE.

---

## Project Structure

### Documentation (feature)

```text
specs/002-architecture-big-bang/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── migration-map.md
├── contracts/
└── tasks.md          # regenerate after this plan via /speckit-tasks
```

### Source (create only with consumers)

```text
apps/desktop/                 # REQUIRED — Main|Preload|Solid renderer|workers
apps/presentation/            # isolate; non-product
# apps/web                    # DO NOT CREATE

packages/
  frontend/{design-system,platform,data,state,canvas,logs,terminal,utils}
  features/{shell,commander,architecture,processes,workflows,integrations,
            operations,approvals,logs,resources,settings,knowledge,agents}
  contracts/  generated/  configuration/  testing/

backend/
  go.work
  runtimes/{core,cli,mcp}     # no server/, node/
  domains/{identity,projects,architecture,resources,integrations,
           operations,workflows,logs,policies,approvals,audit,agents,knowledge}
  providers/{sqlite,process,git,pkg,declarative,handoff}
  platform/{db/sqlite,transport,security,observability,resilience,gen}
  compositions/{local-core,command-line,mcp-server}

schemas/polterware/{common,local,events}/v1/
tooling/  tests/  docs/
```

**Remove after validation:** mock product path, UI-only ops façade, React product + React ui product dep, `@polterware/core` authority, skill preview/setup product surfaces, `legacy/tui` imports.

### Dependency direction

```text
apps/desktop → packages/features → packages/frontend → contracts/generated
backend/runtimes → compositions → domains → ports
providers implement ports; platform has no business rules
```

### Normative terminology (CHK039)

| Term | Definition |
| --- | --- |
| **Provisioning** | Declarative create/configure of an external resource via provider — **DEFERRED** as its own domain/package; no empty package |
| **Operation** | Auditable execution unit |
| **Workflow** | Ordered composition of operations |
| **Integration** | Provider connection + capabilities |
| **Resource** | Real managed/discovered entity |
| **Command** | Concrete instruction inside an operation |

Current create/configure behavior lives under **operations** + **integrations** until a real product case justifies a provisioning domain.

### Organizations / memberships (CHK038 / CHK153)

| Concept | Completion status |
| --- | --- |
| Cloud Organization | **DEFERRED** — not implemented; no default fictional org |
| Cloud Membership / Team | **DEFERRED** |
| Tenant isolation | **NOT APPLICABLE** |
| Organization isolation | **NOT APPLICABLE** |
| Workspace / System / Environment / Resource isolation | **Required** |
| Human vs agent principal isolation | **Required** |

FR-010 is the **local** model only (see amended spec). No scenic org tables.

### Dependency boundary enforcement (CHK049)

**Strategy**: ESLint restricted imports for TypeScript + shell/`go list` architectural checks for Go; both in root `pnpm verify`.

| Rule (TS) | Enforcement |
| --- | --- |
| apps may consume features and public packages | `eslint-plugin-boundaries` or `no-restricted-imports` zones |
| features must not import other features' internals | ban `packages/features/*/src/**` deep imports across features |
| frontend packages must not import apps | restricted patterns `apps/*` |
| web-independent packages must not import Electron | ban `electron` outside `apps/desktop` |
| renderer must not import Node builtins | ban `node:*` in renderer/Solid packages |
| generated must not be hand-edited | generate gate + `// Code generated` header check |
| public entrypoints only | ban deep imports outside package `exports` |

| Rule (Go) | Enforcement |
| --- | --- |
| domains ↛ concrete providers / sqlite drivers / connect handlers / electron | `tooling/check-domain-imports.sh` + unit arch test |
| compositions may wire adapters | allowed |
| runtimes depend on compositions/public APIs | `go list` import allowlist |
| cycles rejected | `go list` + script fail |

Document details: `docs/architecture/dependency-boundaries.md` (T018). Scripts: T019–T020 + T187 (eslint boundaries config).

### Stream / queue limits (CHK058 / CHK110)

Normative defaults in [contracts/events.md](./contracts/events.md). Summary: chunk 64 KiB; op event buffer 256; log UI 10k / batch 200; line 1 MiB; RPC 4 MiB; op timeout 10m; consumer idle 30s; concurrent ops 4; terminal sessions 4; no local heartbeat product.

### Session bootstrap (CHK059)

File-mode only for CLI/MCP defaults; see [contracts/session.md](./contracts/session.md).

### Retry taxonomy (CHK117)

See [contracts/overview.md](./contracts/overview.md) matrix — transient/resource retryable; policy/validation/auth not retryable; mutations need idempotency_key.

### Performance budgets (CHK113)

| Metric | Target |
| --- | --- |
| Desktop interactive cold start | ≤ 10 s (measure; document host) |
| Core Health ready | ≤ 3 s |
| Renderer idle RSS soft | ≤ 512 MiB |
| Core idle RSS soft | ≤ 128 MiB |

Recorded in `tests/performance/startup.md` (T158).

### Future web parity (CHK068)

**Not required** for this feature. Web is DEFERRED (FR-063). No empty `apps/web`.

### Future node secure defaults (CHK105)

Non-binding appendix: [research.md R20](./research.md). No node packages this feature.

### Secret lifecycle (CHK086)

SecretReference create/list/delete + resolve-in-core only; rotate/TTL product DEFERRED. See data-model.

### File-level inventory (CHK018)

Phase 0 inventory MUST be **file-level for relevant modules** (not package names only). Categories: entrypoints, Main, preload, renderer, workers/async, services, persistence, IPC, integrations, scripts, configs, tests, updater, functional assets, legacy stores, React entry, preserve/migrate/remove. Task T001 + disposition T008.

### SQLite migration files (CHK079)

Named list in [data-model.md](./data-model.md) (`0001_…`–`0008_…`). T053 authors them.

### Troubleshooting runbook requirement (CHK143)

Completion docs MUST include `docs/development/troubleshooting.md` covering at least: session file missing/stale; core spawn failure; quarantine/corrupt store; DISCARD reset; token not in logs; concurrency/stream exhausted; CLI/MCP fail-closed. Task T165; outline also in quickstart §Troubleshooting.

### Completion set vs future architecture

| In completion set | Explicitly out (document only) |
| --- | --- |
| pnpm, Turborepo, go.work, Electron, electron-vite, Solid, TW4, polterware-core, SQLite, Connect local, Buf/proto, CLI, MCP, architecture/resources/ops/workflows/terminal/logs/approvals/policies/audit/knowledge/local integrations, updater preserve, React removal | Web app, polterware-server, polterware-node, cloud orgs/memberships/teams, PostgreSQL/pgx/sqlc, NATS, cloud sync/collab, billing, cloud entitlements |

---

## Runtime matrix

| Runtime | Owns | Must not |
| --- | --- | --- |
| Main | Lifecycle, windows, menus, shortcuts, **updater**, spawn core, **session file write**, native perms | Domain, streams, policy |
| Preload | Typed bridge only | Rules, raw token persistence |
| Renderer | Solid UI only (final) | Node, secrets, exec, updates |
| polterware-core | SQLite, ops, roles/policy, audit, knowledge, session token mint | UI, multi-tenant cloud |
| CLI / MCP | Same contracts | Policy bypass |

---

## Domain ownership (completion)

| Domain | Notes |
| --- | --- |
| identity | Principals + roles Owner/Operator/Viewer/Agent |
| projects | Workspace, System, Environment (no cloud org) |
| architecture | Layout revision FR-059 |
| resources | external_key dedupe |
| integrations | execute\|status\|handoff |
| operations | State machine + FR-027a/b |
| workflows | Local pipelines |
| logs | Bounded buffers |
| policies | role+capability+approval |
| approvals | Human approve; agent no self-approve |
| audit | FR-027a/b fields |
| agents | MCP principal binding |
| knowledge | Light CRUD + associate |

**No domains this feature:** cloud org, billing, nodes, sync product.

---

## Authz & session (implementation notes)

1. **Session (FR-067):** Main starts core → core binds `127.0.0.1:0` → mints token → writes runtime session descriptor (endpoint, token, pid, started_at) mode 0600 / Windows ACL → Preload Connect client uses token without renderer localStorage of token → on quit delete file; stale = dead pid or health fail.
2. **Authz:** `Authorize(principal, capability, resource_ctx)` = role defaults ∩ explicit caps ∩ policy ∩ approval gate.
3. **Default install:** single Owner human principal; agents registered with restricted capability sets.

---

## Persistence

- SQLite migrations for data-model entities including knowledge, roles bindings.
- **DISCARD** legacy: conf pins/pipelines/repos, `.polterware/config.json`, old polterware.yaml as required format, mock state (see migration-map).
- **No CONVERT** this feature.
- App icon preference: reimplement greenfield (capability preserve).
- FR-065 quarantine + ResetLocalStore.

---

## Security strategy

- contextIsolation true; nodeIntegration false; sandbox when compatible; CSP restrictive (default-src 'self'; script-src 'self'; no unsafe-eval in prod).
- FR-067 session tests.
- Role×capability security suite (SC-007).
- Audit field asserts (SC-006).
- MCP min tools; no generic exec; no skills product surface.
- Updater only Main; sanitized logs.

---

## Single renderer cutover (normative order)

**No permanent feature flag** to switch architectures. **No dual product path** as final state.

| Step | Gate | Tasks |
| --- | ---: | --- |
| 1 | Implement Solid app scaffold + shared packages | T102–T115 |
| 2 | Migrate all completion-set features to Solid packages | T122–T135 |
| 3 | Integrate polterware-core (Main supervisor, preload, session) | T116–T121 |
| 4 | Integration tests (core ops, roles, audit, session) | T085–T088, T174 |
| 5 | **E2E of the new renderer** (Solid as under-test product path) | **T145**, T146 |
| 6 | Validate Solid is the sole **configured** product entry (after switch) | T137, T142 |
| 7 | **Atomic cutover** — single entry in electron-vite/package.json | **T137** |
| 8 | **Remove React** product tree + runtime deps | **T138**, **T139** — **only after T145 green** |
| 9 | Remove React routes, scripts, builds, residual deps | T139b / T150 |
| 10 | General legacy removal (mock path, TS core authority, skills product, tui imports) | **T143+** |
| 11 | Verify absence of alternate renderer / react product dep | T142, T183 |

**Hard rule**: React purge tasks (T138/T139) **MUST NOT** run before E2E T145 is green.

Final repository MUST NOT contain: React as runtime product dependency; React entrypoint; React product route; React product build script; permanent renderer feature flag; alternate product renderer; permanent compat package.

---

## Testing strategy

| Suite | Covers |
| --- | --- |
| Domain unit | Roles, transitions, dedupe, knowledge rules |
| Contract | Health, Project, Operation, Process, Approval, Knowledge (not only Health) |
| Integration | SQLite, session file, FR-064 ops, FR-065 |
| Security | Isolation, token, role denies, MCP parity, no token in logs |
| E2E desktop | Quickstart E1–E8 + knowledge + single entry |
| CLI/MCP | SC-005/015 |
| Performance | SC-010 |
| Legacy absence | SC-011/012/017 |

---

## Internal phases (branch)

| # | Outcome |
| --- | --- |
| 0 | Baseline inventory + characterization (no deletes) |
| 1 | Monorepo pnpm/turbo/go.work + verify scripts |
| 2 | Schemas local+events + generate + contract tests |
| 3 | Domains + role policy |
| 4 | SQLite + adapters |
| 5 | polterware-core + session + FR-064 ops |
| 6 | CLI + MCP + session client |
| 7 | Frontend Solid foundation + canvas/logs/terminal |
| 8 | Desktop Main supervisor + updater + bridge + Solid features (incl. knowledge) |
| 9 | Atomic Solid cutover → **E2E T145** → React purge (T138/T139 after e2e) |
| 10 | Remove core TS authority, mock path, skills product, legacy/tui |
| 11 | Security + perf + docs + sign-off |

Web/server/node: **documentation-only deferred notes**, never empty packages.

---

## Root commands (target)

`pnpm install` · `pnpm generate` · `pnpm dev:desktop` · `pnpm lint` · `typecheck` · `test` · `test:integration` · `test:e2e` · `test:security` · `test:performance` · `build` · `package` · `pnpm verify` · `go test ./backend/...`

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Scope creep cloud | FR-063 hard; no packages |
| Dual React/Solid | FR-039b + verify |
| Token leakage | FR-067 tests |
| DISCARD surprise | User-reset docs |
| Thin contract tests | Per-service contract suite |
| Skills reintroduction | Disposition REMOVE + grep gate |

---

## Technical completion criteria

Map to SC-001–SC-024 and FR-063–067: core supervised; one renderer; no React product; contracts tested; roles enforced; audit fields complete; session secure; updater Main; knowledge light; DISCARD documented; legacy gone; verify green.

---

## Complexity Tracking

| Topic | Why | Rejected simpler |
| --- | --- | --- |
| Solid replaces React | Spec/plan stack + big bang UI | Dual UI frameworks permanent |
| Go core replaces TS core | Single backend language CLI/MCP/core | Permanent dual domain |
| Session file for CLI/MCP | Multi-client local without cloud | IPC-only blocks CLI/MCP |

---

## Next step

`/speckit-tasks` — regenerate full task list against this plan and amended spec (supersede previous tasks.md).
