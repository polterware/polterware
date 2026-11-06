//! Structure (`StructureView` in `packages/features/architecture`, `structure.css`): the
//! project's domains and components as a compact tree on the left, and the selected node's
//! document beside it.
//!
//! The toolbar leads with the tree's toggle (a texture `SidebarLeftIcon`, selected while the
//! tree shows). The tree pane (`.pw-structure__tree`, 288px with its 8px inset) closes like a
//! document's rail (`createTreePane`): its width springs open on the slow tier and closes on
//! its quicker exit tween while the 280px card, kept whole, slides 16px left and fades. The
//! card (`Card`, 12px in, scrolling past the pane's height) holds the tree's search
//! (`Search structure`) and its filters (Level, and Kind once a component has one; F opens
//! them), then the [`TreeView`]: the project at the root, its domains, and their components,
//! each row led by its 20px level tile, a branch's Add child (a ghost `+`) showing while its
//! line is hovered. A search keeps the nodes it names with their whole branch, a filter only
//! the nodes that pass it, each with the ancestors that lead to it, and every branch of a match
//! stays open; the clauses show under the toolbar.
//!
//! The document (`NodeDocument`, on the 654px column 48px from the top) leads with the hero: a
//! 36px level tile and the level (a picker where a node right under the project may switch
//! between domain and component), the title (24px, edited in place), and the summary (15px
//! secondary, edited in place). A component then shows its head lines, in groups set apart by
//! a rule (a 104px muted label column, then the values): Domain (a picker that moves it),
//! Kind and Technology as tokens (a typed technology adds one); Repository (a picker of the
//! project's checkouts and Choose folder…, with the GitHub button) and Folder; and Deploys as
//! tokens that open the deploy Composer, lead to the platform, and show the latest check, with
//! Check deploys, a danger chip per failed deploy, and a warning per platform whose CLI is not
//! connected. Then the record sections (Rules, Cards, Decisions, Docs; a 16px heading whose
//! `+` links one more), each row opening its record and a direct link's × unlinking it, and the
//! node's Activity. With no node yet, the project's document is followed by the agent prompts.
//!
//! N, the tree's `+`, and Commander's Add structure node open the node Composer: a domain under
//! the project, or a component under the selected domain (or the selected component's), with
//! its Domain pill. Opening a component asks its platforms for the latest deploys. Every write
//! goes through the app's state.

mod records;

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, Focusable, FontWeight,
    IntoElement, PathPromptOptions, Render, SharedString, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::card::card;
use polterware_ui::chip::chip_rounded;
use polterware_ui::data_table::{Column, DataTable, primary_cell, sans};
use polterware_ui::filter::{FilterOption, FilterSpec, Filters};
use polterware_ui::filter_popover::{FilterField, FilterPopover, FiltersChanged};
use polterware_ui::floating_toolbar::{FloatingToolbar, floating_toolbar};
use polterware_ui::icon_tile::{IconTile, TileTone};
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::motion::{SpringTier, Tween};
use polterware_ui::patterns::{
    AnchorSlot, CollapsibleSection, Composer, ComposerEvent, ComposerSpec, MascotMood, PromptCard,
    Property, PropertyOption, PropertyPicker, PropertyPickerEvent, PropertyPills,
    PropertyPillsEvent, PropertyTokenEvent, TreeNode, page_empty_with, prompt_card_grid, tree_view,
    window_anchor,
};
use polterware_ui::service_mark::option_mark;
use polterware_ui::skeleton::text_skeleton;
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::{StatusTone, Theme};
use polterware_ui::toast::ToastTone;
use polterware_ui::window::window_drag::WindowDragRegion;

use crate::contracts::{KnowledgeItem, NodeDeployment, ProjectRepo, Rule, StructureNode, WorkCard};
use crate::state::{
    AppState, DeployInput, Load, NodeDraft, NodePatch, RecordKind, Structure, list_values,
};
use crate::views::knowledge::decision_state;
use crate::views::knowledge_page::{cites, document_activity};

/// The project's own row and document (`PROJECT_ID`).
const PROJECT: &str = "structure:project";
/// The tree card's width and the pane's inset before it (`TREE_PANE_WIDTH`).
const TREE_CARD: f32 = 280.0;
const TREE_INSET: f32 = 8.0;
/// The card's slide while the pane closes.
const TREE_SLIDE: f32 = 16.0;
/// The head lines' label column.
const LABEL_COLUMN: f32 = 104.0;
/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
/// The Repository picker's Choose folder… (`CHOOSE_FOLDER`).
const CHOOSE_FOLDER: &str = "\u{0}choose-folder";
/// The reason's placeholder.
const WHY: &str = "Optional. Recorded in Activity";

/// The integrations catalog's providers (`CATALOG_PROVIDERS`).
const PROVIDERS: [(&str, &str); 14] = [
    ("github", "GitHub"),
    ("gitlab", "GitLab"),
    ("aws", "AWS"),
    ("google-cloud", "Google Cloud"),
    ("azure", "Azure"),
    ("vercel", "Vercel"),
    ("supabase", "Supabase"),
    ("terraform", "Terraform"),
    ("docker", "Docker"),
    ("netlify", "Netlify"),
    ("cloudflare", "Cloudflare"),
    ("fly", "Fly.io"),
    ("railway", "Railway"),
    ("render", "Render"),
];

/// The canvas's component kinds, their palette labels, and their icons (`COMPONENTS`,
/// `componentIcons`).
const COMPONENT_KINDS: [(&str, &str, Icon); 42] = [
    ("frontend", "Web app", Icon::Globe),
    ("mobile", "Mobile app", Icon::Smartphone),
    ("desktop", "Desktop app", Icon::Monitor),
    ("cli", "CLI", Icon::SquareTerminal),
    ("service", "Service", Icon::Server),
    ("api", "API", Icon::Plug),
    ("gateway", "Gateway", Icon::Network),
    ("worker", "Worker", Icon::Cog),
    ("function", "Function", Icon::Zap),
    ("scheduler", "Scheduler", Icon::CalendarClock),
    ("database", "Database", Icon::Database),
    ("cache", "Cache", Icon::MemoryStick),
    ("queue", "Queue", Icon::ListOrdered),
    ("stream", "Event stream", Icon::Activity),
    ("storage", "Object storage", Icon::HardDrive),
    ("search", "Search index", Icon::Search),
    ("warehouse", "Warehouse", Icon::Warehouse),
    ("auth", "Identity", Icon::KeyRound),
    ("cdn", "CDN", Icon::Globe2),
    ("cluster", "Cluster", Icon::Cpu),
    ("monitoring", "Monitoring", Icon::Gauge),
    ("external", "External API", Icon::Cloud),
    ("llm", "AI model", Icon::AiBrain01Icon),
    ("payment", "Payments", Icon::CreditCard),
    ("email", "Email", Icon::Inbox),
    ("user", "User", Icon::User),
    ("team", "Team", Icon::AddTeam),
    ("agent", "Agent", Icon::RoboticIcon),
    ("mcp", "MCP server", Icon::McpServerIcon),
    ("vector", "Vector index", Icon::AiSearchIcon),
    ("secrets", "Secrets", Icon::SquareLock02Icon),
    ("firewall", "Firewall", Icon::FirewallIcon),
    ("kms", "Key management", Icon::Key01Icon),
    ("logs", "Logs", Icon::ScrollText),
    ("tracing", "Tracing", Icon::Route),
    ("alerting", "Alerting", Icon::Bell),
    ("pipeline", "Pipeline", Icon::Workflow),
    ("ci", "CI/CD", Icon::InfinityIcon),
    ("webhook", "Webhook", Icon::Webhook),
    ("config", "Config", Icon::SlidersHorizontalIcon),
    ("spec", "Spec", Icon::FileText),
    ("dataset", "Dataset", Icon::GridTableIcon),
];

/// `STRUCTURE_AGENT_PROMPTS`' shared opening and closing.
const PREAMBLE: &str = "Run `polterware format` once to learn the Polter grammar and `polterware status` to read the current project state.";
const CLOSING: &str = "Write it as Polter text with a clear summary and rationale, check it with `polterware apply --dry-run`, then apply it with `polterware apply`. Only add what you can back with the code.";

/// `STRUCTURE_AGENT_PROMPTS`: the title, the description, the tile, the level its icon shows,
/// and the request between the preamble and the closing.
const PROMPTS: [(&str, &str, TileTone, &str, &str); 2] = [
    (
        "Map the project",
        "Propose domains and components from the repo.",
        TileTone::Architecture,
        "domain",
        "Read this repository and propose its Structure: the domains under the project and the components that make them up (`# path: Title` with `level:` and `under:`; a domain has no `under:`). Link each component to this clone with `dir:` and point its `path:` at the folder that holds it, and add a one-line intent. Check `polterware query structure` first and do not repeat a node that exists.",
    ),
    (
        "Register deploys",
        "Link components to where they run per environment.",
        TileTone::Infrastructure,
        "component",
        "Read the deployment config of this project (CI workflows, Dockerfiles, platform config files) and the existing Structure. For each component that is deployed, set `deploys:` as `env=provider:resource` using the project's environments. Leave components without a deploy target alone.",
    ),
];

/// `KindIcon`: the project's house, a domain's boxes, a component's block.
fn kind_icon(level: &str) -> Icon {
    match level {
        "project" => Icon::House,
        "component" => Icon::Blockchain01Icon,
        _ => Icon::Boxes,
    }
}

/// `kindLabel`.
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// `componentKindLabel`: the palette label, or the kind as written.
fn component_label(kind: &str) -> String {
    COMPONENT_KINDS
        .iter()
        .find(|(id, ..)| *id == kind)
        .map_or_else(|| kind.to_string(), |(_, label, _)| label.to_string())
}

fn provider_name(id: &str) -> String {
    PROVIDERS
        .iter()
        .find(|(provider, _)| *provider == id)
        .map_or_else(|| id.to_string(), |(_, name)| name.to_string())
}

/// `deployLabel`: the platform, then the environment.
fn deploy_label(deploy: &NodeDeployment) -> String {
    let environment = if deploy.environment.is_empty() {
        &deploy.environment_id
    } else {
        &deploy.environment
    };
    format!("{} · {environment}", provider_name(&deploy.provider))
}

/// `DEPLOY_STATES`: a deploy's latest check in plain words.
fn deploy_state(state: &str) -> String {
    match state {
        "ok" => "Live".into(),
        "building" => "Building".into(),
        "failed" => "Failed".into(),
        "stopped" => "Stopped".into(),
        "unknown" => "Unknown".into(),
        other => other.to_string(),
    }
}

/// `repositoryLabel`: owner/repo for a GitHub remote, else its name.
fn repository_label(repo: &ProjectRepo) -> String {
    repo.remote
        .strip_prefix("github.com/")
        .map_or_else(|| repo.name.clone(), str::to_string)
}

/// One thing that cites a node (`StructureRecord`): a card by its slug, a decision, a Rule, or
/// a doc, with a muted caption such as its stage; `direct` is a link the node owns, which its
/// document can remove.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub id: String,
    pub title: String,
    pub caption: String,
    pub direct: bool,
}

/// What cites a node, by kind (`StructureRecords`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Records {
    pub rules: Vec<Record>,
    pub cards: Vec<Record>,
    pub decisions: Vec<Record>,
    pub docs: Vec<Record>,
}

impl Records {
    pub fn of(&self, kind: RecordKind) -> &[Record] {
        match kind {
            RecordKind::Rule => &self.rules,
            RecordKind::Card => &self.cards,
            RecordKind::Decision => &self.decisions,
            RecordKind::Doc => &self.docs,
        }
    }
}

/// A record section's title and the icon its rows lead with (`RECORD_SECTIONS`).
fn section_of(kind: RecordKind) -> (&'static str, Icon) {
    match kind {
        RecordKind::Rule => ("Rules", Icon::ShieldCheck),
        RecordKind::Card => ("Cards", Icon::FolderKanban),
        RecordKind::Decision => ("Decisions", Icon::Scale),
        RecordKind::Doc => ("Docs", Icon::FileText),
    }
}

fn card_record(card: &WorkCard) -> Record {
    Record {
        id: card.slug.clone(),
        title: card.title.clone(),
        caption: capitalize(&card.stage),
        direct: false,
    }
}

fn decision_record(item: &KnowledgeItem) -> Record {
    Record {
        id: item.id.clone(),
        title: item.title.clone(),
        caption: capitalize(decision_state(&item.status)),
        direct: false,
    }
}

fn rule_record(rule: &Rule, caption: &str) -> Record {
    Record {
        id: rule.id.clone(),
        title: if rule.statement.is_empty() {
            rule.slug.clone()
        } else {
            rule.statement.clone()
        },
        caption: caption.to_string(),
        direct: false,
    }
}

fn doc_record(item: &KnowledgeItem) -> Record {
    Record {
        id: item.id.clone(),
        title: item.title.clone(),
        caption: item.tags.join(", "),
        direct: false,
    }
}

/// Everything in the project, for the project's own document.
pub fn project_records(structure: &Structure) -> Records {
    Records {
        rules: structure
            .rules
            .iter()
            .map(|rule| rule_record(rule, ""))
            .collect(),
        cards: structure.cards.iter().map(card_record).collect(),
        decisions: structure
            .items
            .iter()
            .filter(|item| item.kind == "decision")
            .map(decision_record)
            .collect(),
        docs: structure
            .items
            .iter()
            .filter(|item| item.kind == "doc")
            .map(doc_record)
            .collect(),
    }
}

/// `structureRecords`' node: the cards under it, the decisions and docs scoped to it (and the
/// decisions it governs), and every Rule that holds there: its own (direct), an ancestor's
/// (`From …`), or the whole project's.
pub fn node_records(structure: &Structure, node: &StructureNode) -> Records {
    let mut ancestors: Vec<&StructureNode> = Vec::new();
    let mut parent = structure
        .nodes
        .iter()
        .find(|other| other.id == node.parent_id);
    while let Some(next) = parent {
        if ancestors.iter().any(|seen| seen.id == next.id) {
            break;
        }
        ancestors.push(next);
        parent = structure
            .nodes
            .iter()
            .find(|other| other.id == next.parent_id);
    }
    let scoped = |ids: &[String]| ids.contains(&node.id);
    Records {
        rules: structure
            .rules
            .iter()
            .filter_map(|rule| {
                if scoped(&rule.scope_node_ids) {
                    return Some(Record {
                        direct: true,
                        ..rule_record(rule, "")
                    });
                }
                if rule.scope_node_ids.is_empty() {
                    return Some(rule_record(rule, "Whole project"));
                }
                ancestors
                    .iter()
                    .find(|ancestor| rule.scope_node_ids.contains(&ancestor.id))
                    .map(|from| rule_record(rule, &format!("From {}", from.title)))
            })
            .collect(),
        cards: structure
            .cards
            .iter()
            .filter(|card| card.under == node.path)
            .map(|card| Record {
                direct: true,
                ..card_record(card)
            })
            .collect(),
        decisions: structure
            .items
            .iter()
            .filter(|item| item.kind == "decision")
            .filter(|item| {
                scoped(&item.scope_node_ids)
                    || node
                        .decisions
                        .iter()
                        .any(|reference| cites(reference, &item.id, &item.title))
            })
            .map(|item| Record {
                direct: scoped(&item.scope_node_ids),
                ..decision_record(item)
            })
            .collect(),
        docs: structure
            .items
            .iter()
            .filter(|item| item.kind == "doc" && scoped(&item.scope_node_ids))
            .map(|item| Record {
                direct: true,
                ..doc_record(item)
            })
            .collect(),
    }
}

/// A node and the nodes under it (`StructureTreeItem`).
#[derive(Clone, Debug, PartialEq)]
pub struct TreeItem {
    pub node: StructureNode,
    pub children: Vec<TreeItem>,
}

/// `buildStructureTree`: the nodes in their order under their parents, orphans and a cycle
/// shown once at the root. A query or `matches` keeps the nodes that pass both with the
/// ancestors that lead to them; a text match keeps its whole branch, a filter only what
/// passes it.
pub fn build_tree(
    nodes: &[StructureNode],
    query: &str,
    matches: Option<&dyn Fn(&StructureNode) -> bool>,
) -> Vec<TreeItem> {
    let ids: Vec<&str> = nodes.iter().map(|node| node.id.as_str()).collect();
    let mut children: HashMap<&str, Vec<&StructureNode>> = HashMap::new();
    for node in nodes {
        let parent = if !node.parent_id.is_empty() && ids.contains(&node.parent_id.as_str()) {
            node.parent_id.as_str()
        } else {
            ""
        };
        children.entry(parent).or_default().push(node);
    }
    fn build<'a>(
        node: &'a StructureNode,
        children: &HashMap<&str, Vec<&'a StructureNode>>,
        seen: &mut Vec<&'a str>,
    ) -> Option<TreeItem> {
        if seen.contains(&node.id.as_str()) {
            return None;
        }
        seen.push(&node.id);
        let kids = children
            .get(node.id.as_str())
            .map(|kids| {
                kids.iter()
                    .filter_map(|kid| build(kid, children, seen))
                    .collect()
            })
            .unwrap_or_default();
        Some(TreeItem {
            node: node.clone(),
            children: kids,
        })
    }
    let mut seen: Vec<&str> = Vec::new();
    let mut roots: Vec<TreeItem> = children
        .get("")
        .map(|top| {
            top.iter()
                .filter_map(|node| build(node, &children, &mut seen))
                .collect()
        })
        .unwrap_or_default();
    for node in nodes {
        if let Some(item) = build(node, &children, &mut seen) {
            roots.push(item);
        }
    }
    let term = query.trim().to_lowercase();
    if term.is_empty() && matches.is_none() {
        return roots;
    }
    let passes = |node: &StructureNode| {
        let text = format!(
            "{} {} {} {} {}",
            node.title, node.level, node.path, node.kind, node.tech
        )
        .to_lowercase();
        (term.is_empty() || text.contains(&term)) && matches.is_none_or(|matches| matches(node))
    };
    fn filter(
        items: Vec<TreeItem>,
        passes: &dyn Fn(&StructureNode) -> bool,
        strict: bool,
    ) -> Vec<TreeItem> {
        items
            .into_iter()
            .filter_map(|item| {
                let descendants = filter(item.children.clone(), passes, strict);
                if passes(&item.node) {
                    Some(TreeItem {
                        children: if strict { descendants } else { item.children },
                        node: item.node,
                    })
                } else if !descendants.is_empty() {
                    Some(TreeItem {
                        node: item.node,
                        children: descendants,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
    filter(roots, &passes, matches.is_some())
}

/// `flattenStructureTree`.
pub fn flatten(items: &[TreeItem]) -> Vec<&StructureNode> {
    items
        .iter()
        .flat_map(|item| std::iter::once(&item.node).chain(flatten(&item.children)))
        .collect()
}

/// Which picker of the document is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Pick {
    Level,
    Domain,
    Repository,
    Link(RecordKind),
}

/// The deploy open in its Composer: an index into the component's deploys, or none for a new
/// one.
#[derive(Clone, Debug, PartialEq, Eq)]
struct DeployDraft {
    node_id: String,
    index: Option<usize>,
    environment_id: String,
    provider: String,
    resource: String,
    target_id: String,
}

/// A head field edited in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    Title,
    Summary,
    Folder,
}

pub struct StructureView {
    state: Entity<AppState>,
    /// The tree pane shows, and how far open it is.
    tree_open: bool,
    toolbar_layout: Entity<FloatingToolbar>,
    pane: Tween,
    /// The open branches while nothing filters (`structure.expanded`).
    expanded: Vec<SharedString>,
    search: Entity<TextInput>,
    query: String,
    filters: Entity<FilterPopover>,
    /// The kinds the Kind filter offers.
    filter_kinds: Option<Vec<String>>,
    /// The node the head fields were set for, and what each was set to.
    doc: Option<String>,
    title: Entity<TextInput>,
    summary: Entity<TextInput>,
    folder: Entity<TextInput>,
    shown: HashMap<&'static str, String>,
    kinds: Entity<PropertyPills>,
    techs: Entity<PropertyPills>,
    deploys: Entity<PropertyPills>,
    /// What the head tokens were set from.
    pills_for: Option<String>,
    picker: Entity<PropertyPicker>,
    picking: Option<Pick>,
    anchors: HashMap<Pick, AnchorSlot>,
    sections: Vec<(RecordKind, Entity<CollapsibleSection>)>,
    composer: Entity<Composer>,
    record_editor: Entity<Composer>,
    editing: Option<(RecordKind, String)>,
    record_card: super::compose::CardProps,
    record_contexts: Vec<String>,
    /// The node a new one goes under: the project takes a domain, a domain a component.
    create_under: String,
    deploy_composer: Entity<Composer>,
    deploy: Option<DeployDraft>,
    /// The selection the ancestors were opened for, and the component whose deploys were
    /// checked when it opened.
    followed: Option<String>,
    checked: Option<String>,
}

impl StructureView {
    /// The structure filters, which F opens.
    pub fn filters(&self) -> &Entity<FilterPopover> {
        &self.filters
    }

    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let toolbar_layout =
            cx.new(|cx| FloatingToolbar::new(Theme::global(cx).page_header_height));
        cx.observe(&toolbar_layout, |_, _, cx| cx.notify()).detach();
        let search = cx.new(|cx| TextInput::new(cx).placeholder("Search structure").pill());
        cx.observe(&search, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.query {
                this.query = text;
                cx.notify();
            }
        })
        .detach();
        let filters =
            cx.new(|cx| FilterPopover::new("structure-filters", "Filter structure", true, cx));
        cx.subscribe(&filters, |_, _, _: &FiltersChanged, cx| cx.notify())
            .detach();
        cx.observe(&filters, |_, _, cx| cx.notify()).detach();
        let field = |placeholder: &'static str,
                     build: fn(TextInput) -> TextInput,
                     which: Field,
                     window: &mut Window,
                     cx: &mut Context<Self>| {
            let submit = cx.weak_entity();
            let cancel = cx.weak_entity();
            let input = cx.new(|cx| {
                build(TextInput::new(cx).placeholder(placeholder))
                    // Enter commits as blur does; Escape puts the stored value back first.
                    .on_submit(move |_, window, cx| {
                        let _ = submit.update(cx, |_, _| {});
                        window.blur();
                    })
                    .on_cancel(move |window, cx| {
                        let _ = cancel.update(cx, |this, cx| this.restore(which, cx));
                        window.blur();
                    })
            });
            let focus = input.read(cx).focus_handle(cx);
            cx.on_blur(&focus, window, move |this, _, cx| this.commit(which, cx))
                .detach();
            input
        };
        let title = field(
            "Untitled",
            |input| input.prose(24.0, 32.0, FontWeight::SEMIBOLD).wrap(),
            Field::Title,
            window,
            cx,
        );
        let summary = field(
            "Add a short summary…",
            |input| {
                input
                    .prose(15.0, 24.0, FontWeight::NORMAL)
                    .secondary()
                    .chat()
            },
            Field::Summary,
            window,
            cx,
        );
        let folder = field(
            "Folder",
            |input| input.prose(12.0, 18.0, FontWeight::NORMAL).mono(),
            Field::Folder,
            window,
            cx,
        );
        let pills = |cx: &mut Context<Self>| {
            let pills = cx.new(PropertyPills::new);
            cx.observe(&pills, |_, _, cx| cx.notify()).detach();
            cx.subscribe(&pills, Self::token_changed).detach();
            pills
        };
        let (kinds, techs, deploys) = (pills(cx), pills(cx), pills(cx));
        cx.subscribe_in(&deploys, window, Self::token_asked)
            .detach();
        let picker = cx.new(PropertyPicker::new);
        cx.observe(&picker, |_, _, cx| cx.notify()).detach();
        cx.subscribe_in(&picker, window, Self::picked).detach();
        let sections = RecordKind::ALL
            .into_iter()
            .map(|kind| {
                let section = cx.new(|cx| {
                    let mut section = CollapsibleSection::new(section_of(kind).0, cx);
                    section.set_large_title(true);
                    section
                });
                cx.observe(&section, |_, _, cx| cx.notify()).detach();
                (kind, section)
            })
            .collect();
        let composer = cx.new(Composer::new);
        cx.observe(&composer, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&composer, Self::node_composed).detach();
        let record_editor = cx.new(Composer::new);
        cx.observe(&record_editor, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&record_editor, Self::record_composed).detach();
        let deploy_composer = cx.new(Composer::new);
        cx.observe(&deploy_composer, |_, _, cx| cx.notify())
            .detach();
        cx.subscribe(&deploy_composer, Self::deploy_composed)
            .detach();
        cx.observe_in(&state, window, |this, state, window, cx| {
            if state.update(cx, |state, _| state.take_add_node_request()) {
                this.open_create(window, cx);
            }
            cx.notify();
        })
        .detach();
        let (tree_open, expanded) = crate::prefs::structure_tree(&crate::prefs::user_data_dir());
        Self {
            state,
            tree_open,
            toolbar_layout,
            pane: Tween::new(if tree_open { 1.0 } else { 0.0 }),
            expanded: expanded.into_iter().map(Into::into).collect(),
            search,
            query: String::new(),
            filters,
            filter_kinds: None,
            doc: None,
            title,
            summary,
            folder,
            shown: HashMap::new(),
            kinds,
            techs,
            deploys,
            pills_for: None,
            picker,
            picking: None,
            anchors: HashMap::new(),
            sections,
            composer,
            record_editor,
            editing: None,
            record_card: Default::default(),
            record_contexts: Vec::new(),
            create_under: PROJECT.into(),
            deploy_composer,
            deploy: None,
            followed: None,
            checked: None,
        }
    }

    fn structure<'a>(&self, cx: &'a App) -> Option<&'a Structure> {
        match self.state.read(cx).structure() {
            Load::Ready(structure) => Some(structure),
            _ => None,
        }
    }

    fn project_name(&self, cx: &App) -> String {
        self.state
            .read(cx)
            .project()
            .map(|project| project.name.clone())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Project".into())
    }

    /// The selected node, none for the project's document.
    fn selected(&self, cx: &App) -> Option<StructureNode> {
        let id = self.state.read(cx).structure_node()?.to_string();
        self.structure(cx)?
            .nodes
            .iter()
            .find(|node| node.id == id)
            .cloned()
    }

    /// The tree's toggle (`setTreeOpen`).
    fn toggle_tree(&mut self, cx: &mut Context<Self>) {
        self.tree_open = !self.tree_open;
        self.remember_tree();
        let tier = SpringTier::Slow;
        if self.tree_open {
            self.pane.animate_to(1.0, tier.duration(), tier.curve());
        } else {
            self.pane
                .animate_to(0.0, tier.exit_duration(), tier.exit_curve());
        }
        cx.notify();
    }

    fn remember_tree(&self) {
        let expanded: Vec<String> = self.expanded.iter().map(ToString::to_string).collect();
        let _ = crate::prefs::save_structure_tree(
            &crate::prefs::user_data_dir(),
            self.tree_open,
            &expanded,
        );
    }

    /// Selects a row: its document opens beside the tree.
    fn open(&mut self, id: &str, cx: &mut Context<Self>) {
        let id = (id != PROJECT).then(|| id.to_string());
        self.state
            .update(cx, |state, cx| state.select_structure_node(id, cx));
    }

    /// Opens the node Composer (`onOpenCreate`): from a component, the next one goes beside it,
    /// under its domain.
    pub fn open_create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(structure) = self.structure(cx) else {
            return;
        };
        let node = self.selected(cx);
        let domain = node.as_ref().and_then(|node| {
            if node.level == "domain" {
                Some(node.id.clone())
            } else {
                structure
                    .nodes
                    .iter()
                    .find(|other| other.id == node.parent_id && other.level == "domain")
                    .map(|domain| domain.id.clone())
            }
        });
        self.create_under = domain.unwrap_or_else(|| PROJECT.into());
        let component = self.create_under != PROJECT;
        let spec = ComposerSpec::new(
            if component {
                "Component name"
            } else {
                "Domain name"
            },
            if component {
                "Create component"
            } else {
                "Create domain"
            },
        )
        .crumbs({
            let mut crumbs = vec![self.project_name(cx)];
            if component {
                crumbs.push(
                    self.structure(cx)
                        .and_then(|s| s.nodes.iter().find(|n| n.id == self.create_under))
                        .map(|n| n.title.clone())
                        .unwrap_or_default(),
                );
            }
            crumbs.push(
                if component {
                    "New component"
                } else {
                    "New domain"
                }
                .into(),
            );
            crumbs
        })
        .description("What it owns…")
        .why(WHY);
        let properties = self.create_properties(cx);
        self.composer.update(cx, |composer, cx| {
            composer.open(spec, properties, window, cx)
        });
    }

    /// The node Composer's Domain pill, for a component.
    fn create_properties(&self, cx: &App) -> Vec<Property> {
        if self.create_under == PROJECT {
            return Vec::new();
        }
        let options = self
            .structure(cx)
            .map(|structure| {
                structure
                    .nodes
                    .iter()
                    .filter(|node| node.level == "domain")
                    .map(|node| {
                        PropertyOption::new(node.id.clone(), node.title.clone()).icon(Icon::Boxes)
                    })
                    .collect()
            })
            .unwrap_or_default();
        vec![
            Property::single("domain", "Domain", options, self.create_under.clone())
                .icon(Icon::Boxes),
        ]
    }

    /// `addChild`: selects the row, then opens the Composer under it.
    fn add_child(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.open(id, cx);
        self.open_create(window, cx);
    }

    fn node_composed(
        &mut self,
        composer: Entity<Composer>,
        event: &ComposerEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ComposerEvent::Submit(draft) => {
                let component = self.create_under != PROJECT;
                let node = NodeDraft {
                    title: draft.title.trim().to_string(),
                    intent: draft.description.trim().to_string(),
                    why: draft.why.clone(),
                    level: if component { "component" } else { "domain" }.into(),
                    parent_id: if component {
                        self.create_under.clone()
                    } else {
                        String::new()
                    },
                };
                let composer = composer.downgrade();
                self.state.update(cx, |state, cx| {
                    state.create_structure_node(
                        node,
                        move |written, cx| {
                            let _ =
                                composer.update(cx, |composer, cx| composer.finish(written, cx));
                        },
                        cx,
                    )
                });
            }
            ComposerEvent::Property { id, values } if id.as_ref() == "domain" => {
                if let Some(value) = values.first() {
                    self.create_under = value.to_string();
                }
                let properties = self.create_properties(cx);
                let allowed = self
                    .deploy
                    .as_ref()
                    .is_some_and(|d| !d.environment_id.is_empty() && !d.provider.is_empty());
                composer.update(cx, |composer, cx| {
                    composer.set_can_submit(allowed, cx);
                    composer.set_properties(properties, cx);
                });
            }
            _ => {}
        }
    }

    /// What the deploy's Composer's pills hold: the environment, the platform, and the target
    /// (None, the targets the checkout declares, the account's, and a linked one the list no
    /// longer shows).
    fn deploy_properties(&self, cx: &App) -> Vec<Property> {
        let Some(draft) = &self.deploy else {
            return Vec::new();
        };
        let environments = self
            .structure(cx)
            .map(|structure| {
                structure
                    .environments
                    .iter()
                    .map(|environment| {
                        PropertyOption::new(environment.id.clone(), environment.name.clone())
                            .icon(Icon::Layers)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let providers = PROVIDERS
            .iter()
            .map(|(id, name)| {
                PropertyOption::new(*id, *name).mark(move |size, _, cx| option_mark(id, size, cx))
            })
            .collect();
        let listed = self
            .state
            .read(cx)
            .deploy_targets()
            .filter(|listed| listed.node_id == draft.node_id && listed.provider == draft.provider)
            .map(|listed| listed.targets.clone());
        let (targets, hint) = match &listed {
            Some(Load::Ready((targets, message))) => (targets.clone(), message.clone()),
            _ => (Vec::new(), "Loading the platform's projects…".to_string()),
        };
        let mut none = PropertyOption::new("", "None").icon(Icon::XIcon);
        if !hint.is_empty() {
            none = none.hint(hint);
        }
        let mut options = vec![none];
        for target in &targets {
            let key = if target.id.is_empty() {
                format!("name:{}", target.name)
            } else {
                target.id.clone()
            };
            let mut option = PropertyOption::new(key, target.name.clone())
                .mark({
                    let provider = draft.provider.clone();
                    move |size, _, cx| option_mark(&provider, size, cx)
                })
                .group(if target.detected {
                    "In this repository"
                } else {
                    "Account"
                });
            if !target.kind.is_empty() {
                option = option.hint(target.kind.clone());
            }
            options.push(option);
        }
        if !draft.target_id.is_empty() && !targets.iter().any(|target| target.id == draft.target_id)
        {
            let label = if draft.resource.is_empty() {
                draft.target_id.clone()
            } else {
                draft.resource.clone()
            };
            options.push(
                PropertyOption::new(draft.target_id.clone(), label)
                    .mark({
                        let provider = draft.provider.clone();
                        move |size, _, cx| option_mark(&provider, size, cx)
                    })
                    .group("Linked")
                    .hint(draft.target_id.clone()),
            );
        }
        vec![
            Property::single(
                "environment",
                "Environment",
                environments,
                draft.environment_id.clone(),
            )
            .icon(Icon::Layers),
            Property::single("provider", "Platform", providers, draft.provider.clone())
                .icon(Icon::Cloud),
            Property::single("target", "Target", options, draft.target_id.clone())
                .icon(Icon::Cloud)
                .creatable()
                .picker_width(320.0),
        ]
    }

    /// `openDeploy`: a deploy in its Composer, or a new one on the first environment and the
    /// platform the component already uses.
    fn open_deploy(&mut self, index: Option<usize>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(node) = self.selected(cx) else {
            return;
        };
        let deploy = index.and_then(|index| node.deploys.get(index));
        let draft = match deploy {
            Some(deploy) => DeployDraft {
                node_id: node.id.clone(),
                index,
                environment_id: deploy.environment_id.clone(),
                provider: deploy.provider.clone(),
                resource: deploy.resource.clone(),
                target_id: deploy.target_id.clone(),
            },
            None => DeployDraft {
                node_id: node.id.clone(),
                index: None,
                environment_id: self
                    .structure(cx)
                    .and_then(|structure| structure.environments.first())
                    .map(|environment| environment.id.clone())
                    .unwrap_or_default(),
                provider: node.deploys.first().map_or_else(
                    || PROVIDERS[0].0.to_string(),
                    |deploy| deploy.provider.clone(),
                ),
                resource: String::new(),
                target_id: String::new(),
            },
        };
        let mut spec = ComposerSpec::new(
            "Resource name",
            if draft.index.is_none() {
                "Add deploy"
            } else {
                "Save"
            },
        )
        .crumbs([
            node.title.clone(),
            if draft.index.is_none() {
                "New deploy"
            } else {
                "Deploy"
            }
            .into(),
        ])
        .can_submit(!draft.environment_id.is_empty() && !draft.provider.is_empty())
        .initial(polterware_ui::patterns::ComposerDraft {
            title: draft.resource.clone(),
            ..Default::default()
        });
        if draft.index.is_some() {
            spec = spec.edit();
        }
        let (node_id, provider) = (draft.node_id.clone(), draft.provider.clone());
        self.deploy = Some(draft);
        self.state.update(cx, |state, cx| {
            state.load_deploy_targets(node_id, provider, cx)
        });
        let properties = self.deploy_properties(cx);
        self.deploy_composer.update(cx, |composer, cx| {
            composer.open(spec, properties, window, cx)
        });
    }

    /// The component's deploys as the document writes them.
    fn deploys_of(node: &StructureNode) -> Vec<DeployInput> {
        node.deploys
            .iter()
            .map(|deploy| DeployInput {
                environment_id: deploy.environment_id.clone(),
                provider: deploy.provider.clone(),
                resource: deploy.resource.clone(),
                target_id: deploy.target_id.clone(),
            })
            .collect()
    }

    fn deploy_composed(
        &mut self,
        composer: Entity<Composer>,
        event: &ComposerEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ComposerEvent::Submit(form) => {
                let Some(draft) = self.deploy.clone() else {
                    composer.update(cx, |composer, cx| composer.finish(false, cx));
                    return;
                };
                if draft.environment_id.is_empty() || draft.provider.is_empty() {
                    self.state.update(cx, |state, _| {
                        state.toast("Pick an environment and a platform.", true)
                    });
                    composer.update(cx, |composer, cx| composer.finish(false, cx));
                    return;
                }
                let Some(node) = self
                    .structure(cx)
                    .and_then(|structure| {
                        structure.nodes.iter().find(|node| node.id == draft.node_id)
                    })
                    .cloned()
                else {
                    composer.update(cx, |composer, cx| composer.finish(false, cx));
                    return;
                };
                let deploy = DeployInput {
                    environment_id: draft.environment_id.clone(),
                    provider: draft.provider.clone(),
                    resource: form.title.trim().to_string(),
                    target_id: draft.target_id.clone(),
                };
                let mut deploys = Self::deploys_of(&node);
                match draft.index {
                    Some(index) if index < deploys.len() => deploys[index] = deploy,
                    _ => deploys.push(deploy),
                }
                let composer = composer.downgrade();
                self.state.update(cx, |state, cx| {
                    state.save_deploys(
                        &node.id,
                        deploys,
                        move |written, cx| {
                            let _ =
                                composer.update(cx, |composer, cx| composer.finish(written, cx));
                        },
                        cx,
                    )
                });
            }
            ComposerEvent::Property { id, values } => {
                let value = values
                    .first()
                    .map(|value| value.to_string())
                    .unwrap_or_default();
                let targets = self
                    .state
                    .read(cx)
                    .deploy_targets()
                    .and_then(|listed| match &listed.targets {
                        Load::Ready((targets, _)) => Some(targets.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let Some(draft) = self.deploy.as_mut() else {
                    return;
                };
                match id.as_ref() {
                    "environment" => draft.environment_id = value,
                    "provider" => {
                        if draft.provider != value {
                            draft.target_id.clear();
                        }
                        draft.provider = value;
                        let (node, provider) = (draft.node_id.clone(), draft.provider.clone());
                        self.state.update(cx, |state, cx| {
                            state.load_deploy_targets(node, provider, cx)
                        });
                    }
                    "target" => {
                        // `pickTarget`: a listed target links its id and name; a typed name or
                        // a target with no id is a resource name alone.
                        let target = targets.iter().find(|target| {
                            (!target.id.is_empty() && target.id == value)
                                || format!("name:{}", target.name) == value
                        });
                        match target {
                            Some(target) => {
                                draft.target_id = target.id.clone();
                                draft.resource = target.name.clone();
                            }
                            None if value.is_empty() => draft.target_id.clear(),
                            None => {
                                let linked = draft.target_id == value;
                                if !linked {
                                    draft.target_id.clear();
                                    draft.resource =
                                        value.strip_prefix("name:").unwrap_or(&value).to_string();
                                }
                            }
                        }
                    }
                    _ => {}
                }
                let properties = self.deploy_properties(cx);
                composer.update(cx, |composer, cx| composer.set_properties(properties, cx));
            }
            ComposerEvent::Closed => {
                self.deploy = None;
                self.state
                    .update(cx, |state, _| state.clear_deploy_targets());
            }
            ComposerEvent::Action => {}
        }
    }

    /// A head token list changed: Kind, Technology, or a removed deploy.
    fn token_changed(
        &mut self,
        pills: Entity<PropertyPills>,
        event: &PropertyPillsEvent,
        cx: &mut Context<Self>,
    ) {
        let PropertyPillsEvent::Changed { id, values } = event;
        let Some(node) = self.selected(cx) else {
            return;
        };
        let joined = values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        match id.as_ref() {
            "kind" => self.patch(
                &node.id,
                NodePatch {
                    kind: Some(joined),
                    ..NodePatch::default()
                },
                cx,
            ),
            "tech" => self.patch(
                &node.id,
                NodePatch {
                    tech: Some(joined),
                    ..NodePatch::default()
                },
                cx,
            ),
            "deploys" if pills == self.deploys => {
                let kept: Vec<String> = values.iter().map(|value| value.to_string()).collect();
                if let Some(removed) =
                    (0..node.deploys.len()).find(|index| !kept.contains(&index.to_string()))
                {
                    let mut deploys = Self::deploys_of(&node);
                    deploys.remove(removed);
                    self.state.update(cx, |state, cx| {
                        state.save_deploys(&node.id, deploys, |_, _| {}, cx)
                    });
                }
            }
            _ => {}
        }
    }

    /// A deploy token asked to be edited, opened on its platform, or added.
    fn token_asked(
        &mut self,
        _: &Entity<PropertyPills>,
        event: &PropertyTokenEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(node) = self.selected(cx) else {
            return;
        };
        match event {
            PropertyTokenEvent::Edit { value, .. } => {
                let index = value.parse().ok();
                self.open_deploy(index, window, cx);
            }
            PropertyTokenEvent::Add { .. } => self.open_deploy(None, window, cx),
            PropertyTokenEvent::Open { value, .. } => {
                let deploy = value
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| node.deploys.get(index));
                if let Some(deploy) = deploy {
                    let url = if deploy.state == "failed" && !deploy.state_url.is_empty() {
                        &deploy.state_url
                    } else {
                        &deploy.url
                    };
                    if !url.is_empty() {
                        cx.open_url(url);
                    }
                }
            }
        }
    }

    fn patch(&mut self, id: &str, patch: NodePatch, cx: &mut Context<Self>) {
        self.state
            .update(cx, |state, cx| state.edit_structure_node(id, patch, cx));
    }

    /// The node the head fields were set for, none for the project's document. A field blurs
    /// after a press on another row has moved the selection, so its commit goes to this one.
    fn doc_node(&self, cx: &App) -> Option<StructureNode> {
        let id = self.doc.as_deref().filter(|id| *id != PROJECT)?;
        self.structure(cx)?
            .nodes
            .iter()
            .find(|node| node.id == id)
            .cloned()
    }

    /// The stored value a head field shows.
    fn stored(&self, field: Field, cx: &App) -> String {
        match (self.doc_node(cx), field) {
            (Some(node), Field::Title) => node.title,
            (Some(node), Field::Summary) => node.intent,
            (Some(node), Field::Folder) => node.folder,
            (None, Field::Title) => self.project_name(cx),
            (None, Field::Summary) => self
                .structure(cx)
                .map(|structure| structure.settings.description.clone())
                .unwrap_or_default(),
            (None, Field::Folder) => String::new(),
        }
    }

    fn input(&self, field: Field) -> &Entity<TextInput> {
        match field {
            Field::Title => &self.title,
            Field::Summary => &self.summary,
            Field::Folder => &self.folder,
        }
    }

    fn key(field: Field) -> &'static str {
        match field {
            Field::Title => "title",
            Field::Summary => "summary",
            Field::Folder => "folder",
        }
    }

    /// Escape: the stored value again.
    fn restore(&mut self, field: Field, cx: &mut Context<Self>) {
        let value = self.stored(field, cx);
        self.shown.insert(Self::key(field), value.clone());
        self.input(field)
            .clone()
            .update(cx, |input, cx| input.set_text(value, cx));
    }

    /// `InlineText`'s commit, on blur or Enter: a changed value is written; an empty title is
    /// refused and shows the stored one again.
    fn commit(&mut self, field: Field, cx: &mut Context<Self>) {
        if self.structure(cx).is_none() || self.doc.is_none() {
            return;
        }
        let text = self.input(field).read(cx).text().trim().to_string();
        let stored = self.stored(field, cx);
        if field == Field::Title && text.is_empty() {
            self.restore(field, cx);
            return;
        }
        if text == stored.trim() {
            return;
        }
        self.shown.insert(Self::key(field), text.clone());
        match self.doc_node(cx) {
            Some(node) => {
                let patch = match field {
                    Field::Title => NodePatch {
                        title: Some(text),
                        ..NodePatch::default()
                    },
                    Field::Summary => NodePatch {
                        intent: Some(text),
                        ..NodePatch::default()
                    },
                    Field::Folder => NodePatch {
                        folder: Some(text),
                        ..NodePatch::default()
                    },
                };
                self.patch(&node.id, patch, cx);
            }
            None => {
                let (name, description) = match field {
                    Field::Title => (Some(text), None),
                    Field::Summary => (None, Some(text)),
                    Field::Folder => return,
                };
                self.state.update(cx, |state, cx| {
                    state.edit_structure_project(name, description, cx)
                });
            }
        }
    }

    /// Shows the open document's stored values in its fields: all of them for another node,
    /// else what changed and is not being typed.
    fn sync_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let id = self
            .state
            .read(cx)
            .structure_node()
            .unwrap_or(PROJECT)
            .to_string();
        let fresh = self.doc.as_deref() != Some(id.as_str());
        if fresh {
            self.doc = Some(id);
            self.shown.clear();
        }
        for field in [Field::Title, Field::Summary, Field::Folder] {
            let value = self.stored(field, cx);
            let input = self.input(field).clone();
            let typing = input.read(cx).focus_handle(cx).is_focused(window);
            let key = Self::key(field);
            if (fresh || !typing) && self.shown.get(key) != Some(&value) {
                self.shown.insert(key, value.clone());
                input.update(cx, |input, cx| input.set_text(value, cx));
            }
        }
    }

    /// The head tokens for `node`, set again only when what they show changed.
    fn sync_pills(&mut self, node: &StructureNode, structure: &Structure, cx: &mut Context<Self>) {
        let suggestions: Vec<String> = {
            let mut techs: Vec<String> = structure
                .nodes
                .iter()
                .flat_map(|node| list_values(&node.tech))
                .collect();
            techs.sort_by_key(|tech| tech.to_lowercase());
            techs.dedup();
            techs
        };
        let key = format!(
            "{}\u{0}{}\u{0}{}\u{0}{:?}\u{0}{}",
            node.id,
            node.kind,
            node.tech,
            node.deploys,
            suggestions.join(",")
        );
        if self.pills_for.as_deref() == Some(key.as_str()) {
            return;
        }
        self.pills_for = Some(key);
        let kinds = COMPONENT_KINDS
            .iter()
            .map(|(kind, label, icon)| PropertyOption::new(*kind, *label).icon(*icon))
            .collect();
        let picked = |text: &str| -> Vec<SharedString> {
            list_values(text)
                .into_iter()
                .map(SharedString::from)
                .collect()
        };
        let kind = Property::tokens("kind", "Kind", kinds, picked(&node.kind), "Kind")
            .loose()
            .icon(Icon::Package);
        let techs = suggestions
            .iter()
            .map(|tech| PropertyOption::new(tech.clone(), tech.clone()).icon(Icon::Cpu))
            .collect();
        let tech = Property::tokens(
            "tech",
            "Technology",
            techs,
            picked(&node.tech),
            "Technology",
        )
        .loose()
        .creatable()
        .icon(Icon::Cpu);
        let options = node
            .deploys
            .iter()
            .enumerate()
            .map(|(index, deploy)| {
                let mut option =
                    PropertyOption::new(index.to_string(), deploy_label(deploy)).mark({
                        let provider = deploy.provider.clone();
                        move |size, _, cx| option_mark(&provider, size, cx)
                    });
                if !deploy.state.is_empty() {
                    option = option.hint(deploy_state(&deploy.state));
                }
                let url = if deploy.state == "failed" && !deploy.state_url.is_empty() {
                    &deploy.state_url
                } else {
                    &deploy.url
                };
                if !url.is_empty() {
                    option = option.opens(format!("Open in {}", provider_name(&deploy.provider)));
                }
                option
            })
            .collect();
        let values = (0..node.deploys.len())
            .map(|index| SharedString::from(index.to_string()))
            .collect();
        let deploys = Property::tokens("deploys", "Deploys", options, values, "Where it runs")
            .loose()
            .editable()
            .adds()
            .icon(Icon::Cloud);
        self.kinds
            .update(cx, |pills, cx| pills.set_properties(vec![kind], cx));
        self.techs
            .update(cx, |pills, cx| pills.set_properties(vec![tech], cx));
        self.deploys
            .update(cx, |pills, cx| pills.set_properties(vec![deploys], cx));
    }

    /// The Level picker's options, where the node may switch: right under the project, with
    /// nothing under it.
    fn level_options(node: &StructureNode, structure: &Structure) -> Vec<PropertyOption> {
        if !node.parent_id.is_empty()
            || structure
                .nodes
                .iter()
                .any(|other| other.parent_id == node.id)
        {
            return Vec::new();
        }
        ["domain", "component"]
            .into_iter()
            .map(|level| PropertyOption::new(level, capitalize(level)).icon(kind_icon(level)))
            .collect()
    }

    /// The picker `pick` opens with, and the value it starts on.
    fn pick_options(&self, pick: Pick, cx: &App) -> (String, Vec<PropertyOption>, Option<String>) {
        let Some(structure) = self.structure(cx) else {
            return (String::new(), Vec::new(), None);
        };
        let node = self.selected(cx);
        match (pick, node) {
            (Pick::Level, Some(node)) => (
                "Level".into(),
                Self::level_options(&node, structure),
                Some(node.level.clone()),
            ),
            (Pick::Domain, Some(node)) => {
                let options = std::iter::once(
                    PropertyOption::new(PROJECT, self.project_name(cx)).icon(Icon::House),
                )
                .chain(
                    structure
                        .nodes
                        .iter()
                        .filter(|other| other.id != node.id && other.level == "domain")
                        .map(|other| {
                            PropertyOption::new(other.id.clone(), other.title.clone())
                                .icon(Icon::Boxes)
                        }),
                )
                .collect();
                let value = if node.parent_id.is_empty() {
                    PROJECT.to_string()
                } else {
                    node.parent_id.clone()
                };
                ("Domain".into(), options, Some(value))
            }
            (Pick::Repository, Some(node)) => {
                let options = std::iter::once(PropertyOption::new("", "None").icon(Icon::XIcon))
                    .chain(structure.repos.iter().map(|repo| {
                        PropertyOption::new(repo.name.clone(), repository_label(repo)).icon(
                            if repo.remote.is_empty() {
                                Icon::GitBranch
                            } else {
                                Icon::Github
                            },
                        )
                    }))
                    .chain(std::iter::once(
                        PropertyOption::new(CHOOSE_FOLDER, "Choose folder…").icon(Icon::Folder),
                    ))
                    .collect();
                ("Repository".into(), options, Some(node.repo.clone()))
            }
            (Pick::Link(kind), node) => {
                let records = match &node {
                    Some(node) => node_records(structure, node),
                    None => project_records(structure),
                };
                let linked: Vec<&str> = records
                    .of(kind)
                    .iter()
                    .map(|record| record.id.as_str())
                    .collect();
                let (title, icon) = section_of(kind);
                let options = project_records(structure)
                    .of(kind)
                    .iter()
                    .filter(|record| !linked.contains(&record.id.as_str()))
                    .map(|record| {
                        PropertyOption::new(record.id.clone(), record.title.clone()).icon(icon)
                    })
                    .collect();
                (
                    format!("Link {}", title.trim_end_matches('s').to_lowercase()),
                    options,
                    None,
                )
            }
            _ => (String::new(), Vec::new(), None),
        }
    }

    /// Opens the document's picker for `pick`, or closes it when it is the open one.
    fn toggle_pick(&mut self, pick: Pick, window: &mut Window, cx: &mut Context<Self>) {
        if self.picking == Some(pick) && self.picker.read(cx).is_open() {
            self.picker.update(cx, |picker, cx| picker.close(cx));
            return;
        }
        let (label, options, value) = self.pick_options(pick, cx);
        self.picking = Some(pick);
        self.picker.update(cx, |picker, cx| {
            picker.set_options(label, options, value.map(SharedString::from), 240.0, cx);
            picker.set_trigger_height(28.0);
            picker.open(window, cx);
        });
        cx.notify();
    }

    fn picked(
        &mut self,
        _: &Entity<PropertyPicker>,
        event: &PropertyPickerEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let PropertyPickerEvent::Pick(value) = event else {
            return;
        };
        let value = value.to_string();
        let Some(pick) = self.picking else {
            return;
        };
        let node = self.selected(cx);
        match (pick, node) {
            (Pick::Level, Some(node)) if value != node.level => {
                let patch = if value == "component" {
                    NodePatch {
                        level: Some(value),
                        ..NodePatch::default()
                    }
                } else {
                    NodePatch {
                        level: Some(value),
                        kind: Some(String::new()),
                        repo: Some(String::new()),
                        folder: Some(String::new()),
                        ..NodePatch::default()
                    }
                };
                self.patch(&node.id, patch, cx);
            }
            (Pick::Domain, Some(node)) => {
                let parent = if value == PROJECT {
                    String::new()
                } else {
                    value
                };
                if parent != node.parent_id {
                    self.state.update(cx, |state, cx| {
                        state.move_structure_node(&node.id, &parent, cx)
                    });
                }
            }
            (Pick::Repository, Some(node)) if value == CHOOSE_FOLDER => {
                self.pick_folder(node.id.clone(), window, cx);
            }
            (Pick::Repository, Some(node)) if value != node.repo => {
                self.patch(
                    &node.id,
                    NodePatch {
                        repo: Some(value),
                        ..NodePatch::default()
                    },
                    cx,
                );
            }
            (Pick::Link(kind), Some(node)) => {
                self.state.update(cx, |state, cx| {
                    state.link_structure_record(kind, value, &node.id, true, cx)
                });
            }
            _ => {}
        }
    }

    /// Choose folder…: links the local clone picked in the folder dialog (`dir:`).
    fn pick_folder(&mut self, node: String, _: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            let Some(dir) = paths
                .into_iter()
                .next()
                .map(|path: PathBuf| path.display().to_string())
            else {
                return;
            };
            let _ = this.update(cx, |this, cx| {
                this.patch(
                    &node,
                    NodePatch {
                        dir: Some(dir),
                        ..NodePatch::default()
                    },
                    cx,
                )
            });
        })
        .detach();
    }

    /// `copyAgentPrompt`.
    fn copy_prompt(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some((.., request)) = PROMPTS.get(index) else {
            return;
        };
        let prompt = format!("{PREAMBLE}\n\n{request}\n\n{CLOSING}");
        cx.write_to_clipboard(ClipboardItem::new_string(prompt));
        self.state.update(cx, |state, _| {
            state.push_toast(
                "Prompt copied. Paste it into your agent.",
                ToastTone::Success,
            )
        });
    }

    /// The anchor of `pick`'s trigger.
    fn anchor(&mut self, pick: Pick) -> AnchorSlot {
        self.anchors.entry(pick).or_default().clone()
    }

    /// `PickerValue`: a small ghost button with the value's muted icon and the value, or a muted
    /// placeholder, that opens the shared picker.
    fn picker_value(
        &mut self,
        pick: Pick,
        icon: Icon,
        value: Option<String>,
        placeholder: &str,
        mono: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let slot = self.anchor(pick);
        let view = cx.entity().downgrade();
        let shown = value.clone().filter(|value| !value.is_empty());
        let label = div()
            .text_size(px(13.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.color_text_primary)
            .when(shown.is_none(), |label| {
                label
                    .text_color(theme.color_text_muted)
                    .font_weight(FontWeight::NORMAL)
            })
            .when(mono && shown.is_some(), |label| {
                label
                    .font_family(theme.font_mono)
                    .text_size(px(12.0))
                    .font_weight(FontWeight::NORMAL)
            })
            .child(shown.unwrap_or_else(|| placeholder.to_string()));
        div()
            .relative()
            .flex_none()
            .mx(px(-8.0))
            .child(
                Button::new(
                    SharedString::from(format!("structure-pick-{pick:?}")),
                    ButtonKind::Ghost,
                )
                .size(ButtonSize::Sm)
                .leading(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(icon_colored(icon, 14.0, theme.color_text_muted))
                        .child(label),
                )
                .on_click(move |window, cx| {
                    let _ = view.update(cx, |this, cx| this.toggle_pick(pick, window, cx));
                }),
            )
            .child(slot.probe())
            .into_any_element()
    }
}

/// `HeadRow`: a muted label column, then the values inline, wrapping 4px by 20px apart.
fn head_row(label: &str, values: Vec<AnyElement>, theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .min_h(px(28.0))
        .flex()
        .items_start()
        .child(
            div()
                .flex_none()
                .w(px(LABEL_COLUMN))
                .min_h(px(28.0))
                .flex()
                .items_center()
                .text_size(px(13.0))
                .text_color(theme.color_text_muted)
                .child(label.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .min_h(px(28.0))
                .flex()
                .flex_wrap()
                .items_center()
                .gap_x(px(20.0))
                .gap_y(px(4.0))
                .children(values),
        )
        .into_any_element()
}

/// `pw-structure__head-group`: lines 10px apart, set 20px under a rule after the first.
fn head_group(first: bool, rows: Vec<AnyElement>, theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .when(!first, |group| {
            group
                .pt(px(20.0))
                .border_t_1()
                .border_color(theme.color_border_subtle)
        })
        .children(rows)
        .into_any_element()
}

impl Render for StructureView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        let picker_width = self
            .picking
            .and_then(|pick| self.anchors.get(&pick)?.get())
            .map(|bounds| f32::from(bounds.size.width));
        self.picker.update(cx, |picker, _| {
            picker.set_align_end(matches!(self.picking, Some(Pick::Link(_))));
            if let Some(width) = picker_width {
                picker.set_trigger_width(width);
            }
        });
        if self.pane.is_animating() {
            window.request_animation_frame();
        }
        let load = self.state.read(cx).structure().clone();
        let structure = match &load {
            Load::Ready(structure) => Some(structure.clone()),
            _ => None,
        };
        // The Kind filter offers the kinds the components use.
        let kinds: Vec<String> = structure
            .as_ref()
            .map(|structure| {
                let mut kinds: Vec<String> = structure
                    .nodes
                    .iter()
                    .flat_map(|node| list_values(&node.kind))
                    .collect();
                kinds.sort();
                kinds.dedup();
                kinds
            })
            .unwrap_or_default();
        if self.filter_kinds.as_ref() != Some(&kinds) {
            let mut fields = vec![
                FilterField::new(
                    FilterSpec::multi(
                        "level",
                        "Level",
                        ["domain", "component"]
                            .map(|level| FilterOption::new(level, capitalize(level)))
                            .to_vec(),
                    ),
                    Some(Icon::Layers),
                )
                .option_icons([
                    ("domain".into(), Icon::Boxes),
                    ("component".into(), Icon::Blockchain01Icon),
                ]),
            ];
            if !kinds.is_empty() {
                fields.push(
                    FilterField::new(
                        FilterSpec::multi(
                            "kind",
                            "Kind",
                            kinds
                                .iter()
                                .map(|kind| FilterOption::new(kind.clone(), component_label(kind)))
                                .collect(),
                        ),
                        Some(Icon::Boxes),
                    )
                    .option_icons(kinds.iter().map(|kind| {
                        (
                            kind.clone(),
                            COMPONENT_KINDS
                                .iter()
                                .find(|(id, ..)| id == kind)
                                .map_or(Icon::Blockchain01Icon, |(_, _, icon)| *icon),
                        )
                    })),
                );
            }
            self.filters
                .update(cx, |popover, _| popover.set_fields(fields));
            self.filter_kinds = Some(kinds);
        }
        let selected = self.selected(cx);
        let selected_id = self
            .state
            .read(cx)
            .structure_node()
            .map(str::to_string)
            .unwrap_or_else(|| PROJECT.to_string());
        // The selected row stays in view: its ancestors open when the selection moves. Opening
        // a component asks its platforms for the latest deploys.
        if self.followed.as_deref() != Some(selected_id.as_str()) && structure.is_some() {
            self.followed = Some(selected_id.clone());
            if let (Some(structure), Some(node)) = (&structure, &selected) {
                let mut parent = node.parent_id.clone();
                let mut ancestors = vec![SharedString::from(PROJECT)];
                while !parent.is_empty() && !ancestors.iter().any(|id| id.as_ref() == parent) {
                    ancestors.push(parent.clone().into());
                    parent = structure
                        .nodes
                        .iter()
                        .find(|other| other.id == parent)
                        .map(|other| other.parent_id.clone())
                        .unwrap_or_default();
                }
                for id in ancestors {
                    if !self.expanded.contains(&id) {
                        self.expanded.push(id);
                    }
                }
                self.remember_tree();
                let (subject, state) = (node.id.clone(), self.state.clone());
                cx.defer(move |cx| {
                    state.update(cx, |state, cx| state.load_document_activity(subject, cx))
                });
            }
        }
        let check = selected
            .as_ref()
            .filter(|node| node.level == "component" && !node.deploys.is_empty())
            .map(|node| node.id.clone());
        if check.is_some() && self.checked != check {
            self.checked = check.clone();
            if let Some(id) = check {
                let state = self.state.clone();
                cx.defer(move |cx| {
                    state.update(cx, |state, cx| state.check_deploys(id, false, cx))
                });
            }
        } else if check.is_none() {
            self.checked = None;
        }
        if structure.is_some() {
            self.sync_fields(window, cx);
        }
        let toolbar = self.toolbar(window, cx);
        let body = match &load {
            Load::Idle | Load::Loading => div()
                .size_full()
                .p(px(theme.space_4))
                .child(text_skeleton(4, 12.0, window, cx))
                .into_any_element(),
            Load::Failed(message) => div()
                .size_full()
                .child(page_empty_with(
                    Some("Could not read Structure"),
                    MascotMood::Sad,
                    Some(
                        div()
                            .text_size(px(13.0))
                            .text_color(theme.color_text_muted)
                            .child(message.clone())
                            .into_any_element(),
                    ),
                    cx,
                ))
                .into_any_element(),
            Load::Ready(structure) => {
                let tree = self.tree_pane(structure, &selected_id, window, cx);
                let document = self.document(structure, selected.as_ref(), window, cx);
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .child(tree)
                    .child(document)
                    .into_any_element()
            }
        };
        let overlay = (self.picker.read(cx).is_presented())
            .then(|| self.picking.and_then(|pick| self.anchors.get(&pick)?.get()))
            .flatten()
            .map(|anchor| window_anchor(anchor, self.picker.clone()));
        let pills_overlay = [&self.kinds, &self.techs, &self.deploys]
            .into_iter()
            .filter_map(|pills| pills.read(cx).overlay(cx))
            .collect::<Vec<_>>();
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(floating_toolbar(
                self.toolbar_layout.clone(),
                toolbar,
                div().size_full().flex().flex_col().child(body),
            ))
            .children(overlay)
            .children(pills_overlay)
            .child(self.composer.clone())
            .child(self.deploy_composer.clone())
            .child(self.record_editor.clone())
    }
}

impl StructureView {
    /// `structure.css`: both panes include the floating PageToolbar height in their inset.
    fn toolbar_inset(&self, cx: &App) -> f32 {
        self.toolbar_layout.read(cx).height()
    }

    /// `PageToolbar` with the tree's toggle leading, the clauses under it.
    fn toolbar(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let toggle = Button::new("structure-tree-toggle", ButtonKind::Secondary)
            .size(ButtonSize::Icon)
            .icon(Icon::SidebarLeftIcon)
            .selected(self.tree_open)
            .tooltip(if self.tree_open {
                "Hide structure tree"
            } else {
                "Show structure tree"
            })
            .on_click(move |_, cx| {
                let _ = view.update(cx, |this, cx| this.toggle_tree(cx));
            });
        // Draws the tree on the project's Structure canvas, which Canvas then opens on.
        let state = self.state.clone();
        let draw = Button::new("structure-draw-canvas", ButtonKind::Secondary)
            .size(ButtonSize::Icon)
            .icon(Icon::SchemeIcon)
            .tooltip("Draw as canvas")
            .disabled(self.state.read(cx).drawing_structure())
            .on_click(move |_, cx| {
                state.update(cx, |state, cx| {
                    let nodes = match state.structure() {
                        Load::Ready(structure) => structure.nodes.clone(),
                        _ => Vec::new(),
                    };
                    state.draw_structure_canvas(nodes, cx);
                    cx.notify();
                });
            });
        let bar = div()
            .flex_none()
            .h(px(theme.page_header_height))
            .pt(px(theme.space_2))
            .px(px(theme.space_2))
            .pb(px(theme.space_1))
            .flex()
            .items_center()
            .gap(px(theme.space_3))
            .child(toggle)
            .child(div().flex_1().h_full().window_drag_region())
            .child(draw);
        let clauses = FilterPopover::bar(&self.filters, window, cx).map(|bar| {
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
            .children(clauses)
            .into_any_element()
    }

    /// The tree pane: the card with the search, the filters, and the tree.
    fn tree_pane(
        &mut self,
        structure: &Structure,
        selected: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let open = self.pane.value().clamp(0.0, 1.0);
        let filters = self.filters.read(cx).filters().clone();
        let filtering = !self.query.trim().is_empty() || !filters.is_empty();
        let matches = |node: &StructureNode| {
            let kinds = list_values(&node.kind);
            let kinds: Vec<&str> = kinds.iter().map(String::as_str).collect();
            clause_match(&filters, &node.level, &kinds)
        };
        let shown = build_tree(
            &structure.nodes,
            &self.query,
            (!filters.is_empty()).then_some(&matches as &dyn Fn(&StructureNode) -> bool),
        );
        let expanded: Vec<SharedString> = if filtering {
            std::iter::once(SharedString::from(PROJECT))
                .chain(
                    flatten(&shown)
                        .into_iter()
                        .map(|node| node.id.clone().into()),
                )
                .collect()
        } else {
            self.expanded.clone()
        };
        let view = cx.entity().downgrade();
        let add = |id: String, label: String, view: gpui::WeakEntity<Self>| {
            Button::new(
                SharedString::from(format!("structure-add-{id}")),
                ButtonKind::Ghost,
            )
            .size(ButtonSize::Icon)
            .icon(Icon::Plus)
            .tooltip(label)
            .on_click(move |window, cx| {
                let _ = view.update(cx, |this, cx| this.add_child(&id, window, cx));
            })
        };
        fn tile(level: &str) -> IconTile {
            IconTile::new(kind_icon(level)).size(20.0, 12.0)
        }
        fn node_of(
            item: TreeItem,
            view: &gpui::WeakEntity<StructureView>,
            add: &dyn Fn(String, String, gpui::WeakEntity<StructureView>) -> Button,
        ) -> TreeNode {
            let mut row = TreeNode::new(item.node.id.clone(), item.node.title.clone())
                .icon(tile(&item.node.level));
            if item.node.level != "component" {
                row = row
                    .trailing(add(
                        item.node.id.clone(),
                        format!("Add component to {}", item.node.title),
                        view.clone(),
                    ))
                    .trailing_on_hover();
            }
            row.children(
                item.children
                    .into_iter()
                    .map(|child| node_of(child, view, add))
                    .collect(),
            )
        }
        let empty = filtering && shown.is_empty();
        let root = TreeNode::new(PROJECT, self.project_name(cx))
            .icon(tile("project"))
            .trailing(add(PROJECT.into(), "Add domain".into(), view.clone()))
            .trailing_on_hover()
            .children(
                shown
                    .into_iter()
                    .map(|item| node_of(item, &view, &add))
                    .collect(),
            );
        let (toggled, picked) = (view.clone(), view.clone());
        let tree = if empty {
            div()
                .px(px(8.0))
                .py(px(12.0))
                .text_size(px(12.5))
                .text_color(theme.color_text_muted)
                .child("No matching nodes")
                .into_any_element()
        } else {
            tree_view("structure-tree", vec![root])
                .expanded(expanded)
                .selected(Some(selected.to_string().into()))
                .on_toggle(Rc::new(move |id, open, _, cx| {
                    let _ = toggled.update(cx, |this, cx| {
                        let filtering = !this.query.trim().is_empty()
                            || !this.filters.read(cx).filters().is_empty();
                        if filtering {
                            return;
                        }
                        this.expanded.retain(|kept| *kept != id);
                        if open {
                            this.expanded.push(id);
                        }
                        this.remember_tree();
                        cx.notify();
                    });
                }))
                .on_select(Rc::new(move |id, _, cx| {
                    let _ = picked.update(cx, |this, cx| this.open(&id, cx));
                }))
                .build(window, cx)
        };
        let tools = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .mb(px(8.0))
            .child(div().flex_1().min_w(px(0.0)).child(self.search.clone()))
            .child(self.filters.clone());
        let tree_card = card(cx)
            .id("structure-tree-card")
            .w(px(TREE_CARD))
            .max_h_full()
            .flex_none()
            .overflow_y_scroll()
            .p(px(12.0))
            .child(tools)
            .child(tree);
        div()
            .flex_none()
            .w(px((TREE_CARD + TREE_INSET) * open))
            .h_full()
            .min_h(px(0.0))
            .overflow_hidden()
            .pt(px(8.0 + self.toolbar_inset(cx)))
            .pb(px(8.0))
            .pl(px(TREE_INSET.min((TREE_CARD + TREE_INSET) * open)))
            .flex()
            .items_start()
            .when(open <= 0.001, |pane| pane.invisible())
            .child(
                div()
                    .relative()
                    .left(px(-TREE_SLIDE * (1.0 - open)))
                    .opacity(open)
                    .max_h_full()
                    .flex()
                    .child(tree_card),
            )
            .into_any_element()
    }

    /// The selected node's document, or the project's.
    fn document(
        &mut self,
        structure: &Structure,
        node: Option<&StructureNode>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let level = node.map_or("project", |node| node.level.as_str());
        let records = match node {
            Some(node) => node_records(structure, node),
            None => project_records(structure),
        };
        // The hero: the level by its tile (a picker where it may switch), the title, the summary.
        let level_options = node
            .map(|node| Self::level_options(node, structure))
            .unwrap_or_default();
        let level_mark: AnyElement = if level_options.len() > 1 {
            self.picker_value(
                Pick::Level,
                kind_icon(level),
                Some(capitalize(level)),
                "Level",
                false,
                cx,
            )
        } else {
            div()
                .flex_none()
                .text_size(px(13.0))
                .text_color(theme.color_text_muted)
                .child(capitalize(level))
                .into_any_element()
        };
        let hero = div()
            .w_full()
            .flex()
            .flex_col()
            .items_start()
            .mb(px(28.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(IconTile::new(kind_icon(level)).size(36.0, 18.0))
                    .child(level_mark),
            )
            .child(
                div()
                    .w_full()
                    .mt(px(16.0))
                    .mb(px(4.0))
                    .child(self.title.clone()),
            )
            .child(div().w_full().child(self.summary.clone()));
        let head = match node {
            Some(node) if node.level == "component" => Some(self.head(structure, node, cx)),
            _ => None,
        };
        let sections = self.record_sections(structure, node, &records, cx);
        let activity = node.and_then(|node| document_activity(&self.state, &node.id, cx));
        let start = structure.nodes.is_empty().then(|| {
            let cards = PROMPTS
                .iter()
                .map(|(title, description, tone, level, _)| {
                    PromptCard::new(*title, *description, kind_icon(level), *tone)
                })
                .collect();
            let view = cx.entity().downgrade();
            let grid = prompt_card_grid(
                "structure-prompts",
                cards,
                Rc::new(move |index, _, cx| {
                    let _ = view.update(cx, |this, cx| this.copy_prompt(index, cx));
                }),
                window,
                cx,
            );
            div()
                .mt(px(24.0))
                .child(page_empty_with(
                    Some("No structure yet"),
                    MascotMood::Sleeping,
                    Some(grid),
                    cx,
                ))
                .into_any_element()
        });
        let column = div()
            .w_full()
            .max_w(px(polterware_ui::patterns::DOCUMENT_COLUMN))
            .mx_auto()
            .pt(px(48.0 + self.toolbar_inset(cx)))
            .flex()
            .flex_col()
            .child(hero)
            .children(head)
            .children(sections)
            .children(activity)
            .children(start);
        div()
            .id("structure-document")
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .overflow_y_scroll()
            .px(px(48.0))
            .pb(px(64.0))
            .child(column)
            .into_any_element()
    }

    /// A component's head lines: what it is, where its code lives, where it runs.
    fn head(
        &mut self,
        structure: &Structure,
        node: &StructureNode,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        self.sync_pills(node, structure, cx);
        let parent = structure
            .nodes
            .iter()
            .find(|other| other.id == node.parent_id);
        let domain = self.picker_value(
            Pick::Domain,
            Icon::Boxes,
            Some(parent.map_or_else(|| self.project_name(cx), |parent| parent.title.clone())),
            "Domain",
            false,
            cx,
        );
        let what = head_group(
            true,
            vec![
                head_row("Domain", vec![domain], &theme),
                head_row("Kind", vec![self.kinds.clone().into_any_element()], &theme),
                head_row(
                    "Technology",
                    vec![self.techs.clone().into_any_element()],
                    &theme,
                ),
            ],
            &theme,
        );
        let repository = structure.repos.iter().find(|repo| repo.name == node.repo);
        let repo_icon = if repository.is_some_and(|repo| !repo.remote.is_empty()) {
            Icon::Github
        } else {
            Icon::GitBranch
        };
        let repo_label = repository.map_or_else(|| node.repo.clone(), repository_label);
        let mut repo_values = vec![self.picker_value(
            Pick::Repository,
            repo_icon,
            Some(repo_label),
            "Repository",
            true,
            cx,
        )];
        if let Some(repo) = repository
            && let Some(path) = repo.remote.strip_prefix("github.com/")
        {
            let url = format!("https://github.com/{path}");
            repo_values.push(
                Button::new("structure-github", ButtonKind::Ghost)
                    .size(ButtonSize::Icon)
                    .icon(Icon::Github)
                    .tooltip(format!("Open {} on GitHub", repository_label(repo)))
                    .on_click(move |_, cx| cx.open_url(&url))
                    .into_any_element(),
            );
        }
        let folder = div()
            .flex_1()
            .min_w(px(72.0))
            .flex()
            .items_center()
            .gap(px(2.0))
            .child(icon_colored(Icon::Folder, 14.0, theme.color_text_muted))
            .child(
                div()
                    .flex_1()
                    .min_w(px(72.0))
                    .px(px(6.0))
                    .child(self.folder.clone()),
            )
            .into_any_element();
        let code = head_group(
            false,
            vec![
                head_row("Repository", repo_values, &theme),
                head_row("Folder", vec![folder], &theme),
            ],
            &theme,
        );
        let mut deploy_values = vec![
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(6.0))
                .child(self.deploys.clone())
                .when(!node.deploys.is_empty(), |deploys| {
                    let state = self.state.clone();
                    let id = node.id.clone();
                    deploys.child(
                        Button::new("structure-check-deploys", ButtonKind::Ghost)
                            .size(ButtonSize::Icon)
                            .icon(Icon::RefreshCw)
                            .tooltip("Check deploys")
                            .disabled(self.state.read(cx).deploys_checking())
                            .on_click(move |_, cx| {
                                let id = id.clone();
                                state.update(cx, |state, cx| state.check_deploys(id, true, cx));
                            }),
                    )
                })
                .into_any_element(),
        ];
        for deploy in node
            .deploys
            .iter()
            .filter(|deploy| deploy.state == "failed")
        {
            let environment = if deploy.environment.is_empty() {
                &deploy.environment_id
            } else {
                &deploy.environment
            };
            let summary = if deploy.state_summary.is_empty() {
                String::new()
            } else {
                format!(": {}", deploy.state_summary)
            };
            deploy_values.push(
                chip_rounded(
                    format!("{environment} failed{summary}"),
                    StatusTone::Danger,
                    CHIP_RADIUS,
                    cx,
                )
                .into_any_element(),
            );
        }
        let mut providers: Vec<&str> = Vec::new();
        for deploy in &node.deploys {
            if !providers.contains(&deploy.provider.as_str()) {
                providers.push(&deploy.provider);
            }
        }
        for provider in providers {
            let connected = structure.integrations.iter().any(|connection| {
                connection.provider_id == provider && connection.status == "connected"
            });
            if !connected {
                deploy_values.push(
                    chip_rounded(
                        format!("{} CLI not connected", provider_name(provider)),
                        StatusTone::Warn,
                        CHIP_RADIUS,
                        cx,
                    )
                    .into_any_element(),
                );
            }
        }
        let runs = head_group(
            false,
            vec![head_row("Deploys", deploy_values, &theme)],
            &theme,
        );
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .mb(px(56.0))
            .child(what)
            .child(code)
            .child(runs)
            .into_any_element()
    }

    /// One collapsible table section per kind of record that cites the node; one that is
    /// empty shows only where a + can link one.
    fn record_sections(
        &mut self,
        structure: &Structure,
        node: Option<&StructureNode>,
        records: &Records,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let theme = *Theme::global(cx);
        let mut out = Vec::new();
        for (kind, section) in self.sections.clone() {
            let rows = records.of(kind).to_vec();
            // The project's document lists; a node's links, a card only to a component.
            let linkable =
                node.is_some_and(|node| kind != RecordKind::Card || node.level == "component");
            let candidates = linkable && !self.pick_options(Pick::Link(kind), cx).1.is_empty();
            if rows.is_empty() && !linkable {
                continue;
            }
            let (title, icon) = section_of(kind);
            let slot = self.anchor(Pick::Link(kind));
            let view = cx.entity().downgrade();
            let actions: Option<polterware_ui::patterns::SectionBody> = linkable.then(|| {
                let view = view.clone();
                let label = format!("Link {}", title.trim_end_matches('s').to_lowercase());
                Rc::new(move |_: &mut Window, _: &mut App| {
                    let view = view.clone();
                    div()
                        .relative()
                        .child(
                            Button::new(
                                SharedString::from(format!("structure-link-{kind:?}")),
                                ButtonKind::Ghost,
                            )
                            .size(ButtonSize::Icon)
                            .icon(Icon::Plus)
                            .tooltip(label.clone())
                            .disabled(!candidates)
                            .on_click(move |window, cx| {
                                let _ = view.update(cx, |this, cx| {
                                    this.toggle_pick(Pick::Link(kind), window, cx)
                                });
                            }),
                        )
                        .child(slot.probe())
                        .into_any_element()
                }) as polterware_ui::patterns::SectionBody
            });
            let node_id = node.map(|node| node.id.clone());
            let body: polterware_ui::patterns::SectionBody = {
                let view = view.clone();
                let empty = format!("No {} yet", title.to_lowercase());
                Rc::new(move |_: &mut Window, cx: &mut App| {
                    if rows.is_empty() {
                        return page_empty_with(
                            Some(&empty),
                            MascotMood::Sleeping,
                            Some(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(Theme::global(cx).color_text_muted)
                                    .child("Link one with + to show it here.")
                                    .into_any_element(),
                            ),
                            cx,
                        )
                        .into_any_element();
                    }
                    let count = rows.len();
                    let ids: Vec<String> = rows.iter().map(|row| row.id.clone()).collect();
                    let activate = view.clone();
                    let rows = rows.clone();
                    let node_id = node_id.clone();
                    let view = view.clone();
                    div()
                        .w_full()
                        .h(px(count as f32 * 44.0 + 8.0))
                        .child(
                            DataTable::new(
                                SharedString::from(format!("structure-records-{kind:?}")),
                                vec![Column::new("").shrinks()],
                                count,
                                move |row, _, cx| {
                                    vec![record_row(
                                        kind,
                                        &rows[row.index],
                                        icon,
                                        node_id.clone(),
                                        view.clone(),
                                        cx,
                                    )]
                                },
                            )
                            .on_activate(move |index, window, cx| {
                                if let Some(id) = ids.get(index) {
                                    let _ = activate.update(cx, |this, cx| {
                                        this.open_record_editor(kind, id.clone(), window, cx)
                                    });
                                }
                            })
                            .header(0.0, 11.0)
                            .row_height(40.0)
                            .gutters(0.0),
                        )
                        .into_any_element()
                })
            };
            section.update(cx, |section, _| {
                section.set_body(body);
                section.set_actions(actions);
            });
            out.push(
                div()
                    .w_full()
                    .mb(px(28.0))
                    .child(section)
                    .into_any_element(),
            );
        }
        let _ = (&theme, structure);
        out
    }
}

/// A record section's row (`RecordTable`): the section's icon tile and the title, then its
/// caption muted and, for a direct link, Unlink while the row is hovered.
fn record_row(
    kind: RecordKind,
    record: &Record,
    icon: Icon,
    node: Option<String>,
    view: gpui::WeakEntity<StructureView>,
    cx: &App,
) -> AnyElement {
    let theme = *Theme::global(cx);
    let group: SharedString = format!("structure-record-{kind:?}-{}", record.id).into();
    let unlink = (record.direct).then_some(node).flatten().map(|node| {
        let id = record.id.clone();
        let view = view.clone();
        div()
            .invisible()
            .group_hover(group.clone(), |slot| slot.visible())
            .child(
                Button::new(
                    SharedString::from(format!("structure-unlink-{}", record.id)),
                    ButtonKind::Ghost,
                )
                .size(ButtonSize::Icon)
                .icon(Icon::XIcon)
                .tooltip(format!("Unlink {}", record.title))
                .on_click(move |_, cx| {
                    let (id, node) = (id.clone(), node.clone());
                    let _ = view.update(cx, |this, cx| {
                        this.state.update(cx, |state, cx| {
                            state.link_structure_record(kind, id, &node, false, cx)
                        })
                    });
                }),
            )
    });
    div()
        .id(SharedString::from(format!(
            "structure-record-row-{kind:?}-{}",
            record.id
        )))
        .group(group)
        .w_full()
        // DataTable owns the 40px row and its 5px/12px cell padding.
        .h_full()
        .flex()
        .items_center()
        .gap(px(8.0))
        .rounded(px(theme.radius_item))
        .cursor_pointer()
        .text_size(px(12.5))
        .text_color(theme.color_text_secondary)
        .child(div().flex_1().min_w(px(0.0)).child(primary_cell(
            IconTile::new(icon).size(24.0, 14.0),
            SharedString::from(record.title.clone()),
            sans(&theme, FontWeight::MEDIUM),
            None,
            cx,
        )))
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .when(!record.caption.is_empty(), |end| {
                    end.child(
                        div()
                            .text_color(theme.color_text_muted)
                            .child(record.caption.clone()),
                    )
                })
                .children(unlink),
        )
        .into_any_element()
}

/// The tree's Level and Kind clauses for one node (`matchClauses`).
fn clause_match(filters: &Filters, level: &str, kinds: &[&str]) -> bool {
    filters.matches(|field| match field {
        "level" => Some(vec![level]),
        "kind" => Some(kinds.to_vec()),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, parent: &str, level: &str, title: &str) -> StructureNode {
        StructureNode {
            id: id.into(),
            parent_id: parent.into(),
            level: level.into(),
            title: title.into(),
            path: id.into(),
            ..StructureNode::default()
        }
    }

    #[test]
    fn a_search_keeps_its_branch_and_a_filter_only_what_passes() {
        let nodes = vec![
            node("billing", "", "domain", "Billing"),
            node("invoices", "billing", "component", "Invoices"),
            node("payouts", "billing", "component", "Payouts"),
            node("identity", "", "domain", "Identity"),
            node("orphan", "gone", "component", "Orphan"),
        ];
        let all = build_tree(&nodes, "", None);
        let roots: Vec<&str> = all.iter().map(|item| item.node.id.as_str()).collect();
        assert_eq!(roots, ["billing", "identity", "orphan"]);
        let searched = build_tree(&nodes, "billing", None);
        assert_eq!(searched.len(), 1);
        assert_eq!(
            searched[0].children.len(),
            2,
            "a text match keeps its whole branch"
        );
        let components = |node: &StructureNode| node.level == "component";
        let filtered = build_tree(&nodes, "", Some(&components));
        let ids: Vec<&str> = flatten(&filtered)
            .iter()
            .map(|node| node.id.as_str())
            .collect();
        assert_eq!(ids, ["billing", "invoices", "payouts", "orphan"]);
    }

    #[test]
    fn a_node_lists_what_cites_it_and_the_rules_that_hold_there() {
        let billing = node("billing", "", "domain", "Billing");
        let invoices = StructureNode {
            path: "billing/invoices".into(),
            decisions: vec!["dec-1".into()],
            ..node("invoices", "billing", "component", "Invoices")
        };
        let rule = |id: &str, scope: &[&str]| Rule {
            id: id.into(),
            slug: id.into(),
            scope_node_ids: scope.iter().map(|id| id.to_string()).collect(),
            ..Rule::default()
        };
        let structure = Structure {
            nodes: vec![billing, invoices.clone()],
            cards: vec![WorkCard {
                slug: "export".into(),
                title: "Export".into(),
                stage: "build".into(),
                under: "billing/invoices".into(),
                ..WorkCard::default()
            }],
            items: vec![
                KnowledgeItem {
                    id: "dec-1".into(),
                    title: "Decision 1".into(),
                    kind: "decision".into(),
                    status: "accepted".into(),
                    ..KnowledgeItem::default()
                },
                KnowledgeItem {
                    id: "doc-1".into(),
                    title: "Runbook".into(),
                    kind: "doc".into(),
                    tags: vec!["ops".into()],
                    scope_node_ids: vec!["invoices".into()],
                    ..KnowledgeItem::default()
                },
            ],
            rules: vec![
                rule("own", &["invoices"]),
                rule("whole", &[]),
                rule("inherited", &["billing"]),
                rule("elsewhere", &["identity"]),
            ],
            ..Structure::default()
        };
        let records = node_records(&structure, &invoices);
        assert_eq!(
            records
                .rules
                .iter()
                .map(|record| (record.id.as_str(), record.caption.as_str(), record.direct))
                .collect::<Vec<_>>(),
            [
                ("own", "", true),
                ("whole", "Whole project", false),
                ("inherited", "From Billing", false)
            ]
        );
        assert_eq!(records.cards[0].caption, "Build");
        assert!(records.cards[0].direct);
        assert_eq!(records.decisions[0].caption, "Active");
        assert!(
            !records.decisions[0].direct,
            "a governed decision is not a direct link"
        );
        assert_eq!(records.docs[0].caption, "ops");
        let project = project_records(&structure);
        assert_eq!(project.rules.len(), 4);
        assert_eq!(project.docs.len(), 1);
    }
}
