# W3-POLISH request 5: memory list rendering and query failure recovery

Owner: W2-MEMORIES / W2-MINDS / design-system owner. Status: investigate.

In the browser fake-bridge audit the Memories counter reports 8 entries while virtualized DataTable renders an empty tbody at 800, 1280 and 1920 widths. Non-virtualized Search and Review tables render. This is browser-harness evidence, not a packaged Electron reproduction. First rule out bundler/harness configuration, then verify the virtualizer lifecycle/count/scroll-element wiring against the packaged renderer; fix the rendering path with an integration regression if confirmed.

A second fake-bridge case rejecting every memories.list call (including limit:1 Mind counts) reaches the top-level ErrorBoundary through MindsSidebarTree's count resource, removing the shell. A feature-only query failure still retains the shell. Requested behavior: guard failed count resources, retain navigation and retry UI, and add a failed-count recovery test.

W3-POLISH preserves data/virtualizer behavior and stores actual empty-list screenshots. Screenshot evidence must not be counted as populated-list/10k virtualization acceptance until verified.
