# Apply the search query carried by a packaged deep link

Owner: Sparky Search / Shell feature owner. Product files are outside W3-PACKAGING ownership.

Observed in the real ad-hoc-signed release ZIP (Electron 44.4.5): a second instance of `Sparky.app/Contents/MacOS/Sparky`, with the same isolated `--user-data-dir` and `--sparky-e2e-env` as the running app, receives `sparky://search?q=packaging`. The visible main route changes to `aria-label="Search"`, but the `Search memories` searchbox stays empty for the full five-second assertion timeout.

Main parses the route and forwards the URL; Shell `routeLink` appends `url.search` to the route. Inspect `packages/features/sparky-search/src/search-page.tsx` / its store for consuming the `q` route parameter. Preserve existing FTS-only behavior and validation. Add behavior coverage in the feature's owned tests; W3-PACKAGING does not edit specs or features.

Expected: the actual search input/query becomes `packaging`, and search executes as if entered manually. Do not count route navigation alone as query propagation.

Evidence: initial smoke report was written under `apps/sparky-desktop/release/0.1.0/smoke-report.json` and the failed synthetic profile was `/private/var/folders/bg/zjwnpvh52wg23tybw4xm2kt40000gn/T/sparky-w3-packaging-mNcLTq/sample-0`. The final smoke retains this separate query result while also verifying the memory detail route; it does not turn the search query defect into a pass.
