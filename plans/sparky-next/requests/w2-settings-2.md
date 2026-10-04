---
summary: Allow loopback MCP HTTP from the renderer for Test connection
rationale: Agents > Access Test connection must POST initialize + tools/list to http://127.0.0.1:<port>/mcp. CSP connect-src is currently 'self' (plus OpenFreeMap when map.online).
---

Owner: W1-ELECTRON / integrator.

Requested CSP addition, loopback only:

```diff
- `connect-src 'self'`
+ `connect-src 'self' http://127.0.0.1:*`
```

Do not add `http://localhost:*` unless Host-header checks in the MCP guard already allow it. Keep the rest of the CSP unchanged.

Until this lands, Settings still reports failure honestly when MCP is off (no network). When MCP is on, a blocked fetch is shown as a failed test with the browser error text.
