# Contracts Overview

**Tooling**: Protocol Buffers → Buf → ConnectRPC → generated Go + TypeScript  
**Completion contexts only**: `local`, `events`  
**Deferred (no packages this feature)**: `public`, `internal`, `node`

## Principles

1. Single source of types (constitution V / FR-046).
2. No hand-maintained twins in Desktop/CLI/MCP.
3. Semantic RPCs — no unrestricted `exec` from untrusted surfaces.
4. Buf lint + breaking detection in `verify`.
5. Auth: Bearer ephemeral session token (FR-067). See [session.md](./session.md).

## Generation

```text
pnpm generate → buf lint → buf generate
  → packages/generated (TS)
  → backend/platform/gen (Go)
```

Generated code is **non-editable**. Hand edits are a verify failure.

## Authn metadata

```text
authorization: Bearer <session-token>
```

Token minted by core; distributed via session descriptor file to CLI/MCP; Desktop bridge uses privileged channel without persisting token in renderer storage.

## Error model

Connect codes + `ErrorDetail{ code, message, retryable, correlation_id, details? }`.

### Retry classification matrix (normative — closes CHK117)

| Class | Examples | `retryable` | Client action |
| --- | --- | --- | --- |
| **Transient network** | connection reset, temporary unavailable, connect timeout before accept | **true** | Retry with backoff + idempotency key for mutations |
| **Resource pressure** | `resource_exhausted` (queue full, concurrency cap) | **true** | Retry with longer backoff; surface to UI |
| **Deadline** | operation or RPC timeout | **false** for same op id; new op may be requested | Do not auto-retry same op without user/agent decision |
| **Policy deny** | unauthorized, permission_denied, approval denied | **false** | Never retry without human change |
| **Validation** | invalid_argument, failed_precondition (schema) | **false** | Fix request |
| **Not found / conflict** | missing entity, revision conflict | **false** | Refresh state; user resolve |
| **Internal bug** | internal, data loss | **false** | Report; do not blind retry mutations |
| **Unauthenticated session** | missing/invalid/stale token | **false** | Re-bootstrap session (restart Desktop/core) |

**NFR-004**: Retries MUST NOT double-apply mutations — mutating RPCs that may retry MUST accept `idempotency_key` where defined in [operations.md](./operations.md).

## Stream & payload limits

Normative defaults live in [events.md](./events.md) (chunk 64 KiB, queue 256, log buffer 10k, etc.).

## Terminology (contracts align with domain)

See plan/data-model **Normative terminology**: Provisioning (DEFERRED domain), Operation, Workflow, Integration, Resource, Command.

## Related

- [local-api.md](./local-api.md)
- [operations.md](./operations.md)
- [events.md](./events.md)
- [desktop-bridge.md](./desktop-bridge.md)
- [session.md](./session.md)
