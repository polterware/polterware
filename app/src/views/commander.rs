//! The in-app Commander, using the shared palette for its rows and keyboard behavior.

use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window,
};
use polterware_ui::{Commander, CommanderEvent, CommanderGroup, CommanderItem, Icon, TileTone};

use super::commander_route::Route;
use crate::shortcuts::{self, Overrides};
use crate::state::{
    AppState, CommanderSelection, ComposeRequest, Section, SettingsTab, default_tier,
    load_commander_sources,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Item {
    id: &'static str,
    label: &'static str,
    group: &'static str,
    icon: Icon,
    tone: TileTone,
    hint: Option<&'static str>,
}

const ITEMS: &[Item] = &[
    item(
        "nav.inbox",
        "Inbox",
        "Navigation",
        Icon::Inbox,
        TileTone::Knowledge,
        Some("go.inbox"),
    ),
    item(
        "nav.milestones",
        "Milestones",
        "Navigation",
        Icon::Briefcase06Icon,
        TileTone::Execution,
        Some("go.milestones"),
    ),
    item(
        "nav.canvas",
        "Canvas",
        "Navigation",
        Icon::Pencil,
        TileTone::Neutral,
        Some("shell.open_canvas"),
    ),
    item(
        "nav.structure",
        "Structure",
        "Navigation",
        Icon::ChartRelationshipIcon,
        TileTone::Architecture,
        Some("go.structure"),
    ),
    item(
        "nav.truth",
        "Knowledge",
        "Navigation",
        Icon::Layers,
        TileTone::Knowledge,
        Some("go.truth"),
    ),
    item(
        "nav.activity",
        "Activity",
        "Navigation",
        Icon::History,
        TileTone::Neutral,
        Some("go.activity"),
    ),
    item(
        "nav.settings",
        "Open Project settings",
        "Navigation",
        Icon::Settings,
        TileTone::Neutral,
        None,
    ),
    item(
        "nav.app-settings",
        "Open App settings",
        "Navigation",
        Icon::Settings,
        TileTone::Neutral,
        None,
    ),
    item(
        "nav.platforms",
        "Open Project settings > Platforms",
        "Navigation",
        Icon::ShieldCheck,
        TileTone::Integration,
        None,
    ),
    item(
        "nav.cli-reference",
        "Open App settings > CLI reference",
        "Navigation",
        Icon::Terminal,
        TileTone::Identity,
        None,
    ),
    item(
        "action.open-sync",
        "Open App settings > Local core",
        "Actions",
        Icon::Activity,
        TileTone::Execution,
        None,
    ),
    item(
        "action.create-card",
        "New card",
        "Actions",
        Icon::Box,
        TileTone::Architecture,
        Some("shell.create"),
    ),
    item(
        "action.create-decision",
        "New decision",
        "Actions",
        Icon::Box,
        TileTone::Architecture,
        None,
    ),
    item(
        "action.create-doc",
        "New doc",
        "Actions",
        Icon::Box,
        TileTone::Architecture,
        None,
    ),
    item(
        "action.create-node",
        "Add structure node",
        "Actions",
        Icon::Plus,
        TileTone::Architecture,
        Some("structure.add_node"),
    ),
    item(
        "action.connect-integration",
        "Connect integration",
        "Actions",
        Icon::Plug,
        TileTone::Integration,
        None,
    ),
    item(
        "action.show-shortcuts",
        "Keyboard shortcuts",
        "Actions",
        Icon::Keyboard,
        TileTone::Neutral,
        Some("shell.show_shortcuts"),
    ),
    item(
        "recent.payment",
        "Payment Service",
        "Recent",
        Icon::Box,
        TileTone::Architecture,
        None,
    ),
    item(
        "recent.postgres",
        "prod-postgres",
        "Recent",
        Icon::Box,
        TileTone::Architecture,
        None,
    ),
];
const CARD_ITEMS: &[Item] = &[
    item(
        "action.copy-brief",
        "Copy agent brief",
        "Actions",
        Icon::Box,
        TileTone::Architecture,
        None,
    ),
    item(
        "action.dispatch-card",
        "Send this card to an agent",
        "Actions",
        Icon::Box,
        TileTone::Architecture,
        None,
    ),
];
const fn item(
    id: &'static str,
    label: &'static str,
    group: &'static str,
    icon: Icon,
    tone: TileTone,
    hint: Option<&'static str>,
) -> Item {
    Item {
        id,
        label,
        group,
        icon,
        tone,
        hint,
    }
}

fn filtered_items(card_open: bool, query: &str) -> Vec<Item> {
    let query = query.trim().to_lowercase();
    CARD_ITEMS
        .iter()
        .filter(|_| card_open)
        .chain(ITEMS.iter())
        .copied()
        .filter(|item| item.label.to_lowercase().contains(&query))
        .collect()
}

fn hint_text(item: &Item, overrides: &Overrides) -> Option<String> {
    item.hint.map(|hint| {
        shortcuts::command(hint)
            .map(|command| shortcuts::resolve(command, overrides).format())
            .or_else(|| shortcuts::sequence_hint(hint))
            .unwrap_or_else(|| hint.to_string())
    })
}

fn groups(card_open: bool, query: &str, overrides: &Overrides) -> Vec<CommanderGroup> {
    let items = filtered_items(card_open, query);
    let mut groups: Vec<(&str, Vec<CommanderItem>)> = Vec::new();
    for item in items {
        let mut row = CommanderItem::new(item.id, item.label).tile(item.icon, item.tone);
        if let Some(hint) = hint_text(&item, overrides) {
            row = row.hint(hint);
        }
        if let Some((_, rows)) = groups.iter_mut().find(|(group, _)| *group == item.group) {
            rows.push(row);
        } else {
            groups.push((item.group, vec![row]));
        }
    }
    groups
        .into_iter()
        .map(|(name, rows)| CommanderGroup::new(name.to_string()).items(rows))
        .collect()
}

fn card_prompt(title: &str, slug: &str) -> String {
    format!("{title}: work on card {slug}. Read `polterware brief {slug}` first.")
}

fn run_mode(query: &str, count: usize) -> bool {
    !query.trim().is_empty() && count == 0
}
fn prompt(title: &str, details: &str) -> String {
    let title = title.trim();
    let details = details.trim();
    if details.is_empty() {
        title.to_string()
    } else {
        format!("{title}\n\n{details}")
    }
}

pub struct CommanderView {
    state: Entity<AppState>,
    palette: Entity<Commander>,
    route: Route,
    load_id: u64,
}
impl CommanderView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let palette = cx.new(|cx| {
            let mut palette = Commander::new(window, cx);
            palette.set_placeholder("Run an agent or search…", cx);
            palette.set_filterable(false, cx);
            palette
        });
        cx.subscribe_in(
            &palette,
            window,
            |this, _, event: &CommanderEvent, window, cx| match event {
                CommanderEvent::Query(query) => {
                    let state = this.state.read(cx);
                    let card = state.board_route().and_then(|(_, page)| page).is_some();
                    let rows = groups(card, query, state.shortcut_overrides());
                    this.palette.update(cx, |palette, cx| {
                        palette.set_groups(rows, cx);
                        palette
                            .set_run_mode(run_mode(query, filtered_items(card, query).len()), cx);
                    });
                }
                CommanderEvent::Confirm(id) => this.run(id.as_ref(), cx),
                CommanderEvent::Cancel => this.state.update(cx, |state, cx| {
                    state.set_commander(false);
                    cx.notify();
                }),
                CommanderEvent::Run { title, details } => {
                    let text = prompt(title, details);
                    let selection = this.route.selection();
                    this.state.update(cx, |state, cx| {
                        state.start_commander_run(text, selection, cx);
                        state.set_commander(false);
                        cx.notify();
                    });
                }
                CommanderEvent::Property { id, values } => {
                    this.route
                        .change(id, values.first().map_or("", |v| v.as_ref()));
                    let properties = this.route.properties();
                    this.palette.update(cx, |palette, cx| {
                        palette.set_run_properties(properties, cx);
                        palette.focus_title(window, cx);
                    });
                }
                CommanderEvent::Select(_) => {}
            },
        )
        .detach();
        Self {
            state,
            palette,
            route: Route::new(),
            load_id: 0,
        }
    }
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.state.read(cx);
        let card = state.board_route().and_then(|(_, page)| page).is_some();
        let rows = groups(card, "", state.shortcut_overrides());
        let open_card = state
            .board_route()
            .and_then(|(_, page)| page)
            .unwrap_or("")
            .to_string();
        let source = state.commander_source_context();
        self.route.clear_project();
        self.route.preset(&open_card);
        self.load_id += 1;
        let load_id = self.load_id;
        if let Some((core, project)) = source {
            cx.spawn(async move |this, cx| {
                let sources = cx
                    .background_spawn(async move { load_commander_sources(core, project) })
                    .await;
                let _ = this.update(cx, |this, cx| {
                    if this.load_id != load_id {
                        return;
                    }
                    let previous = this.route.card.clone();
                    if !this.route.route_touched {
                        this.route.tier = default_tier(sources.auto.then_some("configured")).into();
                    }
                    this.route.sources = sources;
                    if !previous.is_empty() {
                        this.route.preset(&previous);
                    }
                    let properties = this.route.properties();
                    this.palette
                        .update(cx, |palette, cx| palette.set_run_properties(properties, cx));
                });
            })
            .detach();
        }
        let properties = self.route.properties();
        self.palette.update(cx, |palette, cx| {
            palette.set_groups(rows, cx);
            palette.set_run_properties(properties, cx);
            palette.set_open(true, window, cx);
        });
    }
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.palette
            .update(cx, |palette, cx| palette.set_open(false, window, cx));
    }

    fn run(&mut self, id: &str, cx: &mut Context<Self>) {
        let selection = self.route.selection();
        self.state.update(cx, |state, cx| {
            let card = state
                .board_route()
                .and_then(|(_, page)| page)
                .map(str::to_owned);
            match id {
                "nav.inbox" => state.go_section(Section::Inbox, cx),
                "nav.milestones" => state.go_section(Section::Milestones, cx),
                "nav.canvas" => state.go_section(Section::Canvas, cx),
                "nav.structure" => state.go_section(Section::Structure, cx),
                "nav.truth" => state.go_section(Section::Knowledge, cx),
                "nav.activity" => state.go_section(Section::Activity, cx),
                "nav.settings" => state.open_settings(SettingsTab::General, cx),
                "nav.app-settings" => state.open_settings(SettingsTab::AgentsMcp, cx),
                "nav.platforms" => state.open_settings(SettingsTab::Platforms, cx),
                "action.connect-integration" => state.open_connect_integration(cx),
                "nav.cli-reference" => state.open_settings(SettingsTab::CliReference, cx),
                "action.open-sync" => state.open_settings(SettingsTab::LocalCore, cx),
                "action.show-shortcuts" => state.open_settings(SettingsTab::Shortcuts, cx),
                "action.create-card" => state.compose(
                    ComposeRequest::Card {
                        stage: None,
                        milestone: String::new(),
                    },
                    cx,
                ),
                "action.create-decision" => state.compose(ComposeRequest::Decision, cx),
                "action.create-doc" => state.compose(ComposeRequest::Doc, cx),
                "action.create-node" => state.add_structure_node(cx),
                "action.copy-brief" => {
                    if let Some(slug) = card {
                        state.copy_commander_brief(slug, cx);
                    }
                }
                "action.dispatch-card" => {
                    if let Some(slug) = card {
                        let title = match state.work() {
                            crate::state::Load::Ready(work) => work
                                .card(&slug)
                                .map(|card| card.title.clone())
                                .unwrap_or_else(|| slug.clone()),
                            _ => slug.clone(),
                        };
                        state.start_commander_run(
                            card_prompt(&title, &slug),
                            CommanderSelection {
                                card_slug: slug,
                                ..selection
                            },
                            cx,
                        );
                    }
                }
                _ => {}
            }
            state.set_commander(false);
            cx.notify();
        });
    }
}
impl Focusable for CommanderView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.palette.read(cx).focus_handle(cx)
    }
}
impl Render for CommanderView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.palette.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn order_groups_filter_and_card_actions_follow_electron() {
        let normal = filtered_items(false, "");
        assert_eq!(normal[0].id, "nav.inbox");
        assert_eq!(normal[1].id, "nav.milestones");
        assert_eq!(normal[10].group, "Actions");
        assert_eq!(
            filtered_items(true, "")[..2]
                .iter()
                .map(|i| i.id)
                .collect::<Vec<_>>(),
            ["action.copy-brief", "action.dispatch-card"]
        );
        assert_eq!(
            filtered_items(false, "SETTINGS > CLI")[0].id,
            "nav.cli-reference"
        );
        assert!(filtered_items(false, "copy agent brief").is_empty());
    }
    #[test]
    fn run_mode_and_prompt_follow_palette_body() {
        assert!(!run_mode("  ", 0));
        assert!(!run_mode("settings", 1));
        assert!(run_mode("do work", 0));
        assert_eq!(prompt("  Fix it ", "  "), "Fix it");
        assert_eq!(
            prompt("  Fix it ", " Check tests "),
            "Fix it\n\nCheck tests"
        );
    }
    #[test]
    fn hints_use_remaps_and_sequences() {
        let mut overrides = Overrides::new();
        overrides.insert(
            "shell.create".into(),
            shortcuts::OwnedChord::parse("shift+c").unwrap(),
        );
        assert_eq!(
            hint_text(&ITEMS[0], &overrides).as_deref(),
            Some("G then I")
        );
        assert_eq!(hint_text(&ITEMS[11], &overrides).as_deref(), Some("⇧C"));
    }
    #[test]
    fn send_card_prompt_matches_electron() {
        assert_eq!(
            card_prompt("Fix it", "fix-it"),
            "Fix it: work on card fix-it. Read `polterware brief fix-it` first."
        );
    }
}
