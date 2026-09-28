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

Rules:
- `## slug: Title` is a card; the lowercase-kebab slug is its identity.
- Stages: shape, plan, build, verify, delivered.
- Tasks: [ ] open, [x] done, [-] dropped; T<n> and a trailing `scope` glob are optional.
- `?` is an open question, `!` a contract invariant.
- A text only changes what it mentions. Remove with status: removed, [-], or a trailing [removed].
- Check with `polterware lint file.pw.md`; write with `polterware apply file.pw.md [--dry-run]`.
```

## Workflow

1. Read current cards first: `polterware query cards` (one line per card) or `polterware query "card <slug>"`.
2. Write only what changes. Unmentioned cards, tasks, and fields stay as they are.
3. Check the plan: `polterware apply file.pw.md --dry-run` (or pipe text to `polterware apply - --dry-run`).
4. Apply: `polterware apply file.pw.md`. Every apply needs `summary` and `rationale` in the frontmatter.
5. Fix errors from the line-numbered messages; each one says how.

Writes in this session join one AgentRun per project automatically; do not start or complete runs yourself.
