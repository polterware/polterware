//! Port of `backend/compositions/wedge/sessions.go`.

#[allow(unused_imports)]
use super::*;

use std::collections::{HashMap, HashSet};

use time::OffsetDateTime;

use crate::domains::agentrun;
use crate::domains::work;
use crate::providers::sqlite::{self, Store};

/// How many recent runs a session read considers.
pub const SESSION_RUN_WINDOW: i64 = 200;
/// How many recent handoffs `agent_sessions` answers.
pub const RECENT_HANDOFFS_MAX: usize = 10;

/// One task an agent holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionClaim {
    pub slug: String,
    pub number: i32,
    pub title: String,
    pub scope: String,
}

/// One agent working on the project now: an open agent run and the claims its
/// label holds. A label with claims and no open run still shows, with an empty
/// `run_id`.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentSession {
    pub run_id: String,
    pub agent: String,
    pub task: String,
    pub started_at: OffsetDateTime,
    pub claims: Vec<SessionClaim>,
}

impl Default for AgentSession {
    fn default() -> Self {
        Self {
            run_id: String::new(),
            agent: String::new(),
            task: String::new(),
            started_at: OffsetDateTime::UNIX_EPOCH,
            claims: Vec::new(),
        }
    }
}

/// One recorded handoff, newest first in [`agent_sessions`].
#[derive(Debug, Clone, PartialEq)]
pub struct SessionHandoff {
    pub slug: String,
    pub branch: String,
    pub goal: String,
    pub done: String,
    pub in_progress: String,
    pub next: String,
    pub agent: String,
    pub at: OffsetDateTime,
}

impl Default for SessionHandoff {
    fn default() -> Self {
        Self {
            slug: String::new(),
            branch: String::new(),
            goal: String::new(),
            done: String::new(),
            in_progress: String::new(),
            next: String::new(),
            agent: String::new(),
            at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// Lists the live sessions of a project and its most recent handoffs. `branch`
/// is the project checkout's branch, shared by all sessions because the core
/// does not track a checkout per session.
pub fn agent_sessions(
    store: &Store,
    project_id: &str,
) -> sqlite::Result<(Vec<AgentSession>, Vec<SessionHandoff>, String)> {
    let snapshot = store.load_work_snapshot(project_id)?;
    let runs = store.list_agent_runs(project_id, SESSION_RUN_WINDOW)?;

    let mut claims: HashMap<String, Vec<SessionClaim>> = HashMap::new();
    for card in &snapshot.cards {
        if card.status != work::CARD_ACTIVE {
            continue;
        }
        for task in &card.tasks {
            let Some(claim) = &task.claim else {
                continue;
            };
            claims
                .entry(claim.holder.clone())
                .or_default()
                .push(SessionClaim {
                    slug: card.slug.clone(),
                    number: task.number,
                    title: task.title.clone(),
                    scope: claim.scope.clone(),
                });
        }
    }

    let mut sessions: Vec<AgentSession> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for run in &runs {
        if run.status != agentrun::STATUS_OPEN || run.actor_kind == agentrun::ACTOR_KIND_HUMAN {
            continue;
        }
        seen.insert(run.agent_label.clone());
        sessions.push(AgentSession {
            run_id: run.id.clone(),
            agent: run.agent_label.clone(),
            task: run.task.clone(),
            started_at: run.created_at,
            claims: claims.get(&run.agent_label).cloned().unwrap_or_default(),
        });
    }
    for (holder, held) in &claims {
        if !seen.contains(holder) {
            sessions.push(AgentSession {
                agent: holder.clone(),
                claims: held.clone(),
                ..AgentSession::default()
            });
        }
    }
    sessions.sort_by(|a, b| {
        if a.started_at == b.started_at {
            a.agent.cmp(&b.agent)
        } else {
            b.started_at.cmp(&a.started_at)
        }
    });

    let mut handoffs: Vec<SessionHandoff> = Vec::new();
    for card in &snapshot.cards {
        for h in &card.handoffs {
            handoffs.push(SessionHandoff {
                slug: card.slug.clone(),
                branch: h.branch.clone(),
                goal: h.goal.clone(),
                done: h.done.clone(),
                in_progress: h.in_progress.clone(),
                next: h.next.clone(),
                agent: h.agent.clone(),
                at: h.created_at,
            });
        }
    }
    handoffs.sort_by(|a, b| b.at.cmp(&a.at));
    handoffs.truncate(RECENT_HANDOFFS_MAX);
    Ok((sessions, handoffs, project_git(store, project_id).branch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    #[test]
    fn a_label_with_claims_and_an_open_run_is_one_session() {
        let TempStore { store, .. } = TempStore::new();
        let project =
            crate::compositions::wedge::work::test_support::init_test_project(&store, "Demo");
        let roadmap = "---\npolter: 1\nsummary: Add claim cards\nrationale: Exercise sessions\n---\n\n## git-import: Git import\nstage: build\n\n- [ ] T1 Evidence upsert `backend/import/*`\n";
        let claim = polter::coord_text(
            polter::COORD_CLAIM,
            &polter::parse_task_ref("git-import#T1").expect("task ref"),
            "codex",
        )
        .expect("coord text");
        for text in [roadmap.to_string(), claim] {
            apply_polter(
                &store,
                ApplyInput {
                    project_id: project.id.clone(),
                    text,
                    auto_start_label: "codex".to_string(),
                    agent: "codex".to_string(),
                    ..ApplyInput::default()
                },
            )
            .expect("apply");
        }

        // The claim holder shows without an open run.
        let (sessions, _, _) = agent_sessions(&store, &project.id).expect("sessions");
        assert_eq!(sessions.len(), 1, "sessions: {sessions:?}");
        assert_eq!(sessions[0].agent, "codex");
        assert_eq!(sessions[0].run_id, "");
        assert_eq!(
            sessions[0].claims.len(),
            1,
            "claims: {:?}",
            sessions[0].claims
        );
        assert_eq!(sessions[0].claims[0].slug, "git-import");
        assert_eq!(sessions[0].claims[0].number, 1);
        assert_eq!(sessions[0].claims[0].scope, "backend/import/*");

        // An open agent run on the same label is the same session, with the run.
        let run = agentrun::start_with_actor(
            &project.id,
            "Work the import",
            "codex",
            agentrun::ACTOR_KIND_AGENT,
            Some(OffsetDateTime::now_utc()),
        )
        .expect("run");
        store.create_agent_run(run.clone()).expect("save run");
        let (sessions, _, _) = agent_sessions(&store, &project.id).expect("sessions");
        assert_eq!(sessions.len(), 1, "sessions: {sessions:?}");
        assert_eq!(sessions[0].run_id, run.id);
        assert_eq!(sessions[0].task, "Work the import");
        assert_eq!(sessions[0].claims.len(), 1);
    }

    #[test]
    fn a_project_without_runs_has_no_sessions() {
        let TempStore { store, .. } = TempStore::new();
        save_project(&store, "p", OffsetDateTime::UNIX_EPOCH);
        let (sessions, handoffs, branch) = agent_sessions(&store, "p").expect("sessions");
        assert!(sessions.is_empty(), "sessions: {sessions:?}");
        assert!(handoffs.is_empty(), "handoffs: {handoffs:?}");
        assert_eq!(branch, "");
    }
}
