# Research: Resources Process Detail

**Feature**: `021-process-detail`  
**Date**: 2026-08-12

## Baseline

- `process-detail.tsx` is a ~100-line stub: raw status string, always-visible control actions, no PageLoading/PageError, no StatusChip, no host hand-off, no tests.
- `PROCESS_FIXTURES` covers Running processes with partial fields; missing explicit Stopped/Unknown and a deliberate sparse composition for coverage.
- Processes table prints status as plain text.

## Peer patterns (reuse)

| Peer | Package | Reuse |
| --- | --- | --- |
| Nodes detail | `feature-resources` | loading/error/not-found shows, StatusChip metrics, source honesty, action gating |
| Inventory detail | `feature-resources` | mock toast when no live handler, optional hand-off callbacks |
| Run detail | `feature-operations` | status helpers + Show-gated danger/primary actions |

## Decisions

1. **Dedicated module stays** — elevate `process-detail.tsx`; do not fold into index or node-detail.
2. **Status vocabulary** — keep `Running | Stopped | Unknown` (fixture contract). Labels capitalize; tones: Running=healthy, Stopped=muted, Unknown=unknown.
3. **Action honesty** — View logs + Open terminal available as mock/hand-off for all known processes; Restart and Stop only when status is Running. When callbacks omitted, `pushToast` acknowledges mock intent.
4. **Host open-node** — optional `onOpenNode(nodeId)` switches to Resources Nodes detail for the host; does not reimplement node-detail.
5. **CPU visualization** — `DitherSegmentedBar` only when `cpuPercent` is a finite number; no history series.
6. **Related context** — optional fixture fields only (`relatedComponent` / `relatedResource` / `relatedOperation`); omit section when all absent.
7. **Not-found mount** — mirror Nodes: show ProcessDetail when processes tab is active and scope target exists or is missing, so not-found renders.

## Alternatives rejected

- Inventing time-series CPU charts from a single percent sample.
- Claiming live stop/restart success.
- Second process detail page or drawer.
- Merging process detail into node-detail “processes on host” list as the primary inspector.
