# Quickstart Validation Guide

**Feature**: `002-architecture-big-bang`  
**Scope**: Desktop + polterware-core + CLI + MCP (local-first)  
**Related**: [plan.md](./plan.md) · [contracts/](./contracts/) · [spec.md](./spec.md)

> Target monorepo commands. Legacy conf/`.polterware` data is **DISCARD** — start clean.

---

## Prerequisites

- macOS (primary) or Linux/Windows for core  
- pnpm, Go, Buf  
- Optional: git + package manager for FR-064 fixtures  

---

## 1. Install & generate

```bash
pnpm install
pnpm generate
```

**Expect**: generated TS/Go clients; buf clean.

---

## 2. Backend unit + integration

```bash
go test ./backend/...
pnpm test:integration
```

**Expect**: domains, SQLite, FR-065, session security (SC-022), role denies (SC-007), audit fields (SC-006), stream limits, retry class.

---

## 3. Security

```bash
pnpm test:security
```

**Expect**: isolation; token perms/stale/invalid; no token in logs; MCP parity; Viewer/Agent denies.

---

## 4. Dev desktop

```bash
pnpm dev:desktop
```

**Expect**:

1. Main starts polterware-core (loopback + session file)  
2. Health OK  
3. **Single** Solid product UI (post-cutover)  
4. Core kill → visible error  
5. Unpackaged: updater no-op  

---

## 5. E2E scenarios

```bash
pnpm test:e2e
```

| # | Scenario | SC |
| --- | --- | --- |
| E1 | System map from core not mock | SC-002 |
| E2 | Inspector + knowledge CRUD associate | SC-024 |
| E3 | Real git or pkg op + full audit fields | SC-006/019 |
| E4 | Process start → logs → stop; UI responsive | SC-010/019 |
| E5 | Local pipeline | SC-019 |
| E6 | Declarative plan | SC-019 |
| E7 | Approval + role deny | SC-007 |
| E8 | Offline no cloud | SC-008 |
| E9 | Single renderer / no react product dep | SC-012/017 |
| E10 | DISCARD: no legacy conf required | SC-009 |

**Cutover rule**: E2E (this section / T145) MUST pass on the Solid product path **before** React product tree purge (T138/T139).

---

## 6. CLI (command-complete — CHK136 / SC-005)

Session file is written when Desktop (or supervised core) is running. Default path: platform runtime dir (`$XDG_RUNTIME_DIR/polterware/session.json` or OS equivalent). Override: `POLTERWARE_SESSION_FILE`.

```bash
# Require: Desktop or polterware-core already up with valid session file
polterware health
polterware health --json

polterware systems list
polterware systems list --json

# Request a local op (fixture: git status in a temp repo or allowed pkg script)
polterware ops request --type git.status --system <system-id> --cwd <path>
polterware ops request --type git.status --system <system-id> --cwd <path> --json

polterware ops get --id <operation-id>
polterware ops get --id <operation-id> --json

polterware ops cancel --id <operation-id>

polterware pipelines run --id <workflow-id>
polterware knowledge list --system <system-id>
```

**Fail-closed fixture** (no Desktop):

```bash
env -u POLTERWARE_SESSION_FILE polterware health ; test $? -ne 0
```

**Parity**: same op allow/deny as Desktop suite (SC-005). Human + `--json` formatting tests (T095/T096).

---

## 7. MCP (command-complete — CHK136 / SC-015)

```bash
# MCP server reads the same session file; fails closed if missing/stale
polterware-mcp   # or documented binary name from packaging
```

Tool surface (min-scope — OperationTypes only; **no skills tools**; **no generic exec**):

| Tool | Maps to | Notes |
| --- | --- | --- |
| `polterware.health` | Health.Check | |
| `polterware.ops.request` | OperationService.Request | agent principal; may await approval |
| `polterware.ops.get` | OperationService.Get | |
| `polterware.ops.cancel` | OperationService.Cancel | if capability allows |
| `polterware.systems.list` | Project/System list | |

**Agent rules**: registered as Agent role; cannot self-approve; out-of-scope tools absent. Security suite: `tests/security/mcp_parity_test.go` (T100).

**Fail-closed**: start MCP without session file → exit non-zero / no tools executable.

---

## 8. Performance

```bash
pnpm test:performance
```

SC-010 + startup measurements (NFR-014).

---

## 9. Verify / sign-off

```bash
pnpm verify
go test ./backend/...
```

Plus: scenic package check; legacy import grep; domain import check; boundary lint; docs match architecture.

---

## 10. Contributor timing (SC-014)

Clean clone → install → generate → dev:desktop ≤ 60 minutes with docs only.

---

## Troubleshooting outline (CHK143)

Full runbook: `docs/development/troubleshooting.md` (T165). Required topics:

| Symptom | First checks |
| --- | --- |
| `polterware health` fails | Session file exists? perms 0600? core pid alive? Desktop started? |
| Desktop blank / core error | Main logs; core stderr; quarantine FR-065 |
| Corrupt store | Health isolate; Maintenance ResetLocalStore; DISCARD user-reset doc |
| Token in logs | Fail security test; never log token |
| `resource_exhausted` | Concurrent ops >4 or stream buffer full — retry later or raise config |
| Stream stalled | Consumer idle >30s; resubscribe |
| Approval stuck | Role must be Owner/Operator with approve capability; Agent cannot self-approve |
| React still present | Cutover incomplete — T138/T139/T183 |

---

## Rollback

Git revision only (FR-066). No dual-stack product switch.

---

## Out of scope for this guide

Web app · polterware-server · polterware-node · PostgreSQL · pgx · sqlc · NATS · cloud orgs/memberships/teams · cloud sync
