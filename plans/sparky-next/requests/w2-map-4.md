---
summary: Add expectedRev to PlaceService Set and Clear
rationale: Feature packages send expectedRev as a string on writes. PlaceService.Set/Clear bump the memory revision but the frozen v1.4 proto has no expectedRev field, so the core cannot reject a stale Place save.
---

W2-MAP requests a contracts follow-up (Erick / W1-CORE):

Add optional `int64 expected_rev = 2` to `SetPlaceRequest` and `ClearPlaceRequest`. The core should compare it to the memory revision before `Set`/`Clear` (same conflict model as `MemoryService.Update`).

W2-MAP already sends `expectedRev` as a decimal string on both RPCs (validated locally; unknown fields are ignored on the current wire). The UI loads the memory `rev` first. Until this lands, stale Place saves are last-write-wins.

W2-MAP does not edit `sparky/schemas/**` or generated contracts.
