# W3-POLISH request 4: duplicate Map settings IDs

Owner: W2-SETTINGS / W2-MAP / integrator. Status: open.

sparky-map registers { id: "map", label: "Tiles", group: "Map" }. sparky-settings registers { id: "map", label: "Map", group: "App" }. activeSettings uses find(id), so choosing App > Map renders the first Map > Tiles panel. Map provider and Address search controls are unreachable in the integrated Settings dialog.

Requested change: use one canonical Map panel, or assign unique IDs (e.g. map-tiles and map) and update selection/registry integration tests. Verify that App > Map reaches Address search and provider, and that selected rail state is unique. W3-POLISH captures the integrated behavior honestly and separately previews the unreachable panel for visual review without changing production IDs.
