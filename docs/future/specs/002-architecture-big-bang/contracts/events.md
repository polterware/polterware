# Contract: Events & Streams

**Feature**: `002-architecture-big-bang`  
**Status**: Normative for completion set (local-first)

## Envelope

| Field | Required |
| --- | --- |
| event_id | yes |
| type | yes |
| at | yes (RFC3339) |
| correlation_id | yes |
| operation_id | when tied to an op |
| principal_id | when actor known |
| system_id | when in system context |
| sequence | yes for ordered streams |

## Types (completion)

`operation.state_changed` · `operation.progress` · `operation.log` · `process.output` · `process.exited` · `audit.recorded` · `approval.updated` · `discovery.completed` · `store.health`

## Numeric limits (defaults — configurable, not immutable constants)

These are **initial defaults**. Implementations MUST load them from configuration (with these values as defaults). Changing defaults does not require a contract version bump if behavior remains bounded and signalled.

| Limit | Default | Behavior on exceed |
| --- | --- | --- |
| stdout chunk max | **64 KiB** | Split into multiple events; single event payload MUST NOT exceed max |
| stderr chunk max | **64 KiB** | Same as stdout |
| Event buffer per operation | **256** messages | Backpressure: block producer or return structured `resource_exhausted`; **never** unbounded growth |
| Visible log buffer (renderer) | **10_000** entries | Drop-oldest with **explicit truncation signal** in UI/API metadata |
| Render batch size | **up to 200** entries | UI must batch apply; no full re-render of entire buffer each tick |
| Max single log line | **1 MiB** | Truncate line with `truncated=true` marker; do not reject whole stream |
| Max non-streaming ConnectRPC payload | **4 MiB** | Structured `invalid_argument` or `resource_exhausted` |
| Default operation timeout | **10 minutes** | Terminal `timeout` error; cancel underlying process when applicable |
| Local heartbeat | **NOT USED** this feature | N/A — no product heartbeat protocol on local loopback |
| Max time without stream consumer | **30 seconds** | Pause or cancel producer stream; operation may continue if independent of UI subscription |
| Max concurrent local operations | **4** default | Additional requests queue or deny with `resource_exhausted` |
| Max simultaneous terminal sessions | **4** default | Additional open denied with structured error |

### Buffer rules (normative)

1. **All buffers are bounded** — unbounded in-memory queues are forbidden.
2. **Logs MUST NOT grow indefinitely in memory** — enforce visible buffer + optional disk overflow only if explicitly designed; completion uses bounded in-memory + drop-oldest.
3. **Overflow MUST be signalled** — drop-oldest for logs only; never drop audit events.
4. **Secrets MUST be redacted** before entering log buffers, stream events, or audit payloads.
5. **Streams MUST honor cancellation** and backpressure from the consumer.
6. **Exceeding limits** produces either a structured error (`resource_exhausted`, `invalid_argument`, `deadline_exceeded`) or explicit truncation metadata — never silent corruption of audit.

## Rules

- Chunked payloads only for process/log streams.
- Client cancel of a **subscription** stops the stream; it does **not** imply operation cancel unless the client also calls `CancelOperation` (or equivalent linked cancel).
- Optional `since_sequence` for replaying in-memory buffers within bounds.
- Correlation IDs propagate across Desktop / core / CLI / MCP.

## Required tests (map to SC-010, NFR-003/005, FR-049)

| Case | Expect |
| --- | --- |
| Slow consumer | Producer pauses or drops per policy; no OOM; no unbounded queue |
| Full buffer (256) | `resource_exhausted` or controlled block; no silent loss of audit |
| Excessive line (>1 MiB) | Line truncated + signal; stream continues |
| Excessive non-stream payload (>4 MiB) | Reject with structured error |
| Operation timeout (10m default) | Terminal timeout error |
| Cancel subscription | Stream ends; op independent unless linked cancel |
| Truncation signal | Client can observe truncation flag/metadata |
| No unbounded growth | Fixture fills faster than consumer; memory stays within bound |

See also: [overview.md](./overview.md) error model · [operations.md](./operations.md)
