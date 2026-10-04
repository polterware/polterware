# w1-domain-b status

- Result: done
- Branch / last commit: `sparky/w1-domain-b` / `sparky(w1-domain-b): add status`
- Acceptance criteria:
  - ≥ 90% statement coverage in `domains/{memory,mind,tag,review}` → **pass** (94.5%, 99.1%, 98.6%, 100.0% via `go test ./domains/memory/... ./domains/mind/... ./domains/tag/... ./domains/review/... -cover`)
  - Table tests for Mind/Memory/Tag/Focus/Review/Activity/Settings rules → **pass** (`service_test.go` in each owned domain)
  - Review reproduces Swift mood thresholds and day periods → **pass** (`TestSwiftMoodThresholds`, `TestSwiftPeriodsAndStreak` in `review/service_test.go`)
  - `check-sparky-boundaries.sh` green → **pass** (`bash tooling/check-sparky-boundaries.sh`)
  - Verify command → **pass** (`cd sparky/backend && go test ./domains/... -race -count=1 -cover`, exit 0)
- Deviations from the plan (and why):
  - Plan **Do** lists "in-memory fakes of every repository port"; **Owns** limits this WS to memory, mind, tag, focus, review, activity, settings. Fakes for attachments, place, maintenance, search, reminder, schedule, clock, ids, tx remain stubbed — see request `plans/sparky-next/requests/w1-domain-1.md`.
  - Shared `tests/repotest/**` suite is W1-STORE scope; owned fakes carry contract tests beside memory, mind, and tag fakes; focus, settings, and review fakes are exercised through `activity/fake/testkit` and service tests.
- Requests filed (plans/sparky-next/requests/...): `w1-domain-1.md` (non-owned repository fakes)
- Follow-ups for other WS:
  - **W1-STORE**: implement SQLite repos; run shared `tests/repotest` against SQLite and owned fakes.
  - **Owners of schedule, reminder, attachments, place, search, maintenance, clock, ids, tx**: implement remaining fake ports per `w1-domain-1.md`.
