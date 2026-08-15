# Benchmark notes (T118)

**Date**: 2026-08-03

Generator: `packages/frontend/canvas/src/bench/generate-doc.ts`

| Scenario | Nodes | Edges | Notes |
| --- | --- | --- | --- |
| Default | 500 | 1000 | Target usability (pan/zoom/drag selection) |
| Stress | 1000 | 2000 | Documented stress path |

## Method

1. `generateDoc(n, e)` builds synthetic ArchitectureDocument in memory.  
2. Host uses viewport culling (`visibleDocumentRect` + node AABB).  
3. Edge updates use SVG paths; only world transform for pan/zoom.  
4. History uses structured document snapshots with eventually coalescing for drag.

## Results (implementation baseline)

- Unit/integration pure-path tests pass without DOM.  
- Interactive FPS to be recorded on Desktop when operator runs manual [quickstart.md](./quickstart.md) §5.6.  
- Mitigations in place: culling, LOD zoom thresholds, debounced autosave (800ms), no serialize on pointermove.

## Follow-up

Record wall-clock pan/zoom session on M-series / common desktop and attach numbers before release claim.
