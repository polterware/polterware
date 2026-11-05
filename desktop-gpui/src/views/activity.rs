//! Activity (`activity-page.tsx`, `agent-run-history-card.tsx`): the Unified Change History,
//! the project's latest 50 justified changes from people and agents (`ListChanges`), merged by
//! time with the runs Polterware Dispatch started, merged, or failed (`ListHistory`; none
//! without the Dispatch app). The toolbar holds the All, Human, and Agent tabs, which read the
//! changes again for that actor, and the Tier and Agent filters, which narrow Dispatch's rows
//! and offer the tiers and agents they named. The table reads by day (Today, Yesterday, the
//! weekday, then the date): each row on its actor's bot or initials with its summary, the
//! actor, its run's status, and the time. Nothing opens on its own; a click opens a change in
//! the detail (its rationale, the audit chain of its run from `GetAgentRun` with the current
//! step marked, and its properties) or a Dispatch entry (its run summary, with Open in
//! Dispatch, and the event).

use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FontWeight, IntoElement, Render,
    SharedString, Window, div, prelude::*, px, relative,
};
use polterware_ui::bots::agent_avatar;
use polterware_ui::chip::chip_rounded;
use polterware_ui::data_table::{
    Column, DataTable, SortDirection, TableGroup, mono, next_sort, sans, tabular, text_width,
};
use polterware_ui::filter::{FilterOption, FilterSpec};
use polterware_ui::filter_popover::{FilterField, FilterPopover, FiltersChanged};
use polterware_ui::icon_button::icon_button;
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::patterns::{
    DetailPanel, MascotMood, detail_properties, detail_property, detail_section,
    detail_section_with_aside, detail_text, page_empty, page_empty_with,
};
use polterware_ui::skeleton::text_skeleton;
use polterware_ui::tabs::Tabs;
use polterware_ui::theme::{StatusTone, Theme, tw};
use polterware_ui::window::window_drag::WindowDragRegion;

use crate::contracts::{GetAgentRunResponse, ProjectChange};
use crate::dispatch::{self, contracts::DispatchRunEvent};
use crate::format::{
    Day, ago, date_time, day_label, day_of, instant_millis, now_seconds, time_of_day, today,
};
use crate::state::{ActorFilter, AppState, DISPATCH_ROW, Load};
use crate::views::core_status::core_status;
use crate::views::scope::{ScopeFilter, set_page_fields};

/// The room the floating detail keeps beside the list: its 400px and its 8px inset.
const DETAIL_SPACE: f32 = 408.0;
/// `.pw-data-table-scroll`'s padding-inline.
const TABLE_INSET: f32 = 12.0;
/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
/// `ActorIcon`'s bot in a row.
const ACTOR_SIZE: f32 = 28.0;
/// The summary's and the run's lines: 12.5px and 11.5px text at 1.5.
const SUMMARY_LINE: f32 = 18.75;
const TASK_LINE: f32 = 17.25;
/// A list row is 40px and grows to its cells: a run's two lines in the cell's 5px padding
/// make 46.
const ROW: f32 = 40.0;
const TWO_LINE_ROW: f32 = 10.0 + SUMMARY_LINE + TASK_LINE;

/// `runTone`.
fn run_tone(status: &str) -> StatusTone {
    if status == "completed" {
        StatusTone::Ok
    } else {
        StatusTone::Warn
    }
}

/// `dispatchStatusTone`: a Dispatch run's status.
fn dispatch_tone(status: &str) -> StatusTone {
    match status.to_lowercase().as_str() {
        "merged" | "completed" | "passed" | "ok" | "ready" => StatusTone::Ok,
        "running" | "started" | "starting" | "queued" | "pending" | "active" => StatusTone::Syncing,
        "failed" | "error" | "blocked" | "orphaned" => StatusTone::Danger,
        "cancelled" | "canceled" | "timed_out" | "abandoned" => StatusTone::Warn,
        _ => StatusTone::Muted,
    }
}

fn is_dispatch(change: &ProjectChange) -> bool {
    change.step_id.starts_with(DISPATCH_ROW)
}

/// The Run column's chip tone: Dispatch's for its rows, else the run's.
fn row_tone(change: &ProjectChange) -> StatusTone {
    if is_dispatch(change) {
        dispatch_tone(&change.run_status)
    } else {
        run_tone(&change.run_status)
    }
}

/// `dispatchEventRow`: a Dispatch entry as an Activity row.
fn dispatch_row(event: &DispatchRunEvent) -> ProjectChange {
    let agent_label = match (event.agent.is_empty(), event.tier.is_empty()) {
        (true, _) => "dispatch".to_string(),
        (false, true) => event.agent.clone(),
        (false, false) => format!("{} / {}", event.agent, event.tier),
    };
    ProjectChange {
        step_id: format!("{DISPATCH_ROW}{}", event.id),
        agent_run_id: event.run_id.clone(),
        project_id: event.project_id.clone(),
        actor_kind: "agent".into(),
        agent_label,
        run_status: event.kind.clone(),
        run_task: if event.tier.is_empty() {
            "Agent run".into()
        } else {
            format!("Agent run ({})", event.tier)
        },
        summary: if event.text.is_empty() {
            format!("Agent run {}", event.kind)
        } else {
            event.text.clone()
        },
        payload_json: "{}".into(),
        created_at: event.at.clone(),
    }
}

/// `mergeActivityRows`: the changes and Dispatch's entries in one feed, newest first, in their
/// order when the times tie, and those without a readable time last.
fn merge_rows<'a>(
    changes: &[ProjectChange],
    events: impl IntoIterator<Item = &'a DispatchRunEvent>,
) -> Vec<ProjectChange> {
    let mut rows: Vec<(i64, ProjectChange)> = changes
        .iter()
        .cloned()
        .chain(events.into_iter().map(dispatch_row))
        .map(|row| (instant_millis(&row.created_at).unwrap_or(i64::MIN), row))
        .collect();
    // A stable sort, so rows at the same instant keep their order.
    rows.sort_by(|(a, _), (b, _)| b.cmp(a));
    rows.into_iter().map(|(_, row)| row).collect()
}

/// The filter fields: the tiers (the defaults, then those Dispatch named) and the agents.
fn filter_fields(tiers: &[String], agents: &[String]) -> Vec<FilterField> {
    let options = |values: &[String]| {
        values
            .iter()
            .map(|value| FilterOption::new(value.clone(), value.clone()))
            .collect()
    };
    vec![
        FilterField::new(
            FilterSpec::multi("tier", "Tier", options(tiers)),
            Some(Icon::Gauge),
        ),
        FilterField::new(
            FilterSpec::multi("agent", "Agent", options(agents)),
            Some(Icon::Bot),
        ),
    ]
}

/// `actorTone`.
fn actor_tone(kind: &str) -> StatusTone {
    if kind == "human" {
        StatusTone::Syncing
    } else {
        StatusTone::Muted
    }
}

pub(crate) fn actor_kind(change: &ProjectChange) -> &str {
    if change.actor_kind.is_empty() {
        "agent"
    } else {
        &change.actor_kind
    }
}

fn run_status(change: &ProjectChange) -> &str {
    if change.run_status.is_empty() {
        "open"
    } else {
        &change.run_status
    }
}

/// The run's second line under the summary (`Agent run (hard)`), when it says more.
fn run_line(change: &ProjectChange) -> Option<String> {
    (!change.run_task.is_empty() && change.run_task != change.summary)
        .then(|| change.run_task.clone())
}

/// The Actor column: the agent's label, else Human or Agent.
fn actor_label(change: &ProjectChange) -> String {
    if !change.agent_label.is_empty() {
        change.agent_label.clone()
    } else if actor_kind(change) == "human" {
        "Human".into()
    } else {
        "Agent".into()
    }
}

/// `personInitials`.
fn initials(name: &str) -> String {
    let mut words = name.split_whitespace();
    let first = words.next().and_then(|word| word.chars().next());
    let second = words.next().and_then(|word| word.chars().next());
    let initials: String = first
        .into_iter()
        .chain(second)
        .collect::<String>()
        .to_uppercase();
    if initials.is_empty() {
        "?".into()
    } else {
        initials
    }
}

/// `ActorIcon`: an agent's bot (its mark, still unless its run works), or a person's
/// initials on a round identity tile.
pub(crate) fn actor_icon(change: &ProjectChange, size: f32, cx: &App) -> AnyElement {
    let theme = Theme::global(cx);
    if actor_kind(change) == "human" {
        let name = if change.agent_label.is_empty() {
            change.agent_run_id.as_str()
        } else {
            change.agent_label.as_str()
        };
        return div()
            .size(px(size))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(theme.material_face_top)
            .text_size(px(if size <= 20.0 { 10.0 } else { 11.5 }))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.color_text_secondary)
            .child(initials(name))
            .into_any_element();
    }
    let agent = if change.agent_label.is_empty() {
        change.agent_run_id.as_str()
    } else {
        change.agent_label.as_str()
    };
    agent_avatar(agent, &change.run_status, size.max(20.0)).into_any_element()
}

/// The parts of a change's payload the detail shows (`parseChangeEnvelope`).
#[derive(Default)]
struct Envelope {
    summary: String,
    rationale: String,
    tool: String,
    subjects: Vec<String>,
    paths: Vec<String>,
}

fn envelope(payload: &str) -> Envelope {
    let Ok(serde_json::Value::Object(raw)) = serde_json::from_str::<serde_json::Value>(payload)
    else {
        return Envelope::default();
    };
    let text = |key: &str| {
        raw.get(key)
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string()
    };
    let list = |key: &str| {
        raw.get(key)
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    Envelope {
        summary: text("summary"),
        rationale: text("rationale"),
        tool: text("tool"),
        subjects: list("subject_ids"),
        paths: list("paths"),
    }
}

/// `stepKindLabel`.
fn step_kind_label(kind: &str) -> String {
    let mut chars = kind.chars();
    match chars.next() {
        None => "Step".into(),
        Some(first) => first
            .to_uppercase()
            .chain(chars)
            .collect::<String>()
            .replace('_', " "),
    }
}

pub struct ActivityView {
    state: Entity<AppState>,
    filters: Entity<FilterPopover>,
    /// The tiers and agents the filters offer, as last set.
    offered: (Vec<String>, Vec<String>),
    sort: Option<(usize, SortDirection)>,
    scope: ScopeFilter,
}

impl ActivityView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let filters =
            cx.new(|cx| FilterPopover::new("activity-filters", "Activity filters", true, cx));
        cx.subscribe(&filters, |this, filters, _: &FiltersChanged, cx| {
            this.scope.push(&filters, &this.state, cx);
            cx.notify();
        })
        .detach();
        cx.observe(&filters, |_, _, cx| cx.notify()).detach();
        cx.observe(&state, |view, _, cx| {
            view.offer_known(cx);
            cx.notify();
        })
        .detach();
        let mut view = Self {
            state,
            filters,
            offered: (Vec::new(), Vec::new()),
            sort: None,
            scope: ScopeFilter::new(true),
        };
        view.offer_known(cx);
        view
    }

    /// Offers the tiers and agents Dispatch's entries have named, once they change.
    fn offer_known(&mut self, cx: &mut Context<Self>) {
        let state = self.state.read(cx);
        let known = (
            state.dispatch_tiers().to_vec(),
            state.dispatch_agents().to_vec(),
        );
        if known == self.offered && !self.filters.read(cx).fields().is_empty() {
            return;
        }
        let fields = filter_fields(&known.0, &known.1);
        self.filters
            .update(cx, |popover, _| set_page_fields(popover, fields));
        self.offered = known;
    }

    pub fn filters(&self) -> &Entity<FilterPopover> {
        &self.filters
    }

    /// The changes in table order: by day, newest first, each day in the sorted order.
    fn ordered(&self, changes: &[ProjectChange]) -> (Vec<ProjectChange>, Vec<TableGroup>) {
        let mut days: Vec<(Option<Day>, Vec<ProjectChange>)> = Vec::new();
        for change in changes {
            let day = day_of(&change.created_at);
            match days.iter_mut().find(|(key, _)| *key == day) {
                Some((_, rows)) => rows.push(change.clone()),
                None => days.push((day, vec![change.clone()])),
            }
        }
        // Newest day first; an unreadable one last, as "Earlier".
        days.sort_by(|(a, _), (b, _)| match (a, b) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, _) => std::cmp::Ordering::Greater,
            (_, None) => std::cmp::Ordering::Less,
            (Some(a), Some(b)) => b.cmp(a),
        });
        let now = today();
        let mut rows = Vec::with_capacity(changes.len());
        let mut groups = Vec::with_capacity(days.len());
        for (day, mut items) in days {
            if let Some((column, direction)) = self.sort {
                items.sort_by(|a, b| {
                    let order = match column {
                        0 => a.summary.cmp(&b.summary),
                        1 => actor_kind(a).cmp(actor_kind(b)),
                        2 => a.run_status.cmp(&b.run_status),
                        _ => a.created_at.cmp(&b.created_at),
                    };
                    match direction {
                        SortDirection::Ascending => order,
                        SortDirection::Descending => order.reverse(),
                    }
                });
            }
            let (key, label) = match day {
                Some(day) => (
                    format!("{}-{:02}-{:02}", day.0, day.1, day.2),
                    day_label(day, now),
                ),
                None => ("earlier".to_string(), "Earlier".to_string()),
            };
            let start = rows.len();
            rows.extend(items);
            groups.push(TableGroup::new(key, label, start..rows.len()));
        }
        (rows, groups)
    }

    /// The detail's frame: `title` after `leading`, closed by its button or by a click outside
    /// it that opened no other row.
    fn panel(&self, title: String, leading: AnyElement, cx: &mut Context<Self>) -> DetailPanel {
        let close_state = self.state.clone();
        DetailPanel::new("activity-detail", title)
            .leading(leading)
            .on_close(Rc::new(move |_, cx: &mut App| {
                close_state.update(cx, |state, cx| {
                    state.close_change();
                    cx.notify();
                });
            }))
            .on_outside_click({
                // A click on a row opened another change; any other click outside closes.
                let state = self.state.clone();
                let presses = state.read(cx).activity_presses();
                Rc::new(move |_, cx: &mut App| {
                    state.update(cx, |state, cx| {
                        if state.activity_presses() == presses && state.activity_open().is_some() {
                            state.close_change();
                            cx.notify();
                        }
                    });
                })
            })
    }

    /// `DispatchEventSummaryDetail`: a Dispatch entry's run summary (its status, tier, agent,
    /// and error), with Open in Dispatch, and the event (its kind, run id, and time) in the
    /// rail.
    fn dispatch_detail(&self, event: &DispatchRunEvent, cx: &mut Context<Self>) -> DetailPanel {
        let theme = *Theme::global(cx);
        let title = if event.text.is_empty() {
            format!("Dispatch {}", event.kind)
        } else {
            event.text.clone()
        };
        let panel = self.panel(
            title,
            icon_colored(Icon::Route, 16.0, theme.color_text_secondary),
            cx,
        );
        let or_dash = |value: &str| {
            SharedString::from(if value.is_empty() {
                "\u{2014}".to_string()
            } else {
                value.to_string()
            })
        };
        let kind_chip = |cx: &mut Context<Self>| {
            chip_rounded(
                event.kind.clone(),
                dispatch_tone(&event.kind),
                CHIP_RADIUS,
                cx,
            )
        };
        let error = if event.kind == "failed" {
            event.text.as_str()
        } else {
            ""
        };
        let url = dispatch::run_url(&event.run_id);
        let open = icon_button(
            "activity-open-in-dispatch",
            Icon::ExternalLink,
            "Open in Dispatch",
            Rc::new(move |_, cx: &mut App| dispatch::open_in_dispatch(&url, cx)),
            cx,
        );
        let summary = detail_properties([
            detail_property("Status", kind_chip(cx), cx),
            detail_property("Tier", or_dash(&event.tier), cx),
            detail_property("Agent", or_dash(&event.agent), cx),
            detail_property("Error", or_dash(error), cx),
        ]);
        let run_id = div()
            .font(mono(&theme, FontWeight::NORMAL))
            .text_size(px(12.5))
            .child(or_dash(&event.run_id));
        let facts = detail_properties([
            detail_property("Kind", kind_chip(cx), cx),
            detail_property("Run id", run_id, cx),
            detail_property("When", SharedString::from(date_time(&event.at)), cx),
        ]);
        panel
            .section(detail_section_with_aside(
                "Run summary",
                Icon::FileText,
                open,
                summary,
                cx,
            ))
            .rail(detail_section("Event", Icon::Settings, facts, cx))
    }

    fn detail(
        &self,
        change: &ProjectChange,
        run: Option<&GetAgentRunResponse>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> DetailPanel {
        let theme = *Theme::global(cx);
        let run = run.filter(|run| {
            run.run
                .as_ref()
                .is_some_and(|detail| detail.id == change.agent_run_id)
        });
        // The step this change is in the run, else the run's change step, else the row's own
        // payload.
        let step = run.and_then(|run| {
            run.steps
                .iter()
                .find(|step| step.id == change.step_id)
                .or_else(|| run.steps.iter().find(|step| step.kind == "change"))
        });
        let envelope = envelope(step.map_or(&change.payload_json, |step| &step.payload_json));
        let title = [&envelope.summary, &change.summary]
            .into_iter()
            .find(|text| !text.is_empty())
            .cloned()
            .unwrap_or_else(|| "Change".into());
        let mut panel = self.panel(title, actor_icon(change, 16.0, cx), cx);

        let rationale = envelope.rationale.trim();
        let rationale = if rationale.is_empty() {
            detail_text("No reason given.", cx)
                .text_color(theme.color_text_muted)
                .into_any_element()
        } else {
            detail_text(rationale.to_string(), cx).into_any_element()
        };
        panel = panel.section(detail_section("Rationale", Icon::FileText, rationale, cx));

        let chain: AnyElement = match run {
            None => text_skeleton(2, 12.0, window, cx).into_any_element(),
            Some(run) => {
                let now = now_seconds();
                // `flex-col gap-3 text-[12.5px]` on the body's 1.5 line, each step `gap-1 px-2
                // py-1.5`, in rem on the 13px root.
                div()
                    .flex()
                    .flex_col()
                    .gap(px(tw(3.0)))
                    .text_size(px(12.5))
                    .line_height(relative(1.5))
                    .children(run.steps.iter().map(|step| {
                        let current = step.id == change.step_id;
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tw(1.0)))
                            .px(px(tw(2.0)))
                            .py(px(tw(1.5)))
                            .rounded(px(theme.radius_item))
                            .when(current, |item| item.bg(theme.color_surface_selected))
                            .child(
                                div()
                                    .flex()
                                    .items_baseline()
                                    .justify_between()
                                    .gap(px(tw(3.0)))
                                    .text_size(px(11.5))
                                    .child(
                                        div()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.color_text_secondary)
                                            .child(step_kind_label(&step.kind)),
                                    )
                                    .child(
                                        div()
                                            .font(tabular(sans(&theme, FontWeight::NORMAL)))
                                            .text_color(theme.color_text_muted)
                                            .child(ago(&step.created_at, now)),
                                    ),
                            )
                            .child(div().text_color(theme.color_text_primary).child(
                                if step.summary.is_empty() {
                                    "\u{2014}".to_string()
                                } else {
                                    step.summary.clone()
                                },
                            ))
                    }))
                    .into_any_element()
            }
        };
        panel = panel.section(detail_section("Audit chain", Icon::History, chain, cx));

        let mono_font = mono(&theme, FontWeight::NORMAL);
        // `font-mono` in the value's 12.5px.
        let mono_value = |value: String| {
            div()
                .font(mono_font.clone())
                .text_size(px(12.5))
                .child(value)
                .into_any_element()
        };
        let mut rows = vec![detail_property(
            "Actor",
            chip_rounded(
                actor_kind(change).to_string(),
                actor_tone(actor_kind(change)),
                CHIP_RADIUS,
                cx,
            ),
            cx,
        )];
        if !change.agent_label.is_empty() {
            rows.push(detail_property(
                "Agent",
                SharedString::from(change.agent_label.clone()),
                cx,
            ));
        }
        rows.push(detail_property(
            "Run",
            chip_rounded(
                run_status(change).to_string(),
                run_tone(&change.run_status),
                CHIP_RADIUS,
                cx,
            ),
            cx,
        ));
        if !change.run_task.is_empty() {
            rows.push(detail_property(
                "Task",
                SharedString::from(change.run_task.clone()),
                cx,
            ));
        }
        rows.push(detail_property(
            "When",
            SharedString::from(date_time(&change.created_at)),
            cx,
        ));
        if !envelope.tool.is_empty() {
            rows.push(detail_property(
                "Tool",
                mono_value(envelope.tool.clone()),
                cx,
            ));
        }
        if !envelope.subjects.is_empty() {
            rows.push(detail_property(
                "Subjects",
                mono_value(envelope.subjects.join(", ")),
                cx,
            ));
        }
        if !envelope.paths.is_empty() {
            rows.push(detail_property(
                "Paths",
                mono_value(envelope.paths.join(", ")),
                cx,
            ));
        }
        if !change.agent_run_id.is_empty() {
            rows.push(detail_property(
                "Run id",
                mono_value(change.agent_run_id.clone()),
                cx,
            ));
        }
        panel.rail(detail_section(
            "Properties",
            Icon::Settings,
            detail_properties(rows),
            cx,
        ))
    }
}

impl Render for ActivityView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scope.sync(&self.filters, &self.state, cx);
        let theme = *Theme::global(cx);
        if let Some(page) = core_status(&self.state, cx) {
            return div().size_full().child(page);
        }
        let app = self.state.read(cx);
        let actor = app.activity_actor();
        let open = app.activity_open().map(str::to_string);
        let run = app.activity_run().cloned();
        // Dispatch's entries the Tier and Agent clauses keep (`matchClauses`).
        let filters = self.filters.read(cx).filters();
        let events: Vec<DispatchRunEvent> = app
            .activity_dispatch()
            .iter()
            .filter(|event| {
                filters.matches(|field| match field {
                    "tier" => Some(vec![event.tier.as_str()]),
                    "agent" => Some(vec![event.agent.as_str()]),
                    _ => None,
                })
            })
            .cloned()
            .collect();
        let changes = match app.activity() {
            Load::Ready(changes) => Some(changes.clone()),
            Load::Failed(message) => {
                let message = message.clone();
                return div().size_full().child(page_empty_with(
                    Some("Could not load change history"),
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

        let tab_state = self.state.clone();
        let tabs = Tabs::new("activity-actor")
            .tab("All")
            .tab("Human")
            .tab("Agent")
            .selected(
                ActorFilter::ALL
                    .iter()
                    .position(|a| *a == actor)
                    .unwrap_or(0),
            )
            .on_select(move |index, _, cx| {
                if let Some(actor) = ActorFilter::ALL.get(index).copied() {
                    tab_state.update(cx, |state, cx| {
                        state.set_activity_actor(actor, cx);
                        cx.notify();
                    });
                }
            });
        // `PageToolbar` with the tabs leading: 44px, padded 8px 8px 4px, the empty bar
        // dragging the window.
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
            .child(tabs)
            .child(div().flex_1().h_full().window_drag_region())
            .child(self.filters.clone());
        let filter_bar = FilterPopover::bar(&self.filters, window, cx).map(|bar| {
            div()
                .flex_none()
                .px(px(theme.space_2))
                .pb(px(theme.space_2))
                .child(bar)
        });

        let merged = changes
            .as_deref()
            .map(|changes| merge_rows(changes, &events));
        let (rows, groups) = merged
            .as_deref()
            .map(|rows| self.ordered(rows))
            .unwrap_or_default();
        let rows = Rc::new(rows);
        let active = open
            .as_ref()
            .and_then(|id| rows.iter().find(|change| &change.step_id == id).cloned());
        let list: AnyElement = match &merged {
            None => div()
                .size_full()
                .px(px(TABLE_INSET))
                .child(text_skeleton(4, 12.0, window, cx))
                .into_any_element(),
            Some(rows) if rows.is_empty() => {
                page_empty("No changes yet", MascotMood::Sleeping, cx).into_any_element()
            }
            // The gutters sit inside the scroll, whose whole width the columns share out, and the
            // widths measure every row, as the web table renders them all, and freeze once.
            Some(_) => div()
                .relative()
                .size_full()
                .child(
                    activity_table(rows.clone(), groups, self, cx)
                        .gutters(TABLE_INSET)
                        .measure_every_row()
                        .freeze_once(),
                )
                .into_any_element(),
        };
        let detail = active.as_ref().map(|change| {
            let event = change
                .step_id
                .strip_prefix(DISPATCH_ROW)
                .and_then(|id| events.iter().find(|event| event.id == id));
            match event {
                Some(event) => self.dispatch_detail(event, cx).into_any_element(),
                None => self
                    .detail(change, run.as_ref(), window, cx)
                    .into_any_element(),
            }
        });
        let has_detail = detail.is_some();
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .child(bar)
            .children(filter_bar)
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .child(div().flex_1().min_w(px(0.0)).h_full().child(list))
                    .when(has_detail, |body| {
                        body.child(div().flex_none().w(px(DETAIL_SPACE)))
                    }),
            )
            .children(detail.map(|detail| {
                div()
                    .absolute()
                    .top(px(theme.page_header_height))
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .child(detail)
            }))
    }
}

/// The Activity `DataTable`: Change (the actor and the summary, the run's task under it when
/// it says more), Actor, Run (its status chip), and When (the time), every column sorting,
/// grouped by day, its rows the list density's.
/// No row is marked active: the web table takes no `activeRowId`, so the open change keeps
/// a plain row and J and K move nothing.
fn activity_table(
    rows: Rc<Vec<ProjectChange>>,
    groups: Vec<TableGroup>,
    view: &ActivityView,
    cx: &mut Context<ActivityView>,
) -> DataTable {
    let columns = vec![
        Column::new("Change").sortable().shrinks(),
        Column::new("Actor").sortable(),
        Column::new("Run").sortable(),
        Column::new("When").sortable(),
    ];
    let cells = rows.clone();
    let opened = rows.clone();
    let measured = rows.clone();
    let tall = rows.clone();
    let open_state = view.state.clone();
    let sorted = cx.entity();
    DataTable::new("activity", columns, rows.len(), move |row, _window, cx| {
        let theme = *Theme::global(cx);
        let change = &cells[row.index];
        let task = run_line(change);
        let summary = if change.summary.is_empty() {
            "\u{2014}".to_string()
        } else {
            change.summary.clone()
        };
        let title = div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .items_center()
            // The cell's leading slot, `gap-2.5`.
            .gap(px(tw(2.5)))
            .child(actor_icon(change, ACTOR_SIZE, cx))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    // The cell's own ink, secondary and primary under the pointer, as the
                    // web summary inherits it.
                    .child(
                        div()
                            .text_size(px(12.5))
                            .line_height(px(SUMMARY_LINE))
                            .child(summary),
                    )
                    .children(task.map(|task| {
                        div()
                            .text_size(px(11.5))
                            .line_height(px(TASK_LINE))
                            .text_color(theme.color_text_muted)
                            .child(task)
                    })),
            )
            .into_any_element();
        vec![
            title,
            div()
                .text_size(px(12.5))
                .text_color(theme.color_text_secondary)
                .whitespace_nowrap()
                .child(actor_label(change))
                .into_any_element(),
            chip_rounded(
                run_status(change).to_string(),
                row_tone(change),
                CHIP_RADIUS,
                cx,
            )
            .into_any_element(),
            div()
                .text_size(px(12.5))
                .font(tabular(sans(&theme, FontWeight::NORMAL)))
                .text_color(theme.color_text_muted)
                .whitespace_nowrap()
                .child(time_of_day(&change.created_at))
                .into_any_element(),
        ]
    })
    .measure(move |range, window, cx| {
        let theme = *Theme::global(cx);
        let normal = sans(&theme, FontWeight::NORMAL);
        let medium = sans(&theme, FontWeight::MEDIUM);
        let mut widest = [0.0_f32; 4];
        for change in measured.get(range).unwrap_or_default() {
            let summary =
                ACTOR_SIZE + tw(2.5) + text_width(&change.summary, normal.clone(), 12.5, window);
            let actor = text_width(&actor_label(change), normal.clone(), 12.5, window);
            // The chip: its 1px frame and 9px padding each side around 11px/500 text.
            let chip = 2.0 * 10.0 + text_width(run_status(change), medium.clone(), 11.0, window);
            let when = text_width(
                &time_of_day(&change.created_at),
                tabular(normal.clone()),
                12.5,
                window,
            );
            for (slot, width) in widest.iter_mut().zip([summary, actor, chip, when]) {
                *slot = slot.max(width);
            }
        }
        widest.to_vec()
    })
    .row_height(ROW)
    .row_heights(move |row| {
        if tall.get(row).and_then(run_line).is_some() {
            TWO_LINE_ROW
        } else {
            ROW
        }
    })
    .groups(groups)
    .sort(view.sort, move |column, _, cx| {
        sorted.update(cx, |view, cx| {
            view.sort = next_sort(view.sort, column);
            cx.notify();
        });
    })
    .on_activate(move |index, _window, cx| {
        if let Some(change) = opened.get(index) {
            let step = change.step_id.clone();
            open_state.update(cx, |state, cx| {
                state.open_change(step, cx);
                cx.notify();
            });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(id: &str, kind: &str, agent: &str, tier: &str, at: &str) -> DispatchRunEvent {
        DispatchRunEvent {
            id: id.into(),
            run_id: format!("run-{id}"),
            project_id: "p1".into(),
            kind: kind.into(),
            agent: agent.into(),
            tier: tier.into(),
            at: at.into(),
            ..DispatchRunEvent::default()
        }
    }

    fn change(step: &str, at: &str) -> ProjectChange {
        ProjectChange {
            step_id: step.into(),
            created_at: at.into(),
            ..ProjectChange::default()
        }
    }

    #[test]
    fn a_run_with_its_second_line_stands_46_as_electron_draws_it() {
        assert_eq!(TWO_LINE_ROW, 46.0);
        let run = dispatch_row(&event("e1", "merged", "codex", "hard", ""));
        assert_eq!(run_line(&run).as_deref(), Some("Agent run (hard)"));
        // A change whose summary is its run's task has one line.
        let same = ProjectChange {
            summary: "Agent run".into(),
            run_task: "Agent run".into(),
            ..ProjectChange::default()
        };
        assert_eq!(run_line(&same), None);
        assert_eq!(run_line(&ProjectChange::default()), None);
    }

    #[test]
    fn a_dispatch_entry_reads_as_an_agent_row() {
        let row = dispatch_row(&event(
            "e1",
            "merged",
            "codex",
            "hard",
            "2026-11-04T10:00:00Z",
        ));
        assert_eq!(row.step_id, "dispatch:e1");
        assert_eq!(row.agent_run_id, "run-e1");
        assert_eq!(row.actor_kind, "agent");
        assert_eq!(row.agent_label, "codex / hard");
        assert_eq!(row.run_status, "merged");
        assert_eq!(row.run_task, "Agent run (hard)");
        assert_eq!(row.summary, "Agent run merged");
        assert_eq!(row.payload_json, "{}");

        let bare = dispatch_row(&DispatchRunEvent {
            text: "Fixed the build".into(),
            ..event("e2", "failed", "", "", "")
        });
        assert_eq!(bare.agent_label, "dispatch");
        assert_eq!(bare.run_task, "Agent run");
        assert_eq!(bare.summary, "Fixed the build");
        let untiered = dispatch_row(&event("e3", "started", "claude", "", ""));
        assert_eq!(untiered.agent_label, "claude");
    }

    #[test]
    fn the_feed_merges_newest_first_ties_in_order_and_unreadable_times_last() {
        let changes = [
            change("c1", "2026-11-04T10:00:00.500Z"),
            change("c2", "someday"),
            change("c3", "2026-11-04T09:00:00Z"),
        ];
        let events = [
            event("d1", "merged", "codex", "", "2026-11-04T10:00:00.500Z"),
            event("d2", "started", "codex", "", "2026-11-04T10:00:00.900Z"),
        ];
        let order: Vec<String> = merge_rows(&changes, &events)
            .into_iter()
            .map(|row| row.step_id)
            .collect();
        assert_eq!(order, ["dispatch:d2", "c1", "dispatch:d1", "c3", "c2"]);
    }

    #[test]
    fn a_dispatch_status_takes_the_web_tone() {
        assert_eq!(dispatch_tone("merged"), StatusTone::Ok);
        assert_eq!(dispatch_tone("Started"), StatusTone::Syncing);
        assert_eq!(dispatch_tone("failed"), StatusTone::Danger);
        assert_eq!(dispatch_tone("timed_out"), StatusTone::Warn);
        assert_eq!(dispatch_tone("other"), StatusTone::Muted);
        // A change's run keeps its own tone.
        let mut row = change("c1", "");
        row.run_status = "failed".into();
        assert_eq!(row_tone(&row), StatusTone::Warn);
        row.step_id = "dispatch:e1".into();
        assert_eq!(row_tone(&row), StatusTone::Danger);
    }
}
