# W1-ELECTRON request 6: Solid dependency transforms in scaffold config

Owner: W0-SCAFFOLD / integrator. File: `apps/sparky-desktop/electron.vite.config.ts` (explicitly outside Owns).

`pnpm --filter @polterware/sparky-desktop build` compiles Main and preload, then fails on the renderer:

> @corvu/utils/dist/chunk/UKBPAEF5.jsx: "jsx" is not exported by solid-js/dist/solid.js.

The Polterware config already handles this dependency. Exact proposed patch in Sparky's config:

```diff
           /node_modules[\\/]@ark-ui[\\/]solid[\\/].*\.jsx$/,
+          /node_modules[\\/]\.pnpm[\\/].*@corvu\+.*[\\/].*\.jsx$/,
+          /node_modules[\\/]@corvu[\\/].*\.jsx$/,
```

```diff
-      exclude: ["@ark-ui/solid"],
+      exclude: ["@ark-ui/solid", "@corvu/resizable", "@corvu/utils"],
```

```diff
-      noExternal: ["@ark-ui/solid"],
+      noExternal: ["@ark-ui/solid", "@corvu/resizable", "@corvu/utils"],
```

Then rerun desktop build and dev smoke, waiting for the stub renderer body to mount. CDP verified preload isolation and validated deep-link dispatch, but did not establish that the stub UI mounted: its body was empty at the observation. No renderer boot claim is made. This gate failure belongs to the shared scaffold; its fix/verification sub-task is stopped per protocol §5.
