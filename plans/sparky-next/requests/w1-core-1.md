# W1-CORE dependency wiring

- Owner: W1-DOMAIN, W1-STORE, W1-RECURRENCE, integrator
- Blocked sub-task: production SQLite/domain/planner wiring and real-database RPC acceptance.
- Evidence: this worktree contains repository and service interfaces only; the repository fakes panic, `providers/sqlite` contains only the frozen baseline, and `domains/schedule` has no planner implementation.
- Required change outside W1-CORE Owns: deliver the W1-DOMAIN services/fakes, W1-STORE database/repositories/change feed, and W1-RECURRENCE planner in their existing owned paths. No contracts need to change.
- Integration seam: populate `compositions/core.OpenRuntime` with the resulting constructors and close callback, adapting the store's committed change feed to `ChangeFeed`. Populate `Services` with domain services, `ScheduleUseCases`, and relation readers. Do not report an unavailable backend as a working database.
- Exact shared change: none until the provider constructor APIs exist. W1-CORE deliberately does not invent or edit another workstream's APIs.

The production wiring sub-task is stopped pending these merges. Transport and process lifecycle proceed independently using injected service ports.
