//! Inbox (`inbox-page.tsx`, `inbox-item-detail.tsx`): items that need a person, from
//! `WorkService.ListInbox`. The toolbar holds Search inbox and the Type and Status filters;
//! the table groups the items by kind in triage order (Conflicts, Deploys, Questions, Setup),
//! open before Later within each, with the selection column and the active item, whose
//! detail floats on the right. The first item starts active; a click, Enter, or J and K open
//! another. H moves the selection, or the active item, to Later and D dismisses it; the
//! detail's footer runs each of the item's actions through its confirm.

use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FontWeight, IntoElement, KeyDownEvent,
    Render, SharedString, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::chip::chip_rounded;
use polterware_ui::data_table::{
    Column, DataTable, SelectionChange, SortDirection, TableGroup, next_sort, primary_cell_ink,
    sans, text_width,
};
use polterware_ui::filter::{FilterOption, FilterSpec};
use polterware_ui::filter_popover::{FilterField, FilterPopover, FiltersChanged};
use polterware_ui::icon_tile::{IconTile, TileTone};
use polterware_ui::icons::Icon;
use polterware_ui::menu::MenuItem;
use polterware_ui::patterns::{
    ContextMenu, DetailPanel, MascotMood, Morph, MorphAlign, detail_numbered_list,
    detail_properties, detail_property, detail_section, detail_text, page_empty, page_empty_with,
    selection_toolbar,
};
use polterware_ui::radio_group::{RadioOption, radio_group};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::{StatusTone, Theme, tw};
use polterware_ui::window::window_drag::WindowDragRegion;

use crate::contracts::InboxItem;
use crate::state::{
    ACTION_DECIDE, ACTION_DEFER, ACTION_DELEGATE, ACTION_DISMISS, AppState, INBOX_KINDS, Load,
    inbox_order,
};
use crate::views::core_status::core_status;
use crate::views::scope::{ScopeFilter, set_page_fields};

/// The room the floating detail keeps beside the list: its 400px and its 8px inset.
const DETAIL_SPACE: f32 = 408.0;
/// `.pw-data-table-scroll`'s padding-inline.
const TABLE_INSET: f32 = 12.0;
/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
/// `.pw-field-control.control-field-sm`'s radius.
const FIELD_RADIUS: f32 = 6.0;

/// The actions a selection can take at once: those that need no answer.
const BULK_ACTIONS: [&str; 2] = [ACTION_DEFER, ACTION_DISMISS];

fn kind_entry(kind: &str) -> (&'static str, &'static str, &'static str, Icon) {
    INBOX_KINDS
        .iter()
        .copied()
        .find(|(id, ..)| *id == kind)
        .unwrap_or(("setup", "Setup", "Setup", Icon::ShieldCheck))
}

pub fn kind_label(kind: &str) -> &'static str {
    kind_entry(kind).2
}

fn action_label(action: &str) -> &'static str {
    match action {
        ACTION_DECIDE => "Decide",
        ACTION_DELEGATE => "Delegate",
        ACTION_DEFER => "Later",
        _ => "Dismiss",
    }
}

fn action_icon(action: &str) -> Icon {
    match action {
        ACTION_DECIDE => Icon::Check,
        ACTION_DELEGATE => Icon::Bot,
        ACTION_DEFER => Icon::Clock,
        _ => Icon::Eye,
    }
}

/// `ACTION_HINT`: what each action does, matching what the core records.
fn action_hint(action: &str, kind: &str) -> &'static str {
    match action {
        ACTION_DECIDE if kind == "conflict" => {
            "Decide this conflict. The agent that raised it reads the answer and goes on."
        }
        ACTION_DECIDE => {
            "Answer this question. The answer is written on the card and the item leaves the Inbox."
        }
        ACTION_DELEGATE => {
            "Hand this question to an agent. It leaves your Inbox and shows up in polterware status for the next agent session."
        }
        ACTION_DEFER => {
            "Set this aside. It stays in the Inbox, dimmed under Later, until you come back to it."
        }
        _ => "This needs no action. The item leaves the Inbox.",
    }
}

/// `ACTION_CONFIRM`: the confirm's question, what happens, and the button that does it.
fn action_confirm(action: &str) -> (&'static str, &'static str, &'static str) {
    match action {
        ACTION_DELEGATE => (
            "Delegate to an agent?",
            "The question leaves your Inbox. The next agent session sees it in polterware status and answers it.",
            "Delegate",
        ),
        ACTION_DEFER => (
            "Move to Later?",
            "The item stays in the Inbox, dimmed under Later, until you come back to it.",
            "Move to Later",
        ),
        _ => (
            "Dismiss this item?",
            "It needs no action. The item leaves the Inbox.",
            "Dismiss",
        ),
    }
}

/// The row's status: "Later" for a deferred item, "Blocking" for one that blocks, else
/// "Open"; only a blocking open item takes the danger tone.
fn status_of(item: &InboxItem) -> (&'static str, StatusTone) {
    if item.state == "deferred" {
        ("Later", StatusTone::Muted)
    } else if item.blocking {
        ("Blocking", StatusTone::Danger)
    } else {
        ("Open", StatusTone::Muted)
    }
}

/// `statusLabel`, which the Status column sorts by.
fn status_sort_label(item: &InboxItem) -> String {
    let kind = kind_label(&item.kind);
    if item.state == "deferred" {
        format!("{kind} for later")
    } else if item.blocking {
        format!("Blocking {}", kind.to_lowercase())
    } else {
        kind.to_string()
    }
}

fn status_id(item: &InboxItem) -> &'static str {
    if item.state == "deferred" {
        "deferred"
    } else if item.blocking {
        "blocking"
    } else {
        "open"
    }
}

pub struct InboxView {
    state: Entity<AppState>,
    search: Entity<TextInput>,
    filters: Entity<FilterPopover>,
    menu: Entity<ContextMenu>,
    /// Decide's answer when the item has no options, and a conflict's "Another answer".
    answer: Entity<TextInput>,
    /// The option picked in Decide.
    picked: Option<String>,
    query: String,
    sort: Option<(usize, SortDirection)>,
    fields_key: String,
    scope: ScopeFilter,
}

impl InboxView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new(cx).placeholder("Search inbox").pill());
        cx.observe(&search, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.query {
                this.query = text;
                cx.notify();
            }
        })
        .detach();
        let filters = cx.new(|cx| FilterPopover::new("inbox-filters", "Inbox filters", true, cx));
        cx.subscribe(&filters, |this, filters, _: &FiltersChanged, cx| {
            this.scope.push(&filters, &this.state, cx);
            cx.notify();
        })
        .detach();
        cx.observe(&filters, |_, _, cx| cx.notify()).detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        let menu = cx.new(ContextMenu::new);
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        let answer = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("Your answer")
                .radius(FIELD_RADIUS)
        });
        cx.observe(&answer, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            search,
            filters,
            menu,
            answer,
            picked: None,
            query: String::new(),
            sort: None,
            fields_key: String::new(),
            scope: ScopeFilter::new(false),
        }
    }

    pub fn filters(&self) -> &Entity<FilterPopover> {
        &self.filters
    }

    /// Type and Status, each option counting the items it holds.
    fn sync_fields(&mut self, items: &[InboxItem], cx: &mut Context<Self>) {
        let type_counts: Vec<usize> = INBOX_KINDS
            .iter()
            .map(|(kind, ..)| items.iter().filter(|item| item.kind == *kind).count())
            .collect();
        let statuses = [
            ("blocking", "Blocking"),
            ("open", "Open"),
            ("deferred", "Later"),
        ];
        let status_counts: Vec<usize> = statuses
            .iter()
            .map(|(id, _)| items.iter().filter(|item| status_id(item) == *id).count())
            .collect();
        let key = format!("{type_counts:?}{status_counts:?}");
        if key == self.fields_key {
            return;
        }
        self.fields_key = key;
        let option = |id: &str, label: &str, count: usize| {
            let mut option = FilterOption::new(id, label);
            option.trailing = Some(count.to_string());
            option
        };
        let types = FilterSpec::multi(
            "type",
            "Type",
            INBOX_KINDS
                .iter()
                .zip(&type_counts)
                .map(|((id, label, ..), count)| option(id, label, *count))
                .collect(),
        );
        let mut status = FilterSpec::multi(
            "status",
            "Status",
            statuses
                .iter()
                .zip(&status_counts)
                .map(|((id, label), count)| option(id, label, *count))
                .collect(),
        );
        status.plural_label = Some("statuses".into());
        self.filters.update(cx, |popover, _| {
            set_page_fields(
                popover,
                vec![
                    FilterField::new(types, Some(Icon::Scale)),
                    FilterField::new(status, None),
                ],
            );
        });
    }

    /// The items on show, in display order: grouped by kind, open before Later, searched and
    /// filtered, and sorted by status inside each group when Status sorts.
    fn visible(&self, items: &[InboxItem], cx: &App) -> Vec<InboxItem> {
        let needle = self.query.trim().to_lowercase();
        let filters = self.filters.read(cx).filters();
        let mut rows: Vec<InboxItem> = inbox_order(items)
            .into_iter()
            .filter(|item| {
                needle.is_empty()
                    || item.title.to_lowercase().contains(&needle)
                    || item.detail.to_lowercase().contains(&needle)
            })
            .filter(|item| {
                filters.matches(|field| match field {
                    "type" => Some(vec![item.kind.as_str()]),
                    "status" => Some(vec![status_id(item)]),
                    _ => None,
                })
            })
            .cloned()
            .collect();
        if let Some((1, direction)) = self.sort {
            let group =
                |item: &InboxItem| INBOX_KINDS.iter().position(|(kind, ..)| *kind == item.kind);
            rows.sort_by(|a, b| {
                group(a).cmp(&group(b)).then_with(|| {
                    let order = status_sort_label(a).cmp(&status_sort_label(b));
                    match direction {
                        SortDirection::Ascending => order,
                        SortDirection::Descending => order.reverse(),
                    }
                })
            });
        }
        rows
    }

    fn activate(&mut self, id: String, cx: &mut Context<Self>) {
        let changed = self.state.read(cx).inbox_open() != Some(id.as_str());
        if changed {
            self.reset_answer(cx);
        }
        self.state.update(cx, |state, cx| {
            state.open_inbox_item(id);
            cx.notify();
        });
    }

    fn reset_answer(&mut self, cx: &mut Context<Self>) {
        self.picked = None;
        self.answer.update(cx, |input, cx| input.set_text("", cx));
    }

    /// Opens `action`'s confirm on the item, as the row menu does.
    fn request(&mut self, id: String, action: String, cx: &mut Context<Self>) {
        self.activate(id, cx);
        self.open_confirm(Some(action), cx);
    }

    fn open_confirm(&mut self, action: Option<String>, cx: &mut Context<Self>) {
        if action.as_deref() == Some(ACTION_DECIDE) {
            // A single option is the answer already.
            let only = self
                .active_item(cx)
                .and_then(|item| (item.options.len() == 1).then(|| item.options[0].clone()));
            self.picked = only;
            self.answer.update(cx, |input, cx| input.set_text("", cx));
        }
        self.state.update(cx, |state, cx| {
            state.set_inbox_confirm(action);
            cx.notify();
        });
    }

    fn active_item(&self, cx: &App) -> Option<InboxItem> {
        let state = self.state.read(cx);
        let id = state.inbox_open()?;
        match state.inbox() {
            Load::Ready(inbox) => inbox.items.iter().find(|item| item.id == id).cloned(),
            _ => None,
        }
    }

    /// The answer Decide records: the picked option, else what was typed.
    fn decision(&self, cx: &App) -> String {
        let typed = self.answer.read(cx).text().trim().to_string();
        if !typed.is_empty() {
            return typed;
        }
        self.picked.clone().unwrap_or_default()
    }

    fn run(&mut self, ids: Vec<String>, action: &str, cx: &mut Context<Self>) {
        let answer = if action == ACTION_DECIDE {
            self.decision(cx)
        } else {
            String::new()
        };
        if action == ACTION_DECIDE && answer.is_empty() {
            return;
        }
        let action = action.to_string();
        self.state.update(cx, |state, cx| {
            state.resolve_inbox(ids, &action, answer, cx);
            cx.notify();
        });
    }

    /// H and D (`keyActionTargets`): the selection when every picked item offers the action,
    /// else the active item when it does.
    fn key_action(&mut self, action: &str, visible: &[InboxItem], cx: &mut Context<Self>) -> bool {
        let state = self.state.read(cx);
        if state.inbox_pending() || state.inbox_confirm().is_some() {
            return false;
        }
        let selected = state.inbox_selected().clone();
        let picked: Vec<&InboxItem> = visible
            .iter()
            .filter(|item| selected.contains(&item.id))
            .collect();
        let targets: Vec<String> = if !picked.is_empty() {
            if picked
                .iter()
                .all(|item| item.actions.iter().any(|a| a == action))
            {
                picked.iter().map(|item| item.id.clone()).collect()
            } else {
                Vec::new()
            }
        } else {
            let open = state.inbox_open().map(str::to_string);
            visible
                .iter()
                .filter(|item| Some(&item.id) == open.as_ref())
                .filter(|item| item.actions.iter().any(|a| a == action))
                .map(|item| item.id.clone())
                .collect()
        };
        if targets.is_empty() {
            return false;
        }
        self.run(targets, action, cx);
        true
    }

    fn row_menu(
        &mut self,
        item: &InboxItem,
        at: gpui::Point<gpui::Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let entity = cx.entity();
        let pending = self.state.read(cx).inbox_pending();
        if pending {
            return;
        }
        let items: Vec<MenuItem> = item
            .actions
            .iter()
            .map(|action| {
                let entity = entity.clone();
                let id = item.id.clone();
                let action = action.clone();
                MenuItem::action(
                    action_label(&action),
                    Some(action_icon(&action)),
                    Rc::new(move |_, cx: &mut App| {
                        let (id, action) = (id.clone(), action.clone());
                        entity.update(cx, |view, cx| view.request(id, action, cx));
                    }),
                )
            })
            .collect();
        if items.is_empty() {
            return;
        }
        self.menu
            .update(cx, |menu, cx| menu.open(at, items, window, cx));
    }
}

impl Render for InboxView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        if let Some(page) = core_status(&self.state, cx) {
            return div().size_full().child(page);
        }
        let inbox = match self.state.read(cx).inbox() {
            Load::Ready(inbox) => Some(inbox.clone()),
            Load::Failed(message) => {
                return div().size_full().child(page_empty_with(
                    Some("Could not read the Inbox"),
                    MascotMood::Sad,
                    Some(
                        div()
                            .text_size(px(theme.font_size_sm))
                            .text_color(theme.color_text_muted)
                            .child(SharedString::from(message.clone()))
                            .into_any_element(),
                    ),
                    cx,
                ));
            }
            _ => None,
        };
        if let Some(inbox) = &inbox {
            self.sync_fields(&inbox.items, cx);
        }
        self.scope.sync(&self.filters, &self.state, cx);
        let visible = Rc::new(
            inbox
                .as_ref()
                .map(|inbox| self.visible(&inbox.items, cx))
                .unwrap_or_default(),
        );

        let actions = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(div().w(px(200.0)).child(self.search.clone()))
            .child(self.filters.clone());
        // `PageToolbar` with no leading content: 44px, padded 8px 8px 4px, its actions at
        // the end and the empty bar dragging the window. It stretches across the page: a
        // percentage width does not resolve in this view.
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
            .child(div().flex_1().h_full().window_drag_region())
            .child(actions);
        let filter_bar = FilterPopover::bar(&self.filters, window, cx).map(|bar| {
            div()
                .flex_none()
                .px(px(theme.space_2))
                .pb(px(theme.space_2))
                .child(bar)
        });

        let state = self.state.read(cx);
        let open_id = state.inbox_open().map(str::to_string);
        let active = open_id
            .as_ref()
            .and_then(|id| visible.iter().find(|item| &item.id == id).cloned());
        let selected = state.inbox_selected().clone();
        let pending = state.inbox_pending();
        let confirm = state.inbox_confirm().map(str::to_string);
        let shows_detail = state.inbox_detail_open();

        let list: AnyElement = match &inbox {
            None => div().into_any_element(),
            Some(inbox) if inbox.items.is_empty() => {
                page_empty("All quiet", MascotMood::Sleeping, cx).into_any_element()
            }
            Some(_) if visible.is_empty() => {
                page_empty("No matches", MascotMood::Lost, cx).into_any_element()
            }
            Some(_) => {
                let table = inbox_table(visible.clone(), &active, &selected, self, cx);
                // The gutters sit inside the scroll, whose whole width the columns share out, and the
                // widths measure every row, as the web table renders them all, and freeze once.
                let mut list = div()
                    .relative()
                    .size_full()
                    .child(table.gutters(TABLE_INSET).measure_every_row().freeze_once());
                if !selected.is_empty() {
                    list = list.child(self.selection_bar(&visible, &selected, pending, cx));
                }
                list.into_any_element()
            }
        };

        let detail = active.as_ref().filter(|_| shows_detail).map(|item| {
            self.detail(item, confirm.as_deref(), pending, window, cx)
                .into_any_element()
        });
        let has_detail = detail.is_some();

        let keys_visible = visible.clone();
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .on_key_down(cx.listener(move |view, event: &KeyDownEvent, _, cx| {
                if event.is_held {
                    return;
                }
                // H and D, or what Settings › Shortcuts remapped them to.
                let overrides = view.state.read(cx).shortcut_overrides();
                let keystroke = &event.keystroke;
                let action = if crate::shortcuts::matches("inbox.defer", keystroke, overrides) {
                    ACTION_DEFER
                } else if crate::shortcuts::matches("inbox.dismiss", keystroke, overrides) {
                    ACTION_DISMISS
                } else {
                    return;
                };
                if view.key_action(action, &keys_visible, cx) {
                    cx.stop_propagation();
                }
            }))
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
            .child(self.menu.clone())
    }
}

impl InboxView {
    fn selection_bar(
        &self,
        visible: &[InboxItem],
        selected: &std::collections::BTreeSet<String>,
        pending: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let picked: Vec<&InboxItem> = visible
            .iter()
            .filter(|item| selected.contains(&item.id))
            .collect();
        let shared: Vec<&str> = BULK_ACTIONS
            .iter()
            .copied()
            .filter(|action| {
                !picked.is_empty()
                    && picked
                        .iter()
                        .all(|item| item.actions.iter().any(|a| a == action))
            })
            .collect();
        let ids: Vec<String> = picked.iter().map(|item| item.id.clone()).collect();
        let entity = cx.entity();
        let buttons = shared
            .into_iter()
            .map(|action| {
                let ids = ids.clone();
                let entity = entity.clone();
                Button::new(
                    gpui::ElementId::Name(format!("inbox-bulk-{action}").into()),
                    ButtonKind::Ghost,
                )
                .size(ButtonSize::Sm)
                .icon(action_icon(action))
                .label(action_label(action))
                .disabled(pending)
                .on_click(move |_, cx| {
                    let ids = ids.clone();
                    entity.update(cx, |view, cx| view.run(ids, action, cx));
                })
                .into_any_element()
            })
            .collect();
        let clear = self.state.clone();
        selection_toolbar(
            picked.len(),
            buttons,
            "No action fits every selected item",
            Rc::new(move |_, cx: &mut App| {
                clear.update(cx, |state, cx| {
                    state.set_inbox_selected(Vec::new());
                    cx.notify();
                });
            }),
            cx,
        )
    }

    /// `InboxItemDetail`: what the item is, where it came from, and its actions.
    fn detail(
        &self,
        item: &InboxItem,
        confirm: Option<&str>,
        pending: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> DetailPanel {
        let (status, tone) = status_of(item);
        let close_state = self.state.clone();
        let mut panel = DetailPanel::new("inbox-detail", item.title.clone())
            .aside(chip_rounded(
                kind_label(&item.kind),
                StatusTone::Muted,
                CHIP_RADIUS,
                cx,
            ))
            .aside(chip_rounded(status, tone, CHIP_RADIUS, cx))
            .on_close(Rc::new(move |_, cx: &mut App| {
                close_state.update(cx, |state, cx| {
                    state.close_inbox_detail();
                    cx.notify();
                });
            }))
            .on_outside_click({
                // A click on a row opened or picked one; any other click outside closes.
                let state = self.state.clone();
                let presses = state.read(cx).inbox_row_presses();
                Rc::new(move |_, cx: &mut App| {
                    state.update(cx, |state, cx| {
                        if state.inbox_row_presses() == presses && state.inbox_detail_open() {
                            state.close_inbox_detail();
                            cx.notify();
                        }
                    });
                })
            });
        let detail = |key: &str| {
            item.details
                .get(key)
                .filter(|value| !value.is_empty())
                .cloned()
        };
        if item.kind == "conflict" && (detail("rule").is_some() || detail("decision").is_some()) {
            let mut rows = Vec::new();
            if let Some(rule) = detail("rule") {
                rows.push(detail_property("Rule", SharedString::from(rule), cx));
            }
            if let Some(decision) = detail("decision") {
                rows.push(detail_property(
                    "Decision",
                    SharedString::from(decision),
                    cx,
                ));
            }
            panel = panel.section(detail_section(
                "Goes against",
                Icon::AlertDiamond,
                detail_properties(rows),
                cx,
            ));
        }
        if item.kind == "deploy" && (detail("url").is_some() || detail("dashboard").is_some()) {
            let mut links = div().flex().flex_wrap().gap(px(8.0));
            if let Some(url) = detail("url") {
                links = links.child(
                    Button::new("inbox-deploy-open", ButtonKind::Secondary)
                        .size(ButtonSize::Sm)
                        .icon(Icon::ArrowUpRight01Icon)
                        .label("Open the failed deploy")
                        .on_click(move |_, cx| cx.open_url(&url)),
                );
            }
            if let Some(url) = detail("dashboard") {
                let platform = detail("platform")
                    .or_else(|| detail("provider"))
                    .unwrap_or_else(|| "the platform".into());
                links = links.child(
                    Button::new("inbox-deploy-dashboard", ButtonKind::Ghost)
                        .size(ButtonSize::Sm)
                        .icon(Icon::ArrowUpRight01Icon)
                        .label(format!("Open in {platform}"))
                        .on_click(move |_, cx| cx.open_url(&url)),
                );
            }
            panel = panel.section(detail_section("Deploy", Icon::Cloud, links, cx));
        }
        if !item.detail.is_empty() && item.kind != "conflict" {
            panel = panel.section(detail_section(
                "Detail",
                Icon::FileText,
                detail_text(item.detail.clone(), cx),
                cx,
            ));
        }
        if !item.options.is_empty() {
            panel = panel.section(detail_section(
                "Options",
                Icon::ListFilter,
                detail_numbered_list(item.options.iter().cloned(), cx),
                cx,
            ));
        }
        let mut context = Vec::new();
        if !item.r#where.is_empty() {
            context.push(detail_property(
                "Where",
                SharedString::from(item.r#where.clone()),
                cx,
            ));
        }
        if !item.source.is_empty() {
            context.push(detail_property(
                "Source",
                SharedString::from(item.source.clone()),
                cx,
            ));
        }
        panel = panel.section(detail_section(
            "Context",
            Icon::Network,
            detail_properties(context),
            cx,
        ));

        if !item.actions.is_empty() {
            let count = item.actions.len();
            let mut footer = div().w_full().flex().items_center().gap(px(6.0));
            for (index, action) in item.actions.iter().enumerate() {
                footer = footer.child(
                    self.action_morph(item, action, index, count, confirm, pending, window, cx),
                );
            }
            panel = panel.footer(footer);
        }
        panel
    }

    #[allow(clippy::too_many_arguments)]
    fn action_morph(
        &self,
        item: &InboxItem,
        action: &str,
        index: usize,
        count: usize,
        confirm: Option<&str>,
        pending: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Morph {
        let theme = *Theme::global(cx);
        let trigger = Button::new(
            gpui::ElementId::Name(format!("inbox-action-{action}").into()),
            if action == ACTION_DECIDE {
                ButtonKind::Primary
            } else {
                ButtonKind::Secondary
            },
        )
        .size(ButtonSize::Sm)
        .icon(action_icon(action))
        .label(action_label(action))
        .tooltip(action_hint(action, &item.kind))
        .disabled(pending);
        let entity = cx.entity();
        let open_action = action.to_string();
        let align = if count < 2 || index == 0 {
            MorphAlign::Start
        } else if index + 1 == count {
            MorphAlign::End
        } else {
            MorphAlign::Center
        };
        let title: SharedString = if action == ACTION_DECIDE {
            item.title.clone().into()
        } else {
            action_confirm(action).0.into()
        };
        let mut morph = Morph::new(
            gpui::ElementId::Name(format!("inbox-morph-{action}").into()),
            trigger,
            title,
        )
        .grow()
        .align(align)
        .open(confirm == Some(action), move |open, _, cx| {
            let action = open.then(|| open_action.clone());
            entity.update(cx, |view, cx| view.open_confirm(action, cx));
        });
        let id = item.id.clone();
        let run_entity = cx.entity();
        let run_action = action.to_string();
        let run = Rc::new(move |_: &mut Window, cx: &mut App| {
            let (id, action) = (id.clone(), run_action.clone());
            run_entity.update(cx, |view, cx| view.run(vec![id], &action, cx));
        });
        if action == ACTION_DECIDE {
            let picked = self.picked.clone();
            let typed = self.answer.read(cx).text().trim().to_string();
            let answer_ready = !typed.is_empty() || picked.is_some();
            let mut body = div().flex().flex_col();
            if item.options.is_empty() {
                body = body.child(self.answer.clone());
            } else {
                let options: Vec<RadioOption> = item
                    .options
                    .iter()
                    .map(|option| RadioOption {
                        label: option.clone().into(),
                    })
                    .collect();
                let selected = picked
                    .as_ref()
                    .filter(|_| typed.is_empty())
                    .and_then(|picked| item.options.iter().position(|option| option == picked))
                    .unwrap_or(usize::MAX);
                let pick_entity = cx.entity();
                let choices = item.options.clone();
                let answer = self.answer.clone();
                body = body.child(radio_group(
                    "inbox-decide",
                    &options,
                    selected,
                    Rc::new(move |index, _, cx: &mut App| {
                        let choice = choices.get(index).cloned();
                        answer.update(cx, |input, cx| input.set_text("", cx));
                        pick_entity.update(cx, |view, cx| {
                            view.picked = choice;
                            cx.notify();
                        });
                    }),
                    cx,
                ));
                if item.kind == "conflict" {
                    self.answer
                        .update(cx, |input, _| input.set_placeholder("Another answer"));
                    // `mt-2` on the 13px root.
                    body = body.child(div().mt(px(tw(2.0))).child(self.answer.clone()));
                }
            }
            if item.options.is_empty() {
                self.answer
                    .update(cx, |input, _| input.set_placeholder("Your answer"));
            }
            let record = run.clone();
            morph = morph
                .body(body)
                .action(
                    Button::new("inbox-record", ButtonKind::Primary)
                        .size(ButtonSize::Sm)
                        .label(if pending {
                            "Saving\u{2026}"
                        } else {
                            "Record decision"
                        })
                        .full_width()
                        .disabled(!answer_ready || pending)
                        .on_click(move |window, cx| record(window, cx)),
                )
                .on_enter(move |window, cx| {
                    if answer_ready {
                        run(window, cx);
                    }
                });
        } else {
            let (_, body, confirm_label) = action_confirm(action);
            let confirm_run = run.clone();
            morph = morph
                .body(div().text_color(theme.color_text_secondary).child(body))
                .action(
                    Button::new(
                        gpui::ElementId::Name(format!("inbox-confirm-{action}").into()),
                        ButtonKind::Primary,
                    )
                    .size(ButtonSize::Sm)
                    .label(if pending {
                        "Saving\u{2026}"
                    } else {
                        confirm_label
                    })
                    .full_width()
                    .disabled(pending)
                    .on_click(move |window, cx| confirm_run(window, cx)),
                )
                .on_enter(move |window, cx| run(window, cx));
        }
        morph
    }
}

/// The Inbox `DataTable`: the selection column, Item (the kind's tile and the title), and
/// Status (112px, sorting), grouped by kind.
fn inbox_table(
    rows: Rc<Vec<InboxItem>>,
    active: &Option<InboxItem>,
    selected: &std::collections::BTreeSet<String>,
    view: &InboxView,
    cx: &mut Context<InboxView>,
) -> DataTable {
    let columns = vec![
        Column::new("Item").shrinks(),
        Column::new("Status").width(112.0).sortable(),
    ];
    let mut groups = Vec::new();
    let mut start = 0;
    for (kind, label, _, icon) in INBOX_KINDS {
        let count = rows[start..]
            .iter()
            .take_while(|item| item.kind == kind)
            .count();
        if count > 0 {
            groups.push(TableGroup::new(kind, label, start..start + count).icon(icon));
            start += count;
        }
    }
    let active_index = active
        .as_ref()
        .and_then(|active| rows.iter().position(|item| item.id == active.id));
    let picked: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, item)| selected.contains(&item.id))
        .map(|(index, _)| index)
        .collect();
    let theme = *Theme::global(cx);
    let font = sans(&theme, FontWeight::MEDIUM);
    let cells = rows.clone();
    let entity = cx.entity();
    let activate = entity.clone();
    let opened = rows.clone();
    let picks = rows.clone();
    let select_state = view.state.clone();
    let menu_entity = entity.clone();
    let menu_rows = rows.clone();
    let sorted = entity.clone();
    let measured = rows.clone();
    DataTable::new("inbox", columns, rows.len(), move |row, _window, cx| {
        let theme = *Theme::global(cx);
        let item = &cells[row.index];
        let (_, _, _, icon) = kind_entry(&item.kind);
        let tone = if item.blocking && item.state == "open" {
            TileTone::Security
        } else {
            TileTone::Neutral
        };
        let title = primary_cell_ink(
            IconTile::new(icon).size(24.0, 14.0).tone(tone),
            SharedString::from(item.title.clone()),
            font.clone(),
            if item.state == "deferred" {
                theme.color_text_muted
            } else {
                theme.color_text_primary
            },
            None,
        );
        let (status, tone) = status_of(item);
        vec![
            title,
            chip_rounded(status, tone, CHIP_RADIUS, cx).into_any_element(),
        ]
    })
    .measure(move |range, window, cx| {
        let theme = *Theme::global(cx);
        let medium = sans(&theme, FontWeight::MEDIUM);
        let mut widest = [0.0_f32; 2];
        for item in measured.get(range).unwrap_or_default() {
            let title = 24.0 + 10.0 + text_width(&item.title, medium.clone(), 12.5, window);
            // The chip: its 1px frame and 9px padding each side around 11px/500 text.
            let (status, _) = status_of(item);
            let chip = 2.0 * 10.0 + text_width(status, medium.clone(), 11.0, window);
            widest[0] = widest[0].max(title);
            widest[1] = widest[1].max(chip);
        }
        widest.to_vec()
    })
    .groups(groups)
    .active(active_index)
    .selection(picked, move |change, _, cx| {
        select_state.update(cx, |state, cx| {
            match change {
                SelectionChange::Toggle(index) => {
                    if let Some(item) = picks.get(index) {
                        state.toggle_inbox_selected(&item.id);
                    }
                }
                SelectionChange::All(all) => {
                    state.set_inbox_selected(if all {
                        picks.iter().map(|item| item.id.clone()).collect()
                    } else {
                        Vec::new()
                    });
                }
            }
            cx.notify();
        });
    })
    .sort(view.sort, move |column, _, cx| {
        sorted.update(cx, |view, cx| {
            if column == 1 {
                view.sort = next_sort(view.sort, column);
                cx.notify();
            }
        });
    })
    .on_activate(move |index, _window, cx| {
        if let Some(item) = opened.get(index) {
            let id = item.id.clone();
            activate.update(cx, |view, cx| view.activate(id, cx));
        }
    })
    .on_context(move |index, at, window, cx| {
        if let Some(item) = menu_rows.get(index).cloned() {
            menu_entity.update(cx, |view, cx| {
                view.activate(item.id.clone(), cx);
                view.row_menu(&item, at, window, cx);
            });
        }
    })
}
