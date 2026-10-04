# W1-ELECTRON request 5: Turbo dev environment passthrough

Owner: W0-SCAFFOLD / integrator. File: `turbo.json` (outside Owns).

Smoke found Turbo strict environment filtering silently drops `SPARKY_CORE_*` overrides and Electron debugging arguments. Please add to the dev task:

```json
"passThroughEnv": ["SPARKY_CORE_DB", "SPARKY_CORE_ENDPOINT", "SPARKY_CORE_TOKEN", "REMOTE_DEBUGGING_PORT", "ELECTRON_CLI_ARGS"]
```

Main implements `SPARKY_CORE_DB` (absolute DB path; runstate/attachments derive from its directory) for isolated smoke/test data, and paired loopback endpoint/token overrides. Until this shared change lands, use `pnpm dev:sparky --env-mode=loose` or the direct desktop package command for overridden environments. The root configuration sub-task is stopped.
