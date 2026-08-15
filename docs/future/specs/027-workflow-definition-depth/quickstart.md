# Quickstart validation: Workflow Definition Depth

## Prerequisites

- Repo root monorepo install already present  
- No live polter-core workflow engine required  

## Automated checks

```bash
pnpm --filter @polterware/feature-operations test
```

Expect: workflow-status, workflow-model, workflow-builder (or fixture composition) tests pass.

## Manual spot-check (when Desktop UI is running)

1. Operations → Workflows  
2. Open **Deploy service** — header shows Version + StatusChip; approval step has reviewers/condition/expiration; failure strategy on trigger  
3. Open a workflow with retries (e.g. rotate/restart) — command step shows max attempts / backoff  
4. Edit + Save — version increments by 1; toast is fixture-honest  
5. Create workflow — empty pipeline copy; save version 1 → Ready  
6. Recent runs on deploy workflow — control opens/activates a run (existing Run detail), does not embed a second run surface  

## Rubric

- [ ] WorkflowDef depth fields present  
- [ ] ≥3 step types distinct config  
- [ ] StatusChip on definition status  
- [ ] Dedicated tests pass  
- [ ] Run detail not rewritten  
- [ ] Not “done” solely because sequential diagram existed  
