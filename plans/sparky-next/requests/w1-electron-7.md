# W1-ELECTRON request 7: Turbo passthrough for SPARKY_CORE_BIN

Owner: W0-SCAFFOLD / integrator. Optional smoke convenience; not blocking.

Main now honors absolute `SPARKY_CORE_BIN` so isolated verification can point the supervisor at a temp `go build` without writing `dist/sparky`. `turbo.json` `dev.passThroughEnv` already lists `SPARKY_CORE_DB`, `SPARKY_CORE_ENDPOINT`, `SPARKY_CORE_TOKEN`, `REMOTE_DEBUGGING_PORT`, and `ELECTRON_CLI_ARGS`.

Please add `"SPARKY_CORE_BIN"` to that array so `pnpm dev:sparky` keeps the override under Turbo strict env. Until then, use `pnpm --filter @polterware/sparky-desktop exec electron-vite …` or `pnpm dev:sparky --env-mode=loose`.
