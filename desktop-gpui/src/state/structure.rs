//! Structure's state (`StructureView`'s host in `App.tsx`, its writes in `workspace/persist.ts`):
//! the project's Structure nodes and what a node's document shows besides them (the project's
//! repositories, its cards, decisions, docs, and Rules, its platform connections, its
//! environments, and its settings), read together when Structure opens; the node open beside
//! the tree, kept in the route's `page` and moved without a step of history (`setSelectedNode`,
//! `"silent"`); and the writes a node's document and its Composers make.
//!
//! A node is written as Polter text through `WorkService.ApplyPolter`, as an agent writes it
//! (`structureNodeText`): its heading (`# path: Title`), its `level:` (and `under:` when it is
//! created or moved), then the lines it changes. A card is linked by its `under:`; a Rule, a
//! decision, or a doc by its contexts (`SetScope`). The project's name and description go
//! through `SaveProject` and `SaveProjectSettings` (`persistProjectSettings`). A component's
//! deploys are checked against their platforms (`CheckDeploys`) when its document opens, and at
//! once after they change.

use gpui::{App, AppContext as _, Context};
use polterware_ui::toast::ToastTone;

use super::{AppState, BoardRoute, KnowledgeTab, Load, Route, Section, change};
use crate::client::{Core, CoreError};
use crate::contracts::{
    ApplyPolterRequest, CheckDeploysRequest, DeployTarget, Environment, GetProjectSettingsRequest,
    IntegrationConnection, KnowledgeItem, ListDeployTargetsRequest, ListEnvironmentsRequest,
    ListIntegrationConnectionsRequest, ListKnowledgeRequest, ListProjectReposRequest,
    ListRulesRequest, ListStructureRequest, ListWorkCardsRequest, Project, ProjectRepo,
    ProjectSettings, Rule, SaveProjectRequest, SaveProjectSettingsRequest, SetScopeRequest,
    StructureNode, WorkCard,
};
use crate::work::{polter_card, polter_value};

const POLTER_HEADER: &str = "---\npolter: 1\n---\n\n";

/// What Structure reads, together, when it opens and after each write.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Structure {
    pub nodes: Vec<StructureNode>,
    pub repos: Vec<ProjectRepo>,
    pub cards: Vec<WorkCard>,
    /// The decisions and the docs.
    pub items: Vec<KnowledgeItem>,
    pub rules: Vec<Rule>,
    pub integrations: Vec<IntegrationConnection>,
    pub environments: Vec<Environment>,
    pub settings: ProjectSettings,
}

/// The fields of a node its document edits in place (`StructureNodePatch`); a field left out
/// is unchanged, and an empty value clears it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodePatch {
    pub title: Option<String>,
    pub intent: Option<String>,
    pub level: Option<String>,
    pub repo: Option<String>,
    /// Links the local clone at this folder: the core creates or reuses its repository.
    pub dir: Option<String>,
    pub folder: Option<String>,
    pub kind: Option<String>,
    pub tech: Option<String>,
}

/// A deploy as a component's document writes it (`StructureDeployInput`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeployInput {
    pub environment_id: String,
    pub provider: String,
    pub resource: String,
    /// The platform's id for the resource; empty until linked.
    pub target_id: String,
}

/// A new node: a domain under the project, or a component under a domain (`StructureDraft`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodeDraft {
    pub title: String,
    pub intent: String,
    pub why: String,
    pub level: String,
    /// The parent node's id; empty is the project.
    pub parent_id: String,
}

/// A card, a decision, a Rule, or a doc a node's document lists (`StructureRecordKind`), in
/// the order its sections show them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecordKind {
    Rule,
    Card,
    Decision,
    Doc,
}

impl RecordKind {
    pub const ALL: [RecordKind; 4] = [
        RecordKind::Rule,
        RecordKind::Card,
        RecordKind::Decision,
        RecordKind::Doc,
    ];
}

/// The deploy targets a platform lists for a node (`listDeployTargets`), read for the deploy
/// Composer's Target.
#[derive(Clone, Debug, PartialEq)]
pub struct DeployTargets {
    pub node_id: String,
    pub provider: String,
    pub targets: Load<(Vec<DeployTarget>, String)>,
}

/// `structureNodeText`: a node's heading and its `level:`, with `under:` when it is placed (an
/// empty one puts it right under the project), then its intent as the paragraph after.
pub fn polter_node(
    heading: &str,
    title: &str,
    level: &str,
    under: Option<&str>,
    intent: &str,
) -> String {
    let mut meta = vec![format!("level: {level}")];
    if let Some(under) = under {
        meta.push(if under.is_empty() {
            "under:".into()
        } else {
            format!("under: {under}")
        });
    }
    let intent = intent.trim();
    let intent = if intent.is_empty() {
        String::new()
    } else {
        format!("\n{intent}\n")
    };
    format!(
        "{POLTER_HEADER}# {heading}: {}\n{}\n{intent}",
        polter_value(title),
        meta.join(" · ")
    )
}

/// `persistStructureNode`: the node's block with the lines `patch` changes. A domain's level
/// clears its kind with it; Polter keeps the stored intent when the paragraph is empty.
pub fn polter_node_patch(node: &StructureNode, patch: &NodePatch) -> String {
    let title = patch.title.as_deref().unwrap_or(&node.title);
    let level = patch.level.as_deref().unwrap_or(&node.level);
    let mut text = polter_node(&node.path, title, level, None, "");
    let lines = [
        ("repo", &patch.repo, false),
        ("dir", &patch.dir, false),
        ("path", &patch.folder, false),
        ("kind", &patch.kind, false),
        ("tech", &patch.tech, true),
    ];
    for (key, value, quoted) in lines {
        if let Some(value) = value {
            let value = if quoted {
                polter_value(value)
            } else {
                value.clone()
            };
            text.push_str(&format!("{key}: {value}\n"));
        }
    }
    if let Some(intent) = patch.intent.as_deref().map(str::trim)
        && !intent.is_empty()
    {
        text.push_str(&format!("\n{intent}\n"));
    }
    text
}

/// `persistStructureUpdateDeploys`: the node's `deploys:` list
/// (`environment=provider:resource[@target]`); an empty one clears it.
pub fn polter_deploys(node: &StructureNode, deploys: &[DeployInput]) -> String {
    let links: Vec<String> = deploys
        .iter()
        .map(|deploy| {
            let target = if deploy.target_id.is_empty() {
                String::new()
            } else {
                format!("@{}", deploy.target_id)
            };
            format!(
                "{}={}:{}{target}",
                deploy.environment_id, deploy.provider, deploy.resource
            )
        })
        .collect();
    let line = if links.is_empty() {
        "deploys:\n".to_string()
    } else {
        format!("deploys: {}\n", links.join(", "))
    };
    format!(
        "{}{line}",
        polter_node(&node.path, &node.title, &node.level, None, "")
    )
}

/// `uniqueStructureSlug`: the title as a slug no node's slug, path, or last path segment
/// takes, numbered from 2 when one does.
pub fn unique_structure_slug(nodes: &[StructureNode], title: &str) -> String {
    let mut base = String::new();
    for ch in title.trim().to_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            base.push(ch);
        } else if !base.ends_with('-') {
            base.push('-');
        }
    }
    let base = base.trim_matches('-');
    let base = if base.is_empty() { "node" } else { base };
    let taken = |slug: &str| {
        nodes.iter().any(|node| {
            node.slug == slug || node.path == slug || node.path.rsplit('/').next() == Some(slug)
        })
    };
    if !taken(base) {
        return base.to_string();
    }
    (2..)
        .map(|index| format!("{base}-{index}"))
        .find(|slug| !taken(slug))
        .unwrap_or_default()
}

/// `listValues`: a component's kinds or technologies, stored as a comma list, each once.
pub fn list_values(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for value in text
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !out.iter().any(|seen| seen == value) {
            out.push(value.to_string());
        }
    }
    out
}

impl AppState {
    pub fn structure(&self) -> &Load<Structure> {
        &self.structure
    }

    /// The node open beside the tree, on a Structure route; none is the project's document.
    pub fn structure_node(&self) -> Option<&str> {
        match &self.route {
            Route::Project {
                section: Section::Structure,
                page,
                ..
            } => page.as_deref(),
            _ => None,
        }
    }

    /// Opens node `id`'s document beside the tree (none: the project's), in place of the
    /// route's page and with no step of history, as `setSelectedNode` applies it silently.
    pub fn select_structure_node(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        let Route::Project {
            section: Section::Structure,
            page,
            ..
        } = &mut self.route
        else {
            return;
        };
        if *page == id {
            return;
        }
        *page = id;
        if let Some(slot) = self.history.get_mut(self.history_index) {
            *slot = self.route.clone();
        }
        cx.notify();
    }

    /// Opens a card, a decision, a Rule, or a doc a node's document lists, as its own page,
    /// so Back returns to Structure.
    pub fn open_structure_record(&mut self, kind: RecordKind, id: String, cx: &mut Context<Self>) {
        let Some(project) = self.project_id().map(str::to_string) else {
            return;
        };
        let (section, tab) = match kind {
            RecordKind::Card => (Section::Milestones, KnowledgeTab::default()),
            RecordKind::Decision => (Section::Knowledge, KnowledgeTab::Decisions),
            RecordKind::Rule => (Section::Knowledge, KnowledgeTab::Rules),
            RecordKind::Doc => (Section::Knowledge, KnowledgeTab::Docs),
        };
        self.navigate(
            Route::Project {
                id: project,
                section,
                tab,
                board: BoardRoute::default(),
                page: Some(id),
            },
            cx,
        );
    }

    /// Commander's Add structure node (`action.create-node`): opens Structure and its node
    /// Composer.
    pub fn add_structure_node(&mut self, cx: &mut Context<Self>) {
        if self.project_id().is_none() {
            return;
        }
        if self.section() != Some(Section::Structure) {
            self.go_section(Section::Structure, cx);
        }
        self.commander_open = false;
        self.add_node_request = true;
        cx.notify();
    }

    /// Whether Commander asked for the node Composer; Structure takes the request once its
    /// nodes are read.
    pub fn take_add_node_request(&mut self) -> bool {
        if !matches!(self.structure, Load::Ready(_)) {
            return false;
        }
        std::mem::take(&mut self.add_node_request)
    }

    /// A component's deploys are being checked.
    pub fn deploys_checking(&self) -> bool {
        self.deploys_checking
    }

    pub fn deploy_targets(&self) -> Option<&DeployTargets> {
        self.deploy_targets.as_ref()
    }

    /// Reads the project's Structure and what its documents show.
    pub fn load_structure(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        if !matches!(self.structure, Load::Ready(_)) {
            self.structure = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move { read_structure(&core, &project) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.project_id() != Some(wanted.as_str()) {
                    return;
                }
                state.structure = match result {
                    Ok(structure) => Load::Ready(structure),
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// Applies `text` under `summary` and `why`, then reads Structure, the scope's catalog, and
    /// the open node's Activity again; `done` hears whether it went through, and a refusal is a
    /// toast (`failed` when the core gave no reason).
    fn write_structure(
        &mut self,
        text: String,
        summary: String,
        why: String,
        failed: &'static str,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            done(false, cx);
            return;
        };
        let project = project.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.work().apply_polter(&ApplyPolterRequest {
                        project_id: project,
                        text,
                        change: Some(change(&summary, &why)),
                        session_label: "desktop".into(),
                        ..ApplyPolterRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                let ok = state.structure_written(result.map(|_| ()), failed, cx);
                done(ok, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// After a write: reads again what it changed, or says why it was refused.
    fn structure_written(
        &mut self,
        result: Result<(), CoreError>,
        failed: &'static str,
        cx: &mut Context<Self>,
    ) -> bool {
        match result {
            Ok(()) => {
                self.load_structure(cx);
                self.load_catalog(cx);
                if let Some(node) = self.structure_node().map(str::to_string) {
                    self.load_document_activity(node, cx);
                }
                true
            }
            Err(error) => {
                let message = match error {
                    CoreError::NotRunning => self.lost(CoreError::NotRunning, cx),
                    error => error.to_string(),
                };
                self.toast(
                    if message.is_empty() {
                        failed.to_string()
                    } else {
                        message
                    },
                    true,
                );
                false
            }
        }
    }

    fn structure_node_by_id(&self, id: &str) -> Option<StructureNode> {
        match &self.structure {
            Load::Ready(structure) => structure.nodes.iter().find(|node| node.id == id).cloned(),
            _ => None,
        }
    }

    /// `onCreate`: a new node under its parent, named by a slug free in the project; its
    /// document opens.
    pub fn create_structure_node(
        &mut self,
        draft: NodeDraft,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let Load::Ready(structure) = &self.structure else {
            done(false, cx);
            return;
        };
        let parent = structure
            .nodes
            .iter()
            .find(|node| node.id == draft.parent_id)
            .cloned();
        let slug = unique_structure_slug(&structure.nodes, &draft.title);
        let summary = match &parent {
            Some(parent) => format!("Add {} under {}", draft.title, parent.title),
            None => format!("Add {}", draft.title),
        };
        let text = polter_node(
            &slug,
            &draft.title,
            &draft.level,
            parent.as_ref().map(|parent| parent.path.as_str()),
            &draft.intent,
        );
        let this = cx.weak_entity();
        self.write_structure(
            text,
            summary,
            draft.why,
            "Could not save structure.",
            move |ok, cx| {
                if ok {
                    let _ = this.update(cx, |state, _| {
                        state.push_toast("Structure node added.", ToastTone::Success)
                    });
                }
                done(ok, cx);
            },
            cx,
        );
    }

    /// `onEditNode`: writes a node's title, description, level, checkout, kind, or technology.
    pub fn edit_structure_node(&mut self, id: &str, patch: NodePatch, cx: &mut Context<Self>) {
        let Some(node) = self.structure_node_by_id(id) else {
            return;
        };
        let field = if patch.title.is_some() {
            "Rename"
        } else if patch.intent.is_some() {
            "Describe"
        } else {
            "Update"
        };
        let text = polter_node_patch(&node, &patch);
        let summary = format!("{field} {}", node.title);
        self.write_structure(
            text,
            summary,
            String::new(),
            "Could not save the node.",
            |_, _| {},
            cx,
        );
    }

    /// `onMove`: puts a component under domain `parent_id`, or loose under the project.
    pub fn move_structure_node(&mut self, id: &str, parent_id: &str, cx: &mut Context<Self>) {
        let Some(node) = self.structure_node_by_id(id) else {
            return;
        };
        let parent = self.structure_node_by_id(parent_id);
        if !parent_id.is_empty() && parent.is_none() {
            self.toast("Missing structure node.", true);
            return;
        }
        let summary = match &parent {
            Some(parent) => format!("Move {} under {}", node.title, parent.title),
            None => {
                let from = self
                    .structure_node_by_id(&node.parent_id)
                    .map_or_else(|| "its domain".to_string(), |from| from.title);
                format!("Move {} out of {from}", node.title)
            }
        };
        let under = parent.map(|parent| parent.path).unwrap_or_default();
        let text = polter_node(&node.path, &node.title, &node.level, Some(&under), "");
        self.write_structure(
            text,
            summary,
            String::new(),
            "Could not move the node.",
            |_, _| {},
            cx,
        );
    }

    /// `onDeploysChange`: replaces a component's deploys, then checks them at once, so the
    /// platform's page and the latest deploy show without waiting.
    pub fn save_deploys(
        &mut self,
        id: &str,
        deploys: Vec<DeployInput>,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let Some(node) = self.structure_node_by_id(id) else {
            done(false, cx);
            return;
        };
        let text = polter_deploys(&node, &deploys);
        let summary = format!("Update deploys for {}", node.title);
        let this = cx.weak_entity();
        let checked = node.id.clone();
        self.write_structure(
            text,
            summary,
            String::new(),
            "Could not save structure.",
            move |ok, cx| {
                if ok {
                    let _ = this.update(cx, |state, cx| state.check_deploys(checked, true, cx));
                }
                done(ok, cx);
            },
            cx,
        );
    }

    /// `onCheckDeploys`: asks each platform's CLI for a component's latest deploys; the core
    /// skips links checked in the last minute unless `force`. A failed check the person asked
    /// for is a toast.
    pub fn check_deploys(&mut self, id: String, force: bool, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        self.deploys_checking = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.integration().check_deploys(&CheckDeploysRequest {
                        project_id: project,
                        node_id: id,
                        force,
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.deploys_checking = false;
                match result {
                    Ok(_) => state.load_structure(cx),
                    Err(error) if force => {
                        let message = error.to_string();
                        state.toast(
                            if message.is_empty() {
                                "Could not check the deploys.".to_string()
                            } else {
                                message
                            },
                            true,
                        );
                    }
                    Err(_) => {}
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Lists what a deploy on `provider` can link to for node `id`: the targets its checkout
    /// declares, then the account's. A failure is the list's message.
    pub fn load_deploy_targets(&mut self, id: String, provider: String, cx: &mut Context<Self>) {
        if self
            .deploy_targets
            .as_ref()
            .is_some_and(|listed| listed.node_id == id && listed.provider == provider)
        {
            return;
        }
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        self.deploy_targets = Some(DeployTargets {
            node_id: id.clone(),
            provider: provider.clone(),
            targets: Load::Loading,
        });
        cx.notify();
        cx.spawn(async move |this, cx| {
            let (node, platform) = (id.clone(), provider.clone());
            let result = cx
                .background_spawn(async move {
                    core.integration()
                        .list_deploy_targets(&ListDeployTargetsRequest {
                            project_id: project,
                            provider: platform,
                            node_id: node,
                        })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                let Some(listed) = state.deploy_targets.as_mut() else {
                    return;
                };
                if listed.node_id != id || listed.provider != provider {
                    return;
                }
                listed.targets = Load::Ready(match result {
                    Ok(response) => (response.targets, response.message),
                    Err(error) => (Vec::new(), {
                        let message = error.to_string();
                        if message.is_empty() {
                            "Could not list the platform's projects.".to_string()
                        } else {
                            message
                        }
                    }),
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Forgets the targets read for the deploy Composer, as it closes.
    pub fn clear_deploy_targets(&mut self) {
        self.deploy_targets = None;
    }

    /// `onLinkRecord`: a card moves under the node, or out of any component; a decision, a
    /// Rule, or a doc gains or loses this node among its contexts and keeps the others.
    pub fn link_structure_record(
        &mut self,
        kind: RecordKind,
        record: String,
        node_id: &str,
        linked: bool,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let Load::Ready(structure) = &self.structure else {
            return;
        };
        let node = structure.nodes.iter().find(|node| node.id == node_id);
        let with_context = |contexts: &[String]| -> Vec<String> {
            let mut next: Vec<String> = contexts
                .iter()
                .filter(|id| *id != node_id)
                .cloned()
                .collect();
            if linked {
                next.push(node_id.to_string());
            }
            next
        };
        enum Write {
            Polter(String, String),
            Scope(&'static str, Vec<String>),
        }
        let write = match kind {
            RecordKind::Card => {
                let title = structure
                    .cards
                    .iter()
                    .find(|card| card.slug == record)
                    .map_or(record.clone(), |card| card.title.clone());
                let (under, summary) = match node.filter(|_| linked) {
                    Some(node) => (
                        node.path.clone(),
                        format!("Put {title} under {}", node.title),
                    ),
                    None => (String::new(), format!("Take {title} out of its component")),
                };
                Write::Polter(polter_card(&record, &[("under", &under)]), summary)
            }
            RecordKind::Rule => {
                let contexts = structure
                    .rules
                    .iter()
                    .find(|rule| rule.id == record)
                    .map(|rule| rule.scope_node_ids.clone())
                    .unwrap_or_default();
                Write::Scope("rule", with_context(&contexts))
            }
            RecordKind::Decision | RecordKind::Doc => {
                let contexts = structure
                    .items
                    .iter()
                    .find(|item| item.id == record)
                    .map(|item| item.scope_node_ids.clone())
                    .unwrap_or_default();
                Write::Scope("knowledge", with_context(&contexts))
            }
        };
        let project = project.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    match write {
                        Write::Polter(text, summary) => core
                            .work()
                            .apply_polter(&ApplyPolterRequest {
                                project_id: project,
                                text,
                                change: Some(change(&summary, "")),
                                session_label: "desktop".into(),
                                ..ApplyPolterRequest::default()
                            })
                            .map(|_| ()),
                        Write::Scope(kind, nodes) => core
                            .work()
                            .set_scope(&SetScopeRequest {
                                project_id: project,
                                kind: kind.into(),
                                id: record,
                                nodes,
                            })
                            .map(|_| ()),
                    }
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                let failed = if linked {
                    "Could not link it."
                } else {
                    "Could not unlink it."
                };
                state.structure_written(result, failed, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// `onEditProject`: renames the project or rewrites its description, keeping the rest of
    /// its settings.
    pub fn edit_structure_project(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let Load::Ready(structure) = &self.structure else {
            return;
        };
        let project = project.to_string();
        let name = name.unwrap_or_else(|| {
            self.project()
                .map(|project| project.name.clone())
                .unwrap_or_default()
        });
        let settings = ProjectSettings {
            project_id: project.clone(),
            description: description.unwrap_or_else(|| structure.settings.description.clone()),
            ..structure.settings.clone()
        };
        let failed = "Could not save the project.";
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.project().save_project(&SaveProjectRequest {
                        project: Some(Project {
                            id: project,
                            name,
                            ..Project::default()
                        }),
                    })?;
                    core.project()
                        .save_project_settings(&SaveProjectSettingsRequest {
                            settings: Some(settings),
                        })
                        .map(|_| ())
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.structure_written(result, failed, cx) {
                    state.load_projects(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }
}

/// Structure's reads, made together as the web workspace makes them.
fn read_structure(core: &Core, project: &str) -> Result<Structure, CoreError> {
    let project_id = || project.to_string();
    let nodes = core
        .work()
        .list_structure(&ListStructureRequest {
            project_id: project_id(),
            ..ListStructureRequest::default()
        })?
        .nodes;
    let repos = core
        .project()
        .list_project_repos(&ListProjectReposRequest {
            project_id: project_id(),
        })?
        .repos;
    let cards = core
        .work()
        .list_work_cards(&ListWorkCardsRequest {
            project_id: project_id(),
            ..ListWorkCardsRequest::default()
        })?
        .cards;
    let items = core
        .knowledge()
        .list_knowledge(&ListKnowledgeRequest {
            project_id: project_id(),
            ..ListKnowledgeRequest::default()
        })?
        .items;
    let rules = core
        .work()
        .list_rules(&ListRulesRequest {
            project_id: project_id(),
            ..ListRulesRequest::default()
        })?
        .rules;
    // The platform connections only mark a deploy's CLI as not connected.
    let integrations = core
        .integration()
        .list_integration_connections(&ListIntegrationConnectionsRequest {
            project_id: project_id(),
        })
        .map(|response| response.connections)
        .unwrap_or_default();
    let environments = core
        .project()
        .list_environments(&ListEnvironmentsRequest {
            project_id: project_id(),
        })?
        .environments;
    let settings = core
        .project()
        .get_project_settings(&GetProjectSettingsRequest {
            project_id: project_id(),
        })?
        .settings
        .unwrap_or_default();
    Ok(Structure {
        nodes,
        repos,
        cards,
        items,
        rules,
        integrations,
        environments,
        settings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, slug: &str, path: &str) -> StructureNode {
        StructureNode {
            id: id.into(),
            slug: slug.into(),
            path: path.into(),
            title: "Billing".into(),
            level: "domain".into(),
            ..StructureNode::default()
        }
    }

    #[test]
    fn a_node_is_written_as_an_agent_writes_it() {
        assert_eq!(
            polter_node(
                "billing",
                "Billing · core",
                "domain",
                None,
                "  Owns money.  "
            ),
            "---\npolter: 1\n---\n\n# billing: Billing, core\nlevel: domain\n\nOwns money.\n"
        );
        assert_eq!(
            polter_node("billing/invoices", "Invoices", "component", Some(""), ""),
            "---\npolter: 1\n---\n\n# billing/invoices: Invoices\nlevel: component · under:\n"
        );
        let invoices = StructureNode {
            level: "component".into(),
            ..node("n1", "invoices", "billing/invoices")
        };
        let patch = NodePatch {
            folder: Some("services/invoices".into()),
            tech: Some("Go · Postgres".into()),
            ..NodePatch::default()
        };
        assert_eq!(
            polter_node_patch(&invoices, &patch),
            "---\npolter: 1\n---\n\n# billing/invoices: Billing\nlevel: component\npath: services/invoices\ntech: Go, Postgres\n"
        );
        let deploys = [
            DeployInput {
                environment_id: "prod".into(),
                provider: "vercel".into(),
                resource: "web".into(),
                target_id: "prj_1".into(),
            },
            DeployInput {
                environment_id: "staging".into(),
                provider: "fly".into(),
                resource: "api".into(),
                target_id: String::new(),
            },
        ];
        assert!(
            polter_deploys(&invoices, &deploys)
                .ends_with("level: component\ndeploys: prod=vercel:web@prj_1, staging=fly:api\n")
        );
        assert!(polter_deploys(&invoices, &[]).ends_with("level: component\ndeploys:\n"));
    }

    #[test]
    fn a_new_slug_is_free_across_the_project() {
        let nodes = vec![
            node("a", "billing", "billing"),
            node("b", "invoices", "billing/invoices"),
            node("c", "billing-2", "billing-2"),
        ];
        assert_eq!(unique_structure_slug(&nodes, "Billing"), "billing-3");
        assert_eq!(unique_structure_slug(&nodes, "Invoices!"), "invoices-2");
        assert_eq!(
            unique_structure_slug(&nodes, "  Payouts & Refunds "),
            "payouts-refunds"
        );
        assert_eq!(unique_structure_slug(&nodes, "???"), "node");
        assert_eq!(
            list_values(" service, database,,service "),
            ["service", "database"]
        );
    }
}
