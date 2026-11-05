//! Knowledge (the `truth` route: `index.tsx`'s `KnowledgeView`, `truth-rules-page.tsx`): the
//! Decisions, Rules, and Docs tabs on one toolbar with the tab's search, its filters, and its
//! "+". Decisions (`library-table.tsx`) lists every Decision, the principles (those with no
//! context) pinned on top, by Title, Context, Status, and Updated, filtered by lifecycle state;
//! Rules (`rules-panel.tsx`) lists each Rule by its statement and Context; Docs
//! (`docs-table.tsx`) lists the docs by Title, Context, Tags, and Updated, filtered by tag.
//! Every Context is a chip per Structure node, "Whole project" when there is none. Decisions
//! and Docs mark their first row; a row opens its record as a page in place of the list.
//!
//! The toolbar's "+" and C open the app's shared Composer (`KnowledgeComposer`) for a new
//! decision or doc, or this page's (`RuleFormDialog`) for a new Rule (its statement, its Context
//! as tokens, and an optional reason). A Rule opens in the Composer to edit over the list, with
//! Remove rule; its writes are Polter rule blocks (`ruleText`), and closing it goes
//! back. On Docs the archive button swaps the list for the archived docs (`ArchivedList`), each
//! restored or, once confirmed, deleted for good.

use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, FontWeight, IntoElement,
    Render, SharedString, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::chip::chip_rounded;
use polterware_ui::data_table::{
    Column, DataTable, SortDirection, next_sort, primary_cell, sans, text_width,
};
use polterware_ui::filter::{FilterOption, FilterSpec};
use polterware_ui::filter_popover::{FilterField, FilterPopover, FiltersChanged};
use polterware_ui::icon_tile::{IconTile, TileTone};
use polterware_ui::icons::Icon;
use polterware_ui::patterns::{
    ArchivedItem, ArchivedList, ArchivedListEvent, Composer, ComposerDraft, ComposerEvent,
    ComposerSpec, ConfirmDialog, MascotMood, Property, PropertyOption, modal_layer, page_empty,
    page_empty_with, window_layer,
};
use polterware_ui::skeleton::text_skeleton;
use polterware_ui::sort::{SortValue, sort_rows};
use polterware_ui::tabs::Tabs;
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::{StatusTone, Theme};
use polterware_ui::window::window_drag::WindowDragRegion;

use crate::contracts::{KnowledgeItem, Rule, StructureNode};
use crate::format::{archive_date, format_instant};
use crate::state::{AppState, ComposeRequest, Knowledge, KnowledgeTab, Load};
use crate::views::core_status::core_status;
use crate::views::knowledge_page::KnowledgePage;
use crate::views::scope::{ScopeFilter, clear_page, set_page_fields};

/// `.pw-data-table-scroll`'s padding-inline.
const TABLE_INSET: f32 = 12.0;
/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
/// A default-density row: the 24px tile in the cell's 5px padding.
const ROW_HEIGHT: f32 = 34.0;
/// `ContextChips` shows this many nodes, then "+N".
const CONTEXT_CHIPS: usize = 3;
/// `ContextChips`' `gap-1`.
const CHIP_GAP: f32 = 4.0;
/// A chip's width around its label: the 1px frame and 9px padding on each side.
const CHIP_FRAME: f32 = 20.0;
/// A Rule's statement placeholder (`RuleFormDialog`).
const RULE_PLACEHOLDER: &str = "The API returns only fields declared in the contract";
/// The Composer's reason placeholder (`WHY`).
const WHY: &str = "Optional. Recorded in Activity";
/// A summary's longest (`slice(0, 120)`).
const SUMMARY_MAX: usize = 120;

/// What the Composer is open for: a Rule, new or the one with this id.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Compose {
    Rule(Option<String>),
}

/// The lifecycle states the Status filter offers, in its order (`DECISION_STATE_LABEL`).
const DECISION_STATES: [(&str, &str); 4] = [
    ("active", "Active"),
    ("challenged", "Under review"),
    ("proposed", "Proposed"),
    ("superseded", "Replaced"),
];

/// `decisionState`: a stored status as a lifecycle state; the legacy `accepted` reads as
/// active and anything unknown as proposed.
pub fn decision_state(status: &str) -> &'static str {
    let value = status.trim().to_lowercase();
    if value == "accepted" {
        return "active";
    }
    DECISION_STATES
        .iter()
        .map(|(id, _)| *id)
        .find(|id| *id == value)
        .unwrap_or("proposed")
}

/// `DECISION_STATE_LABEL`.
pub fn decision_state_label(state: &str) -> &'static str {
    DECISION_STATES
        .iter()
        .find(|(id, _)| *id == state)
        .map_or("Proposed", |(_, label)| *label)
}

/// `decisionStateTone`.
pub fn decision_state_tone(state: &str) -> StatusTone {
    match state {
        "active" => StatusTone::Ok,
        "challenged" => StatusTone::Warn,
        _ => StatusTone::Muted,
    }
}

/// A Decision as its row shows it (`LibraryRow`).
#[derive(Clone, Debug, PartialEq)]
struct DecisionRow {
    id: String,
    title: String,
    scope: Vec<String>,
    state: &'static str,
    updated: String,
    /// A principle: no context, so it applies to the whole project and stays on top.
    pinned: bool,
}

/// `libraryRowsFromWorkspace`: the decisions in the core's order, the principles first.
fn decision_rows(items: &[KnowledgeItem]) -> Vec<DecisionRow> {
    let rows: Vec<DecisionRow> = items
        .iter()
        .filter(|item| item.kind == "decision")
        .map(|item| DecisionRow {
            id: item.id.clone(),
            title: item.title.clone(),
            scope: item.scope_node_ids.clone(),
            state: decision_state(&item.status),
            updated: format_instant(&item.updated_at),
            pinned: item.scope_node_ids.is_empty(),
        })
        .collect();
    let (pinned, rest): (Vec<_>, Vec<_>) = rows.into_iter().partition(|row| row.pinned);
    pinned.into_iter().chain(rest).collect()
}

/// The record a click on the tab's first row opens, in the list's own order: the first
/// decision (a principle when there is one), the first rule (in its Composer), or the first doc.
pub fn first_row(tab: KnowledgeTab, knowledge: &Knowledge) -> Option<String> {
    match tab {
        KnowledgeTab::Decisions => decision_rows(&knowledge.items)
            .first()
            .map(|row| row.id.clone()),
        KnowledgeTab::Docs => doc_rows(&knowledge.items).first().map(|row| row.id.clone()),
        KnowledgeTab::Rules => knowledge.rules.first().map(|rule| rule.id.clone()),
    }
}

/// A doc as its row shows it.
#[derive(Clone, Debug, PartialEq)]
struct DocRow {
    id: String,
    title: String,
    summary: String,
    owner: String,
    tags: Vec<String>,
    scope: Vec<String>,
    updated: String,
}

/// `mapDocs`: the items of kind doc, in the core's order.
fn doc_rows(items: &[KnowledgeItem]) -> Vec<DocRow> {
    items
        .iter()
        .filter(|item| item.kind == "doc")
        .map(|item| DocRow {
            id: item.id.clone(),
            title: item.title.clone(),
            summary: item.summary.clone(),
            owner: if item.owner.is_empty() {
                "Platform".into()
            } else {
                item.owner.clone()
            },
            tags: item.tags.clone(),
            scope: item.scope_node_ids.clone(),
            updated: format_instant(&item.updated_at),
        })
        .collect()
}

/// `distinctDocTags`: every doc's tags once, sorted.
fn doc_tags(docs: &[DocRow]) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    for tag in docs.iter().flat_map(|doc| &doc.tags) {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    tags.sort();
    tags
}

/// The Structure nodes `ids` names, in the tree's order (`ContextChips`).
fn context_nodes<'a>(ids: &[String], nodes: &'a [StructureNode]) -> Vec<&'a StructureNode> {
    nodes.iter().filter(|node| ids.contains(&node.id)).collect()
}

/// A Rule's Context as text (`aboutOf`): its nodes' titles, else "Whole project".
fn about(ids: &[String], nodes: &[StructureNode]) -> String {
    let titles: Vec<&str> = context_nodes(ids, nodes)
        .iter()
        .map(|node| node.title.as_str())
        .collect();
    if titles.is_empty() {
        "Whole project".into()
    } else {
        titles.join(", ")
    }
}

/// `ContextChips`' labels: the first three nodes, then "+N", or "Whole project".
pub(crate) fn context_labels(ids: &[String], nodes: &[StructureNode]) -> Vec<String> {
    let found = context_nodes(ids, nodes);
    if found.is_empty() {
        return vec!["Whole project".into()];
    }
    let mut labels: Vec<String> = found
        .iter()
        .take(CONTEXT_CHIPS)
        .map(|node| node.title.clone())
        .collect();
    if found.len() > CONTEXT_CHIPS {
        labels.push(format!("+{}", found.len() - CONTEXT_CHIPS));
    }
    labels
}

/// `scopeContextOptions`: the domains, then the components, each under its title (its path
/// when another node has the same title) and the mark `ScopeIcon` gives its level.
pub(crate) fn context_options(nodes: &[StructureNode]) -> Vec<PropertyOption> {
    let shared = |title: &str| nodes.iter().filter(|node| node.title == title).count() > 1;
    [
        ("domain", Icon::Boxes),
        ("component", Icon::Blockchain01Icon),
    ]
    .into_iter()
    .flat_map(|(level, icon)| {
        nodes
            .iter()
            .filter(move |node| node.level == level)
            .map(move |node| {
                let label = if shared(&node.title) && !node.path.is_empty() {
                    &node.path
                } else {
                    &node.title
                };
                PropertyOption::new(node.id.clone(), label.clone()).icon(icon)
            })
    })
    .collect()
}

/// `ContextPicker`: the nodes a record is about as tokens, none being the whole project.
pub(crate) fn context_property(ids: &[String], nodes: &[StructureNode]) -> Property {
    Property::tokens(
        "context",
        "Context",
        context_options(nodes),
        ids.iter().map(|id| id.clone().into()).collect(),
        "Whole project",
    )
    .loose()
    .icon(Icon::Layers)
}

/// `oneLine`: a Polter heading's value on one line.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `slice(0, 120)`: at most the first 120 characters.
fn summary(text: &str) -> String {
    text.chars().take(SUMMARY_MAX).collect()
}

/// `ruleHandle`: what a Polter block names a Rule by, its slug or else its id.
fn rule_handle(rule: &Rule) -> &str {
    if rule.slug.is_empty() {
        &rule.id
    } else {
        &rule.slug
    }
}

/// `cardSlugFromTitle`: the title in kebab case, at most 48 characters, or "card".
fn slug_from_title(title: &str) -> String {
    let mut slug = String::new();
    for ch in title.trim().to_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            slug.push(ch);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug: String = slug.trim_matches('-').chars().take(48).collect();
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "card".into()
    } else {
        slug.into()
    }
}

/// `uniqueCardSlug`: the title's slug, or the first numbered one from 2 not taken yet.
fn unique_slug(taken: &[&str], title: &str) -> String {
    let base = slug_from_title(title);
    if !taken.contains(&base.as_str()) {
        return base;
    }
    (2..)
        .map(|suffix| format!("{base}-{suffix}"))
        .find(|slug| !taken.contains(&slug.as_str()))
        .unwrap_or(base)
}

/// `ruleText`: a Rule's Polter block, `#### rule <handle>: Statement`, then `scope:` with the
/// node paths when they change (none clears them) and `status: removed` to retire it.
fn rule_text(handle: &str, statement: &str, scope: Option<&[String]>, remove: bool) -> String {
    let statement = if statement.is_empty() {
        String::new()
    } else {
        format!(": {}", one_line(statement))
    };
    let mut text = format!("---\npolter: 1\n---\n\n#### rule {handle}{statement}\n");
    if let Some(paths) = scope {
        if paths.is_empty() {
            text.push_str("scope:\n");
        } else {
            text.push_str(&format!("scope: {}\n", paths.join(", ")));
        }
    }
    if remove {
        text.push_str("status: removed\n");
    }
    text
}

/// The paths of the nodes `ids` names, sorted (`pathsOf`).
fn node_paths(ids: &[String], nodes: &[StructureNode]) -> Vec<String> {
    let mut paths: Vec<String> = nodes
        .iter()
        .filter(|node| ids.contains(&node.id))
        .map(|node| node.path.clone())
        .collect();
    paths.sort();
    paths
}

/// `saveRule`'s text and summary: an edit of `current` keeps its handle and names its scope
/// only when it changed; a new Rule takes a slug from its statement and names a scope only
/// when it has one.
fn rule_save(
    current: Option<&Rule>,
    rules: &[Rule],
    statement: &str,
    ids: &[String],
    nodes: &[StructureNode],
) -> String {
    let paths = node_paths(ids, nodes);
    let changed = current.is_none_or(|rule| {
        let mut was = rule.scope_paths.clone();
        was.sort();
        was != paths
    });
    let handle = match current {
        Some(rule) => rule_handle(rule).to_string(),
        None => {
            let taken: Vec<&str> = rules.iter().map(|rule| rule.slug.as_str()).collect();
            unique_slug(&taken, statement)
        }
    };
    let scope = (changed && (current.is_some() || !paths.is_empty())).then_some(paths.as_slice());
    rule_text(&handle, statement, scope, false)
}

pub(crate) fn chips(labels: &[String], cx: &App) -> AnyElement {
    div()
        .min_w(px(0.0))
        .flex()
        .items_center()
        .gap(px(CHIP_GAP))
        .overflow_hidden()
        .children(labels.iter().map(|label| {
            chip_rounded(label.clone(), StatusTone::Muted, CHIP_RADIUS, cx).flex_none()
        }))
        .into_any_element()
}

pub(crate) fn chips_width(labels: &[String], window: &Window, cx: &App) -> f32 {
    let theme = Theme::global(cx);
    let medium = sans(theme, FontWeight::MEDIUM);
    let widths: f32 = labels
        .iter()
        .map(|label| CHIP_FRAME + text_width(label, medium.clone(), 11.0, window))
        .sum();
    widths + CHIP_GAP * labels.len().saturating_sub(1) as f32
}

pub(crate) fn muted_cell(text: impl Into<SharedString>, cx: &App) -> AnyElement {
    let theme = Theme::global(cx);
    div()
        .text_size(px(12.5))
        .text_color(theme.color_text_muted)
        .whitespace_nowrap()
        .child(text.into())
        .into_any_element()
}

/// A row's title after its knowledge-toned tile (`DataTable.PrimaryCell`).
fn title_cell(icon: Icon, title: &str, cx: &App) -> AnyElement {
    let theme = Theme::global(cx);
    primary_cell(
        IconTile::new(icon)
            .size(24.0, 16.0)
            .tone(TileTone::Knowledge),
        SharedString::from(title.to_string()),
        sans(theme, FontWeight::MEDIUM),
        None,
        cx,
    )
}

pub(crate) fn title_width(title: &str, window: &Window, cx: &App) -> f32 {
    let theme = Theme::global(cx);
    24.0 + 10.0 + text_width(title, sans(theme, FontWeight::MEDIUM), 12.5, window)
}

fn text(value: &str) -> SortValue {
    SortValue::Text(value.to_string())
}

pub struct KnowledgeView {
    state: Entity<AppState>,
    search: Entity<TextInput>,
    /// One filter menu per tab, each with its own name (`Library filters`, `Rule filters`,
    /// `Doc filters`).
    filters: [Entity<FilterPopover>; 3],
    query: String,
    /// The tab the search, filters, and sort were set for; another one clears them.
    tab: KnowledgeTab,
    tags: Vec<String>,
    sort: Option<(usize, SortDirection)>,
    /// The marked row of Decisions or Docs; the first one until J, K, or a row moves it.
    active: Option<String>,
    /// The record open as a page.
    page: Entity<KnowledgePage>,
    composer: Entity<Composer>,
    /// What the Composer is open for, and the Context it holds.
    compose: Option<Compose>,
    contexts: Vec<SharedString>,
    /// Docs shows its archive in place of the list.
    show_archived: bool,
    archived: Entity<ArchivedList>,
    /// The archived doc waiting for Delete permanently to be confirmed: its id and title.
    delete_ask: Option<(String, String)>,
    delete_focus: FocusHandle,
    /// The confirm takes the focus on the next frame.
    delete_focus_due: bool,
    /// Each tab's link to the shared scope; Docs offers environments too.
    scopes: [ScopeFilter; 3],
}

impl KnowledgeView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new(cx).placeholder("Search").pill());
        cx.observe(&search, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.query {
                this.query = text;
                cx.notify();
            }
        })
        .detach();
        let filters = [
            ("knowledge-library-filters", "Library filters"),
            ("knowledge-rule-filters", "Rule filters"),
            ("knowledge-doc-filters", "Doc filters"),
        ]
        .map(|(id, label)| {
            let popover = cx.new(|cx| FilterPopover::new(id, label, true, cx));
            cx.subscribe(&popover, |this, popover, _: &FiltersChanged, cx| {
                if let Some(index) = this.filters.iter().position(|kept| *kept == popover) {
                    this.scopes[index].push(&popover, &this.state, cx);
                }
                cx.notify();
            })
            .detach();
            cx.observe(&popover, |_, _, cx| cx.notify()).detach();
            popover
        });
        let statuses = FilterSpec::multi(
            "status",
            "Status",
            DECISION_STATES
                .iter()
                .map(|(id, label)| FilterOption::new(*id, *label))
                .collect(),
        )
        .plural("statuses");
        filters[0].update(cx, |popover, _| {
            popover.set_fields(vec![FilterField::new(statuses, None)]);
        });
        filters[2].update(cx, |popover, _| {
            popover.set_fields(vec![FilterField::new(
                FilterSpec::multi("tags", "Tag", Vec::new()),
                Some(Icon::FileText),
            )]);
        });
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        let page = cx.new(|cx| KnowledgePage::new(state.clone(), cx));
        let composer = cx.new(Composer::new);
        cx.subscribe(&composer, Self::composed).detach();
        let archived = cx.new(|cx| {
            ArchivedList::new("knowledge-archived-docs", Icon::FileText, cx).gutters(TABLE_INSET)
        });
        cx.subscribe(&archived, Self::archive_picked).detach();
        Self {
            page,
            composer,
            compose: None,
            contexts: Vec::new(),
            show_archived: false,
            archived,
            delete_ask: None,
            delete_focus: cx.focus_handle(),
            delete_focus_due: false,
            scopes: [
                ScopeFilter::new(false),
                ScopeFilter::new(false),
                ScopeFilter::new(true),
            ],
            state,
            search,
            filters,
            query: String::new(),
            tab: KnowledgeTab::default(),
            tags: Vec::new(),
            sort: None,
            active: None,
        }
    }

    /// The Context property the Composer shows, while the project has nodes to pick.
    fn context_properties(&self, nodes: &[StructureNode]) -> Vec<Property> {
        if nodes.is_empty() {
            return Vec::new();
        }
        let ids: Vec<String> = self.contexts.iter().map(|id| id.to_string()).collect();
        vec![context_property(&ids, nodes)]
    }

    /// Opens the Composer for `compose`: the "+" of the tab, or a Rule to edit.
    fn open_composer(
        &mut self,
        compose: Compose,
        knowledge: &Knowledge,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let rule = match &compose {
            Compose::Rule(Some(id)) => knowledge.rules.iter().find(|rule| rule.id == *id),
            _ => None,
        };
        let spec = match (&compose, rule) {
            (Compose::Rule(_), Some(rule)) => ComposerSpec::new(RULE_PLACEHOLDER, "Save rule")
                .why(WHY)
                .edit()
                .action("Remove rule")
                .initial(ComposerDraft {
                    title: rule.statement.clone(),
                    ..ComposerDraft::default()
                }),
            (Compose::Rule(_), None) => ComposerSpec::new(RULE_PLACEHOLDER, "Create rule").why(WHY),
        };
        self.contexts = rule.map_or_else(Vec::new, |rule| {
            rule.scope_node_ids
                .iter()
                .map(|id| id.clone().into())
                .collect()
        });
        self.compose = Some(compose);
        let properties = self.context_properties(&knowledge.nodes);
        self.composer.update(cx, |composer, cx| {
            composer.open(spec, properties, window, cx);
        });
    }

    /// The "+" on the toolbar and C (`composeHere`): a decision or a doc in the app's
    /// Composer, a Rule in this page's.
    pub(crate) fn compose_here(
        &mut self,
        tab: KnowledgeTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let request = match tab {
            KnowledgeTab::Decisions => ComposeRequest::Decision,
            KnowledgeTab::Docs => ComposeRequest::Doc,
            KnowledgeTab::Rules => {
                let Load::Ready(knowledge) = self.state.read(cx).knowledge() else {
                    return;
                };
                let knowledge = knowledge.clone();
                return self.open_composer(Compose::Rule(None), &knowledge, window, cx);
            }
        };
        self.state
            .update(cx, |state, cx| state.compose(request, cx));
    }

    fn composed(
        &mut self,
        composer: Entity<Composer>,
        event: &ComposerEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ComposerEvent::Submit(draft) => self.write(draft.clone(), composer, cx),
            ComposerEvent::Property { values, .. } => {
                self.contexts = values.clone();
                if let Load::Ready(knowledge) = self.state.read(cx).knowledge() {
                    let properties = self.context_properties(&knowledge.nodes.clone());
                    composer.update(cx, |composer, cx| composer.set_properties(properties, cx));
                }
            }
            ComposerEvent::Action => self.remove_rule(composer, cx),
            ComposerEvent::Closed => {
                // A Rule's page closes with its Composer, unless the route already left it.
                if let Some(Compose::Rule(Some(id))) = self.compose.take()
                    && matches!(self.state.read(cx).knowledge_route(),
                        Some((KnowledgeTab::Rules, Some(page))) if page == id)
                {
                    self.state.update(cx, |state, cx| {
                        if state.can_go_back() {
                            state.go_back(cx);
                        } else {
                            state.set_knowledge_tab(KnowledgeTab::Rules, cx);
                        }
                        cx.notify();
                    });
                }
            }
        }
    }

    /// Writes what the Composer holds, then tells it whether that went through.
    fn write(&mut self, draft: ComposerDraft, composer: Entity<Composer>, cx: &mut Context<Self>) {
        let done = {
            let composer = composer.downgrade();
            move |written: bool, cx: &mut App| {
                let _ = composer.update(cx, |composer, cx| composer.finish(written, cx));
            }
        };
        let Load::Ready(knowledge) = self.state.read(cx).knowledge() else {
            return done(false, cx);
        };
        let knowledge = knowledge.clone();
        let ids: Vec<String> = self.contexts.iter().map(|id| id.to_string()).collect();
        match self.compose.clone() {
            Some(Compose::Rule(id)) => {
                let current = id
                    .as_ref()
                    .and_then(|id| knowledge.rules.iter().find(|rule| rule.id == *id));
                let text = rule_save(
                    current,
                    &knowledge.rules,
                    &draft.title,
                    &ids,
                    &knowledge.nodes,
                );
                let written = if current.is_some() {
                    "Rule updated."
                } else {
                    "Rule created."
                };
                self.state.update(cx, |state, cx| {
                    state.write_rule(text, summary(&draft.title), draft.why, written, done, cx);
                });
            }
            None => done(false, cx),
        }
    }

    /// Remove rule (`removeRule`): its block retired at once under "Remove rule: …", then the
    /// Composer closes and the page goes back.
    fn remove_rule(&mut self, composer: Entity<Composer>, cx: &mut Context<Self>) {
        let Some(Compose::Rule(Some(id))) = self.compose.clone() else {
            return;
        };
        let app = self.state.read(cx);
        if app.knowledge_pending() {
            return;
        }
        let Load::Ready(knowledge) = app.knowledge() else {
            return;
        };
        let Some(rule) = knowledge.rules.iter().find(|rule| rule.id == id).cloned() else {
            return;
        };
        let text = rule_text(rule_handle(&rule), "", None, true);
        let summary = summary(&format!("Remove rule: {}", rule.statement));
        let composer = composer.downgrade();
        self.state.update(cx, |state, cx| {
            state.write_rule(
                text,
                summary,
                String::new(),
                "Rule removed.",
                move |written, cx| {
                    if written {
                        let _ = composer.update(cx, |composer, cx| composer.close(cx));
                    }
                },
                cx,
            );
        });
    }

    /// The archive button: shows the archived docs, read afresh, or goes back to the list.
    /// The Docs tab's archive toggle: the archived docs, or back to the docs.
    pub(crate) fn toggle_archived(&mut self, cx: &mut Context<Self>) {
        self.show_archived = !self.show_archived;
        if self.show_archived {
            self.state
                .update(cx, |state, cx| state.load_archived_docs(cx));
        }
        cx.notify();
    }

    /// Restore writes at once; Delete permanently asks first.
    fn archive_picked(
        &mut self,
        _: Entity<ArchivedList>,
        event: &ArchivedListEvent,
        cx: &mut Context<Self>,
    ) {
        let (id, delete) = match event {
            ArchivedListEvent::Restore(id) => (id.to_string(), false),
            ArchivedListEvent::Delete(id) => (id.to_string(), true),
        };
        let Load::Ready(items) = self.state.read(cx).archived_docs() else {
            return;
        };
        let Some(title) = items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.title.clone())
        else {
            return;
        };
        if delete {
            self.delete_ask = Some((id, title));
            self.delete_focus_due = true;
            cx.notify();
        } else {
            self.state
                .update(cx, |state, cx| state.restore_doc(id, title, cx));
        }
    }

    /// The archived docs, the loading rows until they are read.
    fn archive(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let app = self.state.read(cx);
        let busy = app.knowledge_pending();
        match app.archived_docs() {
            Load::Ready(items) => {
                let items: Vec<ArchivedItem> = items
                    .iter()
                    .map(|item| ArchivedItem {
                        id: item.id.clone().into(),
                        title: item.title.clone().into(),
                        detail: item.tags.join(", ").into(),
                        archived_at: archive_date(&item.archived_at).into(),
                    })
                    .collect();
                self.archived.update(cx, |list, cx| {
                    list.set_items(items, cx);
                    list.set_busy(busy, cx);
                });
                self.archived.clone().into_any_element()
            }
            Load::Failed(message) => page_empty_with(
                Some("Could not read the archive"),
                MascotMood::Sad,
                Some(
                    div()
                        .text_size(px(theme.font_size_sm))
                        .text_color(theme.color_text_muted)
                        .child(SharedString::from(message.clone()))
                        .into_any_element(),
                ),
                cx,
            )
            .into_any_element(),
            _ => div()
                .size_full()
                .px(px(TABLE_INSET))
                .child(text_skeleton(4, 12.0, window, cx))
                .into_any_element(),
        }
    }

    /// Delete permanently's confirm (`requestConfirm`).
    fn delete_confirm(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (id, title) = self.delete_ask.clone()?;
        if std::mem::take(&mut self.delete_focus_due) {
            self.delete_focus.focus(window);
        }
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let close: polterware_ui::Handler = {
            let view = view.clone();
            Rc::new(move |_, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    this.delete_ask = None;
                    cx.notify();
                });
            })
        };
        let dialog = ConfirmDialog::new(
            "knowledge-delete",
            format!("Delete “{title}” permanently?"),
            "The item, its body, and its links to other records are removed from this project.",
        )
        .recovery("This cannot be undone.")
        .confirm_label("Delete permanently")
        .focus(&self.delete_focus)
        .on_cancel(close.clone())
        .on_confirm(Rc::new(move |_, cx: &mut App| {
            let _ = view.update(cx, |this, cx| {
                this.delete_ask = None;
                let (id, title) = (id.clone(), title.clone());
                this.state
                    .update(cx, |state, cx| state.delete_doc(id, title, cx));
                cx.notify();
            });
        }));
        Some(window_layer(modal_layer(1.0, true, close, dialog, &theme)))
    }

    /// Keeps the Composer on the Rule the route opens, and closes it when the route leaves it.
    fn follow_rule_page(
        &mut self,
        tab: KnowledgeTab,
        page: Option<&str>,
        knowledge: &Knowledge,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let wanted = (tab == KnowledgeTab::Rules)
            .then_some(page)
            .flatten()
            .filter(|id| knowledge.rules.iter().any(|rule| rule.id == *id));
        match (&self.compose, wanted) {
            (Some(Compose::Rule(Some(open))), Some(id)) if open == id => {}
            (_, Some(id)) => {
                self.open_composer(Compose::Rule(Some(id.to_string())), knowledge, window, cx);
            }
            (Some(Compose::Rule(Some(_))), None) => {
                self.compose = None;
                self.composer.update(cx, |composer, cx| composer.close(cx));
            }
            _ => {}
        }
    }

    fn filters_for(&self, tab: KnowledgeTab) -> &Entity<FilterPopover> {
        &self.filters[KnowledgeTab::ALL
            .iter()
            .position(|t| *t == tab)
            .unwrap_or(0)]
    }

    /// `changeTab`: a new tab starts with an empty search, no clauses, no sort, and its
    /// first row marked.
    fn follow_tab(&mut self, tab: KnowledgeTab, cx: &mut Context<Self>) {
        if tab == self.tab {
            return;
        }
        self.tab = tab;
        self.sort = None;
        self.active = None;
        self.show_archived = false;
        self.query.clear();
        self.search.update(cx, |input, cx| {
            input.set_text("", cx);
            input.set_placeholder(if tab == KnowledgeTab::Rules {
                "Search rules"
            } else {
                "Search"
            });
        });
        for popover in &self.filters {
            clear_page(popover, cx);
        }
    }

    /// The Tag field offers every tag the docs use.
    fn sync_tags(&mut self, docs: &[DocRow], cx: &mut Context<Self>) {
        let tags = doc_tags(docs);
        if tags == self.tags {
            return;
        }
        let options = tags
            .iter()
            .map(|tag| FilterOption::new(tag.clone(), tag.clone()))
            .collect();
        self.filters[2].update(cx, |popover, _| {
            set_page_fields(
                popover,
                vec![FilterField::new(
                    FilterSpec::multi("tags", "Tag", options),
                    Some(Icon::FileText),
                )],
            );
        });
        self.tags = tags;
    }

    fn needle(&self) -> String {
        self.query.trim().to_lowercase()
    }

    /// `filterLibraryRows` and the Status clauses, then the sort.
    fn visible_decisions(&self, rows: Vec<DecisionRow>, cx: &App) -> Vec<DecisionRow> {
        let needle = self.needle();
        let filters = self.filters[0].read(cx).filters();
        let rows: Vec<DecisionRow> = rows
            .into_iter()
            .filter(|row| {
                needle.is_empty()
                    || [
                        row.title.as_str(),
                        decision_state_label(row.state),
                        if row.pinned { "principle" } else { "" },
                    ]
                    .iter()
                    .any(|value| value.to_lowercase().contains(&needle))
            })
            .filter(|row| filters.matches(|field| (field == "status").then(|| vec![row.state])))
            .collect();
        match self.sort {
            Some((column, direction)) => sort_rows(&rows, direction, |row| match column {
                0 => text(&row.title),
                2 => text(decision_state_label(row.state)),
                3 => text(&row.updated),
                _ => SortValue::Missing,
            }),
            None => rows,
        }
    }

    /// `filterDocs` and the Tag clauses, then the sort.
    fn visible_docs(&self, rows: Vec<DocRow>, cx: &App) -> Vec<DocRow> {
        let needle = self.needle();
        let filters = self.filters[2].read(cx).filters();
        let rows: Vec<DocRow> = rows
            .into_iter()
            .filter(|row| {
                needle.is_empty()
                    || [&row.title, &row.summary, &row.owner]
                        .into_iter()
                        .chain(&row.tags)
                        .any(|value| value.to_lowercase().contains(&needle))
            })
            .filter(|row| {
                filters.matches(|field| {
                    (field == "tags").then(|| row.tags.iter().map(String::as_str).collect())
                })
            })
            .collect();
        match self.sort {
            Some((column, direction)) => sort_rows(&rows, direction, |row| match column {
                0 => text(&row.title),
                2 => text(&row.tags.join(",")),
                3 => text(&row.updated),
                _ => SortValue::Missing,
            }),
            None => rows,
        }
    }

    /// The Rules the search keeps (statement, slug, or Context), then the sort.
    fn visible_rules(&self, rules: &[Rule], nodes: &[StructureNode]) -> Vec<Rule> {
        let needle = self.needle();
        let rows: Vec<Rule> = rules
            .iter()
            .filter(|rule| {
                needle.is_empty()
                    || [
                        rule.statement.as_str(),
                        rule.slug.as_str(),
                        &about(&rule.scope_node_ids, nodes),
                    ]
                    .iter()
                    .any(|value| value.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect();
        match self.sort {
            Some((column, direction)) => sort_rows(&rows, direction, |rule| match column {
                0 => text(&rule.statement),
                _ => text(&about(&rule.scope_node_ids, nodes)),
            }),
            None => rows,
        }
    }

    /// The marked row: the one J, K, or a row chose while it is listed, else the first.
    fn marked(&self, ids: &[&str]) -> Option<usize> {
        self.active
            .as_deref()
            .and_then(|active| ids.iter().position(|id| *id == active))
            .or(if ids.is_empty() { None } else { Some(0) })
    }

    fn toolbar(
        &self,
        tab: KnowledgeTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let tab_state = self.state.clone();
        let tabs = KnowledgeTab::ALL
            .iter()
            .fold(Tabs::new("knowledge-tab"), |tabs, tab| {
                tabs.tab(tab.label())
            })
            .selected(
                KnowledgeTab::ALL
                    .iter()
                    .position(|t| *t == tab)
                    .unwrap_or(0),
            )
            .on_select(move |index, _, cx| {
                if let Some(tab) = KnowledgeTab::ALL.get(index).copied() {
                    tab_state.update(cx, |state, cx| {
                        state.set_knowledge_tab(tab, cx);
                        cx.notify();
                    });
                }
            });
        let archive = tab == KnowledgeTab::Docs && self.show_archived;
        let create = match tab {
            KnowledgeTab::Decisions => "New Decision",
            KnowledgeTab::Rules => "New rule",
            KnowledgeTab::Docs => "Add doc",
        };
        let actions = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.0))
            .when(tab == KnowledgeTab::Docs, |actions| {
                let view = cx.entity().downgrade();
                actions.child(
                    Button::new("knowledge-archived-docs", ButtonKind::Secondary)
                        .size(ButtonSize::Icon)
                        .icon(Icon::Archive)
                        .selected(self.show_archived)
                        .tooltip(if self.show_archived {
                            "Back to docs"
                        } else {
                            "Archived docs"
                        })
                        .on_click(move |_, cx| {
                            let _ = view.update(cx, |this, cx| this.toggle_archived(cx));
                        }),
                )
            })
            .when(!archive, |actions| {
                actions
                    .child(div().w(px(200.0)).child(self.search.clone()))
                    .child(self.filters_for(tab).clone())
            })
            .when(!archive, |actions| {
                actions.child(
                    Button::new("knowledge-create", ButtonKind::Primary)
                        .size(ButtonSize::Icon)
                        .icon(Icon::Plus)
                        .tooltip(create)
                        .on_click({
                            let view = cx.entity().downgrade();
                            move |window, cx| {
                                let _ =
                                    view.update(cx, |this, cx| this.compose_here(tab, window, cx));
                            }
                        }),
                )
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
            .child(actions);
        let filter_bar = FilterPopover::bar(self.filters_for(tab), window, cx).map(|bar| {
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

    fn decisions(&self, knowledge: &Knowledge, cx: &mut Context<Self>) -> AnyElement {
        let all = decision_rows(&knowledge.items);
        if all.is_empty() {
            return page_empty("No Decisions yet", MascotMood::Sleeping, cx).into_any_element();
        }
        let rows = self.visible_decisions(all, cx);
        if rows.is_empty() {
            return page_empty("No matches", MascotMood::Lost, cx).into_any_element();
        }
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        let marked = self.marked(&ids);
        let nodes = Rc::new(knowledge.nodes.clone());
        let rows = Rc::new(rows);
        let columns = vec![
            Column::new("Title").sortable().shrinks(),
            Column::new("Context"),
            Column::new("Status").sortable(),
            Column::new("Updated").sortable(),
        ];
        let cells = rows.clone();
        let cell_nodes = nodes.clone();
        let measured = rows.clone();
        let measure_nodes = nodes.clone();
        let table = DataTable::new(
            "knowledge-decisions",
            columns,
            rows.len(),
            move |row, _, cx| {
                let decision = &cells[row.index];
                let icon = if decision.pinned {
                    Icon::Scale
                } else {
                    Icon::FileText
                };
                vec![
                    title_cell(icon, &decision.title, cx),
                    chips(&context_labels(&decision.scope, &cell_nodes), cx),
                    chip_rounded(
                        decision_state_label(decision.state),
                        decision_state_tone(decision.state),
                        CHIP_RADIUS,
                        cx,
                    )
                    .into_any_element(),
                    muted_cell(decision.updated.clone(), cx),
                ]
            },
        )
        .measure(move |range, window, cx| {
            let theme = *Theme::global(cx);
            let medium = sans(&theme, FontWeight::MEDIUM);
            let normal = sans(&theme, FontWeight::NORMAL);
            let mut widest = [0.0_f32; 4];
            for decision in measured.get(range).unwrap_or_default() {
                let widths = [
                    title_width(&decision.title, window, cx),
                    chips_width(&context_labels(&decision.scope, &measure_nodes), window, cx),
                    CHIP_FRAME
                        + text_width(
                            decision_state_label(decision.state),
                            medium.clone(),
                            11.0,
                            window,
                        ),
                    text_width(&decision.updated, normal.clone(), 12.5, window),
                ];
                for (slot, width) in widest.iter_mut().zip(widths) {
                    *slot = slot.max(width);
                }
            }
            widest.to_vec()
        });
        // J and K open the next Decision, as `onActiveRowChange` activates it.
        self.table(
            table,
            rows.iter().map(|row| row.id.clone()).collect(),
            marked,
            true,
            cx,
        )
    }

    fn docs(&mut self, knowledge: &Knowledge, cx: &mut Context<Self>) -> AnyElement {
        let all = doc_rows(&knowledge.items);
        self.sync_tags(&all, cx);
        if all.is_empty() {
            return page_empty("No docs yet", MascotMood::Sleeping, cx).into_any_element();
        }
        let rows = self.visible_docs(all, cx);
        if rows.is_empty() {
            return page_empty("No matches", MascotMood::Lost, cx).into_any_element();
        }
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        let marked = self.marked(&ids);
        let nodes = Rc::new(knowledge.nodes.clone());
        let rows = Rc::new(rows);
        let columns = vec![
            Column::new("Title").sortable().shrinks(),
            Column::new("Context"),
            Column::new("Tags").sortable(),
            Column::new("Updated").sortable(),
        ];
        let cells = rows.clone();
        let cell_nodes = nodes.clone();
        let measured = rows.clone();
        let measure_nodes = nodes.clone();
        let table = DataTable::new("knowledge-docs", columns, rows.len(), move |row, _, cx| {
            let doc = &cells[row.index];
            vec![
                title_cell(Icon::FileText, &doc.title, cx),
                chips(&context_labels(&doc.scope, &cell_nodes), cx),
                chips(&doc.tags, cx),
                muted_cell(doc.updated.clone(), cx),
            ]
        })
        .measure(move |range, window, cx| {
            let theme = *Theme::global(cx);
            let normal = sans(&theme, FontWeight::NORMAL);
            let mut widest = [0.0_f32; 4];
            for doc in measured.get(range).unwrap_or_default() {
                let widths = [
                    title_width(&doc.title, window, cx),
                    chips_width(&context_labels(&doc.scope, &measure_nodes), window, cx),
                    chips_width(&doc.tags, window, cx),
                    text_width(&doc.updated, normal.clone(), 12.5, window),
                ];
                for (slot, width) in widest.iter_mut().zip(widths) {
                    *slot = slot.max(width);
                }
            }
            widest.to_vec()
        });
        // J and K only move the marker (`onFocusRow`).
        self.table(
            table,
            rows.iter().map(|row| row.id.clone()).collect(),
            marked,
            false,
            cx,
        )
    }

    fn rules(&self, knowledge: &Knowledge, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        if knowledge.rules.is_empty() {
            return page_empty_with(
                Some("No rules yet"),
                MascotMood::Sleeping,
                Some(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.color_text_muted)
                        .child("Agents read every Rule that holds for their work in the brief.")
                        .into_any_element(),
                ),
                cx,
            )
            .into_any_element();
        }
        let rows = Rc::new(self.visible_rules(&knowledge.rules, &knowledge.nodes));
        let nodes = Rc::new(knowledge.nodes.clone());
        let columns = vec![
            Column::new("Rule").sortable().shrinks(),
            Column::new("Context").sortable(),
        ];
        let cells = rows.clone();
        let cell_nodes = nodes.clone();
        let measured = rows.clone();
        let measure_nodes = nodes.clone();
        let table = DataTable::new("knowledge-rules", columns, rows.len(), move |row, _, cx| {
            let rule = &cells[row.index];
            let statement = if rule.statement.is_empty() {
                &rule.id
            } else {
                &rule.statement
            };
            vec![
                title_cell(Icon::ShieldCheck, statement, cx),
                chips(&context_labels(&rule.scope_node_ids, &cell_nodes), cx),
            ]
        })
        .measure(move |range, window, cx| {
            let mut widest = [0.0_f32; 2];
            for rule in measured.get(range).unwrap_or_default() {
                let widths = [
                    title_width(&rule.statement, window, cx),
                    chips_width(
                        &context_labels(&rule.scope_node_ids, &measure_nodes),
                        window,
                        cx,
                    ),
                ];
                for (slot, width) in widest.iter_mut().zip(widths) {
                    *slot = slot.max(width);
                }
            }
            widest.to_vec()
        });
        // A Rule's row marks nothing until one is opened, then the open one (`activeRowId`
        // follows the opened Rule).
        let open = match self.state.read(cx).knowledge_route() {
            Some((KnowledgeTab::Rules, Some(page))) => Some(page.to_string()),
            _ => None,
        };
        let marked = open
            .or_else(|| self.active.clone())
            .and_then(|id| rows.iter().position(|rule| rule.id == id));
        let ids = rows.iter().map(|rule| rule.id.clone()).collect();
        self.table(table, ids, marked, true, cx)
    }

    /// The shared table setup: default-density rows, the gutters inside the scroll, the widths
    /// measured over every row and frozen once, and the sort; a row opens its record and
    /// keeps the marker. `moves_open`: J and K open the next row too, else they only mark it.
    fn table(
        &self,
        table: DataTable,
        ids: Vec<String>,
        marked: Option<usize>,
        moves_open: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let view = cx.entity();
        let sorted = view.clone();
        let opened = Rc::new(ids);
        let open_state = self.state.clone();
        let open_view = view.clone();
        let open_ids = opened.clone();
        let mut table = table
            .row_height(ROW_HEIGHT)
            .sort(self.sort, move |column, _, cx| {
                sorted.update(cx, |view, cx| {
                    view.sort = next_sort(view.sort, column);
                    cx.notify();
                });
            })
            .on_activate(move |index, _, cx| {
                if let Some(id) = open_ids.get(index).cloned() {
                    open_view.update(cx, |view, _| view.active = Some(id.clone()));
                    open_state.update(cx, |state, cx| {
                        state.open_knowledge_page(id, cx);
                        cx.notify();
                    });
                }
            });
        if marked.is_some() {
            table = table.active(marked);
        }
        if !moves_open {
            let move_ids = opened.clone();
            table = table.on_move(move |index, _, cx| {
                if let Some(id) = move_ids.get(index).cloned() {
                    view.update(cx, |view, cx| {
                        view.active = Some(id);
                        cx.notify();
                    });
                }
            });
        }
        div()
            .relative()
            .size_full()
            .child(table.gutters(TABLE_INSET).measure_every_row().freeze_once())
            .into_any_element()
    }
}

impl Render for KnowledgeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        if let Some(page) = core_status(&self.state, cx) {
            return div().size_full().child(page);
        }
        let app = self.state.read(cx);
        let Some((tab, page)) = app.knowledge_route() else {
            return div().size_full();
        };
        let page_open = page.is_some();
        let knowledge = match app.knowledge() {
            Load::Ready(knowledge) => Some(knowledge.clone()),
            Load::Failed(message) => {
                let message = message.clone();
                return div().size_full().child(page_empty_with(
                    Some("Could not read Knowledge"),
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
        if let Some(knowledge) = &knowledge {
            let page = page.map(str::to_string);
            self.follow_rule_page(tab, page.as_deref(), knowledge, window, cx);
        }
        if page_open && tab != KnowledgeTab::Rules {
            // The record's page takes the list's place, toolbar and all.
            return div().size_full().child(self.page.clone());
        }
        self.follow_tab(tab, cx);
        let index = KnowledgeTab::ALL
            .iter()
            .position(|t| *t == tab)
            .unwrap_or(0);
        self.scopes[index].sync(&self.filters[index], &self.state, cx);
        let toolbar = self.toolbar(tab, window, cx);
        let list = match &knowledge {
            None => div()
                .size_full()
                .px(px(TABLE_INSET))
                .child(text_skeleton(4, 12.0, window, cx))
                .into_any_element(),
            Some(_) if tab == KnowledgeTab::Docs && self.show_archived => self.archive(window, cx),
            Some(knowledge) => match tab {
                KnowledgeTab::Decisions => self.decisions(knowledge, cx),
                KnowledgeTab::Rules => self.rules(knowledge, cx),
                KnowledgeTab::Docs => self.docs(knowledge, cx),
            },
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(div().flex_1().min_h(px(0.0)).child(list))
            .child(self.composer.clone())
            .children(self.delete_confirm(window, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, kind: &str, status: &str, scope: &[&str]) -> KnowledgeItem {
        KnowledgeItem {
            id: id.into(),
            title: format!("Title {id}"),
            kind: kind.into(),
            status: status.into(),
            scope_node_ids: scope.iter().map(|s| s.to_string()).collect(),
            ..KnowledgeItem::default()
        }
    }

    fn node(id: &str, title: &str) -> StructureNode {
        StructureNode {
            id: id.into(),
            title: title.into(),
            ..StructureNode::default()
        }
    }

    #[test]
    fn principles_lead_the_decisions_in_the_core_order() {
        let items = [
            item("d1", "decision", "active", &["n1"]),
            item("k1", "doc", "", &[]),
            item("d2", "decision", "accepted", &[]),
            item("d3", "decision", "weird", &["n2"]),
            item("d4", "decision", "challenged", &[]),
        ];
        let rows = decision_rows(&items);
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, ["d2", "d4", "d1", "d3"]);
        assert_eq!(rows[0].state, "active");
        assert_eq!(rows[1].state, "challenged");
        assert_eq!(decision_state_label(rows[1].state), "Under review");
        assert_eq!(rows[3].state, "proposed");
        assert!(rows[0].pinned && !rows[2].pinned);
    }

    #[test]
    fn a_context_names_three_nodes_then_counts_the_rest() {
        let nodes = [
            node("n1", "API"),
            node("n2", "Web"),
            node("n3", "Sync"),
            node("n4", "Billing"),
        ];
        let ids = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(context_labels(&ids(&[]), &nodes), ["Whole project"]);
        // The tree's order, not the record's.
        assert_eq!(context_labels(&ids(&["n2", "n1"]), &nodes), ["API", "Web"]);
        assert_eq!(
            context_labels(&ids(&["n4", "n3", "n2", "n1"]), &nodes),
            ["API", "Web", "Sync", "+1"]
        );
        assert_eq!(about(&ids(&["n3", "n1"]), &nodes), "API, Sync");
        assert_eq!(about(&ids(&["gone"]), &nodes), "Whole project");
    }

    fn rule(id: &str, slug: &str, statement: &str, paths: &[&str]) -> Rule {
        Rule {
            id: id.into(),
            slug: slug.into(),
            statement: statement.into(),
            scope_paths: paths.iter().map(|path| path.to_string()).collect(),
            ..Rule::default()
        }
    }

    #[test]
    fn a_rule_block_names_its_handle_then_its_scope_and_a_removal() {
        assert_eq!(
            rule_text(
                "one-writer",
                "Keep  one\nwriter",
                Some(&["core".into(), "web".into()]),
                false
            ),
            "---\npolter: 1\n---\n\n#### rule one-writer: Keep one writer\nscope: core, web\n"
        );
        assert_eq!(
            rule_text("one-writer", "", Some(&[]), false),
            "---\npolter: 1\n---\n\n#### rule one-writer\nscope:\n"
        );
        assert_eq!(
            rule_text("r-1", "", None, true),
            "---\npolter: 1\n---\n\n#### rule r-1\nstatus: removed\n"
        );
    }

    #[test]
    fn a_new_rule_takes_a_free_slug_and_an_edit_names_its_scope_only_when_it_changed() {
        assert_eq!(
            slug_from_title("  The API returns only declared fields!  "),
            "the-api-returns-only-declared-fields"
        );
        assert_eq!(slug_from_title("¿?"), "card");
        assert_eq!(slug_from_title(&"a".repeat(60)).len(), 48);
        assert_eq!(
            unique_slug(&["keep-it", "keep-it-2"], "Keep it"),
            "keep-it-3"
        );
        let nodes = [node("n1", "Core"), node("n2", "Web")].map(|node| StructureNode {
            path: node.title.to_lowercase(),
            ..node
        });
        let rules = [rule("r1", "keep-it", "Keep it", &["web", "core"])];
        let ids = ["n1".to_string(), "n2".to_string()];
        // The same nodes in another order are no change.
        assert_eq!(
            rule_save(Some(&rules[0]), &rules, "Keep it", &ids, &nodes),
            "---\npolter: 1\n---\n\n#### rule keep-it: Keep it\n"
        );
        assert_eq!(
            rule_save(Some(&rules[0]), &rules, "Keep it", &ids[..1], &nodes),
            "---\npolter: 1\n---\n\n#### rule keep-it: Keep it\nscope: core\n"
        );
        // A new Rule for the whole project names no scope.
        assert_eq!(
            rule_save(None, &rules, "Keep it", &[], &nodes),
            "---\npolter: 1\n---\n\n#### rule keep-it-2: Keep it\n"
        );
        // One whose statement has no letters is named by its id.
        assert_eq!(rule_handle(&rule("r9", "", "", &[])), "r9");
    }

    #[test]
    fn docs_offer_each_tag_once_sorted() {
        let mut a = item("k1", "doc", "", &[]);
        a.tags = vec!["runbook".into(), "api".into()];
        let mut b = item("k2", "doc", "", &[]);
        b.tags = vec!["api".into(), "billing".into()];
        let docs = doc_rows(&[a, b, item("d1", "decision", "", &[])]);
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].owner, "Platform");
        assert_eq!(doc_tags(&docs), ["api", "billing", "runbook"]);
    }
}
