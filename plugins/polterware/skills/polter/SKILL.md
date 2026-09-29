---
name: polter
description: Create or update Polterware Work cards (roadmap items, tasks, open questions, invariants) in Polter format with polterware apply. Use when planning work, recording tasks or decisions-in-progress, or moving a card between stages in a project tracked by Polterware.
---

# Polter format

Write Work cards as Polter text and apply them with the `polterware` CLI. The Polterware store is the source of truth; a `.pw.md` file is only input.

```markdown
Polter format v1: markdown that writes Polterware Work cards.

---
polter: 1
summary: One line saying what changes
rationale: Why it changes
---

## git-import: Git metadata import
stage: plan · owner: erick · under: core/import · decisions: ADR-0007

Intent paragraph: what we want and why.

- [ ] T1 Upsert commit evidence `backend/import/*`
- [x] T2 Stamp revision on facts
? Include merge commits? [blocking] (yes | no)
! No network calls during scan

handoff:
branch: feat/git-import
agent: codex
goal: import HEAD, branch and recent commits as evidence
done: commit evidence upsert in core
in_progress: stamping revision on observed facts
next: wire into ImportProjectSources
touched: backend/import.go, docs/spec.md

Rules:
- `## slug: Title` is a card; the lowercase-kebab slug is its identity.
- Stages: shape, plan, build, verify, delivered.
- Tasks: [ ] open, [x] done, [-] dropped; T<n> and a trailing `scope` glob are optional.
- `?` is an open question, `!` a contract invariant.
- `handoff:` records one session on `branch:` (required): goal, done, in_progress, next, touched, agent.
- A text only changes what it mentions. Remove with status: removed, [-], or a trailing [removed].
- Start with `polterware status`. End with `polterware handoff --card <slug> --note "..."`.
- Check with `polterware lint file.pw.md`; write with `polterware apply file.pw.md [--dry-run]`.
```

## Workflow

1. Start from `polterware status` (current card, branch, last handoff, open questions).
2. Read current cards first: `polterware query cards` (one line per card) or `polterware query "card <slug>"`.
3. Write only what changes. Unmentioned cards, tasks, and fields stay as they are.
4. Check the plan: `polterware apply file.pw.md --dry-run` (or pipe text to `polterware apply - --dry-run`).
5. Apply: `polterware apply file.pw.md`. Every apply needs `summary` and `rationale` in the frontmatter.
6. Fix errors from the line-numbered messages; each one says how.
7. End the session with `polterware handoff --card <slug> --note "..."`. A `touched:` list that does not match the git diff is a hint, not an error.

Writes in this session join one AgentRun per project automatically; do not start or complete runs yourself.
