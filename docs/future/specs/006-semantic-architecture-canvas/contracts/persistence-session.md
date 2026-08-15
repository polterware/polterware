# Contract: Persistence & Session

**Inherits**: `005` autosave + layout RPC  
**Wire**: `ArchitectureService.GetLayout` / `UpdateLayout` — **no proto change required**

---

## 1. Save port (existing)

```ts
type LayoutSavePort = {
  save(
    systemId: string,
    expectedRevision: number,
    bodyJson: string,
  ): Promise<{ revision: number }>;
};
```

Body = `serializeDocument(ArchitectureDocument)`.

Optimistic concurrency: revision mismatch → conflict; never silent overwrite.

---

## 2. What triggers autosave

| Event | Schedule save? |
| --- | --- |
| Confirmed node create | yes |
| Confirmed relationship | yes |
| Rename / type change / delete | yes |
| Move (gesture end) | yes (coalesced) |
| Align / layout / paste | yes |
| Viewport-only pan/zoom | optional (prefer include in document viewport; debounce longer or with next doc save) |
| Catalog open | **no** |
| Placement / provisional | **no** |
| Pending relationship | **no** |
| Highlight mode | **no** |
| Favorites toggle | **no** (local prefs) |

---

## 3. Status UX

| Status | User-visible |
| --- | --- |
| clean | (empty or idle) |
| dirty | Unsaved |
| saving | Saving… |
| saved | Saved (brief) |
| error | Save failed + **Retry** |

Retry: `flush` same body + current expected revision (or refetch policy on conflict).

---

## 4. Soft-fail (mandatory)

On ConnectError / network failure:

- **Do not** set full-page Architecture error that unmounts canvas
- Keep local `doc` intact
- Show status error and/or single toast
- Allow continued local editing and retry

Discovery must not rewrite layout silently (existing domain rule).

---

## 5. Manual save

- Not primary CTA
- If present: secondary “Save now” calling same port
- Must not be required for happy path

---

## 6. Session snapshot (detail round-trip)

```ts
type SessionSnapshot = {
  viewport: Viewport;
  selection: SelectionState;
  environmentId?: string | "all";
};
// keyed by systemId; in-memory Map
```

| When | Action |
| --- | --- |
| Open details | `setSessionSnapshot` + optional autosave flush |
| Return to index | Restore viewport into doc/host; restore env filter; **restore selection** filtering to ids that still exist in the document |
| System switch | Use other key; do not leak selection |

---

## 7. Import / export

| Format | Contract |
| --- | --- |
| JSON document | validate + migrate; preserve type ids |
| SVG / PNG | Honest geometry export (005); type labels when feasible |

Import never executes provider side effects.

---

## 8. App composition duties

`App.tsx`:

- Wire getLayout / updateLayout
- Soft-fail save errors (toast once or status only)
- Route open details / back
- Do not own type registry
