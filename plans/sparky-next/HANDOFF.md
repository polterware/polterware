# Handoff (2026-10-04) — Wave 0 done, Wave 1 not started

Written so a fresh session (opened inside this repo) can continue as **integrator**. Delete with the plan at W3-CLEANUP.

## Where things are
- Repo: this one (`polterware`). Integration branch: `sparky-next` (checked out in worktree `../polterware-wt/integration`). Plan: `plans/sparky-next/`.
- Workstream worktrees: `../polterware-wt/<ws-id>`, branches `sparky/<ws-id>`. Old Swift app / Bun MCP (read-only reference): `/Users/erickpatrickbarcelos/codes/sparky-workspace/{sparky,sparky-mcp}`.
- Launcher: `plans/sparky-next/tools/run-codex.sh` (logs in `~/.sparky-next-logs/`). Check progress by log size/banner and child processes, not by waiting.

## State
- Wave 0 fully merged into `sparky-next`; `bash tooling/verify-sparky.sh` green. Gate G0 passed technically; **Erick must approve Wave 1**.
- Contracts: local tags `sparky-contracts-v1` (also on remote) and `sparky-contracts-v1.1` (local only; text-only search). Use v1.1.
- Decisions of this session (see `00-decisions.md` resolutions log): Go core, Electron+Solid, official go-sdk v1.8.0, MapLibre+OpenFreeMap, FTS5 only (no semantic/Ollama), native N-API reminders module in Electron Main (no launchd helper, ad-hoc signing OK), no users/migration.

## Rules Erick set
- Agents: Codex only from Wave 1 (`gpt-6.1-sol` for hard WS, `gpt-6-luna` for light ones), reasoning effort **low..high, never xhigh/max/ultra**. Always `-c model_provider=openai` (his `~/.codex/config.toml` points to local Ollama) and close stdin (`< /dev/null`).
- Agents never push or tag (except a WS that says "local tag"). Remote currently has 8 `sparky/w0-*` branches and tag `sparky-contracts-v1` (repo is private): **ask Erick before deleting**.
- Pending questions for Erick: (1) approve Wave 1; (2) delete the remote branches/tag?; (3) apply "quit Electron when the window closes" (~120 MB hidden vs ~50-70 MB total) and the cheap perf guardrails (calendar cap per memory, lazy planner + debounce, lazy images); (4) keep or revert the opacity-lint allowlist widening made by W0-SCAFFOLD (DESIGN.md + tooling/check-ui-opacity.mjs).

## Wave 1 launch table (model / effort)
W1-RECURRENCE, W1-CORE, W1-MCP, W1-DS, W1-NATIVE-NOTIFY: gpt-6.1-sol / high. W1-DOMAIN, W1-STORE, W1-ELECTRON: gpt-6.1-sol / medium.
W1-SEARCH, W1-ATTACH: gpt-6-luna / medium. W1-ASSETS: gpt-6-luna / low. Wave file: `08-wave1.md` (6th argument).
Merge order is in `08-wave1.md`. As integrator: verify each branch yourself (ownership check with `git diff --name-only sparky-next...<branch>`, rerun its Verify), merge `--no-ff`, run `verify-sparky.sh`, then stop at G1 for Erick.
Lessons: spikes aren't blocked by unrelated gate failures; ownership blockers must be resolved by the integrator (extend Owns, rerun).
