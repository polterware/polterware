//! A card's page (`work-card-detail.tsx`, `work-card-work.tsx` on `KnowledgeDocumentShell`):
//! a [`DocumentPage`] whose column holds the card's title and its intent (read-only: agents
//! write cards with Polter), then the spec-driven flow, top to bottom, 32px apart and 48px
//! under the intent: Spec, Plan, Tasks, Rules that apply, Analyze, and then what the card left
//! behind (Commits, the last handoff, Runs, the Agent brief), then its Activity. Each group's
//! header carries the stage it belongs to as a chip: Specify, Plan, Implement, or Analyze.
//!
//! Spec holds Behavior, Acceptance criteria (the card's invariants), and Edge cases, with the
//! contract gate (Move to Planned) beside its header and the review's checks that hold it at
//! its end. Behavior and Edge cases are Markdown, edited in place and saved 800ms after the
//! last keystroke as a card block's `behavior:` and `edge-cases:` sections. Plan holds the plan
//! text, edited the same way and written as the card's `plan:` section, with the plan gate
//! (Start work) beside its header. Every empty section reads a one-line hint of what would go
//! there, which a click opens for typing.
//!
//! Tasks lists every task with its box (done and open; a dropped one opens again), its number,
//! title, scope, and claim, and drops an open one from the × under the pointer. Rules that
//! apply lists the Rules in force for the card, each opening its page. Analyze reads whether
//! the tasks, acceptance criteria, and Rules are met, never writing.
//!
//! Commits lists the ones linked to a task or citing it, opens one on GitHub, unlinks a linked
//! one, and links one from the "+" Morph (a task, then a sha). Last handoff, Runs (each opening
//! in Dispatch), and the Agent brief (previewed or copied) follow when there are any.
//!
//! The rail's Properties hold the Stage (a move back asks why), Slug, Kind, Owner, Component,
//! and Updated, then how agents add to the card; Questions lists its questions; the rail ends
//! with Remove from board, which asks why.

use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, Focusable, FontWeight,
    IntoElement, Render, SharedString, Subscription, WeakEntity, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::checkbox::checkbox_mark;
use polterware_ui::chip::chip_rounded;
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::markdown::{MarkdownEditor, MarkdownEditorEvent};
use polterware_ui::menu::MenuItem;
use polterware_ui::patterns::{
    ContextMenu, DocumentPage, Morph, MorphAlign, RailState, document_footer, document_group,
    document_group_with, document_title, rail_properties, rail_property, rail_select,
};
use polterware_ui::radio_group::{RadioOption, radio_group};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::{StatusTone, Theme};
use polterware_ui::wrap::wrap_text;

use crate::contracts::{CardCommit, WorkCard, WorkTask};
use crate::dispatch;
use crate::format::{format_instant, month_day_time};
use crate::state::{AppState, CardReads, CardSpec, Load};
use crate::views::knowledge_page::document_activity;
use crate::views::milestones::MilestonesView;
use crate::work::{self, Stage};

/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
/// `createDocumentAutosave`'s debounce, for the spec sections the page edits.
/// What an empty description reads.
const INTENT_HINT: &str = "Add a description…";
const AUTOSAVE: Duration = Duration::from_millis(800);
/// What an empty Behavior section says to write there.
const BEHAVIOR_HINT: &str = "What the feature does.";
/// What an empty Edge cases section says to write there.
const EDGE_CASES_HINT: &str = "The cases it must handle.";
/// What an empty Plan section says to write there.
const PLAN_HINT: &str = "Architecture, APIs, and files touched.";
/// A sha the Link a commit form takes (`/^[0-9a-f]{4,40}$/i`).
fn sha_ok(sha: &str) -> bool {
    (4..=40).contains(&sha.len()) && sha.chars().all(|letter| letter.is_ascii_hexdigit())
}

/// A run's chip (`runTone`).
fn run_tone(status: &str) -> StatusTone {
    match status {
        "merged" | "completed" => StatusTone::Ok,
        "running" | "queued" | "started" => StatusTone::Syncing,
        "failed" | "cancelled" | "timeout" => StatusTone::Danger,
        _ => StatusTone::Muted,
    }
}

/// A rail group's content, 4px in as its heading's words are.
fn rail_body(content: impl IntoElement) -> gpui::Div {
    div().w_full().min_w(px(0.0)).px(px(4.0)).child(content)
}

/// A spec section's editor: its Markdown, and the hint an empty one reads.
fn spec_editor(
    hint: &'static str,
    source: &str,
    cx: &mut Context<MarkdownEditor>,
) -> MarkdownEditor {
    let mut editor = MarkdownEditor::new(source, cx);
    editor.set_placeholder(hint);
    editor
}

/// A spec section's body: the in-place editor once it holds text or a block is open, and until
/// then one line reading what to write there, which a click opens for typing. The editor keeps a
/// tall minimum height while it is empty, so mounting it would leave blank room under the hint.
fn spec_body(
    id: &'static str,
    hint: &'static str,
    editor: &Entity<MarkdownEditor>,
    cx: &mut Context<CardPage>,
) -> AnyElement {
    let empty = {
        let editor = editor.read(cx);
        editor.source().trim().is_empty() && !editor.is_editing()
    };
    if !empty {
        return editor.clone().into_any_element();
    }
    let theme = *Theme::global(cx);
    let editor = editor.clone();
    div()
        .id(id)
        .w_full()
        .pr(px(8.0))
        .cursor_text()
        .text_size(px(15.0))
        .line_height(px(24.0))
        .font_weight(FontWeight(450.0))
        .text_color(theme.color_text_muted)
        .child(hint)
        .on_click(move |_, window, cx| {
            editor.update(cx, |editor, cx| editor.edit_start(window, cx));
        })
        .into_any_element()
}

pub struct CardPage {
    state: Entity<AppState>,
    host: WeakEntity<MilestonesView>,
    rail: Entity<RailState>,
    menu: Entity<ContextMenu>,
    /// The Link a commit Morph is open, the task it links, and the sha typed.
    linking: bool,
    link_task: Option<i32>,
    sha: Entity<TextInput>,
    link_saving: bool,
    brief_open: bool,
    /// The card whose Activity was read.
    activity_for: Option<String>,
    /// The spec sections' Markdown editors: Behavior, Edge cases, and Plan.
    behavior: Entity<MarkdownEditor>,
    edge_cases: Entity<MarkdownEditor>,
    plan: Entity<MarkdownEditor>,
    /// The card the spec editors hold.
    spec_for: Option<String>,
    /// A section waiting out the autosave (keyed by `CardSpec::index`): the card and the value.
    pending_spec: [Option<(String, String)>; 3],
    /// Bumped by each keystroke, so only the last one's timer saves.
    spec_due: [u64; 3],
    _spec_changes: [Subscription; 3],
    /// The title and the description, edited in place, with the card they hold.
    title: Entity<TextInput>,
    intent: Entity<MarkdownEditor>,
    text_for: Option<String>,
    /// The title and description waiting out the autosave: the card, the title, the text.
    pending_text: Option<(String, String, String)>,
    text_due: u64,
    _intent_changes: Subscription,
}

impl CardPage {
    pub fn new(
        state: Entity<AppState>,
        host: WeakEntity<MilestonesView>,
        cx: &mut Context<Self>,
    ) -> Self {
        let sha = cx.new(|cx| TextInput::new(cx).placeholder("Commit sha"));
        cx.observe(&sha, |_, _, cx| cx.notify()).detach();
        let menu = cx.new(ContextMenu::new);
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        let behavior = cx.new(|cx| spec_editor(BEHAVIOR_HINT, "", cx));
        let edge_cases = cx.new(|cx| spec_editor(EDGE_CASES_HINT, "", cx));
        let plan = cx.new(|cx| spec_editor(PLAN_HINT, "", cx));
        for editor in [&behavior, &edge_cases, &plan] {
            cx.observe(editor, |_, _, cx| cx.notify()).detach();
        }
        let spec_changes = [
            cx.subscribe(&behavior, CardPage::behavior_typed),
            cx.subscribe(&edge_cases, CardPage::edge_cases_typed),
            cx.subscribe(&plan, CardPage::plan_typed),
        ];
        let intent = cx.new(|cx| spec_editor(INTENT_HINT, "", cx));
        cx.observe(&intent, |_, _, cx| cx.notify()).detach();
        let intent_changes = cx.subscribe(&intent, |this, _, _: &MarkdownEditorEvent, cx| {
            this.text_typed(cx)
        });
        let page = cx.weak_entity();
        let title = cx.new(|cx| {
            TextInput::new(cx)
                .prose(24.0, 32.0, FontWeight::SEMIBOLD)
                .wrap()
                .placeholder("Untitled")
                // Enter moves into the description, after the field's own update.
                .on_submit(move |_, window, cx| {
                    let page = page.clone();
                    window.defer(cx, move |window, cx| {
                        let _ = page.update(cx, |this, cx| {
                            this.intent
                                .update(cx, |intent, cx| intent.edit_start(window, cx));
                        });
                    });
                })
        });
        cx.observe(&title, |this, _, cx| this.text_typed(cx))
            .detach();
        Self {
            state,
            host,
            rail: cx.new(|_| RailState::new(true)),
            menu,
            linking: false,
            link_task: None,
            sha,
            link_saving: false,
            brief_open: false,
            activity_for: None,
            behavior,
            edge_cases,
            plan,
            spec_for: None,
            pending_spec: [None, None, None],
            spec_due: [0, 0, 0],
            _spec_changes: spec_changes,
            title,
            intent,
            text_for: None,
            pending_text: None,
            text_due: 0,
            _intent_changes: intent_changes,
        }
    }

    /// Shows `card`'s title and description: another card's (writing what the last one still
    /// held), else what changed elsewhere while nothing waits to save and the title is not
    /// being typed.
    fn sync_text(&mut self, card: &WorkCard, window: &Window, cx: &mut Context<Self>) {
        if self.text_for.as_deref() != Some(card.slug.as_str()) {
            self.flush_text(cx);
            self.text_for = Some(card.slug.clone());
        } else if self.pending_text.is_some()
            || self.title.read(cx).focus_handle(cx).is_focused(window)
        {
            return;
        }
        if self.title.read(cx).text() != card.title {
            let title = card.title.clone();
            self.title.update(cx, |field, cx| field.set_text(title, cx));
        }
        if self.intent.read(cx).source() != card.intent && !self.intent.read(cx).is_editing() {
            let intent = card.intent.clone();
            self.intent
                .update(cx, |editor, cx| editor.set_source(&intent, cx));
        }
        self.pending_text = None;
    }

    /// Keeps the new title and description until no keystroke comes for a while. A blank
    /// title is not a valid card: it waits here until a name is typed.
    fn text_typed(&mut self, cx: &mut Context<Self>) {
        let Some(slug) = self.text_for.clone() else {
            return;
        };
        let title = self.title.read(cx).text().trim().to_string();
        let intent = self.intent.read(cx).source().trim().to_string();
        let saved = self.saved_text(&slug, cx);
        if title.is_empty() || saved.as_ref() == Some(&(title.clone(), intent.clone())) {
            self.pending_text = None;
            return;
        }
        self.pending_text = Some((slug, title, intent));
        self.text_due += 1;
        let due = self.text_due;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AUTOSAVE).await;
            let _ = this.update(cx, |this, cx| {
                if this.text_due == due {
                    this.flush_text(cx);
                }
            });
        })
        .detach();
    }

    /// The title and description as the card holds them now.
    fn saved_text(&self, slug: &str, cx: &App) -> Option<(String, String)> {
        match self.state.read(cx).work() {
            Load::Ready(work) => work.card(slug).map(|card| {
                (
                    card.title.trim().to_string(),
                    card.intent.trim().to_string(),
                )
            }),
            _ => None,
        }
    }

    /// Writes the title and description now, when they wait.
    fn flush_text(&mut self, cx: &mut Context<Self>) {
        let Some((slug, title, intent)) = self.pending_text.take() else {
            return;
        };
        if self.saved_text(&slug, cx) == Some((title.clone(), intent.clone())) {
            return;
        }
        self.state.update(cx, |state, cx| {
            state.save_card_text(slug, title, intent, cx)
        });
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        self.flush_text(cx);
        for field in 0..3 {
            self.flush_spec(field, cx);
        }
        self.state.update(cx, |state, cx| {
            if state.can_go_back() {
                state.go_back(cx);
            } else {
                let view = state
                    .board_route()
                    .and_then(|(board, _)| board.view.clone());
                state.open_board(view, cx);
            }
            cx.notify();
        });
    }

    /// Reads the card's Activity when another card opens, and shows its spec sections.
    fn sync(&mut self, card: &WorkCard, cx: &mut Context<Self>) {
        self.sync_spec(card, cx);
        if self.activity_for.as_deref() == Some(card.id.as_str()) {
            return;
        }
        self.activity_for = Some(card.id.clone());
        self.linking = false;
        self.link_task = None;
        self.brief_open = false;
        self.sha.update(cx, |field, cx| field.set_text("", cx));
        let subject = card.id.clone();
        self.state
            .update(cx, |state, cx| state.load_document_activity(subject, cx));
    }

    /// Shows `card`'s spec sections: new editors for another card (writing what the last one
    /// still held), else only what changed elsewhere and is not being typed.
    fn sync_spec(&mut self, card: &WorkCard, cx: &mut Context<Self>) {
        if self.spec_for.as_deref() != Some(card.id.as_str()) {
            for field in 0..3 {
                self.flush_spec(field, cx);
            }
            self.spec_for = Some(card.id.clone());
            let behavior = card.behavior.clone();
            self.behavior = cx.new(|cx| spec_editor(BEHAVIOR_HINT, &behavior, cx));
            let edge_cases = card.edge_cases.clone();
            self.edge_cases = cx.new(|cx| spec_editor(EDGE_CASES_HINT, &edge_cases, cx));
            let plan = card.plan.clone();
            self.plan = cx.new(|cx| spec_editor(PLAN_HINT, &plan, cx));
            for editor in [&self.behavior, &self.edge_cases, &self.plan] {
                cx.observe(editor, |_, _, cx| cx.notify()).detach();
            }
            self._spec_changes = [
                cx.subscribe(&self.behavior, CardPage::behavior_typed),
                cx.subscribe(&self.edge_cases, CardPage::edge_cases_typed),
                cx.subscribe(&self.plan, CardPage::plan_typed),
            ];
            return;
        }
        let editors = [
            self.behavior.clone(),
            self.edge_cases.clone(),
            self.plan.clone(),
        ];
        for (spec, editor) in CardSpec::ALL.into_iter().zip(editors) {
            if self.pending_spec[spec.index()].is_some() {
                continue;
            }
            let saved = spec.value(card);
            if editor.read(cx).source() != saved {
                editor.update(cx, |editor, cx| editor.set_source(saved, cx));
            }
        }
    }

    fn behavior_typed(
        &mut self,
        _: Entity<MarkdownEditor>,
        event: &MarkdownEditorEvent,
        cx: &mut Context<Self>,
    ) {
        self.spec_typed(CardSpec::Behavior, event, cx);
    }

    fn edge_cases_typed(
        &mut self,
        _: Entity<MarkdownEditor>,
        event: &MarkdownEditorEvent,
        cx: &mut Context<Self>,
    ) {
        self.spec_typed(CardSpec::EdgeCases, event, cx);
    }

    fn plan_typed(
        &mut self,
        _: Entity<MarkdownEditor>,
        event: &MarkdownEditorEvent,
        cx: &mut Context<Self>,
    ) {
        self.spec_typed(CardSpec::Plan, event, cx);
    }

    /// Keeps a section's new Markdown until no keystroke comes for a while.
    fn spec_typed(&mut self, spec: CardSpec, event: &MarkdownEditorEvent, cx: &mut Context<Self>) {
        let MarkdownEditorEvent::Changed(source) = event;
        let Some(slug) = self.spec_for.clone() else {
            return;
        };
        if self.saved_spec(&slug, spec, cx).as_deref() == Some(source.trim()) {
            self.pending_spec[spec.index()] = None;
            return;
        }
        self.pending_spec[spec.index()] = Some((slug, source.clone()));
        self.schedule_spec(spec.index(), cx);
    }

    /// The section's body as the card holds it now.
    fn saved_spec(&self, slug: &str, spec: CardSpec, cx: &App) -> Option<String> {
        match self.state.read(cx).work() {
            Load::Ready(work) => work
                .card(slug)
                .map(|card| spec.value(card).trim().to_string()),
            _ => None,
        }
    }

    /// Saves section `field` once no keystroke came for a while.
    fn schedule_spec(&mut self, field: usize, cx: &mut Context<Self>) {
        self.spec_due[field] += 1;
        let due = self.spec_due[field];
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AUTOSAVE).await;
            let _ = this.update(cx, |this, cx| {
                if this.spec_due[field] == due {
                    this.flush_spec(field, cx);
                }
            });
        })
        .detach();
    }

    /// Writes section `field` now, when it waits.
    fn flush_spec(&mut self, field: usize, cx: &mut Context<Self>) {
        let Some((slug, body)) = self.pending_spec[field].take() else {
            return;
        };
        let spec = CardSpec::ALL[field];
        if self.saved_spec(&slug, spec, cx).as_deref() == Some(body.trim()) {
            return;
        }
        self.state
            .update(cx, |state, cx| state.save_card_spec(slug, spec, body, cx));
    }

    fn set_task(&self, slug: &str, task: &WorkTask, next: &str, cx: &mut Context<Self>) {
        let (slug, task, next) = (slug.to_string(), task.clone(), next.to_string());
        self.state.update(cx, |state, cx| {
            state.set_task_state(slug, &task, &next, cx);
            cx.notify();
        });
    }

    /// Stage: the stages as a menu under the pointer, a move back asking why.
    fn pick_stage(&mut self, card: &WorkCard, window: &mut Window, cx: &mut Context<Self>) {
        let current = Stage::of(card);
        let host = self.host.clone();
        let items = Stage::ALL
            .iter()
            .map(|stage| {
                let (host, stage, slug) = (host.clone(), *stage, card.slug.clone());
                MenuItem::radio(
                    stage.label(),
                    stage == current,
                    Rc::new(move |_, cx: &mut App| {
                        if stage == current {
                            return;
                        }
                        let slug = slug.clone();
                        let _ = host.update(cx, |host, cx| host.ask_stage(slug, stage, cx));
                    }),
                )
            })
            .collect();
        let at = window.mouse_position();
        self.menu
            .update(cx, |menu, cx| menu.open(at, items, window, cx));
    }

    fn properties(&self, card: &WorkCard, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let page = cx.weak_entity();
        let stage_card = card.clone();
        let stage = rail_select(
            "card-stage",
            Stage::of(card).label(),
            Rc::new(move |window: &mut Window, cx: &mut App| {
                let card = stage_card.clone();
                let _ = page.update(cx, |this, cx| this.pick_stage(&card, window, cx));
            }),
            cx,
        );
        let text = |label: &str, value: &str, mono: bool| {
            (!value.is_empty()).then(|| {
                rail_property(
                    label,
                    div().min_w(px(0.0)).child(wrap_text(value.to_string())),
                    mono,
                    cx,
                )
            })
        };
        let mut rows = vec![rail_property("Stage", stage, false, cx)];
        rows.extend(text("Slug", &card.slug, true));
        rows.extend(text(
            "Kind",
            if card.kind == "architecture" {
                "Architecture change"
            } else {
                "Feature"
            },
            false,
        ));
        rows.extend(text("Owner", &card.owner, false));
        rows.extend(text("Component", &card.under, true));
        rows.extend(text("Updated", &format_instant(&card.updated_at), false));
        document_group(
            "Properties",
            Icon::InformationCircle,
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(rail_properties().children(rows))
                .child(
                    div()
                        .px(px(4.0))
                        .text_size(px(12.5))
                        .line_height(px(18.0))
                        .text_color(theme.color_text_muted)
                        .child(wrap_text(
                            "Agents add tasks, questions, and invariants with polterware apply.",
                        )),
                ),
            cx,
        )
        .into_any_element()
    }

    fn questions(&self, card: &WorkCard, cx: &App) -> Option<AnyElement> {
        if card.questions.is_empty() {
            return None;
        }
        let theme = *Theme::global(cx);
        let rows = card.questions.iter().map(|question| {
            let open = question.state == "open";
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .text_size(px(12.5))
                .text_color(theme.color_text_secondary)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(6.0))
                        .child(wrap_text(question.text.clone()))
                        .when(question.blocking && open, |line| {
                            line.child(chip_rounded("Blocking", StatusTone::Warn, CHIP_RADIUS, cx))
                        })
                        .when(question.state == "answered", |line| {
                            line.child(chip_rounded(
                                "Answered",
                                StatusTone::Healthy,
                                CHIP_RADIUS,
                                cx,
                            ))
                        }),
                )
                .when(
                    question.state == "answered" && !question.answer.is_empty(),
                    |row| {
                        row.child(
                            div()
                                .text_color(theme.color_text_muted)
                                .child(wrap_text(question.answer.clone())),
                        )
                    },
                )
                .when(open && !question.options.is_empty(), |row| {
                    row.child(
                        div()
                            .font_family(theme.font_mono)
                            .text_size(px(11.0))
                            .text_color(theme.color_text_muted)
                            .child(question.options.join(" | ")),
                    )
                })
        });
        Some(
            document_group(
                "Questions",
                Icon::HelpCircle,
                div()
                    .px(px(4.0))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .children(rows),
                cx,
            )
            .into_any_element(),
        )
    }

    /// `Rules that apply`: every Rule in force for the card by its statement and status, each
    /// row opening the Rule's page; the law the implementation keeps and Analyze confirms.
    fn rules_applying(&self, card: &WorkCard, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let rules = match self.state.read(cx).knowledge() {
            Load::Ready(knowledge) => knowledge.rules.clone(),
            _ => Vec::new(),
        };
        let content = if card.rules.is_empty() {
            Self::muted("No Rules apply to this card.", cx)
        } else {
            let state = self.state.clone();
            let rows = card.rules.iter().enumerate().map(|(index, handle)| {
                let rule = rules
                    .iter()
                    .find(|rule| &rule.slug == handle || &rule.id == handle);
                let statement = rule.map_or_else(
                    || handle.clone(),
                    |rule| {
                        if rule.statement.is_empty() {
                            handle.clone()
                        } else {
                            rule.statement.clone()
                        }
                    },
                );
                let id = rule.map(|rule| rule.id.clone());
                let page = state.clone();
                div()
                    .id(("card-rule", index))
                    .w_full()
                    .min_w(px(0.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(4.0))
                    .rounded(px(theme.radius_item))
                    .when(id.is_some(), |row| {
                        row.cursor_pointer()
                            .hover(|row| row.bg(theme.color_surface_hover))
                    })
                    .on_click(move |_, _, cx| {
                        if let Some(id) = id.clone() {
                            page.update(cx, |state, cx| state.open_rule(id, cx));
                        }
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .text_size(px(12.5))
                            .line_height(px(18.0))
                            .text_color(theme.color_text_secondary)
                            .child(wrap_text(statement)),
                    )
            });
            div()
                .px(px(4.0))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .children(rows)
                .into_any_element()
        };
        let aside = Self::group_aside(Self::stage_chip("Implement", cx), None);
        document_group_with(
            "Rules that apply",
            Icon::ShieldCheck,
            Some(aside),
            true,
            content,
            cx,
        )
        .into_any_element()
    }

    fn muted(text: impl Into<SharedString>, cx: &App) -> AnyElement {
        let theme = Theme::global(cx);
        div()
            .text_size(px(12.5))
            .line_height(px(18.0))
            .text_color(theme.color_text_muted)
            .child(wrap_text(text.into()))
            .into_any_element()
    }

    /// One check line (`.pw-card-work__check`): its mark, then its message.
    fn check_line(icon: Icon, color: gpui::Rgba, message: &str, cx: &App) -> AnyElement {
        let theme = Theme::global(cx);
        div()
            .flex()
            .items_start()
            .gap(px(8.0))
            .text_size(px(13.0))
            .line_height(px(20.0))
            .text_color(theme.color_text_secondary)
            .child(
                div()
                    .h(px(20.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .child(icon_colored(icon, 14.0, color)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .child(wrap_text(message.to_string())),
            )
            .into_any_element()
    }

    fn tasks(&self, card: &WorkCard, pending: bool, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let (done, total) = work::task_progress(card);
        let progress = (total > 0).then(|| {
            chip_rounded(
                format!("{done}/{total}"),
                if done == total {
                    StatusTone::Healthy
                } else {
                    StatusTone::Muted
                },
                CHIP_RADIUS,
                cx,
            )
            .into_any_element()
        });
        let aside = Self::group_aside(Self::stage_chip("Implement", cx), progress);
        let content = if card.tasks.is_empty() {
            div()
                .px(px(4.0))
                .child(Self::muted("No tasks yet.", cx))
                .into_any_element()
        } else {
            let page = cx.weak_entity();
            let rows = card.tasks.iter().map(|task| {
                let open = task.state == "open";
                let group = SharedString::from(format!("card-task-{}", task.number));
                let (toggle_page, toggle_task, toggle_slug) =
                    (page.clone(), task.clone(), card.slug.clone());
                let next = if open { "done" } else { "open" };
                let mut title = div()
                    .flex_1()
                    .min_w(px(0.0))
                    .when(task.state == "dropped", |title| title.line_through())
                    .child(wrap_text(task.title.clone()));
                if !task.scope.is_empty() {
                    title = title.child(
                        div()
                            .ml(px(8.0))
                            .font_family(theme.font_mono)
                            .text_size(px(11.0))
                            .text_color(theme.color_text_muted)
                            .child(task.scope.clone()),
                    );
                }
                if !task.claim_holder.is_empty() {
                    title = title.child(
                        div()
                            .ml(px(8.0))
                            .text_size(px(11.0))
                            .text_color(theme.color_text_muted)
                            .child(format!("claimed by {}", task.claim_holder)),
                    );
                }
                let drop = open.then(|| {
                    let (drop_page, drop_task, drop_slug) =
                        (page.clone(), task.clone(), card.slug.clone());
                    div()
                        .flex_none()
                        .my(px(-4.0))
                        .invisible()
                        .group_hover(group.clone(), |slot| slot.visible())
                        .child(
                            Button::new(
                                SharedString::from(format!("card-task-drop-{}", task.number)),
                                ButtonKind::Ghost,
                            )
                            .size(ButtonSize::Icon)
                            .icon(Icon::Close)
                            .tooltip(format!("Drop T{}", task.number))
                            .disabled(pending)
                            .on_click(move |_, cx| {
                                let _ = drop_page.update(cx, |this, cx| {
                                    this.set_task(&drop_slug, &drop_task, "dropped", cx)
                                });
                            }),
                        )
                });
                div()
                    .id(SharedString::from(format!("card-task-{}", task.number)))
                    .group(group)
                    .flex()
                    .items_start()
                    .gap(px(8.0))
                    .text_size(px(13.0))
                    .text_color(if open {
                        theme.color_text_secondary
                    } else {
                        theme.color_text_muted
                    })
                    .child(
                        div()
                            .id(SharedString::from(format!("card-task-box-{}", task.number)))
                            .mt(px(1.0))
                            .flex_none()
                            .when(!pending, |mark| mark.cursor_pointer())
                            .on_click(move |_, _, cx| {
                                if pending {
                                    return;
                                }
                                let _ = toggle_page.update(cx, |this, cx| {
                                    this.set_task(&toggle_slug, &toggle_task, next, cx)
                                });
                            })
                            .child(checkbox_mark(task.state == "done", 14.0, &theme)),
                    )
                    .child(
                        div()
                            .w(px(28.0))
                            .flex_none()
                            .font_family(theme.font_mono)
                            .text_size(px(11.0))
                            .line_height(px(20.0))
                            .text_color(theme.color_text_muted)
                            .child(format!("T{}", task.number)),
                    )
                    .child(title)
                    .children(drop)
            });
            div()
                .px(px(4.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(rows)
                .into_any_element()
        };
        document_group_with("Tasks", Icon::ListChecks, Some(aside), true, content, cx)
            .into_any_element()
    }

    /// A group's stage chip, as every group of the card's flow carries one: Specify, Plan,
    /// Implement, or Analyze.
    fn stage_chip(label: &'static str, cx: &App) -> AnyElement {
        chip_rounded(label, StatusTone::Muted, CHIP_RADIUS, cx).into_any_element()
    }

    /// A group's aside: its stage chip, then what its header also holds.
    fn group_aside(chip: AnyElement, extra: Option<AnyElement>) -> AnyElement {
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(chip)
            .children(extra)
            .into_any_element()
    }

    /// A section's name inside a group, above its Markdown or list.
    fn section_label(label: &'static str, cx: &App) -> AnyElement {
        let theme = Theme::global(cx);
        div()
            .text_size(px(12.5))
            .line_height(px(18.0))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(theme.color_text_muted)
            .child(label)
            .into_any_element()
    }

    /// The spec's third part: the acceptance criteria, which are the card's invariants.
    fn acceptance(card: &WorkCard, cx: &App) -> AnyElement {
        let theme = Theme::global(cx);
        let content = if card.invariants.is_empty() {
            Self::muted("No acceptance criteria yet.", cx)
        } else {
            let rows = card.invariants.iter().map(|invariant| {
                div()
                    .flex()
                    .gap(px(8.0))
                    .child(div().flex_none().child("•"))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .child(wrap_text(invariant.text.clone())),
                    )
            });
            div()
                .pl(px(8.0))
                .pr(px(4.0))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .text_size(px(13.0))
                .line_height(px(20.0))
                .text_color(theme.color_text_secondary)
                .children(rows)
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(Self::section_label("Acceptance criteria", cx))
            .child(content)
            .into_any_element()
    }

    /// Spec: what the card does, the acceptance criteria its contract accepts, and the cases it
    /// must handle. The contract gate and what holds it sit here, under the "Specify" chip.
    fn spec(
        &self,
        card: &WorkCard,
        reads: &CardReads,
        gate: Option<work::Gate>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let contract = gate.filter(|gate| gate.step == "contract");
        let aside = Self::group_aside(
            Self::stage_chip("Specify", cx),
            contract.map(|gate| self.gate_action(card, gate, reads)),
        );
        let notes = contract.and_then(|_| self.gate_notes(reads, cx));
        let content = div()
            .px(px(4.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(Self::section_label("Behavior", cx))
                    .child(spec_body(
                        "card-behavior",
                        BEHAVIOR_HINT,
                        &self.behavior,
                        cx,
                    )),
            )
            .child(Self::acceptance(card, cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(Self::section_label("Edge cases", cx))
                    .child(spec_body(
                        "card-edge-cases",
                        EDGE_CASES_HINT,
                        &self.edge_cases,
                        cx,
                    )),
            )
            .children(notes);
        document_group_with("Spec", Icon::FileText, Some(aside), true, content, cx)
            .into_any_element()
    }

    /// Plan: the architecture, APIs, and files touched, with the plan gate's approve action and
    /// what holds it, under the "Plan" chip.
    fn plan_group(
        &self,
        card: &WorkCard,
        reads: &CardReads,
        gate: Option<work::Gate>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let plan = gate.filter(|gate| gate.step == "plan");
        let aside = Self::group_aside(
            Self::stage_chip("Plan", cx),
            plan.map(|gate| self.gate_action(card, gate, reads)),
        );
        let notes = plan.and_then(|_| self.gate_notes(reads, cx));
        let content = div()
            .px(px(4.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(spec_body("card-plan", PLAN_HINT, &self.plan, cx))
            .children(notes);
        document_group_with("Plan", Icon::Route, Some(aside), true, content, cx).into_any_element()
    }

    /// The button that passes `gate` (`AdvanceFeature`), which asks no why.
    fn gate_action(&self, card: &WorkCard, gate: work::Gate, reads: &CardReads) -> AnyElement {
        let blocked = reads.checks.iter().any(|check| check.error);
        let state = self.state.clone();
        let slug = card.slug.clone();
        Button::new("card-gate", ButtonKind::Secondary)
            .size(ButtonSize::Sm)
            .label(if reads.advancing {
                "Moving…"
            } else {
                gate.label
            })
            .disabled(reads.advancing || blocked)
            .on_click(move |_, cx| {
                let slug = slug.clone();
                state.update(cx, |state, cx| state.advance_card(slug, gate.step, cx));
            })
            .into_any_element()
    }

    /// What holds the next gate: the refusal's lines, then the review's checks.
    fn gate_notes(&self, reads: &CardReads, cx: &App) -> Option<AnyElement> {
        if reads.checks.is_empty() && reads.refusal.is_empty() {
            return None;
        }
        let theme = *Theme::global(cx);
        let lines = reads
            .refusal
            .iter()
            .map(|message| Self::check_line(Icon::Close, theme.color_danger, message, cx))
            .chain(reads.checks.iter().map(|check| {
                if check.error {
                    Self::check_line(Icon::Close, theme.color_danger, &check.message, cx)
                } else {
                    Self::check_line(Icon::HelpCircle, theme.color_text_muted, &check.message, cx)
                }
            }));
        Some(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(lines)
                .into_any_element(),
        )
    }

    /// Analyze: what `verify` read about the card, and the button that reads it again.
    fn analyze(&self, reads: &CardReads, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let state = self.state.clone();
        let button = Button::new("card-verify", ButtonKind::Secondary)
            .size(ButtonSize::Sm)
            .label(if reads.verifying {
                "Verifying…"
            } else {
                "Verify"
            })
            .disabled(reads.verifying)
            .on_click(move |_, cx| state.update(cx, |state, cx| state.verify_card(cx)))
            .into_any_element();
        let aside = Self::group_aside(Self::stage_chip("Analyze", cx), Some(button));
        let content = match &reads.verification {
            None => Self::muted(
                "Verify reads whether the card's tasks, acceptance criteria, and Rules are met. It never writes.",
                cx,
            ),
            Some(result) => {
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .child(chip_rounded(
                        if result.passed {
                            "Passed"
                        } else {
                            "Not finished"
                        },
                        if result.passed {
                            StatusTone::Ok
                        } else {
                            StatusTone::Warn
                        },
                        CHIP_RADIUS,
                        cx,
                    ))
                    .child(
                        div()
                            .w_full()
                            .mt(px(8.0))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .children(result.checks.iter().map(
                                |check| match check.status.as_str() {
                                    "pass" => Self::check_line(
                                        Icon::Check,
                                        theme.color_success,
                                        &check.message,
                                        cx,
                                    ),
                                    "fail" => Self::check_line(
                                        Icon::Close,
                                        theme.color_danger,
                                        &check.message,
                                        cx,
                                    ),
                                    _ => Self::check_line(
                                        Icon::HelpCircle,
                                        theme.color_text_muted,
                                        &check.message,
                                        cx,
                                    ),
                                },
                            )),
                    )
                    .into_any_element()
            }
        };
        document_group_with(
            "Analyze",
            Icon::Check,
            Some(aside),
            false,
            rail_body(content),
            cx,
        )
        .into_any_element()
    }

    /// The "+" Morph that links a commit to a task: the task, then the sha.
    fn link_morph(&self, card: &WorkCard, cx: &mut Context<Self>) -> Option<AnyElement> {
        let tasks: Vec<WorkTask> = work::live_tasks(card).into_iter().cloned().collect();
        if tasks.is_empty() {
            return None;
        }
        let page = cx.weak_entity();
        let set_open = {
            let page = page.clone();
            let only = (tasks.len() == 1).then(|| tasks[0].number);
            move |open: bool, _: &mut Window, cx: &mut App| {
                let _ = page.update(cx, |this, cx| {
                    this.linking = open;
                    if open {
                        this.link_task = only;
                        this.sha.update(cx, |field, cx| field.set_text("", cx));
                    }
                    cx.notify();
                });
            }
        };
        let options: Vec<RadioOption> = tasks
            .iter()
            .map(|task| RadioOption {
                label: format!("T{} {}", task.number, task.title).into(),
            })
            .collect();
        let numbers: Vec<i32> = tasks.iter().map(|task| task.number).collect();
        let picked = self
            .link_task
            .and_then(|number| numbers.iter().position(|n| *n == number))
            .unwrap_or(usize::MAX);
        let pick_page = page.clone();
        let pick_numbers = numbers.clone();
        let sha = self.sha.read(cx).text().trim().to_string();
        let ready = self.link_task.is_some() && sha_ok(&sha) && !self.link_saving;
        let link_page = page.clone();
        let link = move |cx: &mut App| {
            let _ = link_page.update(cx, |this, cx| this.link(cx));
        };
        let enter = link.clone();
        let morph = Morph::new(
            "card-link-commit",
            Button::new("card-link-commit-trigger", ButtonKind::Ghost)
                .size(ButtonSize::Icon)
                .icon(Icon::Plus)
                .tooltip("Link a commit to one of this card's tasks."),
            "Link a commit",
        )
        .open(self.linking, set_open)
        .align(MorphAlign::End)
        .body(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(radio_group(
                    "card-link-task",
                    &options,
                    picked,
                    Rc::new(move |index, _, cx: &mut App| {
                        let number = pick_numbers.get(index).copied();
                        let _ = pick_page.update(cx, |this, cx| {
                            this.link_task = number;
                            cx.notify();
                        });
                    }),
                    cx,
                ))
                .child(self.sha.clone()),
        )
        .action(
            Button::new("card-link-commit-save", ButtonKind::Primary)
                .size(ButtonSize::Sm)
                .label(if self.link_saving {
                    "Linking…"
                } else {
                    "Link commit"
                })
                .full_width()
                .disabled(!ready)
                .on_click(move |_, cx| link(cx)),
        )
        .on_enter(move |_, cx| enter(cx));
        Some(morph.into_any_element())
    }

    fn link(&mut self, cx: &mut Context<Self>) {
        let sha = self.sha.read(cx).text().trim().to_string();
        let Some(number) = self.link_task else {
            return;
        };
        if !sha_ok(&sha) || self.link_saving {
            return;
        }
        self.link_saving = true;
        let page = cx.weak_entity();
        self.state.update(cx, |state, cx| {
            state.link_commit(
                number,
                sha,
                true,
                move |ok, cx| {
                    let _ = page.update(cx, |this, cx| {
                        this.link_saving = false;
                        if ok {
                            this.linking = false;
                        }
                        cx.notify();
                    });
                },
                cx,
            );
        });
        cx.notify();
    }

    fn commits(&self, card: &WorkCard, reads: &CardReads, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let aside = self.link_morph(card, cx);
        let content = if reads.commits.is_empty() {
            let code = |text: String| {
                div()
                    .min_w(px(0.0))
                    .max_w_full()
                    .font_family(theme.font_mono)
                    .text_size(px(12.0))
                    .text_color(theme.color_text_secondary)
                    .child(wrap_text(text))
            };
            div()
                .flex()
                .flex_wrap()
                .items_baseline()
                .gap_x(px(4.0))
                .text_size(px(12.5))
                .line_height(px(18.0))
                .text_color(theme.color_text_muted)
                .child("Commits whose message cites")
                .child(code(format!("{}#T1", card.slug)))
                .child("show here, and so do commits linked with")
                .child(code(format!(
                    "polterware task done {}#T1 --commit <sha>",
                    card.slug
                )))
                .into_any_element()
        } else {
            let rows = reads
                .commits
                .iter()
                .enumerate()
                .map(|(index, commit)| self.commit_row(index, commit, cx));
            div().flex().flex_col().children(rows).into_any_element()
        };
        document_group_with(
            "Commits",
            Icon::GitCommit,
            aside,
            false,
            rail_body(content),
            cx,
        )
        .into_any_element()
    }

    fn commit_row(&self, index: usize, commit: &CardCommit, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let github = (!commit.url.is_empty()).then(|| {
            let url = commit.url.clone();
            Button::new(("card-commit-github", index), ButtonKind::Ghost)
                .size(ButtonSize::Icon)
                .icon(Icon::Github)
                .tooltip("Open on GitHub")
                .on_click(move |_, cx| cx.open_url(&url))
        });
        let unlink = (!commit.cited).then(|| {
            let state = self.state.clone();
            let (number, sha) = (commit.task_number, commit.sha.clone());
            Button::new(("card-commit-unlink", index), ButtonKind::Ghost)
                .size(ButtonSize::Icon)
                .icon(Icon::Close)
                .tooltip("Unlink commit")
                .on_click(move |_, cx| {
                    let sha = sha.clone();
                    state.update(cx, |state, cx| {
                        state.link_commit(number, sha, false, |_, _| {}, cx)
                    });
                })
        });
        let short = if commit.short_sha.is_empty() {
            commit.sha.chars().take(7).collect()
        } else {
            commit.short_sha.clone()
        };
        div()
            .h(px(44.0))
            .min_w(px(0.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .pl(px(8.0))
            .pr(px(4.0))
            .text_size(px(13.0))
            .child(icon_colored(Icon::GitCommit, 14.0, theme.color_text_muted))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .text_color(theme.color_text_primary)
                    .child(if commit.subject.is_empty() {
                        short.clone()
                    } else {
                        commit.subject.clone()
                    }),
            )
            .child(chip_rounded(
                format!("T{}", commit.task_number),
                StatusTone::Muted,
                CHIP_RADIUS,
                cx,
            ))
            .child(
                div()
                    .flex_none()
                    .font_family(theme.font_mono)
                    .text_size(px(12.0))
                    .text_color(theme.color_text_muted)
                    .child(short),
            )
            .children(github)
            .children(unlink)
            .into_any_element()
    }

    fn handoff(reads: &CardReads, cx: &App) -> Option<AnyElement> {
        let handoff = reads.handoff.as_ref()?;
        let theme = Theme::global(cx);
        let at = if handoff.at.is_empty() {
            String::new()
        } else {
            month_day_time(&handoff.at)
        };
        let aside: Vec<&str> = [handoff.agent.as_str(), handoff.branch.as_str(), at.as_str()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect();
        let rows = [
            ("Goal", &handoff.goal),
            ("Done", &handoff.done),
            ("In progress", &handoff.in_progress),
            ("Next", &handoff.next),
        ]
        .into_iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(label, value)| {
            div()
                .flex()
                .gap(px(12.0))
                .child(
                    div()
                        .w(px(96.0))
                        .flex_none()
                        .text_color(theme.color_text_muted)
                        .child(label),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .text_color(theme.color_text_secondary)
                        .child(wrap_text(value.clone())),
                )
        });
        Some(
            document_group_with(
                "Last handoff",
                Icon::History,
                Some(Self::muted(aside.join(" / "), cx)),
                false,
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .text_size(px(13.0))
                    .line_height(px(20.0))
                    .children(rows),
                cx,
            )
            .into_any_element(),
        )
    }

    fn runs(reads: &CardReads, cx: &App) -> Option<AnyElement> {
        if reads.runs.is_empty() {
            return None;
        }
        let theme = *Theme::global(cx);
        let rows = reads.runs.iter().enumerate().map(|(index, run)| {
            let url = dispatch::run_url(&run.id);
            let at = month_day_time(work::run_at(run));
            let meta: Vec<&str> = [run.agent.as_str(), at.as_str()]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect();
            div()
                .id(("card-run", index))
                .w_full()
                .min_w(px(0.0))
                .h(px(44.0))
                .flex()
                .items_center()
                .gap(px(12.0))
                .px(px(8.0))
                .rounded(px(theme.radius_item))
                .cursor_pointer()
                .hover(|row| row.bg(theme.color_surface_hover))
                .on_click(move |_, _, cx| dispatch::open_in_dispatch(&url, cx))
                .child(chip_rounded(
                    if run.status.is_empty() {
                        "queued".to_string()
                    } else {
                        run.status.clone()
                    },
                    run_tone(&run.status),
                    CHIP_RADIUS,
                    cx,
                ))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .text_size(px(13.0))
                        .text_color(theme.color_text_primary)
                        .child(if run.prompt.is_empty() {
                            run.id.clone()
                        } else {
                            run.prompt.clone()
                        }),
                )
                .child(
                    div()
                        .flex_none()
                        .text_size(px(12.5))
                        .text_color(theme.color_text_muted)
                        .child(meta.join(" / ")),
                )
        });
        Some(
            document_group_with(
                "Runs",
                Icon::Play,
                Some(Self::muted(reads.runs.len().to_string(), cx)),
                false,
                div().flex().flex_col().children(rows),
                cx,
            )
            .into_any_element(),
        )
    }

    fn brief(
        &self,
        card: &WorkCard,
        reads: &CardReads,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if reads.brief.trim().is_empty() {
            return None;
        }
        let theme = *Theme::global(cx);
        let page = cx.weak_entity();
        let open = self.brief_open;
        let brief = reads.brief.clone();
        let state = self.state.clone();
        let aside = div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(
                Button::new("card-brief-preview", ButtonKind::Ghost)
                    .size(ButtonSize::Sm)
                    .label(if open { "Hide" } else { "Preview" })
                    .on_click(move |_, cx| {
                        let _ = page.update(cx, |this, cx| {
                            this.brief_open = !this.brief_open;
                            cx.notify();
                        });
                    }),
            )
            .child(
                Button::new("card-brief-copy", ButtonKind::Ghost)
                    .size(ButtonSize::Icon)
                    .icon(Icon::Copy)
                    .tooltip("Copy agent brief")
                    .on_click(move |_, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(brief.clone()));
                        state.update(cx, |state, cx| {
                            state.push_toast(
                                "Agent brief copied.",
                                polterware_ui::toast::ToastTone::Success,
                            );
                            cx.notify();
                        });
                    }),
            )
            .into_any_element();
        let content = if open {
            div()
                .w_full()
                .p(px(12.0))
                .rounded(px(theme.radius_item))
                .bg(theme.color_surface_subtle)
                .font_family(theme.font_mono)
                .text_size(px(12.0))
                .line_height(px(18.0))
                .text_color(theme.color_text_secondary)
                .child(wrap_text(reads.brief.clone()))
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_wrap()
                .items_baseline()
                .gap_x(px(4.0))
                .text_size(px(12.5))
                .line_height(px(18.0))
                .text_color(theme.color_text_muted)
                .child("What an agent reads before it works on this card (")
                .child(
                    div()
                        .font_family(theme.font_mono)
                        .text_size(px(12.0))
                        .text_color(theme.color_text_secondary)
                        .child(format!("polterware brief {}", card.slug)),
                )
                .child(").")
                .into_any_element()
        };
        Some(
            document_group_with(
                "Agent brief",
                Icon::SourceCodeIcon,
                Some(aside),
                false,
                rail_body(content),
                cx,
            )
            .into_any_element(),
        )
    }
}

impl Render for CardPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.state.read(cx);
        let Some((_, Some(slug))) = app.board_route() else {
            return div().size_full();
        };
        let slug = slug.to_string();
        let pending = app.work_pending();
        let reads = app.card_reads().clone();
        let Load::Ready(work) = app.work() else {
            return div().size_full();
        };
        let card = work.card(&slug).cloned();
        let page = cx.weak_entity();
        let back: polterware_ui::Handler = Rc::new(move |_, cx: &mut App| {
            let _ = page.update(cx, |this, cx| this.back(cx));
        });
        let Some(card) = card else {
            // `KnowledgeDocumentNotFound`.
            let theme = Theme::global(cx);
            return div().size_full().child(
                DocumentPage::new("card-page")
                    .on_back(back)
                    .child(document_title("Card not found", cx))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(theme.color_text_muted)
                            .child(wrap_text(
                                "This card is not available in the current project knowledge set. It may have been archived or the link is stale.",
                            )),
                    ),
            );
        };
        self.sync(&card, cx);
        self.sync_text(&card, window, cx);
        // The card's Rules carry their status on this page, from the same read Knowledge makes.
        self.state
            .update(cx, |state, cx| state.ensure_knowledge(cx));
        let reads = if reads.slug == card.slug {
            reads
        } else {
            CardReads::default()
        };
        let body = spec_body("card-intent", INTENT_HINT, &self.intent, cx);
        let gate = work::next_gate(Stage::of(&card));
        let groups: Vec<AnyElement> = vec![
            self.spec(&card, &reads, gate, cx),
            self.plan_group(&card, &reads, gate, cx),
            self.tasks(&card, pending, cx),
            self.rules_applying(&card, cx),
        ];
        // What agents do with the card stands in the rail, beside its properties.
        let mut tools: Vec<AnyElement> =
            vec![self.analyze(&reads, cx), self.commits(&card, &reads, cx)];
        tools.extend(Self::handoff(&reads, cx));
        tools.extend(Self::runs(&reads, cx));
        tools.extend(self.brief(&card, &reads, cx));
        let host = self.host.clone();
        let (remove_slug, remove_title) = (card.slug.clone(), card.title.clone());
        let remove = Button::new("card-remove", ButtonKind::Danger)
            .size(ButtonSize::Sm)
            .label("Remove from board")
            .disabled(pending)
            .on_click(move |_, cx| {
                let (slug, title) = (remove_slug.clone(), remove_title.clone());
                let _ = host.update(cx, |host, cx| host.ask_remove(slug, title, cx));
            })
            .into_any_element();
        let mut document = DocumentPage::new("card-page")
            .on_back(back)
            .rail_state(&self.rail)
            .rail(self.properties(&card, cx));
        if let Some(questions) = self.questions(&card, cx) {
            document = document.rail(questions);
        }
        for tool in tools {
            document = document.rail(tool);
        }
        let document = document
            .rail(document_footer([remove], cx))
            .child(
                div()
                    .w_full()
                    .mt(px(32.0))
                    .mb(px(8.0))
                    .child(self.title.clone()),
            )
            .child(body)
            .child(
                div()
                    .w_full()
                    .mt(px(48.0))
                    .flex()
                    .flex_col()
                    .gap(px(32.0))
                    .children(groups),
            )
            .children(document_activity(&self.state, &card.id, cx));
        div().size_full().child(document).child(self.menu.clone())
    }
}
