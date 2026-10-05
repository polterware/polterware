# W1-RECURRENCE: split the verification command for Go fuzzing

- Owner: integrator (plan files are outside W1-RECURRENCE Owns).
- Evidence: the exact mandated command `go test ./domains/schedule/... -race -count=1 -fuzz=. -fuzztime=20s` exits with `cannot use -fuzz flag with multiple packages`, because the frozen scaffold contains both `schedule` and `schedule/fake`.
- Stop only the plan-edit sub-task; retain the fake repository package and validate equivalent coverage with the commands below.
- Requested exact edit to the W1-RECURRENCE Verify line in `plans/sparky-next/08-wave1.md`:

```diff
-**Verify.** `cd sparky/backend && go test ./domains/schedule/... -race -count=1 -fuzz=. -fuzztime=20s`.
+**Verify.** `cd sparky/backend && go test ./domains/schedule/... -race -count=1 && go test ./domains/schedule -race -count=1 -fuzz=. -fuzztime=20s`.
```

- W1-RECURRENCE runs and records both equivalent checks, without changing the shared plan.
