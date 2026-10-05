# Remove the remaining memories keyboard quarantine after list integration

Owner: W3-FIX-LISTS / integrator. `apps/sparky-desktop/tests/e2e/memories.spec.ts` is outside W3-FIX-SHELL ownership.

The shell keyboard bug (w3-e2e-6) is fixed and covered by the passing owned `shell: Enter and Space activate native buttons and Save schedule preserves memory status` spec. The original memories spec also names w3-e2e-3 (virtualized rows), which belongs to W3-FIX-LISTS. Keep that acceptance intact and remove its quarantine after integrating the list fix:

```diff
-// Product gap: plans/sparky-next/requests/w3-e2e-6.md
-test.fixme(process.env.SPARKY_E2E_RECHECK_GAPS !== "1", "w3-e2e-6 and w3-e2e-3: collection keyboard activation and virtualized rows are blocked. Set SPARKY_E2E_RECHECK_GAPS=1 after the fix.");
```

Rerun `keyboard create, edit, pin, complete, undo and delete` against the integrated app. W3-FIX-SHELL removes the owned Mind and Calendar fixmes; it does not edit or claim a pass for the unowned memories spec.
