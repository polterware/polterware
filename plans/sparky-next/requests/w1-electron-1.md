# W1-ELECTRON request 1: real core integration

Owner: W1-CORE / integrator. Blocking acceptance: real-core dev boot, one detached spawn, restart/reconnect, and `window.sparky.memories.list()`.

This worktree contains only `sparky/backend/runtimes/cli/main.go` with stub verbs and no runstate implementation. The stub exits without an API or `core.json`. W1-ELECTRON must not implement another workstream's core.

Please merge W1-CORE and dependencies, then rerun W1-ELECTRON smoke against a temporary database (never the user's real data).
Confirm the frozen desktop integration conventions:
- CLI arguments: `sparky -db <path> serve --listen 127.0.0.1:0`.
- `<db directory>/run/core.json`: `pid`, `endpoint`, `token`, `dbPath`, mode 0600.
- Health `build`: `<binary mtime unix seconds>-<bytes>`; Health `dbPath` identifies the database.
- Maintenance has no Shutdown in v1.1: desktop sends SIGTERM only after authenticated health/database validation, waits, then attaches/starts the replacement.

No contract edits requested. The real-core validation sub-task is stopped until this dependency is present.
