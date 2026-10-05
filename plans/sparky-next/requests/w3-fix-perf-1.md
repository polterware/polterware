# Resolve production Solid exports in packaged renderer builds

Owner: W0-SCAFFOLD / shared renderer configuration owner.

The diagnostic `electron-vite build` renderer contains `const IS_DEV = true`, `devComponent`, and Solid development exports. `apps/sparky-desktop/electron.vite.config.ts` hardcodes the `development` export condition for every build. This is independent of the ASAR asset bug and the row optimizations in W3-FIX-PERF. Do not attribute all measured cost to development mode: the captured CPU profile also establishes repeated row construction and expensive per-row checkbox setup.

Requested exact delta in `renderer.resolve.conditions`:

```diff
- conditions: ["solid", "browser", "development", "import", "module", "default"],
+ conditions: ["solid", "browser", "development|production", "import", "module", "default"],
```

Installed Vite 7 expands the special `development|production` condition according to its production setting. Preserve the Solid condition, deduplication and the development workflow. Rebuild, confirm the packaged renderer resolves production Solid code, and rerun the unchanged 10k scroll spec and startup gate from w3-e2e-9. The shared configuration is outside this workstream's ownership; it remains unchanged here. No frame budget or assertion change is requested.
