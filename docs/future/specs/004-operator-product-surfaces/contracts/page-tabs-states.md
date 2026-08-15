# Contract: Page Tabs & System States

**Owner**: `@polterware/frontend-design-system` + feature pages  
**Consumers**: All seven destinations

## Tabs

```ts
type PageTabsProps = {
  items: { id: string; label: string; disabled?: boolean }[];
  value: string;
  onChange: (id: string) => void;
};
```

Rules:

1. Rendered **inside** MainSurface (below page header or as page toolbar), never in sidebar.  
2. Keyboard: arrow keys within tablist; Tab moves into panel.  
3. Changing tab clears detail page state (section index) unless product explicitly preserves (default: clear).  
4. Active tab uses neutral raised treatment—not brand green fill.

## System states

| State | Component | Required props |
| --- | --- | --- |
| Loading | `PageLoading` | optional label |
| Empty | `PageEmpty` | title, description?, action? |
| Error | `PageError` | message, onRetry? |
| Offline | `PageOffline` | message (cached?), onRetry |
| Forbidden | `PageForbidden` | permission name?, onRequestAccess? |
| Fixture | `FixtureBanner` | none |

Rules:

1. Loading MUST NOT look like finished empty product.  
2. Offline MUST mention cached data when cache is shown.  
3. Forbidden MUST name the capability when known (e.g. `operations.execute`).  
4. Status chips NEVER color-only.

## Dense tables

Minimum interaction contract for list tabs:

- Row activation → detail or drawer per entity policy  
- Search input bound to `route.search`  
- Filter chips/select bound to `route.filter`  
- Optional pagination footer text (“Showing 1–N of M”)

Virtualization allowed via `@tanstack/solid-virtual` when row counts are large.
