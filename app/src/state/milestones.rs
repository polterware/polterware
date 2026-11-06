//! Milestones' state: the boards a project lists and the cards on them (`loadMilestones`,
//! `loadWorkCards`), the place a route keeps (the Roadmap, a board in its Board or List
//! layout, a card's page), the writes the Electron Desktop makes from there (`changeMilestone`
//! and the card handlers in `App.tsx`, `persist.ts`), and what a card's page reads besides the
//! card (`WorkCardWork`).

use gpui::{App, AppContext as _, Context};
use polterware_ui::toast::ToastTone;

use super::{AppState, Load, Route, Section, change};
use crate::client::{Core, CoreError};
use crate::contracts::{
    AdvanceFeatureRequest, ApplyPolterRequest, ArchiveMilestoneRequest, CardCommit,
    CreateMilestoneRequest, LinkTaskCommitRequest, ListAgentSessionsRequest,
    ListCardCommitsRequest, ListMilestonesRequest, ListStructureRequest, ListWorkCardsRequest,
    Milestone, QueryPolterRequest, ReorderMilestonesRequest, ReviewFeatureRequest, SessionHandoff,
    StructureNode, UnlinkTaskCommitRequest, UpdateMilestoneRequest, VerifyFeatureRequest,
    VerifyFeatureResponse, WorkCard, WorkTask,
};
use crate::dispatch::{self, contracts::DispatchRun, contracts::ListRunsRequest};
use crate::work::{self, GateCheck, Stage};

/// How many Dispatch runs a card's page reads to find its own (`listRuns`'s limit).
const RUNS_LIMIT: i32 = 200;

/// What Milestones lists: the project's milestones (archived ones too, as `ListMilestones`
/// returns them), its active cards, and the Structure nodes the cards sit under.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Work {
    pub milestones: Vec<Milestone>,
    pub cards: Vec<WorkCard>,
    pub nodes: Vec<StructureNode>,
}

impl Work {
    pub fn card(&self, slug: &str) -> Option<&WorkCard> {
        self.cards.iter().find(|card| card.slug == slug)
    }

    pub fn milestone(&self, id: &str) -> Option<&Milestone> {
        self.milestones.iter().find(|milestone| milestone.id == id)
    }

    /// A board's name: All cards, the Backlog, or the milestone's.
    pub fn board_name(&self, view: &str) -> String {
        match view {
            work::ALL_CARDS => "All cards".into(),
            work::BACKLOG => "Backlog".into(),
            id => self
                .milestone(id)
                .map(|milestone| milestone.name.clone())
                .unwrap_or_else(|| "Milestone".into()),
        }
    }
}

/// A board's layout (`WorkTabId`: `board` and `specs`, labeled Board and List).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BoardTab {
    #[default]
    Board,
    List,
}

impl BoardTab {
    pub const ALL: [BoardTab; 2] = [BoardTab::Board, BoardTab::List];

    pub fn label(self) -> &'static str {
        match self {
            BoardTab::Board => "Board",
            BoardTab::List => "List",
        }
    }
}

/// Where Milestones is (`ClientRoute.milestone` and `tab`): the board open, `all` or a
/// milestone's id, none being the Roadmap, and the layout it shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoardRoute {
    pub view: Option<String>,
    pub tab: BoardTab,
}

/// A Roadmap write (`MilestoneAction`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MilestoneAction {
    Create {
        name: String,
        goal: String,
        state: String,
        target_date: String,
    },
    Rename {
        id: String,
        name: String,
        target_date: String,
    },
    Archive {
        id: String,
    },
    Reorder {
        ids: Vec<String>,
    },
}

/// What a card's page reads besides the card, read again when the card or its stage changes
/// (`WorkCardWork`'s effect).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CardReads {
    pub slug: String,
    /// The stage these were read at.
    pub stage: String,
    pub checks: Vec<GateCheck>,
    /// A refused gate's lines, shown above its checks.
    pub refusal: Vec<String>,
    pub advancing: bool,
    pub verification: Option<VerifyFeatureResponse>,
    pub verifying: bool,
    pub handoff: Option<SessionHandoff>,
    pub runs: Vec<DispatchRun>,
    pub commits: Vec<CardCommit>,
    pub brief: String,
}

/// A card write: the Polter text, its change, and the toast once it is written.
struct CardWrite {
    text: String,
    summary: String,
    rationale: String,
    written: Option<String>,
    failed: &'static str,
    /// The stages shown before the write (slug, stage), put back at once when it fails.
    restore: Vec<(String, String)>,
}

impl AppState {
    pub fn work(&self) -> &Load<Work> {
        &self.work
    }

    pub fn work_pending(&self) -> bool {
        self.work_pending
    }

    pub fn card_reads(&self) -> &CardReads {
        &self.card_reads
    }

    /// Milestones' place, and the card open as a page, on a Milestones route.
    pub fn board_route(&self) -> Option<(&BoardRoute, Option<&str>)> {
        match &self.route {
            Route::Project {
                section: Section::Milestones,
                board,
                page,
                ..
            } => Some((board, page.as_deref())),
            _ => None,
        }
    }

    fn go_board(&mut self, board: BoardRoute, page: Option<String>, cx: &mut Context<Self>) {
        let Some(id) = self.project_id().map(str::to_string) else {
            return;
        };
        self.navigate(
            Route::Project {
                id,
                section: Section::Milestones,
                tab: Default::default(),
                board,
                page,
            },
            cx,
        );
    }

    /// Opens a board (`all`, `backlog`, or a milestone's id), or the Roadmap with none, in the
    /// layout it was in (`setMilestoneView`).
    pub fn open_board(&mut self, view: Option<String>, cx: &mut Context<Self>) {
        let tab = self
            .board_route()
            .map(|(board, _)| board.tab)
            .unwrap_or_default();
        self.go_board(BoardRoute { view, tab }, None, cx);
    }

    /// Shows the open board as a Board or a List (`setTab`).
    pub fn set_board_tab(&mut self, tab: BoardTab, cx: &mut Context<Self>) {
        let view = self.board_route().and_then(|(board, _)| board.view.clone());
        self.go_board(BoardRoute { view, tab }, None, cx);
    }

    /// Opens a card as a page in place of its board, so Back returns to it.
    pub fn open_card(&mut self, slug: String, cx: &mut Context<Self>) {
        let board = self
            .board_route()
            .map(|(board, _)| board.clone())
            .unwrap_or_default();
        self.go_board(board, Some(slug), cx);
    }

    /// Reads the milestones, the cards, and the Structure nodes; a card page open reads its
    /// own again when its stage moved.
    pub fn load_work(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        let (scope_nodes, reads) = (self.scope_node_ids(), self.scope_reads);
        if !matches!(self.work, Load::Ready(_)) {
            self.work = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move { read_work(&core, &project, scope_nodes) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.project_id() != Some(wanted.as_str()) || state.scope_reads != reads {
                    return;
                }
                state.work = match result {
                    Ok(work) => Load::Ready(work),
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                state.keep_card_reads(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Reads the open card's page again when it is another card, or its stage moved.
    pub(super) fn keep_card_reads(&mut self, cx: &mut Context<Self>) {
        let Some((_, Some(slug))) = self.board_route() else {
            return;
        };
        let slug = slug.to_string();
        let stage = match &self.work {
            Load::Ready(work) => match work.card(&slug) {
                Some(card) => card.stage.clone(),
                None => return,
            },
            _ => return,
        };
        if self.card_reads.slug != slug || self.card_reads.stage != stage {
            self.load_card_reads(slug, stage, cx);
        }
    }

    /// Reads what a card's page shows besides the card: the review's checks for its next
    /// gate, its latest handoff, its Dispatch runs, its commits, and its agent brief. Each
    /// read that fails leaves its part empty.
    fn load_card_reads(&mut self, slug: String, stage: String, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        self.card_reads = CardReads {
            slug: slug.clone(),
            stage: stage.clone(),
            ..CardReads::default()
        };
        cx.spawn(async move |this, cx| {
            let wanted = slug.clone();
            let reads = cx
                .background_spawn(async move { read_card(&core, &project, &slug, &stage) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.card_reads.slug != wanted {
                    return;
                }
                let (checks, handoff, runs, commits, brief) = reads;
                state.card_reads.checks = checks;
                state.card_reads.handoff = handoff;
                state.card_reads.runs = runs;
                state.card_reads.commits = commits;
                state.card_reads.brief = brief;
                cx.notify();
            });
        })
        .detach();
    }

    /// Writes a Roadmap action under its summary, with no reason (`changeMilestone`), then
    /// reads the milestones and cards again (an archive moves its cards to the Backlog).
    /// `done` hears none when it failed, else the milestone the action names (empty after an
    /// archive or a reorder).
    pub fn milestone_action(
        &mut self,
        action: MilestoneAction,
        done: impl FnOnce(Option<String>, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project), Load::Ready(work)) =
            (self.ready_core(), self.project_id(), &self.work)
        else {
            done(None, cx);
            return;
        };
        let project = project.to_string();
        let milestone = match &action {
            MilestoneAction::Create { .. } | MilestoneAction::Reorder { .. } => None,
            MilestoneAction::Rename { id, .. } | MilestoneAction::Archive { id } => {
                match work.milestone(id) {
                    Some(milestone) => Some(milestone.clone()),
                    None => {
                        self.toast("This milestone is no longer available.", true);
                        done(None, cx);
                        return;
                    }
                }
            }
        };
        let old_name = milestone
            .as_ref()
            .map(|milestone| milestone.name.clone())
            .unwrap_or_default();
        let summary = match &action {
            MilestoneAction::Create { name, .. } => format!("Create milestone: {name}"),
            MilestoneAction::Rename { name, .. } => {
                format!("Rename milestone: {old_name} to {name}")
            }
            MilestoneAction::Archive { .. } => format!("Archive milestone: {old_name}"),
            MilestoneAction::Reorder { .. } => "Reorder milestones".into(),
        };
        let toast = match &action {
            MilestoneAction::Archive { .. } => {
                format!("Archived {old_name}; its cards are in the Backlog.")
            }
            _ => "Milestone saved locally.".into(),
        };
        self.work_pending = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let change = Some(change(&summary, ""));
                    let selected = match action {
                        MilestoneAction::Create {
                            name,
                            goal,
                            state,
                            target_date,
                        } => core
                            .work()
                            .create_milestone(&CreateMilestoneRequest {
                                project_id: project,
                                name,
                                goal,
                                state,
                                target_date,
                                change,
                                ..CreateMilestoneRequest::default()
                            })?
                            .milestone
                            .map(|milestone| milestone.id),
                        MilestoneAction::Rename {
                            id,
                            name,
                            target_date,
                        } => {
                            let milestone = milestone.unwrap_or_default();
                            core.work().update_milestone(&UpdateMilestoneRequest {
                                project_id: project,
                                id: id.clone(),
                                name,
                                goal: milestone.goal,
                                exit_criteria: milestone.exit_criteria,
                                target_date,
                                state: milestone.state,
                                change,
                            })?;
                            Some(id)
                        }
                        MilestoneAction::Archive { id } => {
                            core.work().archive_milestone(&ArchiveMilestoneRequest {
                                project_id: project,
                                id,
                                archived: true,
                                change,
                            })?;
                            None
                        }
                        MilestoneAction::Reorder { ids } => {
                            core.work().reorder_milestones(&ReorderMilestonesRequest {
                                project_id: project,
                                milestone_ids: ids,
                                change,
                            })?;
                            None
                        }
                    };
                    Ok::<_, CoreError>(selected)
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.work_pending = false;
                match result {
                    Ok(selected) => {
                        state.load_work(cx);
                        // The shared Composer offers the milestones the catalog holds.
                        state.load_catalog(cx);
                        state.push_toast(toast, ToastTone::Success);
                        done(selected.or(Some(String::new())), cx);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                        done(None, cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Writes card blocks as Polter text, then reads the cards again; `done` hears whether it
    /// went through. A failure reads them again too, which undoes what the board showed.
    fn write_cards(
        &mut self,
        write: CardWrite,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            done(false, cx);
            return;
        };
        let project = project.to_string();
        self.work_pending = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let CardWrite {
                text,
                summary,
                rationale,
                written,
                failed,
                restore,
            } = write;
            let result = cx
                .background_spawn(async move {
                    core.work().apply_polter(&ApplyPolterRequest {
                        project_id: project,
                        text,
                        change: Some(change(&summary, &rationale)),
                        session_label: "desktop".into(),
                        ..ApplyPolterRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.work_pending = false;
                if result.is_err() {
                    state.restore_stages(&restore);
                }
                state.load_work(cx);
                let ok = match result {
                    Ok(_) => {
                        if let Some(written) = written {
                            state.push_toast(written, ToastTone::Success);
                        }
                        true
                    }
                    Err(CoreError::NotRunning) => {
                        let message = state.lost(CoreError::NotRunning, cx);
                        state.toast(message, true);
                        false
                    }
                    Err(error) => {
                        let message = error.to_string();
                        state.toast(
                            if message.is_empty() {
                                failed.into()
                            } else {
                                message
                            },
                            true,
                        );
                        false
                    }
                };
                done(ok, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Shows `slugs` at `stage` before the write lands, as the board's optimistic move does;
    /// returns the stages they had.
    fn show_stage(&mut self, slugs: &[String], stage: Stage) -> Vec<(String, String)> {
        let mut before = Vec::new();
        if let Load::Ready(work) = &mut self.work {
            for card in work
                .cards
                .iter_mut()
                .filter(|card| slugs.contains(&card.slug))
            {
                before.push((
                    card.slug.clone(),
                    std::mem::replace(&mut card.stage, stage.id().into()),
                ));
            }
        }
        before
    }

    fn restore_stages(&mut self, before: &[(String, String)]) {
        if let Load::Ready(work) = &mut self.work {
            for (slug, stage) in before {
                if let Some(card) = work.cards.iter_mut().find(|card| &card.slug == slug) {
                    card.stage = stage.clone();
                }
            }
        }
    }

    /// Moves one card to `stage` (`onSetCardStage`); `rationale` is the why a move back asks.
    pub fn set_card_stage(
        &mut self,
        slug: String,
        stage: Stage,
        rationale: String,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let restore = self.show_stage(std::slice::from_ref(&slug), stage);
        self.write_cards(
            CardWrite {
                text: work::polter_card(&slug, &[("stage", stage.id())]),
                summary: format!("Move card to {}: {slug}", stage.label()),
                rationale,
                written: Some(format!("Card moved to {}.", stage.label())),
                failed: "Could not update card stage.",
                restore,
            },
            done,
            cx,
        );
    }

    /// Moves the selected cards to `stage` in one write (`onSetCardsStage`).
    pub fn set_cards_stage(
        &mut self,
        slugs: Vec<String>,
        stage: Stage,
        rationale: String,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let count = work::card_count(slugs.len());
        let restore = self.show_stage(&slugs, stage);
        self.write_cards(
            CardWrite {
                text: work::polter_cards(&slugs, &[("stage", stage.id())]),
                summary: format!("Move {count} to {}", stage.label()),
                rationale,
                written: Some(format!("{count} moved to {}.", stage.label())),
                failed: "Could not move the cards.",
                restore,
            },
            done,
            cx,
        );
    }

    /// Moves cards to a milestone by its slug, empty being the Backlog (`onSetCardsMilestone`).
    pub fn set_cards_milestone(
        &mut self,
        slugs: Vec<String>,
        milestone: String,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let name = match (&self.work, milestone.as_str()) {
            (_, "") => "the Backlog".to_string(),
            (Load::Ready(work), slug) => work
                .milestones
                .iter()
                .find(|item| item.slug == slug)
                .map(|item| item.name.clone())
                .unwrap_or_else(|| slug.to_string()),
            (_, slug) => slug.to_string(),
        };
        let count = work::card_count(slugs.len());
        self.write_cards(
            CardWrite {
                text: work::polter_cards(&slugs, &[("milestone", &milestone)]),
                summary: format!("Move {count} to {name}"),
                rationale: String::new(),
                written: Some(format!("{count} moved to {name}.")),
                failed: "Could not move the cards.",
                restore: Vec::new(),
            },
            done,
            cx,
        );
    }

    /// Electron App.tsx's archive Undo: capture the project at removal, not at click time.
    fn card_undo_toast(
        &mut self,
        message: String,
        project: String,
        slugs: Vec<String>,
        summary: String,
        cx: &mut Context<Self>,
    ) {
        let Some(core) = self.ready_core() else {
            return;
        };
        self.push_toast(message, ToastTone::Info);
        let id = self.next_toast - 1;
        let weak = cx.entity().downgrade();
        self.toast_actions.insert(
            id,
            polterware_ui::toast::ToastAction {
                label: "Undo".into(),
                on_click: std::rc::Rc::new(move |_, cx| {
                    let (core, project, slugs, summary) = (
                        core.clone(),
                        project.clone(),
                        slugs.clone(),
                        summary.clone(),
                    );
                    let _ = weak.update(cx, |_, cx| {
                        cx.spawn(async move |this, cx| {
                            let requested_project = project.clone();
                            let result = cx
                                .background_spawn(async move {
                                    core.work().apply_polter(&ApplyPolterRequest {
                                        project_id: project,
                                        text: work::polter_cards(&slugs, &[("status", "active")]),
                                        change: Some(change(&summary, "")),
                                        session_label: "desktop".into(),
                                        ..Default::default()
                                    })
                                })
                                .await;
                            let _ = this.update(cx, |state, cx| {
                                if let Err(error) = result {
                                    state.toast(error.to_string(), true);
                                } else if state.project_id() == Some(requested_project.as_str()) {
                                    state.load_work(cx);
                                }
                                cx.notify();
                            });
                        })
                        .detach();
                    });
                }),
            },
        );
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(polterware_ui::toast::TOAST_ACTION_LIFETIME)
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.toasts.iter().any(|toast| toast.id == id) {
                    state.dismiss_toast(id);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Archives the selected cards under one reason (`onRemoveCards`: status removed).
    pub fn archive_cards(
        &mut self,
        slugs: Vec<String>,
        rationale: String,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let count = work::card_count(slugs.len());
        let project = self.project_id().unwrap_or_default().to_string();
        let undo = cx.entity().downgrade();
        let restore = slugs.clone();
        self.write_cards(
            CardWrite {
                text: work::polter_cards(&slugs, &[("status", "removed")]),
                summary: format!("Archive {count}"),
                rationale,
                written: None,
                failed: "Could not archive the cards.",
                restore: Vec::new(),
            },
            move |ok, cx| {
                if ok {
                    let _ = undo.update(cx, |state, cx| {
                        state.card_undo_toast(
                            format!("Archived {count}"),
                            project,
                            restore,
                            format!("Restore {count}"),
                            cx,
                        );
                    });
                }
                done(ok, cx);
            },
            cx,
        );
    }

    /// Removes the open card from the board with its reason (`onRemoveCard`); its page goes
    /// back to the board it left.
    pub fn remove_card(&mut self, slug: String, rationale: String, cx: &mut Context<Self>) {
        let title = match &self.work {
            Load::Ready(work) => work.card(&slug).map(|card| card.title.clone()),
            _ => None,
        }
        .unwrap_or_else(|| slug.clone());
        let project = self.project_id().unwrap_or_default().to_string();
        let back = cx.entity().downgrade();
        self.write_cards(
            CardWrite {
                text: work::polter_card(&slug, &[("status", "removed")]),
                summary: format!("Remove card: {slug}"),
                rationale,
                written: None,
                failed: "Could not remove card.",
                restore: Vec::new(),
            },
            move |ok, cx| {
                if ok {
                    let _ = back.update(cx, |state, cx| {
                        state.card_undo_toast(
                            format!("Removed “{title}” from the board"),
                            project.clone(),
                            vec![slug.clone()],
                            format!("Restore card: {slug}"),
                            cx,
                        );
                        if state.project_id() == Some(project.as_str())
                            && let Some((_, Some(_))) = state.board_route()
                        {
                            state.open_board(
                                state
                                    .board_route()
                                    .and_then(|(board, _)| board.view.clone()),
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            },
            cx,
        );
    }

    /// Marks a card's task done, dropped, or open again (`onSetTaskState`), with no why.
    pub fn set_task_state(
        &mut self,
        slug: String,
        task: &WorkTask,
        state: &str,
        cx: &mut Context<Self>,
    ) {
        let verb = match state {
            "done" => "Complete",
            "dropped" => "Drop",
            _ => "Reopen",
        };
        if let Load::Ready(work) = &mut self.work
            && let Some(card) = work.cards.iter_mut().find(|card| card.slug == slug)
            && let Some(slot) = card
                .tasks
                .iter_mut()
                .find(|slot| slot.number == task.number)
        {
            slot.state = state.into();
        }
        self.write_cards(
            CardWrite {
                text: work::polter_task(&slug, task.number, &task.title, state),
                summary: format!("{verb} {slug} T{}", task.number),
                rationale: String::new(),
                written: None,
                failed: "Could not update the task.",
                restore: Vec::new(),
            },
            |_, _| {},
            cx,
        );
    }

    /// Creates a card from the Composer (`createCard`, `persistWorkCreateCard`) under a slug
    /// no card uses; `done` hears whether it went through.
    pub fn create_card(
        &mut self,
        card: work::NewCard,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        // The slug must be free across the project: the cards on hand are all of them only
        // while Milestones is read with no scope.
        if let Load::Ready(work) = &self.work
            && self.scope_node_ids().is_empty()
        {
            let slug = work::unique_card_slug(
                work.cards.iter().map(|card| card.slug.as_str()),
                &card.title,
            );
            return self.write_new_card(slug, card, done, cx);
        }
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return done(false, cx);
        };
        let project = project.to_string();
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let cards = cx
                .background_spawn(async move {
                    core.work().list_work_cards(&ListWorkCardsRequest {
                        project_id: project,
                        ..ListWorkCardsRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| match cards {
                Ok(response) if state.project_id() == Some(wanted.as_str()) => {
                    let slug = work::unique_card_slug(
                        response.cards.iter().map(|card| card.slug.as_str()),
                        &card.title,
                    );
                    state.write_new_card(slug, card, done, cx);
                }
                Ok(_) => done(false, cx),
                Err(error) => {
                    let message = state.lost(error, cx);
                    state.toast(message, true);
                    cx.notify();
                    done(false, cx);
                }
            });
        })
        .detach();
    }

    fn write_new_card(
        &mut self,
        slug: String,
        card: work::NewCard,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        self.write_cards(
            CardWrite {
                text: work::polter_new_card(&slug, &card),
                summary: format!("Create card: {}", card.title),
                rationale: card.why.clone(),
                written: None,
                failed: "Could not create card.",
                restore: Vec::new(),
            },
            {
                // `Created “title”` with Open, which shows the card's page on Milestones.
                let weak = cx.entity().downgrade();
                let title = card.title.clone();
                let project = self.project_id().unwrap_or_default().to_string();
                move |ok, cx| {
                    if ok {
                        let toast = weak.clone();
                        let _ = weak.update(cx, |state, cx| {
                            state.push_action_toast(
                                format!("Created “{title}”"),
                                ToastTone::Success,
                                "Open",
                                std::rc::Rc::new(move |_, cx| {
                                    let (slug, project) = (slug.clone(), project.clone());
                                    let _ = toast.update(cx, |state, cx| {
                                        if state.project_id() == Some(project.as_str()) {
                                            state.open_card(slug, cx);
                                            cx.notify();
                                        }
                                    });
                                }),
                                cx,
                            );
                        });
                    }
                    done(ok, cx);
                }
            },
            cx,
        );
    }

    /// Passes the open card's next gate (`onAdvanceCard`), which asks no why; a refusal stays
    /// on the page as lines.
    pub fn advance_card(&mut self, slug: String, step: &'static str, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        if self.card_reads.advancing {
            return;
        }
        let project = project.to_string();
        self.card_reads.advancing = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let wanted = slug.clone();
            let summary = if step == "contract" {
                format!("Accept contract for {slug}")
            } else {
                format!("Approve plan for {slug}")
            };
            let result = cx
                .background_spawn(async move {
                    core.work().advance_feature(&AdvanceFeatureRequest {
                        project_id: project,
                        slug,
                        step: step.into(),
                        change: Some(change(&summary, "")),
                        session_label: "desktop".into(),
                        ..AdvanceFeatureRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.card_reads.slug != wanted {
                    return;
                }
                state.card_reads.advancing = false;
                match result {
                    Ok(_) => {
                        state.card_reads.refusal.clear();
                        state.load_work(cx);
                    }
                    Err(CoreError::NotRunning) => {
                        let message = state.lost(CoreError::NotRunning, cx);
                        state.toast(message, true);
                    }
                    Err(error) => {
                        let message = error.to_string();
                        state.card_reads.refusal = work::gate_error_lines(if message.is_empty() {
                            "The gate refused the card."
                        } else {
                            &message
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Verifies the open card, which reads and never writes (`VerifyFeature`).
    pub fn verify_card(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        if self.card_reads.verifying || self.card_reads.slug.is_empty() {
            return;
        }
        let project = project.to_string();
        let slug = self.card_reads.slug.clone();
        self.card_reads.verifying = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let wanted = slug.clone();
            let result = cx
                .background_spawn(async move {
                    core.work().verify_feature(&VerifyFeatureRequest {
                        project_id: project,
                        target: slug,
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.card_reads.slug != wanted {
                    return;
                }
                state.card_reads.verifying = false;
                match result {
                    Ok(mut response) => {
                        response.checks.retain(|check| !check.message.is_empty());
                        state.card_reads.verification = Some(response);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Links commit `sha` to task `number` of the open card, or unlinks it, then reads its
    /// commits again; `done` hears whether it went through.
    pub fn link_commit(
        &mut self,
        number: i32,
        sha: String,
        link: bool,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            done(false, cx);
            return;
        };
        let project = project.to_string();
        let slug = self.card_reads.slug.clone();
        cx.spawn(async move |this, cx| {
            let wanted = slug.clone();
            let result = cx
                .background_spawn(async move {
                    let task = format!("{slug}#T{number}");
                    if link {
                        core.work().link_task_commit(&LinkTaskCommitRequest {
                            project_id: project.clone(),
                            task,
                            sha,
                        })?;
                    } else {
                        core.work().unlink_task_commit(&UnlinkTaskCommitRequest {
                            project_id: project.clone(),
                            task,
                            sha,
                        })?;
                    }
                    Ok::<_, CoreError>(
                        core.work()
                            .list_card_commits(&ListCardCommitsRequest {
                                project_id: project,
                                slug,
                            })
                            .map(|response| response.commits)
                            .ok(),
                    )
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                let ok = match result {
                    Ok(commits) => {
                        if state.card_reads.slug == wanted
                            && let Some(commits) = commits
                        {
                            state.card_reads.commits = commits;
                        }
                        true
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                        false
                    }
                };
                done(ok, cx);
                cx.notify();
            });
        })
        .detach();
    }
}

/// Milestones' three reads, made together as the web workspace makes them.
/// Reads the milestones (project-wide), the cards under `scope_nodes` (none: the whole
/// project), and the project's Structure nodes.
fn read_work(core: &Core, project: &str, scope_nodes: Vec<String>) -> Result<Work, CoreError> {
    let milestones = core
        .work()
        .list_milestones(&ListMilestonesRequest {
            project_id: project.to_string(),
        })?
        .milestones;
    let cards = core
        .work()
        .list_work_cards(&ListWorkCardsRequest {
            project_id: project.to_string(),
            scope_nodes,
            ..ListWorkCardsRequest::default()
        })?
        .cards;
    let nodes = core
        .work()
        .list_structure(&ListStructureRequest {
            project_id: project.to_string(),
            ..ListStructureRequest::default()
        })?
        .nodes;
    Ok(Work {
        milestones,
        cards,
        nodes,
    })
}

type CardRead = (
    Vec<GateCheck>,
    Option<SessionHandoff>,
    Vec<DispatchRun>,
    Vec<CardCommit>,
    String,
);

/// A card page's reads; each one that fails is empty.
fn read_card(core: &Core, project: &str, slug: &str, stage: &str) -> CardRead {
    let stage = Stage::parse(stage).unwrap_or(Stage::Shape);
    let checks = core
        .work()
        .review_feature(&ReviewFeatureRequest {
            project_id: project.to_string(),
            slug: slug.to_string(),
        })
        .ok()
        .and_then(|response| response.review)
        .map(|review| work::gate_checks(&review.checks, stage))
        .unwrap_or_default();
    let handoff = core
        .work()
        .list_agent_sessions(&ListAgentSessionsRequest {
            project_id: project.to_string(),
        })
        .ok()
        .and_then(|response| work::latest_handoff(&response.handoffs, slug).cloned());
    let runs = dispatch::connect()
        .and_then(|dispatch| {
            dispatch
                .dispatch()
                .list_runs(&ListRunsRequest {
                    project_id: project.to_string(),
                    limit: RUNS_LIMIT,
                    ..ListRunsRequest::default()
                })
                .ok()
        })
        .map(|response| {
            work::card_runs(&response.runs, slug)
                .into_iter()
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let commits = core
        .work()
        .list_card_commits(&ListCardCommitsRequest {
            project_id: project.to_string(),
            slug: slug.to_string(),
        })
        .map(|response| response.commits)
        .unwrap_or_default();
    let brief = core
        .work()
        .query_polter(&QueryPolterRequest {
            project_id: project.to_string(),
            query: format!("brief {slug}"),
            ..QueryPolterRequest::default()
        })
        .map(|response| response.text)
        .unwrap_or_default();
    (checks, handoff, runs, commits, brief)
}
