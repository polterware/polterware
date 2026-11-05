//! Milestones (`milestone-list.tsx`, `index.tsx`'s board tabs, `work-board.tsx`,
//! `work-list-table.tsx`, `card-peek.tsx`): the Roadmap, then
//! a board's cards as a Board or a List, and a card's page in place of either.
//!
//! The Roadmap lists the boards, All cards first and then the milestones in order, by Board,
//! State, Progress (the delivered share, green once every card is delivered), and Target. Its
//! search narrows them by name; its "+" opens the Composer for a new milestone (its name, its
//! goal, its State, and its Target date), which opens the board it made. A milestone's "…"
//! and its right click rename it (the Composer again), move it up or down, or archive it once
//! confirmed, its cards moving to the Backlog.
//!
//! A board's toolbar goes back to the Roadmap, switches Board and List, and searches and
//! filters its cards (by stage, and by open questions). The Board holds the five stage
//! columns: a card drags to another column, a move back asking why, and its Tasks open in a
//! Morph where they are checked off; a column's "+" opens the app's shared Composer for a new
//! card there, on the board's milestone, and so does C anywhere on Milestones. The List groups the
//! cards by stage (each group's "+" adds one), orders them by when they changed, and its
//! headers sort by title, stage, or change; picked cards move to a stage or a milestone, or
//! archive under one reason. Space peeks at the marked card; a row or a card opens its page.

use std::collections::BTreeSet;
use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, FontWeight, IntoElement,
    KeyDownEvent, Render, SharedString, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::checkbox::checkbox_mark;
use polterware_ui::chip::chip_rounded;
use polterware_ui::data_table::{
    Column, DataTable, SelectionChange, SortDirection, TableGroup, next_sort, primary_cell, sans,
    tabular, text_width,
};
use polterware_ui::filter::{FilterOption, FilterSpec};
use polterware_ui::filter_popover::{FilterField, FilterPopover, FiltersChanged};
use polterware_ui::icon_tile::{IconTile, TileTone};
use polterware_ui::icons::Icon;
use polterware_ui::menu::MenuItem;
use polterware_ui::patterns::{
    BoardCard, BoardColumn, BoardMove, Composer, ComposerDraft, ComposerEvent, ComposerSpec,
    ConfirmDialog, ContextMenu, DialogPresence, MascotMood, Morph, MorphAlign, Property, board,
    modal_layer, page_empty, page_empty_with, peek, peek_chip, peek_section, selection_toolbar,
    window_layer,
};
use polterware_ui::progress_bar::progress_bar_sm;
use polterware_ui::skeleton::text_skeleton;
use polterware_ui::tabs::Tabs;
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::{StatusTone, Theme};

use crate::contracts::{Milestone, StructureNode, WorkCard};
use crate::format::format_instant;
use crate::state::{AppState, BoardTab, ComposeRequest, Load, MilestoneAction, Work};
use crate::views::card_page::CardPage;
use crate::views::compose::dot_option;
use crate::views::core_status::core_status;
use crate::views::knowledge::{chips, chips_width, context_labels, muted_cell, title_width};
use crate::views::scope::{ScopeFilter, clear_page};
use crate::work::{self, Grouping, Ordering, Property as ListProperty, Stage};
use polterware_ui::window::window_drag::WindowDragRegion;

/// `.pw-data-table-scroll`'s padding-inline.
const TABLE_INSET: f32 = 12.0;
/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
/// A chip's frame around its label: 9px of padding and 1px of border each side.
const CHIP_FRAME: f32 = 20.0;
/// A milestone's states, in the Composer's order.
const STATES: [&str; 3] = ["planned", "active", "done"];
/// The List's columns after the card, in order.
const LIST_PROPERTIES: [ListProperty; 6] = ListProperty::ALL;

/// A milestone state's chip (`STATE_TONE`).
pub fn state_tone(state: &str) -> StatusTone {
    match state {
        "active" => StatusTone::Syncing,
        "done" => StatusTone::Healthy,
        _ => StatusTone::Muted,
    }
}

/// A stage's chip (`workStageTone`).
pub fn stage_tone(stage: Stage) -> StatusTone {
    match stage {
        Stage::Delivered => StatusTone::Healthy,
        Stage::Verify => StatusTone::Syncing,
        Stage::Build => StatusTone::Ok,
        _ => StatusTone::Muted,
    }
}

/// A stage's 20px tile (`STAGE_ICON`, `STAGE_TILE`): one chart hue per stage.
fn stage_tile(stage: Stage, theme: &Theme) -> IconTile {
    let (icon, tone, tint) = match stage {
        Stage::Shape => (Icon::Pencil, TileTone::Knowledge, theme.chart_amber),
        Stage::Plan => (Icon::Route, TileTone::Architecture, theme.chart_blue),
        Stage::Build => (Icon::Hammer, TileTone::Infrastructure, theme.chart_cyan),
        Stage::Verify => (Icon::ShieldCheck, TileTone::Identity, theme.chart_violet),
        Stage::Delivered => (Icon::Check, TileTone::Data, theme.chart_green),
    };
    IconTile::new(icon).tone(tone).tint(tint)
}

/// What the Composer is open for: a new milestone, or the one with this id to rename.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Compose {
    Milestone,
    Rename(String),
}

/// A question asked before a write.
#[derive(Clone)]
enum Ask {
    ArchiveMilestone {
        id: String,
        name: String,
        cards: usize,
    },
    /// A move back to `stage`, which asks why; a board drop waits on it.
    MoveBack {
        slugs: Vec<String>,
        stage: Stage,
        what: String,
        settle: Option<Rc<BoardMove>>,
    },
    ArchiveCards {
        slugs: Vec<String>,
    },
    RemoveCard {
        slug: String,
        title: String,
    },
}

/// A Roadmap row: All cards, or a milestone's board.
#[derive(Clone)]
struct RoadmapRow {
    id: String,
    name: String,
    milestone: Option<Milestone>,
    done: usize,
    total: usize,
}

pub struct MilestonesView {
    state: Entity<AppState>,
    search: Entity<TextInput>,
    query: String,
    card_search: Entity<TextInput>,
    card_query: String,
    filters: Entity<FilterPopover>,
    menu: Entity<ContextMenu>,
    composer: Entity<Composer>,
    compose: Option<Compose>,
    milestone_state: SharedString,
    ask: Option<Ask>,
    ask_focus: FocusHandle,
    ask_focus_due: bool,
    reason: Entity<TextInput>,
    peek: Option<String>,
    peek_presence: DialogPresence,
    peek_focus: FocusHandle,
    /// The List's picked cards, by slug.
    selected: BTreeSet<String>,
    display: work::Display,
    /// The List's marked row, and the Roadmap's.
    active: Option<String>,
    roadmap_active: Option<String>,
    /// The card whose Tasks Morph is open on the Board.
    tasks_open: Option<String>,
    /// A task is being written from the Board.
    task_pending: bool,
    /// The board the search and filters were set for; another one clears them.
    view: Option<String>,
    page: Entity<CardPage>,
    scope: ScopeFilter,
}

impl MilestonesView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new(cx).placeholder("Search milestones").pill());
        cx.observe(&search, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.query {
                this.query = text;
                cx.notify();
            }
        })
        .detach();
        let card_search = cx.new(|cx| TextInput::new(cx).placeholder("Search").pill());
        cx.observe(&card_search, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.card_query {
                this.card_query = text;
                cx.notify();
            }
        })
        .detach();
        let filters =
            cx.new(|cx| FilterPopover::new("milestones-card-filters", "Card filters", true, cx));
        cx.subscribe(&filters, |this, filters, _: &FiltersChanged, cx| {
            this.scope.push(&filters, &this.state, cx);
            cx.notify();
        })
        .detach();
        cx.observe(&filters, |_, _, cx| cx.notify()).detach();
        let stages = FilterSpec::multi(
            "stage",
            "Stage",
            Stage::ALL
                .iter()
                .map(|stage| FilterOption::new(stage.id(), stage.label()))
                .collect(),
        )
        .plural("stages");
        let questions = FilterSpec::multi(
            "questions",
            "Card",
            vec![FilterOption::new("open", "open questions")],
        )
        .single();
        filters.update(cx, |popover, _| {
            popover.set_fields(vec![
                FilterField::new(stages, None),
                FilterField::new(questions, Some(Icon::ListChecks)).section("flags"),
            ]);
        });
        let menu = cx.new(ContextMenu::new);
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        let composer = cx.new(Composer::new);
        cx.subscribe(&composer, Self::composed).detach();
        let reason = cx.new(|cx| TextInput::new(cx).inline());
        cx.observe(&reason, |_, _, cx| cx.notify()).detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        let host = cx.weak_entity();
        let page = cx.new(|cx| CardPage::new(state.clone(), host, cx));
        Self {
            state,
            search,
            query: String::new(),
            card_search,
            card_query: String::new(),
            filters,
            menu,
            composer,
            compose: None,
            milestone_state: "planned".into(),
            ask: None,
            ask_focus: cx.focus_handle(),
            ask_focus_due: false,
            reason,
            peek: None,
            peek_presence: DialogPresence::new(),
            peek_focus: cx.focus_handle(),
            selected: BTreeSet::new(),
            display: work::Display::default(),
            active: None,
            roadmap_active: None,
            tasks_open: None,
            task_pending: false,
            view: None,
            page,
            scope: ScopeFilter::new(false),
        }
    }

    fn work(&self, cx: &App) -> Option<Work> {
        match self.state.read(cx).work() {
            Load::Ready(work) => Some(work.clone()),
            _ => None,
        }
    }

    // The Composer.

    fn milestone_properties(&self) -> Vec<Property> {
        let options = STATES
            .iter()
            .map(|state| dot_option(state, work::milestone_state_label(state), state_tone(state)))
            .collect();
        vec![Property::single(
            "state",
            "State",
            options,
            self.milestone_state.clone(),
        )]
    }

    fn open_composer(&mut self, compose: Compose, window: &mut Window, cx: &mut Context<Self>) {
        let Some(work) = self.work(cx) else {
            return;
        };
        let (spec, properties) = match &compose {
            Compose::Milestone => {
                self.milestone_state = "planned".into();
                (
                    ComposerSpec::new("Milestone name", "Create milestone")
                        .description("Goal: what this milestone delivers…")
                        .field("Target date", "YYYY-MM-DD")
                        .edit(),
                    self.milestone_properties(),
                )
            }
            Compose::Rename(id) => {
                let name = work
                    .milestone(id)
                    .map(|milestone| milestone.name.clone())
                    .unwrap_or_default();
                (
                    ComposerSpec::new("Milestone name", "Save")
                        .edit()
                        .initial(ComposerDraft {
                            title: name,
                            ..ComposerDraft::default()
                        }),
                    Vec::new(),
                )
            }
        };
        self.compose = Some(compose);
        self.composer.update(cx, |composer, cx| {
            composer.open(spec, properties, window, cx)
        });
    }

    /// A column's or a stage group's "+" (`addCardTo`) at `stage`, and C (`composeCard`): a
    /// new card in the app's Composer, on this board's milestone.
    pub(crate) fn compose_card(&mut self, stage: Option<Stage>, cx: &mut Context<Self>) {
        let view = self
            .state
            .read(cx)
            .board_route()
            .and_then(|(board, _)| board.view.clone())
            .unwrap_or_default();
        let milestone = match view.as_str() {
            work::ALL_CARDS | work::BACKLOG | "" => String::new(),
            id => id.to_string(),
        };
        self.state.update(cx, |state, cx| {
            state.compose(ComposeRequest::Card { stage, milestone }, cx)
        });
    }

    fn composed(
        &mut self,
        composer: Entity<Composer>,
        event: &ComposerEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ComposerEvent::Submit(draft) => self.write(draft.clone(), composer, cx),
            ComposerEvent::Property { id, values } => {
                let value = values
                    .first()
                    .map(|value| value.to_string())
                    .unwrap_or_default();
                if id.as_ref() == "state" {
                    self.milestone_state = value.into();
                }
                let properties = match &self.compose {
                    Some(Compose::Milestone) => self.milestone_properties(),
                    _ => Vec::new(),
                };
                composer.update(cx, |composer, cx| composer.set_properties(properties, cx));
            }
            ComposerEvent::Action => {}
            ComposerEvent::Closed => self.compose = None,
        }
    }

    fn write(&mut self, draft: ComposerDraft, composer: Entity<Composer>, cx: &mut Context<Self>) {
        let done = {
            let composer = composer.downgrade();
            move |written: bool, cx: &mut App| {
                let _ = composer.update(cx, |composer, cx| composer.finish(written, cx));
            }
        };
        let Some(work) = self.work(cx) else {
            return done(false, cx);
        };
        match self.compose.clone() {
            Some(Compose::Milestone) | Some(Compose::Rename(_)) => {
                let renaming = match &self.compose {
                    Some(Compose::Rename(id)) => Some(id.clone()),
                    _ => None,
                };
                let target = draft.fields.first().cloned().unwrap_or_default();
                let problem =
                    work::milestone_name_error(&work.milestones, &draft.title, renaming.as_deref())
                        .or_else(|| {
                            (!work::target_date_ok(&target))
                                .then_some("Write the target date as YYYY-MM-DD.")
                        });
                if let Some(problem) = problem {
                    self.state.update(cx, |state, cx| {
                        state.toast(problem, true);
                        cx.notify();
                    });
                    return done(false, cx);
                }
                let action = match renaming {
                    Some(id) => MilestoneAction::Rename {
                        id,
                        name: draft.title.trim().to_string(),
                    },
                    None => MilestoneAction::Create {
                        name: draft.title.trim().to_string(),
                        goal: draft.description.trim().to_string(),
                        state: self.milestone_state.to_string(),
                        target_date: target.trim().to_string(),
                    },
                };
                let create = matches!(action, MilestoneAction::Create { .. });
                let state = self.state.downgrade();
                self.state.update(cx, |app, cx| {
                    app.milestone_action(
                        action,
                        move |made, cx| {
                            done(made.is_some(), cx);
                            // A new milestone opens its board.
                            if create && let Some(id) = made.filter(|id| !id.is_empty()) {
                                let _ = state.update(cx, |state, cx| {
                                    state.open_board(Some(id), cx);
                                    cx.notify();
                                });
                            }
                        },
                        cx,
                    );
                });
            }
            None => done(false, cx),
        }
    }

    // Questions before a write.

    fn ask(&mut self, ask: Ask, cx: &mut Context<Self>) {
        self.reason.update(cx, |field, cx| field.set_text("", cx));
        self.ask = Some(ask);
        self.ask_focus_due = true;
        cx.notify();
    }

    /// Cancel: a board drop that waited on the answer goes back.
    fn dismiss_ask(&mut self, cx: &mut Context<Self>) {
        if let Some(Ask::MoveBack {
            settle: Some(change),
            ..
        }) = self.ask.take()
        {
            change.settle(false, cx);
        }
        cx.notify();
    }

    fn answer(&mut self, cx: &mut Context<Self>) {
        let Some(ask) = self.ask.take() else {
            return;
        };
        let reason = self.reason.read(cx).text().trim().to_string();
        match ask {
            Ask::ArchiveMilestone { id, .. } => {
                self.state.update(cx, |state, cx| {
                    state.milestone_action(MilestoneAction::Archive { id }, |_, _| {}, cx);
                });
            }
            Ask::MoveBack {
                slugs,
                stage,
                settle,
                ..
            } => self.move_cards(slugs, stage, reason, settle, cx),
            Ask::ArchiveCards { slugs } => {
                let view = cx.entity().downgrade();
                self.state.update(cx, |state, cx| {
                    state.archive_cards(
                        slugs,
                        reason,
                        move |ok, cx| {
                            if ok {
                                let _ = view.update(cx, |this, cx| {
                                    this.selected.clear();
                                    cx.notify();
                                });
                            }
                        },
                        cx,
                    );
                });
            }
            Ask::RemoveCard { slug, .. } => {
                self.state
                    .update(cx, |state, cx| state.remove_card(slug, reason, cx));
            }
        }
        cx.notify();
    }

    /// Writes a stage move: one card (a drop, which the board waits on) or the List's pick.
    fn move_cards(
        &mut self,
        slugs: Vec<String>,
        stage: Stage,
        reason: String,
        settle: Option<Rc<BoardMove>>,
        cx: &mut Context<Self>,
    ) {
        let view = cx.entity().downgrade();
        self.state
            .update(cx, |state, cx| match (settle, slugs.as_slice()) {
                (Some(change), [slug]) => state.set_card_stage(
                    slug.clone(),
                    stage,
                    reason,
                    move |ok, cx| change.settle(ok, cx),
                    cx,
                ),
                _ => state.set_cards_stage(
                    slugs,
                    stage,
                    reason,
                    move |ok, cx| {
                        if ok {
                            let _ = view.update(cx, |this, cx| {
                                this.selected.clear();
                                cx.notify();
                            });
                        }
                    },
                    cx,
                ),
            });
    }

    /// Moves `slugs` to `stage`, asking why first when that takes one of them back.
    fn request_stage(
        &mut self,
        slugs: Vec<String>,
        stage: Stage,
        settle: Option<Rc<BoardMove>>,
        cx: &mut Context<Self>,
    ) {
        let Some(work) = self.work(cx) else {
            if let Some(change) = settle {
                change.settle(false, cx);
            }
            return;
        };
        let cards = work.cards.iter().filter(|card| slugs.contains(&card.slug));
        if work::moves_back(cards, stage) {
            let what = if settle.is_some() {
                "the card".to_string()
            } else {
                work::card_count(slugs.len())
            };
            self.ask(
                Ask::MoveBack {
                    slugs,
                    stage,
                    what,
                    settle,
                },
                cx,
            );
        } else {
            self.move_cards(slugs, stage, String::new(), settle, cx);
        }
    }

    /// The question open, as a dialog over the page.
    fn ask_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let ask = self.ask.clone()?;
        if std::mem::take(&mut self.ask_focus_due) {
            self.ask_focus.focus(window);
        }
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let close: polterware_ui::Handler = {
            let view = view.clone();
            Rc::new(move |_, cx: &mut App| {
                let _ = view.update(cx, |this, cx| this.dismiss_ask(cx));
            })
        };
        let confirm: polterware_ui::Handler = Rc::new(move |_, cx: &mut App| {
            let _ = view.update(cx, |this, cx| this.answer(cx));
        });
        let archive_reason = |what: String| {
            (
                format!("Archive {what}?"),
                "Say why, so Activity and agents know it was dropped on purpose. Restore brings it back."
                    .to_string(),
                "Archive",
                "Why archive it?",
            )
        };
        let dialog = match &ask {
            Ask::ArchiveMilestone { name, cards, .. } => ConfirmDialog::new(
                "milestones-archive",
                format!("Archive “{name}”?"),
                if *cards > 0 {
                    format!("Its {} move to the Backlog.", work::card_count(*cards))
                } else {
                    "It has no cards.".to_string()
                },
            )
            .recovery(
                "The milestone stays in the store, archived. Its cards keep their stage and history but do not return to it.",
            )
            .confirm_label("Archive")
            .danger()
            .focus(&self.ask_focus),
            ask => {
                let (title, description, label, placeholder) = match ask {
                    Ask::MoveBack { stage, what, .. } => (
                        format!("Move {what} back to {}?", stage.label()),
                        "Moving back reopens work. Say why, so Activity and agents know what changed."
                            .to_string(),
                        "Move back",
                        "Why move it back?",
                    ),
                    Ask::ArchiveCards { slugs } => archive_reason(work::card_count(slugs.len())),
                    Ask::RemoveCard { title, .. } => archive_reason(format!("“{title}”")),
                    Ask::ArchiveMilestone { .. } => unreachable!("asked above"),
                };
                self.reason
                    .update(cx, |field, _| field.set_placeholder(placeholder));
                ConfirmDialog::new("milestones-reason", title, description)
                    .confirm_label(label)
                    .reason(&self.reason)
            }
        };
        let dialog = dialog.on_cancel(close.clone()).on_confirm(confirm);
        Some(window_layer(modal_layer(1.0, true, close, dialog, &theme)))
    }

    /// Asks before removing the open card from the board, from its page.
    pub(crate) fn ask_remove(&mut self, slug: String, title: String, cx: &mut Context<Self>) {
        self.ask(Ask::RemoveCard { slug, title }, cx);
    }

    /// Asks why the open card's stage moves back, or moves it, from its page.
    pub(crate) fn ask_stage(&mut self, slug: String, stage: Stage, cx: &mut Context<Self>) {
        let Some(work) = self.work(cx) else {
            return;
        };
        if work::moves_back(work.card(&slug), stage) {
            self.ask(
                Ask::MoveBack {
                    slugs: vec![slug],
                    stage,
                    what: "the card".into(),
                    settle: None,
                },
                cx,
            );
        } else {
            self.state.update(cx, |state, cx| {
                state.set_card_stage(slug, stage, String::new(), |_, _| {}, cx);
            });
        }
    }

    // The Roadmap.

    fn roadmap_rows(&self, work: &Work) -> Vec<RoadmapRow> {
        let needle = self.query.trim().to_lowercase();
        let (done, total) = work::progress(&work.cards, work::ALL_CARDS);
        std::iter::once(RoadmapRow {
            id: work::ALL_CARDS.into(),
            name: "All cards".into(),
            milestone: None,
            done,
            total,
        })
        .chain(
            work::listed_milestones(&work.milestones)
                .into_iter()
                .map(|milestone| {
                    let (done, total) = work::progress(&work.cards, &milestone.id);
                    RoadmapRow {
                        id: milestone.id.clone(),
                        name: milestone.name.clone(),
                        milestone: Some(milestone.clone()),
                        done,
                        total,
                    }
                }),
        )
        .filter(|row| needle.is_empty() || row.name.to_lowercase().contains(&needle))
        .collect()
    }

    /// A milestone's actions (`milestoneActions`), for its "…" and its right click.
    fn milestone_actions(
        &self,
        milestone: &Milestone,
        work: &Work,
        cx: &mut Context<Self>,
    ) -> Vec<MenuItem> {
        let view = cx.entity().downgrade();
        let pending = self.state.read(cx).work_pending();
        let mut items = Vec::new();
        let id = milestone.id.clone();
        {
            let view = view.clone();
            let id = id.clone();
            items.push(MenuItem::action(
                "Rename",
                Some(Icon::Pencil),
                Rc::new(move |window, cx: &mut App| {
                    let _ = view.update(cx, |this, cx| {
                        this.open_composer(Compose::Rename(id.clone()), window, cx);
                    });
                }),
            ));
        }
        for (label, icon, direction) in [
            ("Move up", Icon::ArrowUp, -1),
            ("Move down", Icon::ArrowDown, 1),
        ] {
            let Some(ids) = work::move_milestone(&work.milestones, &id, direction) else {
                continue;
            };
            let state = self.state.clone();
            items.push(MenuItem::action(
                label,
                Some(icon),
                Rc::new(move |_, cx: &mut App| {
                    if pending {
                        return;
                    }
                    let ids = ids.clone();
                    state.update(cx, |state, cx| {
                        state.milestone_action(MilestoneAction::Reorder { ids }, |_, _| {}, cx);
                    });
                }),
            ));
        }
        items.push(MenuItem::Separator);
        let cards = work::cards_in_view(&work.cards, &id).len();
        let name = milestone.name.clone();
        items.push(MenuItem::danger(
            "Archive",
            Some(Icon::Archive),
            Rc::new(move |_, cx: &mut App| {
                let (id, name) = (id.clone(), name.clone());
                let _ = view.update(cx, |this, cx| {
                    this.ask(Ask::ArchiveMilestone { id, name, cards }, cx);
                });
            }),
        ));
        items
    }

    fn roadmap_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let ready = self.work(cx).is_some();
        let actions = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(div().w(px(200.0)).child(self.search.clone()))
            .child(
                Button::new("milestones-create", ButtonKind::Primary)
                    .size(ButtonSize::Icon)
                    .icon(Icon::Plus)
                    .tooltip("New milestone")
                    .disabled(!ready)
                    .on_click(move |window, cx| {
                        let _ = view.update(cx, |this, cx| {
                            this.open_composer(Compose::Milestone, window, cx)
                        });
                    }),
            );
        div()
            .flex_none()
            .h(px(theme.page_header_height))
            .pt(px(theme.space_2))
            .px(px(theme.space_2))
            .pb(px(theme.space_1))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(theme.space_3))
            .child(div().flex_1().h_full().window_drag_region())
            .child(actions)
            .into_any_element()
    }

    fn roadmap(&self, work: &Work, cx: &mut Context<Self>) -> AnyElement {
        let rows = self.roadmap_rows(work);
        if rows.is_empty() {
            let theme = Theme::global(cx);
            return page_empty_with(
                Some("No board matches"),
                MascotMood::Lost,
                Some(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.color_text_muted)
                        .child("Try another name.")
                        .into_any_element(),
                ),
                cx,
            )
            .into_any_element();
        }
        let rows = Rc::new(rows);
        let marked = self
            .roadmap_active
            .as_ref()
            .and_then(|id| rows.iter().position(|row| &row.id == id))
            .or(Some(0));
        let columns = vec![
            Column::new("Board").shrinks(),
            Column::new("State"),
            Column::new("Progress"),
            Column::new("Target"),
            Column::new("").width(44.0).fixed().end().resizable(false),
        ];
        let cells = rows.clone();
        let measured = rows.clone();
        let menu = cx.entity().downgrade();
        let table = DataTable::new(
            "milestones-roadmap",
            columns,
            rows.len(),
            move |row, _, cx| {
                let theme = *Theme::global(cx);
                let board = &cells[row.index];
                let icon = if board.milestone.is_some() {
                    Icon::FolderKanban
                } else {
                    Icon::Layers
                };
                let title = primary_cell(
                    IconTile::new(icon)
                        .size(24.0, 16.0)
                        .tone(TileTone::Knowledge),
                    SharedString::from(board.name.clone()),
                    sans(&theme, FontWeight::MEDIUM),
                    None,
                    cx,
                );
                let state = match &board.milestone {
                    Some(milestone) => chip_rounded(
                        work::milestone_state_label(&milestone.state),
                        state_tone(&milestone.state),
                        CHIP_RADIUS,
                        cx,
                    )
                    .into_any_element(),
                    None => div().into_any_element(),
                };
                let complete = board.total > 0 && board.done == board.total;
                let fraction = if board.total > 0 {
                    board.done as f32 / board.total as f32
                } else {
                    0.0
                };
                let progress = div()
                    .flex()
                    .min_w(px(0.0))
                    .items_center()
                    .gap(px(10.0))
                    .child(div().w(px(96.0)).flex_none().child(progress_bar_sm(
                        fraction,
                        if complete {
                            theme.color_success
                        } else {
                            theme.color_accent
                        },
                        cx,
                    )))
                    .child(
                        div()
                            .font(tabular(sans(&theme, FontWeight::NORMAL)))
                            .text_size(px(12.5))
                            .text_color(theme.color_text_muted)
                            .child(if board.total > 0 {
                                format!("{}/{}", board.done, board.total)
                            } else {
                                work::card_count(0)
                            }),
                    )
                    .into_any_element();
                let target = board
                    .milestone
                    .as_ref()
                    .map(|milestone| milestone.target_date.clone())
                    .filter(|date| !date.is_empty())
                    .unwrap_or_else(|| "—".into());
                let more = match (&board.milestone, row.hovered) {
                    (Some(milestone), true) => {
                        let id = milestone.id.clone();
                        let menu = menu.clone();
                        Button::new(("milestones-more", row.index), ButtonKind::Ghost)
                            .size(ButtonSize::Icon)
                            .icon(Icon::MoreHorizontal)
                            .tooltip(format!("Actions for {}", board.name))
                            .on_click(move |window, cx| {
                                let id = id.clone();
                                let _ = menu
                                    .update(cx, |this, cx| this.open_menu(&id, None, window, cx));
                            })
                            .into_any_element()
                    }
                    _ => div().into_any_element(),
                };
                vec![title, state, progress, muted_cell(target, cx), more]
            },
        )
        .measure(move |range, window, cx| {
            let theme = *Theme::global(cx);
            let medium = sans(&theme, FontWeight::MEDIUM);
            let normal = sans(&theme, FontWeight::NORMAL);
            let mut widest = [0.0_f32; 5];
            for board in measured.get(range).unwrap_or_default() {
                let state = board.milestone.as_ref().map_or(0.0, |milestone| {
                    CHIP_FRAME
                        + text_width(
                            work::milestone_state_label(&milestone.state),
                            medium.clone(),
                            11.0,
                            window,
                        )
                });
                let count = if board.total > 0 {
                    format!("{}/{}", board.done, board.total)
                } else {
                    work::card_count(0)
                };
                let target = board
                    .milestone
                    .as_ref()
                    .map_or("—".to_string(), |milestone| {
                        if milestone.target_date.is_empty() {
                            "—".into()
                        } else {
                            milestone.target_date.clone()
                        }
                    });
                let widths = [
                    title_width(&board.name, window, cx),
                    state,
                    96.0 + 10.0 + text_width(&count, tabular(normal.clone()), 12.5, window),
                    text_width(&target, normal.clone(), 12.5, window),
                    0.0,
                ];
                for (slot, width) in widest.iter_mut().zip(widths) {
                    *slot = slot.max(width);
                }
            }
            widest.to_vec()
        });
        let open_state = self.state.clone();
        let open_rows = rows.clone();
        let open_view = cx.entity().downgrade();
        let context_rows = rows.clone();
        let context_view = cx.entity().downgrade();
        let table = table
            .active(marked)
            .on_activate(move |index, _, cx| {
                if let Some(row) = open_rows.get(index) {
                    let id = row.id.clone();
                    let _ = open_view.update(cx, |this, _| this.roadmap_active = Some(id.clone()));
                    open_state.update(cx, |state, cx| {
                        state.open_board(Some(id), cx);
                        cx.notify();
                    });
                }
            })
            .on_context(move |index, at, window, cx| {
                if let Some(row) = context_rows.get(index)
                    && row.milestone.is_some()
                {
                    let id = row.id.clone();
                    let _ = context_view
                        .update(cx, |this, cx| this.open_menu(&id, Some(at), window, cx));
                }
            });
        div()
            .relative()
            .size_full()
            .child(table.gutters(TABLE_INSET).measure_every_row().freeze_once())
            .into_any_element()
    }

    fn open_menu(
        &mut self,
        id: &str,
        at: Option<gpui::Point<gpui::Pixels>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(work) = self.work(cx) else {
            return;
        };
        let Some(milestone) = work.milestone(id).cloned() else {
            return;
        };
        let items = self.milestone_actions(&milestone, &work, cx);
        let at = at.unwrap_or_else(|| window.mouse_position());
        self.menu
            .update(cx, |menu, cx| menu.open(at, items, window, cx));
    }

    // A board.

    /// `changeTab` and a new board: an empty search, no clauses, nothing picked.
    fn follow_view(&mut self, view: Option<&str>, cx: &mut Context<Self>) {
        if self.view.as_deref() == view {
            return;
        }
        self.view = view.map(str::to_string);
        self.card_query.clear();
        self.card_search
            .update(cx, |field, cx| field.set_text("", cx));
        clear_page(&self.filters, cx);
        self.selected.clear();
        self.active = None;
        self.tasks_open = None;
    }

    /// The board's cards that the search and filters keep (`filterWorkCards`).
    fn visible_cards<'a>(&self, work: &'a Work, view: &str, cx: &App) -> Vec<&'a WorkCard> {
        let needle = self.card_query.trim().to_lowercase();
        let filters = self.filters.read(cx).filters();
        let stages = filters.values_of("stage");
        let open_questions = !filters.values_of("questions").is_empty();
        work::cards_in_view(&work.cards, view)
            .into_iter()
            .filter(|card| stages.is_empty() || stages.iter().any(|stage| stage == &card.stage))
            .filter(|card| {
                !open_questions
                    || card
                        .questions
                        .iter()
                        .any(|question| question.state == "open")
            })
            .filter(|card| {
                needle.is_empty()
                    || [
                        &card.title,
                        &card.slug,
                        &card.intent,
                        &card.owner,
                        &card.under,
                    ]
                    .iter()
                    .any(|value| value.to_lowercase().contains(&needle))
            })
            .collect()
    }

    fn board_toolbar(
        &self,
        name: &str,
        tab: BoardTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let back_state = self.state.clone();
        let back = Button::new("milestones-back", ButtonKind::Secondary)
            .size(ButtonSize::Icon)
            .icon(Icon::ChevronLeft)
            .tooltip("Back to Milestones")
            .on_click(move |_, cx| {
                back_state.update(cx, |state, cx| {
                    state.open_board(None, cx);
                    cx.notify();
                });
            });
        let tab_state = self.state.clone();
        let tabs = BoardTab::ALL
            .iter()
            .fold(Tabs::new("milestones-board-tab"), |tabs, tab| {
                tabs.tab(tab.label())
            })
            .selected(BoardTab::ALL.iter().position(|t| *t == tab).unwrap_or(0))
            .on_select(move |index, _, cx| {
                if let Some(tab) = BoardTab::ALL.get(index).copied() {
                    tab_state.update(cx, |state, cx| {
                        state.set_board_tab(tab, cx);
                        cx.notify();
                    });
                }
            });
        let _ = name;
        let actions = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(div().w(px(200.0)).child(self.card_search.clone()))
            .child(self.filters.clone());
        let bar = div()
            .flex_none()
            .h(px(theme.page_header_height))
            .pt(px(theme.space_2))
            .px(px(theme.space_2))
            .pb(px(theme.space_1))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(theme.space_3))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(back)
                    .child(tabs),
            )
            .child(div().flex_1().h_full().window_drag_region())
            .child(actions);
        let filter_bar = FilterPopover::bar(&self.filters, window, cx).map(|bar| {
            div()
                .flex_none()
                .px(px(theme.space_2))
                .pb(px(theme.space_2))
                .child(bar)
        });
        div()
            .flex_none()
            .flex()
            .flex_col()
            .child(bar)
            .children(filter_bar)
            .into_any_element()
    }

    /// A card's Tasks on the Board (`.pw-work-card__expandable`): "Tasks d/t" that morphs into
    /// the list of its tasks, each checked off or opened again.
    fn tasks_morph(&self, card: &WorkCard, cx: &mut Context<Self>) -> Option<AnyElement> {
        let tasks: Vec<_> = work::live_tasks(card).into_iter().cloned().collect();
        if tasks.is_empty() {
            return None;
        }
        let theme = *Theme::global(cx);
        let (done, total) = work::task_progress(card);
        let open = self.tasks_open.as_deref() == Some(card.slug.as_str());
        let view = cx.entity().downgrade();
        let slug = card.slug.clone();
        let trigger = Button::new(
            SharedString::from(format!("board-tasks-{}", card.slug)),
            ButtonKind::Secondary,
        )
        .size(ButtonSize::Sm)
        .full_width()
        .label(format!("Tasks {done}/{total}"))
        .trailing(polterware_ui::icons::icon_colored(
            Icon::ChevronDown,
            14.0,
            theme.color_text_muted,
        ));
        let set_open = {
            let view = view.clone();
            let slug = slug.clone();
            move |open: bool, _: &mut Window, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    this.tasks_open = open.then(|| slug.clone());
                    cx.notify();
                });
            }
        };
        let pending = self.task_pending;
        let rows = tasks.into_iter().map(|task| {
            let checked = task.state == "done";
            let state = self.state.clone();
            let slug = slug.clone();
            let toggle_task = task.clone();
            let view = view.clone();
            div()
                .id(SharedString::from(format!(
                    "board-task-{}-{}",
                    slug, task.number
                )))
                .flex()
                .min_w(px(0.0))
                .items_start()
                .gap(px(8.0))
                .py(px(4.0))
                .text_color(if checked {
                    theme.color_text_muted
                } else {
                    theme.color_text_secondary
                })
                .when(!pending, |row| row.cursor_pointer())
                .on_click(move |_, _, cx| {
                    if pending {
                        return;
                    }
                    let next = if checked { "open" } else { "done" };
                    let task = toggle_task.clone();
                    let slug = slug.clone();
                    let _ = view.update(cx, |this, _| this.task_pending = true);
                    state.update(cx, |state, cx| {
                        state.set_task_state(slug, &task, next, cx);
                        cx.notify();
                    });
                    let _ = view.update(cx, |this, cx| {
                        this.task_pending = false;
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .mt(px(1.0))
                        .flex_none()
                        .child(checkbox_mark(checked, 14.0, &theme)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .gap(px(1.0))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(18.0))
                                .child(task.title.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_x(px(8.0))
                                .gap_y(px(2.0))
                                .text_size(px(11.0))
                                .text_color(theme.color_text_muted)
                                .child(
                                    div()
                                        .font_family(theme.font_mono)
                                        .child(format!("T{}", task.number)),
                                )
                                .child(work::task_status(&task))
                                .when(!task.scope.is_empty(), |meta| {
                                    meta.child(
                                        div()
                                            .max_w_full()
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .whitespace_nowrap()
                                            .font_family(theme.font_mono)
                                            .child(task.scope.clone()),
                                    )
                                }),
                        ),
                )
                .into_any_element()
        });
        let morph = Morph::new(
            SharedString::from(format!("board-tasks-morph-{}", card.slug)),
            trigger,
            format!("{} tasks", card.title),
        )
        .open(open, set_open)
        .width(300.0)
        .align(MorphAlign::Start)
        .cancel_label("Close")
        .body(div().flex().flex_col().gap(px(2.0)).children(rows));
        Some(
            div()
                .flex()
                .px(px(6.0))
                .pb(px(6.0))
                .child(morph)
                .into_any_element(),
        )
    }

    fn board(
        &self,
        work: &Work,
        cards: &[&WorkCard],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let order: std::collections::HashMap<&str, usize> = work
            .cards
            .iter()
            .enumerate()
            .map(|(index, card)| (card.slug.as_str(), index))
            .collect();
        let mut built = board("milestones-board");
        for stage in Stage::ALL {
            let column_cards: Vec<BoardCard> = cards
                .iter()
                .filter(|card| Stage::of(card) == stage)
                .map(|card| {
                    let open_state = self.state.clone();
                    let slug = card.slug.clone();
                    let mut board_card = BoardCard::new(
                        card.slug.clone(),
                        card.title.clone(),
                        card.slug.clone(),
                        order.get(card.slug.as_str()).copied().unwrap_or(usize::MAX),
                    )
                    .on_activate(Rc::new(move |_, cx: &mut App| {
                        let slug = slug.clone();
                        open_state.update(cx, |state, cx| {
                            state.open_card(slug, cx);
                            cx.notify();
                        });
                    }));
                    if let Some(footer) = self.tasks_morph(card, cx) {
                        board_card = board_card.footer(footer);
                    }
                    board_card
                })
                .collect();
            let view = cx.entity().downgrade();
            built = built.column(
                BoardColumn::new(stage.id(), stage.label(), stage_tile(stage, &theme))
                    .cards(column_cards)
                    .on_add(Rc::new(move |_, cx: &mut App| {
                        let _ = view.update(cx, |this, cx| this.compose_card(Some(stage), cx));
                    })),
            );
        }
        let view = cx.entity().downgrade();
        built
            .on_move(Rc::new(move |change: BoardMove, _, cx: &mut App| {
                let Some(stage) = Stage::parse(&change.column) else {
                    change.settle(false, cx);
                    return;
                };
                let slug = change.card.to_string();
                let change = Rc::new(change);
                let settle = change.clone();
                if view
                    .update(cx, |this, cx| {
                        this.request_stage(vec![slug], stage, Some(change), cx)
                    })
                    .is_err()
                {
                    settle.settle(false, cx);
                }
            }))
            .build(window, cx)
    }

    /// The List's columns on show: the card, then the display's properties.
    fn list_columns(&self) -> Vec<ListProperty> {
        LIST_PROPERTIES
            .into_iter()
            .filter(|property| self.display.properties.contains(property))
            .collect()
    }

    fn list(&self, work: &Work, cards: &[&WorkCard], cx: &mut Context<Self>) -> AnyElement {
        if cards.is_empty() {
            return page_empty("No cards", MascotMood::Sleeping, cx).into_any_element();
        }
        let display = self.display.clone();
        let ordered = work::order_cards(cards, display.ordering, display.descending);
        // Rows come in group order, each group in the display's order.
        let group_list = work::list_groups(display.grouping, &work.milestones);
        let mut rows: Vec<WorkCard> = Vec::new();
        let mut groups = Vec::new();
        if group_list.is_empty() {
            rows = ordered.iter().map(|card| (*card).clone()).collect();
        } else {
            for group in &group_list {
                let start = rows.len();
                rows.extend(
                    ordered
                        .iter()
                        .filter(|card| work::list_group_of(card, display.grouping) == group.key)
                        .map(|card| (*card).clone()),
                );
                if rows.len() > start {
                    let mut table_group =
                        TableGroup::new(group.key.clone(), group.label.clone(), start..rows.len());
                    if display.grouping == Grouping::Stage
                        && let Some(stage) = Stage::parse(&group.key)
                    {
                        let view = cx.entity().downgrade();
                        table_group = table_group.on_add(
                            format!("New card in {}", group.label),
                            Rc::new(move |_, cx: &mut App| {
                                let _ =
                                    view.update(cx, |this, cx| this.compose_card(Some(stage), cx));
                            }),
                        );
                    }
                    groups.push(table_group);
                }
            }
        }
        let rows = Rc::new(rows);
        let properties = self.list_columns();
        let mut columns = vec![Column::new("Card").sortable().shrinks()];
        for property in &properties {
            let column = Column::new(property.label());
            columns.push(match property {
                ListProperty::Stage | ListProperty::Updated => column.sortable(),
                _ => column,
            });
        }
        let milestones = Rc::new(work.milestones.clone());
        let nodes = Rc::new(work.nodes.clone());
        let cells = rows.clone();
        let cell_properties = properties.clone();
        let cell_milestones = milestones.clone();
        let cell_nodes = nodes.clone();
        let table = DataTable::new("milestones-list", columns, rows.len(), move |row, _, cx| {
            let theme = *Theme::global(cx);
            let card = &cells[row.index];
            let mut out = vec![primary_cell(
                IconTile::new(Icon::ListChecks)
                    .size(24.0, 16.0)
                    .tone(TileTone::Knowledge),
                SharedString::from(card.title.clone()),
                sans(&theme, FontWeight::MEDIUM),
                None,
                cx,
            )];
            for property in &cell_properties {
                out.push(list_cell(
                    *property,
                    card,
                    &cell_milestones,
                    &cell_nodes,
                    cx,
                ));
            }
            out
        });
        let measured = rows.clone();
        let measure_properties = properties.clone();
        let measure_milestones = milestones.clone();
        let measure_nodes = nodes.clone();
        let table = table.measure(move |range, window, cx| {
            let mut widest = vec![0.0_f32; measure_properties.len() + 1];
            for card in measured.get(range).unwrap_or_default() {
                widest[0] = widest[0].max(title_width(&card.title, window, cx));
                for (slot, property) in widest.iter_mut().skip(1).zip(&measure_properties) {
                    *slot = slot.max(list_cell_width(
                        *property,
                        card,
                        &measure_milestones,
                        &measure_nodes,
                        window,
                        cx,
                    ));
                }
            }
            widest
        });
        let ids: Vec<String> = rows.iter().map(|card| card.slug.clone()).collect();
        let picked: Vec<usize> = ids
            .iter()
            .enumerate()
            .filter(|(_, slug)| self.selected.contains(*slug))
            .map(|(index, _)| index)
            .collect();
        let marked = self
            .active
            .as_ref()
            .and_then(|slug| ids.iter().position(|id| id == slug))
            .or(Some(0));
        // The header shows the display's ordering; a click orders by that column.
        let sort_column = |ordering: Ordering| -> Option<usize> {
            match ordering {
                Ordering::Title => Some(0),
                Ordering::Stage => properties
                    .iter()
                    .position(|p| *p == ListProperty::Stage)
                    .map(|i| i + 1),
                Ordering::Updated => properties
                    .iter()
                    .position(|p| *p == ListProperty::Updated)
                    .map(|i| i + 1),
            }
        };
        let sort = sort_column(display.ordering).map(|column| {
            (
                column,
                if display.descending {
                    SortDirection::Descending
                } else {
                    SortDirection::Ascending
                },
            )
        });
        let column_ordering: Vec<Option<Ordering>> = std::iter::once(Some(Ordering::Title))
            .chain(properties.iter().map(|property| match property {
                ListProperty::Stage => Some(Ordering::Stage),
                ListProperty::Updated => Some(Ordering::Updated),
                _ => None,
            }))
            .collect();
        let view = cx.entity().downgrade();
        let sort_view = view.clone();
        let select_view = view.clone();
        let select_ids = ids.clone();
        let move_view = view.clone();
        let move_ids = ids.clone();
        let open_state = self.state.clone();
        let open_ids = ids.clone();
        let mut table = table
            .groups(groups)
            .active(marked)
            .selection(picked, move |change, _, cx| {
                let _ = select_view.update(cx, |this, cx| {
                    match change {
                        SelectionChange::Toggle(index) => {
                            if let Some(slug) = select_ids.get(index)
                                && !this.selected.remove(slug)
                            {
                                this.selected.insert(slug.clone());
                            }
                        }
                        SelectionChange::All(all) => {
                            this.selected = if all {
                                select_ids.iter().cloned().collect()
                            } else {
                                BTreeSet::new()
                            };
                        }
                    }
                    cx.notify();
                });
            })
            .sort(sort, move |column, _, cx| {
                let _ = sort_view.update(cx, |this, cx| {
                    let Some(Some(ordering)) = column_ordering.get(column).copied() else {
                        return;
                    };
                    match next_sort(sort, column) {
                        Some((_, direction)) => {
                            this.display.ordering = ordering;
                            this.display.descending = direction == SortDirection::Descending;
                        }
                        None => {
                            let default = work::Display::default();
                            this.display.ordering = default.ordering;
                            this.display.descending = default.descending;
                        }
                    }
                    cx.notify();
                });
            })
            .on_move(move |index, _, cx| {
                if let Some(slug) = move_ids.get(index).cloned() {
                    let _ = move_view.update(cx, |this, cx| {
                        this.active = Some(slug);
                        cx.notify();
                    });
                }
            })
            .on_activate(move |index, _, cx| {
                if let Some(slug) = open_ids.get(index).cloned() {
                    let _ = view.update(cx, |this, _| this.active = Some(slug.clone()));
                    open_state.update(cx, |state, cx| {
                        state.open_card(slug, cx);
                        cx.notify();
                    });
                }
            });
        table = table.gutters(TABLE_INSET).measure_every_row().freeze_once();
        let mut list = div()
            .relative()
            .size_full()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                // Space peeks at the marked card (`onPeek`).
                if event.keystroke.key == "space"
                    && !event.keystroke.modifiers.modified()
                    && let Some(slug) = this.active.clone().or_else(|| this.first_list_card(cx))
                {
                    cx.stop_propagation();
                    this.open_peek(slug, window, cx);
                }
            }))
            .child(table);
        if !self.selected.is_empty() {
            list = list.child(self.selection_bar(work, cx));
        }
        list.into_any_element()
    }

    fn first_list_card(&self, cx: &App) -> Option<String> {
        let work = self.work(cx)?;
        let view = self.view.clone().unwrap_or_default();
        let cards = self.visible_cards(&work, &view, cx);
        work::order_cards(&cards, self.display.ordering, self.display.descending)
            .first()
            .map(|card| card.slug.clone())
    }

    /// The List's selection toolbar: Move to…, Milestone…, and Archive.
    fn selection_bar(&self, work: &Work, cx: &mut Context<Self>) -> AnyElement {
        let slugs: Vec<String> = self.selected.iter().cloned().collect();
        let pending = self.state.read(cx).work_pending();
        let view = cx.entity().downgrade();
        let stage_view = view.clone();
        let stage_slugs = slugs.clone();
        let move_to = Button::new("milestones-move-to", ButtonKind::Ghost)
            .size(ButtonSize::Sm)
            .label("Move to…")
            .disabled(pending)
            .on_click(move |window, cx| {
                let slugs = stage_slugs.clone();
                let _ = stage_view.update(cx, |this, cx| this.pick_stage(slugs, window, cx));
            })
            .into_any_element();
        let milestone_view = view.clone();
        let milestone_slugs = slugs.clone();
        let milestones: Vec<(String, String)> =
            std::iter::once((String::new(), "Backlog".to_string()))
                .chain(
                    work::listed_milestones(&work.milestones)
                        .into_iter()
                        .map(|milestone| (milestone.slug.clone(), milestone.name.clone())),
                )
                .collect();
        let to_milestone = Button::new("milestones-to-milestone", ButtonKind::Ghost)
            .size(ButtonSize::Sm)
            .label("Milestone…")
            .disabled(pending)
            .on_click(move |window, cx| {
                let slugs = milestone_slugs.clone();
                let milestones = milestones.clone();
                let _ = milestone_view.update(cx, |this, cx| {
                    this.pick_milestone(slugs, milestones, window, cx)
                });
            })
            .into_any_element();
        let archive_view = view.clone();
        let archive = Button::new("milestones-archive-cards", ButtonKind::Ghost)
            .size(ButtonSize::Sm)
            .icon(Icon::Archive)
            .label("Archive")
            .disabled(pending)
            .on_click(move |_, cx| {
                let slugs = slugs.clone();
                let _ =
                    archive_view.update(cx, |this, cx| this.ask(Ask::ArchiveCards { slugs }, cx));
            })
            .into_any_element();
        let clear = view.clone();
        selection_toolbar(
            self.selected.len(),
            vec![move_to, to_milestone, archive],
            "",
            Rc::new(move |_, cx: &mut App| {
                let _ = clear.update(cx, |this, cx| {
                    this.selected.clear();
                    cx.notify();
                });
            }),
            cx,
        )
    }

    /// Move to…: the stages, as the Stage picker lists them.
    fn pick_stage(&mut self, slugs: Vec<String>, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().downgrade();
        let items = Stage::ALL
            .iter()
            .map(|stage| {
                let (view, slugs, stage) = (view.clone(), slugs.clone(), *stage);
                MenuItem::action(
                    stage.label(),
                    None,
                    Rc::new(move |_, cx: &mut App| {
                        let slugs = slugs.clone();
                        let _ =
                            view.update(cx, |this, cx| this.request_stage(slugs, stage, None, cx));
                    }),
                )
            })
            .collect();
        let at = window.mouse_position();
        self.menu
            .update(cx, |menu, cx| menu.open(at, items, window, cx));
    }

    /// Milestone…: the Backlog, then the milestones.
    fn pick_milestone(
        &mut self,
        slugs: Vec<String>,
        milestones: Vec<(String, String)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = cx.entity().downgrade();
        let state = self.state.clone();
        let items = milestones
            .into_iter()
            .map(|(slug, name)| {
                let (view, state, cards) = (view.clone(), state.clone(), slugs.clone());
                let icon = if slug.is_empty() {
                    Icon::Inbox
                } else {
                    Icon::Folder
                };
                MenuItem::action(
                    name,
                    Some(icon),
                    Rc::new(move |_, cx: &mut App| {
                        let (cards, slug, view) = (cards.clone(), slug.clone(), view.clone());
                        state.update(cx, |state, cx| {
                            state.set_cards_milestone(
                                cards,
                                slug,
                                move |ok, cx| {
                                    if ok {
                                        let _ = view.update(cx, |this, cx| {
                                            this.selected.clear();
                                            cx.notify();
                                        });
                                    }
                                },
                                cx,
                            );
                        });
                    }),
                )
            })
            .collect();
        let at = window.mouse_position();
        self.menu
            .update(cx, |menu, cx| menu.open(at, items, window, cx));
    }

    // The Peek.

    fn open_peek(&mut self, slug: String, window: &mut Window, cx: &mut Context<Self>) {
        self.peek = Some(slug);
        self.peek_presence.set_open(true);
        self.peek_focus.focus(window);
        cx.notify();
    }

    pub(crate) fn close_peek(&mut self, cx: &mut Context<Self>) -> bool {
        if self.peek.is_none() || !self.peek_presence.is_open() {
            return false;
        }
        self.peek_presence.set_open(false);
        cx.notify();
        true
    }

    /// `CardPeek`: the card's stage, slug, and component, its intent, and what is open on it.
    fn peek_layer(
        &mut self,
        work: &Work,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.peek_presence.is_present() {
            self.peek = None;
            return None;
        }
        let slug = self.peek.clone()?;
        let Some(card) = work.card(&slug).cloned() else {
            self.peek = None;
            return None;
        };
        let theme = *Theme::global(cx);
        let pose = self.peek_presence.frame(window);
        let view = cx.entity().downgrade();
        let close: polterware_ui::Handler = {
            let view = view.clone();
            Rc::new(move |_, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    this.close_peek(cx);
                });
            })
        };
        let open_state = self.state.clone();
        let open_slug = card.slug.clone();
        let open: polterware_ui::Handler = {
            let view = view.clone();
            Rc::new(move |_, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    this.close_peek(cx);
                });
                let slug = open_slug.clone();
                open_state.update(cx, |state, cx| {
                    state.open_card(slug, cx);
                    cx.notify();
                });
            })
        };
        let stage = Stage::of(&card);
        let mut properties = vec![
            chip_rounded(stage.label(), stage_tone(stage), CHIP_RADIUS, cx).into_any_element(),
            peek_chip(None, card.slug.clone(), true, cx).into_any_element(),
        ];
        if !card.under.is_empty() {
            properties.push(
                peek_chip(Some(Icon::Layers), card.under.clone(), false, cx).into_any_element(),
            );
        }
        let intent = if card.intent.trim().is_empty() {
            div()
                .text_color(theme.color_text_muted)
                .child("No intent written yet.")
        } else {
            div().child(card.intent.clone())
        };
        let open_tasks: Vec<AnyElement> = card
            .tasks
            .iter()
            .filter(|task| task.state == "open")
            .map(|task| {
                div()
                    .flex()
                    .flex_wrap()
                    .items_baseline()
                    .text_color(theme.color_text_primary)
                    .child(
                        div()
                            .mr(px(8.0))
                            .font_family(theme.font_mono)
                            .text_size(px(12.0))
                            .text_color(theme.color_text_muted)
                            .child(format!("T{}", task.number)),
                    )
                    .child(task.title.clone())
                    .when(!task.claim_holder.is_empty(), |row| {
                        row.child(
                            div()
                                .ml(px(8.0))
                                .text_size(px(12.0))
                                .text_color(theme.color_text_muted)
                                .child(format!("/ {}", task.claim_holder)),
                        )
                    })
                    .into_any_element()
            })
            .collect();
        let open_questions: Vec<AnyElement> = card
            .questions
            .iter()
            .filter(|question| question.state == "open")
            .map(|question| {
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .text_color(theme.color_text_primary)
                    .child(question.text.clone())
                    .when(question.blocking, |row| {
                        row.child(div().ml(px(8.0)).child(chip_rounded(
                            "Blocking",
                            StatusTone::Danger,
                            CHIP_RADIUS,
                            cx,
                        )))
                    })
                    .into_any_element()
            })
            .collect();
        let mut sheet = peek(card.title.clone(), "Card", pose)
            .icon(IconTile::new(Icon::FolderKanban).size(24.0, 14.0))
            .properties(properties)
            .on_close(close.clone())
            .on_open(open)
            .child(intent);
        if !open_tasks.is_empty() {
            sheet = sheet.child(peek_section("Open tasks", open_tasks, cx));
        }
        if !open_questions.is_empty() {
            sheet = sheet.child(peek_section("Open questions", open_questions, cx));
        }
        let body = div()
            .track_focus(&self.peek_focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    cx.stop_propagation();
                    this.close_peek(cx);
                }
            }))
            .child(sheet);
        Some(window_layer(modal_layer(
            pose.opacity,
            self.peek_presence.is_open(),
            close,
            body,
            &theme,
        )))
    }
}

/// A List cell for `property`.
fn list_cell(
    property: ListProperty,
    card: &WorkCard,
    milestones: &[Milestone],
    nodes: &[StructureNode],
    cx: &App,
) -> AnyElement {
    let theme = Theme::global(cx);
    match property {
        ListProperty::Slug => div()
            .font_family(theme.font_mono)
            .text_size(px(12.0))
            .whitespace_nowrap()
            .child(card.slug.clone())
            .into_any_element(),
        ListProperty::Context => chips(&context_labels(&card_context(card, nodes), nodes), cx),
        ListProperty::Stage => {
            let stage = Stage::of(card);
            chip_rounded(stage.label(), stage_tone(stage), CHIP_RADIUS, cx).into_any_element()
        }
        ListProperty::Tasks => muted_cell(task_count(card), cx),
        ListProperty::Milestone => muted_cell(milestone_name(card, milestones), cx),
        ListProperty::Updated => muted_cell(format_instant(&card.updated_at), cx),
    }
}

fn list_cell_width(
    property: ListProperty,
    card: &WorkCard,
    milestones: &[Milestone],
    nodes: &[StructureNode],
    window: &Window,
    cx: &App,
) -> f32 {
    let theme = Theme::global(cx);
    let normal = sans(theme, FontWeight::NORMAL);
    let medium = sans(theme, FontWeight::MEDIUM);
    match property {
        ListProperty::Slug => text_width(
            &card.slug,
            polterware_ui::data_table::mono(theme, FontWeight::NORMAL),
            12.0,
            window,
        ),
        ListProperty::Context => chips_width(
            &context_labels(&card_context(card, nodes), nodes),
            window,
            cx,
        ),
        ListProperty::Stage => {
            CHIP_FRAME + text_width(Stage::of(card).label(), medium, 11.0, window)
        }
        ListProperty::Tasks => text_width(&task_count(card), normal, 12.5, window),
        ListProperty::Milestone => {
            text_width(&milestone_name(card, milestones), normal, 12.5, window)
        }
        ListProperty::Updated => {
            text_width(&format_instant(&card.updated_at), normal, 12.5, window)
        }
    }
}

/// The nodes a card is under (`contextOf`): the one whose path its `under` names.
fn card_context(card: &WorkCard, nodes: &[StructureNode]) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| !card.under.is_empty() && node.path == card.under)
        .map(|node| node.id.clone())
        .collect()
}

fn task_count(card: &WorkCard) -> String {
    let (done, total) = work::task_progress(card);
    if total > 0 {
        format!("{done}/{total}")
    } else {
        "—".into()
    }
}

fn milestone_name(card: &WorkCard, milestones: &[Milestone]) -> String {
    milestones
        .iter()
        .find(|milestone| !card.milestone_id.is_empty() && milestone.id == card.milestone_id)
        .map(|milestone| milestone.name.clone())
        .unwrap_or_else(|| "Backlog".into())
}

impl Render for MilestonesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scope.sync(&self.filters, &self.state, cx);
        let theme = *Theme::global(cx);
        if let Some(page) = core_status(&self.state, cx) {
            return div().size_full().child(page);
        }
        let app = self.state.read(cx);
        let Some((board_route, page)) = app.board_route() else {
            return div().size_full();
        };
        let (view, tab, page_open) = (board_route.view.clone(), board_route.tab, page.is_some());
        let work = match app.work() {
            Load::Ready(work) => Some(work.clone()),
            Load::Failed(message) => {
                let message = message.clone();
                return div().size_full().child(page_empty_with(
                    Some("Could not read Milestones"),
                    MascotMood::Sad,
                    Some(
                        div()
                            .text_size(px(theme.font_size_sm))
                            .text_color(theme.color_text_muted)
                            .child(SharedString::from(message))
                            .into_any_element(),
                    ),
                    cx,
                ));
            }
            _ => None,
        };
        let ask = self.ask_dialog(window, cx);
        if page_open {
            // The card's page takes the board's place, toolbar and all.
            return div()
                .size_full()
                .child(self.page.clone())
                .children(ask)
                .child(self.menu.clone());
        }
        let body = match (&view, &work) {
            (_, None) => div()
                .size_full()
                .flex()
                .flex_col()
                .child(match &view {
                    None => self.roadmap_toolbar(cx),
                    Some(_) => self.board_toolbar("", tab, window, cx),
                })
                .child(
                    div()
                        .px(px(TABLE_INSET))
                        .child(text_skeleton(4, 12.0, window, cx)),
                )
                .into_any_element(),
            (None, Some(work)) => {
                self.follow_view(None, cx);
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .child(self.roadmap_toolbar(cx))
                    .child(div().flex_1().min_h(px(0.0)).child(self.roadmap(work, cx)))
                    .into_any_element()
            }
            (Some(view), Some(work)) => {
                self.follow_view(Some(view), cx);
                let name = work.board_name(view);
                let cards = self.visible_cards(work, view, cx);
                let content = match tab {
                    BoardTab::Board => self.board(work, &cards, window, cx),
                    BoardTab::List => self.list(work, &cards, cx),
                };
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .child(self.board_toolbar(&name, tab, window, cx))
                    .child(div().flex_1().min_h(px(0.0)).child(content))
                    .into_any_element()
            }
        };
        let peek = work
            .as_ref()
            .and_then(|work| self.peek_layer(work, window, cx));
        div()
            .size_full()
            .child(body)
            .child(self.composer.clone())
            .children(peek)
            .children(ask)
            .child(self.menu.clone())
    }
}
