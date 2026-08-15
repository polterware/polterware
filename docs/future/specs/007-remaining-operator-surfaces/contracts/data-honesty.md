# Contract: Data Honesty (007)

**Owner**: All feature packages + App adapters  
**Extends**: 004 core-data-adapter honesty

## DataSource

```ts
type DataSource = "core" | "fixture" | "empty" | "cached" | "unavailable";
```

| Value | UI expectation |
| --- | --- |
| core | Live from polter-core / connected adapters |
| fixture | Explicit demo/fixture labeling |
| empty | Empty state + setup CTA when applicable |
| cached | “Last known” labeling |
| unavailable | Capability not offered (edition, disconnected provider, unimplemented runtime) |

## Surface-specific rules

1. **Observability**: fixture metrics/traces must not read as live production APM.  
2. **Billing**: use `unavailable` when commercial plan does not apply—never invent charges.  
3. **Developer MCP/CLI**: status reflects process reality; “Running” only if running.  
4. **Sync**: offline queue count must not claim provider applied until confirmed.  
5. **Notifications**: fixture inbox labeled when not from real notification store.  
6. **Provisioning executions**: step progress from fixtures must say demo when not core-backed.

## Banner placement

Prefer page-level subtle banner once per surface, not per-row noise, except status chips that already encode health.
