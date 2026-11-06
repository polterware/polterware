//! A card's page (`work-card-detail.tsx`, `work-card-work.tsx` on `KnowledgeDocumentShell`):
//! a [`DocumentPage`] whose column holds the card's title and its intent (read-only: agents
//! write cards with Polter), then what the card needs to move on, 32px apart and 48px under the
//! intent, then its Activity.
//!
//! Tasks lists every task with its box (done and open; a dropped one opens again), its number,
//! title, scope, and claim, and drops an open one from the × under the pointer; Invariants
//! lists them. The next stage gate (Contract gate for an idea, Plan gate for a planned card)
//! shows the review's checks that hold it, an error blocking its button (Move to Planned,
//! Start work), and a refusal's lines above them. Verification reads whether the tasks and
//! checks are done, never writing. Commits lists the ones linked to a task or citing it, opens
//! one on GitHub, unlinks a linked one, and links one from the "+" Morph (a task, then a sha).
//! Last handoff, Runs (each opening in Dispatch), and the Agent brief (previewed or copied)
//! follow when there are any.
//!
//! The rail's Properties hold the Stage (a move back asks why), Slug, Kind, Owner, Component,
//! Decisions, and Updated, then how agents add to the card; Questions lists its questions; the
//! rail ends with Remove from board, which asks why.

use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, IntoElement, Render,
    SharedString, WeakEntity, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::checkbox::checkbox_mark;
use polterware_ui::chip::chip_rounded;
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::markdown::markdown_view;
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
use crate::state::{AppState, CardReads, Load};
use crate::views::knowledge_page::document_activity;
use crate::views::milestones::MilestonesView;
use crate::work::{self, Stage};

/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
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
        }
    }

    fn back(&mut self, cx: &mut Context<Self>) {
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

    /// Reads the card's Activity when another card opens.
    fn sync(&mut self, card: &WorkCard, cx: &mut Context<Self>) {
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
        rows.extend(text("Decisions", &card.decisions.join(", "), false));
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
        let aside = (total > 0).then(|| {
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
        document_group_with("Tasks", Icon::ListChecks, aside, true, content, cx).into_any_element()
    }

    fn invariants(card: &WorkCard, cx: &App) -> Option<AnyElement> {
        if card.invariants.is_empty() {
            return None;
        }
        let theme = Theme::global(cx);
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
        Some(
            document_group_with(
                "Invariants",
                Icon::ShieldCheck,
                None,
                true,
                div()
                    .pl(px(8.0))
                    .pr(px(4.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .text_size(px(13.0))
                    .text_color(theme.color_text_secondary)
                    .children(rows),
                cx,
            )
            .into_any_element(),
        )
    }

    fn gate(
        &self,
        card: &WorkCard,
        reads: &CardReads,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let theme = *Theme::global(cx);
        let gate = work::next_gate(Stage::of(card))?;
        let blocked = reads.checks.iter().any(|check| check.error);
        let state = self.state.clone();
        let slug = card.slug.clone();
        let button = Button::new("card-gate", ButtonKind::Secondary)
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
            .into_any_element();
        let content = if reads.checks.is_empty() && reads.refusal.is_empty() {
            Self::muted("Nothing holds this card back.", cx)
        } else {
            let lines = reads
                .refusal
                .iter()
                .map(|message| Self::check_line(Icon::Close, theme.color_danger, message, cx))
                .chain(reads.checks.iter().map(|check| {
                    if check.error {
                        Self::check_line(Icon::Close, theme.color_danger, &check.message, cx)
                    } else {
                        Self::check_line(
                            Icon::HelpCircle,
                            theme.color_text_muted,
                            &check.message,
                            cx,
                        )
                    }
                }));
            div()
                .mt(px(8.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(lines)
                .into_any_element()
        };
        Some(
            document_group_with(
                gate.title,
                Icon::ShieldCheck,
                Some(button),
                true,
                content,
                cx,
            )
            .into_any_element(),
        )
    }

    fn verification(&self, reads: &CardReads, cx: &mut Context<Self>) -> AnyElement {
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
        let content =
            match &reads.verification {
                None => Self::muted(
                    "Verify reads whether the card's tasks and checks are done. It never writes.",
                    cx,
                ),
                Some(result) => div()
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
                    .into_any_element(),
            };
        document_group_with("Verification", Icon::Check, Some(button), true, content, cx)
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
                    .font_family(theme.font_mono)
                    .text_size(px(12.0))
                    .text_color(theme.color_text_secondary)
                    .child(text)
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
        document_group_with("Commits", Icon::GitCommit, aside, true, content, cx).into_any_element()
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
                true,
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
                true,
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
                true,
                content,
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
        let theme = *Theme::global(cx);
        let reads = if reads.slug == card.slug {
            reads
        } else {
            CardReads::default()
        };
        let body = if card.intent.trim().is_empty() {
            div()
                .text_size(px(15.0))
                .line_height(px(24.0))
                .text_color(theme.color_text_disabled)
                .child("Add a description…")
                .into_any_element()
        } else {
            markdown_view(&card.intent, cx).into_any_element()
        };
        let mut groups: Vec<AnyElement> = vec![self.tasks(&card, pending, cx)];
        groups.extend(Self::invariants(&card, cx));
        groups.extend(self.gate(&card, &reads, cx));
        groups.push(self.verification(&reads, cx));
        groups.push(self.commits(&card, &reads, cx));
        groups.extend(Self::handoff(&reads, cx));
        groups.extend(Self::runs(&reads, cx));
        groups.extend(self.brief(&card, &reads, cx));
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
        let document = document
            .rail(document_footer([remove], cx))
            .child(
                div()
                    .w_full()
                    .mt(px(32.0))
                    .mb(px(8.0))
                    .child(document_title(card.title.clone(), cx)),
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
        let _ = window;
        div().size_full().child(document).child(self.menu.clone())
    }
}
