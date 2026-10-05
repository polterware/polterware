# Nonempty 10k list misses the scroll frame budget

Owner: W3 performance / shared table and Memories owners.

The virtualizer mount fix now renders a real subset: 19 rows out of 10000. The existing `10k memories virtualization and list scrolling FPS trace` reaches tracing and captures 180 animation frames, but reports average 21.978770949720655 ms, 45.49844949417928 FPS and p95 26 ms. The unchanged assertions require average ≤17.2 ms and p95 ≤20 ms. No test assertion or virtualization behavior was disabled.

Evidence: `apps/sparky-desktop/test-results/w3-fix-lists-perf/results.json` contains the `scroll-chromium-trace` and `scroll-performance` attachments; the failure Playwright trace is under `w3-fix-lists-perf/artifacts/perf-desktop-10k-memories--e1bc0-nd-list-scrolling-FPS-trace/trace.zip`. This is an unsigned diagnostic directory package with CLI `--config.asar=false` because the standard package is independently blocked by [request 1](w3-fix-lists-1.md). It uses the real isolated core, SDK seed, Main, preload and renderer, with the existing notification stub. Machine: arm64, macOS 26.6.2, Electron 44.4.5.

Please profile the captured main-thread/layout work and rerun after the standard ASAR asset path is repaired. Do not count an empty list, relax frame limits, disable virtualization or claim a standard-package pass from this diagnostic variant. W3-FIX-LISTS implements the requested row/contract fixes and records the first measurable FPS result; this follow-up tracks performance acceptance separately.
