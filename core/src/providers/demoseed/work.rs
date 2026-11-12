//! The demo project's milestones and cards.

use crate::compositions::wedge;
use crate::providers::sqlite::{self, Store};

use super::seed::PROJECT_ID;

/// Spreads demo Work cards across every stage, with tasks, invariants, open
/// questions (one blocking, so Inbox has a decision), and a handoff on a card
/// in Build.
const ROADMAP_POLTER: &str = r#"---
polter: 1
summary: Seed the demo roadmap
rationale: Show every Work stage with tasks, questions, invariants, and a handoff
---

## billing-webhooks: Billing webhooks
stage: shape · under: services/payments · owner: Platform

Receive payment provider webhooks and record each event exactly once.

? Which provider signs the payload? [blocking] (stripe | adyen | both)
? Retry failed events from a queue or inline?
! Never charge a customer twice for the same event

## session-cache-health: Session cache health
stage: shape · under: data/redis

Explain why the session cache reports degraded health in development.

- [ ] Collect cache latency samples
? Is the degraded status a real regression or a stale probe?

## auth-token-rotation: Auth token rotation
stage: plan · under: services/auth · decisions: decision-sqlite-local · owner: Platform

Rotate signing keys without logging users out.

- [ ] Key rotation schedule in auth service  `services/auth/*`
- [ ] Dual-key validation window
- [ ] Rotation runbook
! Old keys stay valid for one full session lifetime

## git-metadata-import: Git metadata import
stage: build · under: core/import · decisions: decision-sqlite-local, decision-deterministic-import · owner: Platform

Read HEAD, branch, and recent commits so delivered tasks match commits on main.

- [x] T1 Read HEAD and branch from the local checkout  `backend/providers/git/*`
- [x] T2 Link commits to tasks by citation
- [ ] T3 Link a commit by hand from the card  `backend/domains/inbox/*`
- [ ] T4 Show linked commits on the task  `app/src/views/card_page.rs`
- [-] T5 Import tags as releases
? Include merge commits? [blocking] (yes | no | first-parent)
! No network calls while reading Git

handoff:
branch: feat/git-metadata-import
agent: claude-code
goal: Every done task matches a commit on main
done: HEAD and branch are read; cited commits link their tasks
in_progress: Linking commits by hand from the card
next: Show linked commits on the task
touched: backend/providers/git/git.go, backend/compositions/wedge/commits.go

## commits-panel: Commits panel
stage: build · under: reality/commits · decisions: decision-deterministic-import

Show each card's commits with their subject, task, and GitHub link.

- [x] Read the commits that cite a task
- [ ] Group commits by task
- [ ] Empty state when no commit cites the card
! A commit always names its task

## context-compiler: Context compiler
stage: verify · under: agents/context · decisions: decision-bounded-context · owner: Agents

Compile a bounded project context for an agent session.

- [x] Token budget per section
- [x] Include active invariants
- [ ] Verify output against a large project
! The compiled context never includes credentials

## local-sqlite-core: Local SQLite core
stage: delivered · under: core · decisions: decision-sqlite-local · owner: Platform

One local core process owns the SQLite database; every client attaches to it.

- [x] Single core per database
- [x] Live change events
- [x] CLI and Desktop attach to the running core
! Only the core opens SQLite
"#;

/// A second session (another agent) claiming an open task, so the board shows a
/// claim and Agents History shows two runs.
const CLAIM_POLTER: &str = r#"---
polter: 1
summary: Claim the fact detail task
rationale: Pick up the linked-commits UI while the core work continues
---

## git-metadata-import: Git metadata import

claim: T4
"#;

pub(super) fn seed_work(store: &Store) -> sqlite::Result<()> {
    let applies = [
        wedge::ApplyInput {
            project_id: PROJECT_ID.to_string(),
            text: ROADMAP_POLTER.to_string(),
            auto_start_label: "claude-code".to_string(),
            agent: "claude-code".to_string(),
            ..wedge::ApplyInput::default()
        },
        wedge::ApplyInput {
            project_id: PROJECT_ID.to_string(),
            text: CLAIM_POLTER.to_string(),
            auto_start_label: "codex".to_string(),
            agent: "codex".to_string(),
            ..wedge::ApplyInput::default()
        },
    ];
    for input in applies {
        wedge::apply_polter(store, input).map_err(|err| sqlite::Error::message(err.to_string()))?;
    }
    Ok(())
}
