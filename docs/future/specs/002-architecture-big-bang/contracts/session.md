# Contract: Local Session (FR-067)

**Feature**: `002-architecture-big-bang`  
**Status**: Normative for Desktop, CLI, MCP

## Lifecycle

1. Desktop Main starts `polterware-core`.
2. Core binds `127.0.0.1:<ephemeral>`.
3. Core mints session token (cryptographically random, process-lifetime).
4. Writer (Main supervising core, or core under Main) creates **session descriptor file**.
5. Clients (Desktop bridge, CLI, MCP) authenticate with `Authorization: Bearer <token>`.
6. On clean shutdown: delete descriptor; invalidate token.
7. Stale: pid not running **or** Health fails → reject requests and cleanup descriptor.

## Session descriptor file

| Field | Required |
| --- | --- |
| endpoint | yes (`http://127.0.0.1:PORT`) |
| token | yes |
| pid | yes |
| started_at | yes (RFC3339) |

**Path**: OS runtime dir (e.g. `$XDG_RUNTIME_DIR/polterware/session.json` or macOS/Windows equivalent documented in local-setup).

**Permissions**: Unix mode **`0600`**; Windows ACL **current user only**.

**MUST NOT**:

- log the token (including debug logs in fixtures)
- store token in renderer `localStorage` / permanent preferences
- put token in process argv
- prompt interactively for the token in the **default** CLI/MCP path

## CLI / MCP token bootstrap UX (normative — closes CHK059)

| Client | Bootstrap mode | Behavior |
| --- | --- | --- |
| **Desktop** | Privileged Preload bridge | Main/core writes file; bridge reads token without exposing raw token to renderer storage |
| **CLI** | **File mode only** (default) | Read session descriptor from well-known path; validate perms + pid + Health; exit non-zero with structured human/`--json` error if missing/stale |
| **MCP** | **File mode only** (default) | Same as CLI; server process fails closed at startup if session unusable |

### Normative choices

1. **Default = file mode** — no interactive password/token prompt required for normal local operator use.
2. **Optional override**: `POLTERWARE_SESSION_FILE=/path` env var for tests and non-default locations.
3. **Optional override**: `POLTERWARE_ENDPOINT` + `POLTERWARE_TOKEN` **only for automated tests** — MUST NOT be recommended in operator docs as primary path; production packaging uses file mode.
4. **No prompt mode** in completion set — interactive token paste is **DEFERRED** (not acceptance).
5. **Fail closed**: missing file, bad perms, dead pid, health fail, invalid token → deny; never invent a token.

### Operator UX copy (docs must state)

```text
Start Polterware Desktop (or polterware-core under Desktop supervision) so the session file exists.
Then run: polterware health
```

## Client validation sequence

1. Resolve path (`POLTERWARE_SESSION_FILE` or platform default).
2. Read file; reject if unreadable or wrong mode/ACL (where OS allows check).
3. Parse endpoint, token, pid, started_at.
4. Check pid alive (when OS allows).
5. `Health.Check` with Bearer token.
6. On any failure → unauthorized / not ready; do not retry with guessed credentials.

## Tests required (SC-022)

- File permissions (Unix 0600)
- Invalid token rejected
- Stale pid rejected
- Dead process rejected
- Cleanup on shutdown
- Token absent from log sinks in fixture
- CLI fail-closed without Desktop/core
- MCP fail-closed without valid session
- Env override path used only when set
