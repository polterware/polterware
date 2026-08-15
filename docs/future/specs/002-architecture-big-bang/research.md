# Research: Architecture Big Bang Refactor

**Feature**: `002-architecture-big-bang`  
**Date**: 2026-08-02 (updated post-spec amend)  
**Status**: Complete — planning unknowns resolved against amended spec + constitution v2

---

## R1. Backend language

**Decision**: Go for polterware-core, CLI, MCP.  
**Rationale**: Single domain implementation; process isolation from Electron; constitution II/III.  
**Alternatives**: Keep TS core (dual authority); Rust (superseded by stack decision).

## R2. Local transport + session (FR-067)

**Decision**: ConnectRPC on `127.0.0.1` random port; ephemeral bearer token; session descriptor file in OS runtime dir (Unix `0600`, Windows user ACL); fields: endpoint, token, pid, started_at; delete on clean shutdown; stale if pid dead or health fails.  
**Rationale**: CLI/MCP need same contracts without cloud; security default-deny.  
**Alternatives**: Unix socket only (harder TS tooling); token in argv (leaks).

## R3. Contracts tooling

**Decision**: Protobuf + Buf + ConnectRPC; generate Go + TS; contexts **local** + **events** only for completion. No scenic public/internal/node protos.  
**Rationale**: FR-046/V; FR-063.  
**Stream limits (normative defaults)**: see [contracts/events.md](./contracts/events.md) — stdout/stderr chunk **64 KiB**; event buffer/op **256**; visible log buffer **10_000**; render batch **200**; max log line **1 MiB**; max non-stream RPC **4 MiB**; op timeout **10 min**; no local heartbeat this feature; max idle without consumer **30 s**; max concurrent local ops **4**; max terminal sessions **4**; overflow = drop-oldest for logs only (never drop audit); secrets redacted before buffers.

## R4. Frontend

**Decision**: SolidJS + Tailwind v4 + electron-vite. Final product **no React**.  
**Rationale**: Spec stack; XIV single UI stack.  
**Cutover**: Atomic entrypoint switch; then delete React product (FR-039b).

## R5. Design system

**Decision**: Native HTML primitives + Kobalte (complex a11y) + Corvu (resizable panels) + Hugeicons Free Stroke Rounded + TanStack Virtual. No CSS-in-JS runtime.
**Rationale**: DESIGN.md density; constitution I.

## R6. Canvas / logs / terminal

**Decision**: Independent frontend packages; DOM/SVG first; layout worker; terminal presentation-only; process authority in core.  
**Rationale**: Spec US2/US6/FR-019.

## R7. Persistence

**Decision**: SQLite pure-Go preferred; greenfield DISCARD for listed stores; no CONVERT this feature; no PostgreSQL.  
**Rationale**: FR-005; completion set.

## R8. Authz model

**Decision**: Roles Owner/Operator/Viewer/Agent; `role defaults + explicit capabilities + policy + approval`. Tenant/org isolation N/A.  
**Rationale**: Spec FR-024/024a/029 amend (C1/C2).

## R9. Observability

**Decision**: Structured logs + correlation IDs across Desktop/core/CLI/MCP. **OpenTelemetry SDK not required** for acceptance; may be added later without domain coupling.  
**Rationale**: FR-035; avoid unjustified weight; user stack listed OTel as optional verify item.

## R10. Skills helpers

**Decision**: **REMOVE** skill preview/setup from product path (mock UI + secondary helper, not FR-064 control-plane journey). MCP product remains.  
**Rationale**: Spec H4 resolve; evidence packages/core skillSetup + mock UI.

## R11. Updater

**Decision**: **PRESERVE** electron-updater in Main; no-op when unpackaged; no renderer API.  
**Rationale**: FR-039a; `apps/desktop/src/main/updater.ts` functional when packaged.

## R12. Knowledge

**Decision**: Light domain in scope — notes/links/runbook refs; CRUD + associate; no RAG/vectors.  
**Rationale**: FR-011a.

## R13. Deferred runtimes

**Decision**: Web, server, node, cloud org: document only; `tooling` must fail on empty scenic packages.  
**Rationale**: FR-063/XIII.

## R14. Security defaults

**Decision**: contextIsolation true; nodeIntegration false; sandbox when compatible; CSP `default-src 'self'` (refine in tasks); session tests SC-022.

## R15. Testing bar

**Decision**: Contract tests per major local service; role matrix; audit fields; session security; FR-064 e2e; single renderer audit.

## R16. Monorepo

**Decision**: pnpm + Turborepo + go.work; generate before typecheck/build; root `verify`.

## R17. Docs conflict

**Decision**: `docs/architecture.md` Rust/React narrative superseded; rewrite at completion (and early “superseded” banner during branch).

## R18. Async / worker inventory (current → target) — CHK028

Baseline mechanisms found in the existing repo (relevant paths only):

| Mechanism | Location (baseline) | Disposition |
| --- | --- | --- |
| Electron Main process lifecycle | `apps/desktop/src/main/*` | **PRESERVE** (thin Main) |
| electron-updater (Main) | `apps/desktop/src/main/updater.ts` | **PRESERVE** |
| IPC handlers (Main) | `apps/desktop/src/main/ipc.ts` | **MIGRATE TO POLTERWARE-CORE** for ops; Main keeps lifecycle/icons/commander only |
| child_process spawn/exec (runner, processManager, editor, clipboard, declarative status, system) | `packages/core/src/lib/*`, `declarative/*` | **MIGRATE TO POLTERWARE-CORE** (`backend/providers/process` + git/pkg/declarative) |
| node-pty dependency | `packages/core` package.json | **MIGRATE TO POLTERWARE-CORE** process provider (or drop if pure pipes suffice); terminal UI stays presentation-only |
| Process output buffers / streams | `processManager`, workbench types | **MIGRATE TO POLTERWARE-CORE** with events.md bounds |
| Pipeline engine async steps | `packages/core/src/pipeline/*` | **MIGRATE TO POLTERWARE-CORE** workflows domain |
| MCP installer spawnSync (claude mcp add) | `packages/core/src/lib/mcpInstaller.ts` | **REMOVE** from product path if skill-adjacent; MCP **runtime** is new Go process — installer UX not acceptance |
| Vitest worker env | tests only | **NOT APPLICABLE** (test harness) |
| Web Workers | none in product today | **MIGRATE TO WEB WORKER** for canvas layout/heavy parse (`packages/frontend/canvas/worker`) |
| worker_threads | none | **NOT APPLICABLE** unless pure-Go/core needs later |
| Electron utilityProcess | none | **NOT APPLICABLE** this feature |
| Persistent timers / watchers in product for domain | none required | **NOT APPLICABLE** / avoid new Main watchers for ops |
| Renderer async (React effects, promises) | renderer features | **MIGRATE** to Solid stores + Connect streams; no domain in renderer |
| polterware-core as child of Main | new | **MIGRATE TO CHILD PROCESS** (Main supervisor) |

**Normative rule**: heavy layout/parsing → Web Worker; process exec/streams/policy → polterware-core; Main never owns bulk streams.

## R19. Secret lifecycle (completion) — CHK086

**In scope (minimal)**:

- Create/list/delete **SecretReference** records (locator only).
- Use references in ops without loading raw values into audit/logs/UI.
- Backend may resolve via OS keychain/`none` for local fixture — values never written to SQLite as plaintext product default.

**DEFERRED (not acceptance)**:

- Full enterprise rotate/revoke workflows
- TTL/expiry enforcement product
- Multi-backend secret managers
- Shared org secret vaults

## R20. Future node secure defaults (non-acceptance appendix) — CHK105

**Status**: DEFERRED — do not implement packages. When a future feature specifies nodes, defaults MUST include at least:

1. Outbound-only connect from node to control plane (no inbound open admin port by default).
2. TLS required for node↔server (when server exists).
3. Capability-scoped dispatch; human policy/approval for dangerous ops.
4. Bounded queues; no silent success on disconnect.
5. No scenic empty `backend/runtimes/node` until real consumers.

This appendix prevents inventing insecure defaults later; it does **not** expand FR-063.

## R21. Dependency boundary enforcement — CHK049

**TypeScript**: ESLint `no-restricted-imports` (+ optional `eslint-plugin-boundaries`) enforcing:

- apps → features/public packages only
- features ↛ other features' internals
- frontend packages ↛ apps
- web-independent packages ↛ electron
- renderer ↛ node: builtins
- no hand-edit of `packages/generated`

**Go**: `tooling/check-domain-imports.sh` + `go list` architectural tests:

- domains ↛ providers, transport handlers, sqlite drivers, electron
- runtimes → compositions → domains/ports
- cycles fail CI

Both wired into root `pnpm verify`.

## R22. Normative terminology — CHK039

| Term | Meaning |
| --- | --- |
| **Provisioning** | Declarative create/configure of an **external** resource via provider |
| **Operation** | Auditable unit of execution |
| **Workflow** | Ordered composition of operations |
| **Integration** | Provider connection + capabilities (execute \| status \| handoff) |
| **Resource** | Real entity discovered, created, or managed |
| **Command** | Concrete instruction executed inside an operation |

**Provisioning domain package**: **DEFERRED** — no empty domain. Current create/configure stays under **operations** + **integrations** until a real case justifies a domain.

## R23. File-level inventory rigor — CHK018

Package-level inventory is **not sufficient**. Phase 0 MUST produce a **file-level relevant inventory** covering entrypoints, Main, preload, renderer, workers/async, services, persistence, IPC, integrations, scripts, configs, tests, updater, functional assets, legacy stores, React entry, preserve/migrate/remove targets. Trivial static assets and generated code may be omitted. Task T001 delivers `docs/migration/baseline/tree.txt` + file register; T008 disposition covers every row (SC-001).

## R24. Performance budgets (measurement targets) — CHK113

| Metric | Target (default host, local) |
| --- | --- |
| Desktop UI interactive after cold start | ≤ **10 s** (dev) / measure packaged |
| polterware-core Health ready after spawn | ≤ **3 s** |
| Renderer idle memory (soft) | ≤ **512 MiB** |
| Core idle memory (soft) | ≤ **128 MiB** |

Soft budgets are **measurement gates** in `tests/performance/startup.md` — not hard CI fail on all machines; document host and record actuals (SC-014/NFR).

## R25. Future desktop↔web parity — CHK068

**DEFERRED**. Completion does **not** require a desktop↔web parity matrix. When web is specified later, parity will be documented then. Web remains non-acceptance (FR-063).

## Resolved NEEDS CLARIFICATION

None remaining for plan. Spec amend closed C1–C3, H4, H7; checklist amend closed CHK018–CHK153 open gaps at requirements/plan layer.
