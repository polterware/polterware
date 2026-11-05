//! Canvas's state (`CanvasView`, `canvas-workspace.tsx`, `canvasPersistence` in
//! `workspace/persist.ts`): the project's canvases and the types the core reports, the
//! archived ones, the records a node can cite (cards, decisions, Structure nodes, and Facts),
//! and the canvas open in the editor, which lives outside the route as it does in Electron:
//! Back leaves it open, and choosing Canvas again closes it. The canvas open last in a
//! project opens again when Canvas does, while the app runs.
//!
//! Every write goes through `CanvasService` under the change the Electron Desktop records:
//! `Create canvas: <name>`, `Update canvas`, `Archive canvas`, `Restore canvas`.

use std::collections::HashMap;

use gpui::{AppContext as _, Context, Task};
use polterware_ui::toast::ToastTone;

use super::{AppState, Load, Section, change};
use crate::canvas::references::{
    PromotionKind, StoreEntity, created_decision_title, created_fact_id, promotion_rationale,
    promotion_text,
};
use crate::canvas::structure::{NAME as STRUCTURE_NAME, draw_structure, is_structure_canvas};
use crate::client::{Core, CoreError};
use crate::contracts::{
    ApplyPolterRequest, Canvas, CanvasGraph, CanvasTypeDefinition, CreateCanvasRequest,
    DeleteCanvasRequest, GetCanvasRequest, ListCanvasTypesRequest, ListCanvasesRequest,
    ListFactsRequest, ListKnowledgeRequest, ListStructureRequest, ListWorkCardsRequest,
    SetCanvasArchivedRequest, StructureNode, UpdateCanvasRequest,
};

/// The canvases and the types, read together.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasList {
    pub canvases: Vec<Canvas>,
    pub types: Vec<CanvasTypeDefinition>,
}

/// The canvas open in the editor, with its type.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenCanvas {
    pub canvas: Canvas,
    pub definition: CanvasTypeDefinition,
    /// Bumped on every open, so the editor knows a fresh document arrived.
    pub generation: u64,
}

/// A canvas's fields an edit changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasPatch {
    pub name: Option<String>,
    pub agent_visible: Option<bool>,
    pub graph: Option<CanvasGraph>,
    pub layout_json: Option<String>,
}

pub struct CanvasState {
    list: Load<CanvasList>,
    archived: Load<Vec<Canvas>>,
    entities: Load<Vec<StoreEntity>>,
    open: Option<OpenCanvas>,
    generation: u64,
    /// The canvas open last in each project.
    remembered: HashMap<String, String>,
    /// Whether the remembered canvas was tried since the list was read.
    restored: bool,
    busy: bool,
    /// Structure's Draw as canvas is writing.
    drawing: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            list: Load::Idle,
            archived: Load::Idle,
            entities: Load::Idle,
            open: None,
            generation: 0,
            remembered: HashMap::new(),
            restored: false,
            busy: false,
            drawing: false,
        }
    }
}

fn read_list(core: &Core, project: &str) -> Result<CanvasList, CoreError> {
    let canvases = core
        .canvas()
        .list_canvases(&ListCanvasesRequest {
            project_id: project.to_string(),
            ..ListCanvasesRequest::default()
        })?
        .canvases;
    let types = core
        .canvas()
        .list_canvas_types(&ListCanvasTypesRequest::default())?
        .types;
    Ok(CanvasList { canvases, types })
}

/// What a node can cite: the cards, the decisions, the Structure nodes, and the Facts.
fn read_entities(core: &Core, project: &str) -> Result<Vec<StoreEntity>, CoreError> {
    let cards = core
        .work()
        .list_work_cards(&ListWorkCardsRequest {
            project_id: project.to_string(),
            ..ListWorkCardsRequest::default()
        })?
        .cards;
    let items = core
        .knowledge()
        .list_knowledge(&ListKnowledgeRequest {
            project_id: project.to_string(),
            ..ListKnowledgeRequest::default()
        })?
        .items;
    let nodes = core
        .work()
        .list_structure(&ListStructureRequest {
            project_id: project.to_string(),
            ..ListStructureRequest::default()
        })?
        .nodes;
    // A Facts failure leaves the other records citable, as the Electron Desktop shows it apart.
    let facts = core
        .project_model()
        .list_facts(&ListFactsRequest {
            project_id: project.to_string(),
            ..ListFactsRequest::default()
        })
        .map(|response| response.facts)
        .unwrap_or_default();
    let mut out: Vec<StoreEntity> = cards
        .into_iter()
        .map(|card| StoreEntity {
            kind: "card".into(),
            id: card.id,
            title: card.title,
            slug: card.slug,
            ..StoreEntity::default()
        })
        .collect();
    out.extend(
        items
            .into_iter()
            .filter(|item| item.kind == "decision")
            .map(|item| StoreEntity {
                kind: "decision".into(),
                id: item.id,
                title: item.title,
                ..StoreEntity::default()
            }),
    );
    out.extend(nodes.into_iter().map(|node| StoreEntity {
        kind: "node".into(),
        id: node.id,
        title: node.title,
        level: node.level,
        component_kind: node.kind,
        tech: node.tech,
        ..StoreEntity::default()
    }));
    out.extend(facts.into_iter().map(|fact| StoreEntity {
        kind: "fact".into(),
        id: fact.id,
        title: fact.statement,
        ..StoreEntity::default()
    }));
    Ok(out)
}

fn update_request(project: &str, id: &str, patch: CanvasPatch) -> UpdateCanvasRequest {
    UpdateCanvasRequest {
        project_id: project.to_string(),
        id: id.to_string(),
        change: Some(change(
            "Update canvas",
            "Save canvas edits from the Desktop editor.",
        )),
        name: patch.name,
        agent_visible: patch.agent_visible,
        graph: patch.graph,
        layout_json: patch.layout_json,
        ..UpdateCanvasRequest::default()
    }
}

impl AppState {
    pub fn canvas_list(&self) -> &Load<CanvasList> {
        &self.canvas.list
    }

    pub fn archived_canvases(&self) -> &Load<Vec<Canvas>> {
        &self.canvas.archived
    }

    pub fn canvas_entities(&self) -> &[StoreEntity] {
        match &self.canvas.entities {
            Load::Ready(entities) => entities,
            _ => &[],
        }
    }

    pub fn open_canvas(&self) -> Option<&OpenCanvas> {
        self.canvas.open.as_ref()
    }

    pub fn canvas_busy(&self) -> bool {
        self.canvas.busy
    }

    /// Whether Structure's Draw as canvas is writing.
    pub fn drawing_structure(&self) -> bool {
        self.canvas.drawing
    }

    /// The open canvas's name, for the breadcrumb after Canvas.
    pub fn open_canvas_name(&self) -> Option<&str> {
        (self.section() == Some(Section::Canvas))
            .then(|| {
                self.canvas
                    .open
                    .as_ref()
                    .map(|open| open.canvas.name.as_str())
            })
            .flatten()
    }

    /// A new project's canvases are read afresh; what was open in each project is kept.
    pub(super) fn forget_canvases(&mut self) {
        let remembered = std::mem::take(&mut self.canvas.remembered);
        self.canvas = CanvasState {
            remembered,
            ..CanvasState::default()
        };
    }

    /// Reads the canvases and the types, and the records a node can cite.
    pub fn load_canvases(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        if !matches!(self.canvas.list, Load::Ready(_)) {
            self.canvas.list = Load::Loading;
        }
        let entities_core = core.clone();
        let entities_project = project.clone();
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move { read_list(&core, &project) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.project_id() != Some(wanted.as_str()) {
                    return;
                }
                state.canvas.list = match result {
                    Ok(list) => Load::Ready(list),
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                state.restore_canvas(cx);
                cx.notify();
            });
        })
        .detach();
        cx.spawn(async move |this, cx| {
            let wanted = entities_project.clone();
            let result = cx
                .background_spawn(async move { read_entities(&entities_core, &entities_project) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.project_id() != Some(wanted.as_str()) {
                    return;
                }
                state.canvas.entities = match result {
                    Ok(entities) => Load::Ready(entities),
                    Err(error) => Load::Failed(error.to_string()),
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// Reopens the canvas open last in this project once its list arrives, while it exists.
    fn restore_canvas(&mut self, cx: &mut Context<Self>) {
        if self.canvas.restored || self.canvas.open.is_some() {
            return;
        }
        let Load::Ready(list) = &self.canvas.list else {
            return;
        };
        self.canvas.restored = true;
        let Some(project) = self.project_id() else {
            return;
        };
        if let Some(id) = self.canvas.remembered.get(project).cloned()
            && list.canvases.iter().any(|canvas| canvas.id == id)
        {
            self.open_canvas_by_id(id, cx);
        }
    }

    pub fn load_archived_canvases(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        self.canvas.archived = Load::Loading;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.canvas().list_canvases(&ListCanvasesRequest {
                        project_id: project,
                        archived: true,
                        ..ListCanvasesRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.canvas.archived = match result {
                    Ok(response) => Load::Ready(response.canvases),
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// Opens canvas `id` in the editor, read afresh with its type.
    pub fn open_canvas_by_id(&mut self, id: String, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        self.canvas.busy = true;
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move {
                    core.canvas().get_canvas(&GetCanvasRequest {
                        project_id: project,
                        id,
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.canvas.busy = false;
                if state.project_id() != Some(wanted.as_str()) {
                    return;
                }
                match result.map(|response| response.canvas.unwrap_or_default()) {
                    Ok(canvas) => state.show_canvas(canvas, cx),
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

    fn show_canvas(&mut self, canvas: Canvas, cx: &mut Context<Self>) {
        let definition = match &self.canvas.list {
            Load::Ready(list) => list
                .types
                .iter()
                .find(|definition| definition.r#type == canvas.r#type)
                .cloned(),
            _ => None,
        };
        let Some(definition) = definition else {
            self.toast(
                "The server did not report this canvas type. Reload types before editing.",
                true,
            );
            return;
        };
        if let Some(project) = self.project_id().map(str::to_string) {
            self.canvas.remembered.insert(project, canvas.id.clone());
        }
        self.canvas.generation += 1;
        self.canvas.open = Some(OpenCanvas {
            canvas,
            definition,
            generation: self.canvas.generation,
        });
        cx.notify();
    }

    /// Back to the list (the editor saved before asking), read afresh.
    pub fn close_canvas(&mut self, cx: &mut Context<Self>) {
        if self.canvas.open.take().is_none() {
            return;
        }
        if let Some(project) = self.project_id().map(str::to_string) {
            self.canvas.remembered.remove(&project);
        }
        self.load_canvases(cx);
        cx.notify();
    }

    /// Creates a canvas of the core's one type and opens it; `done` hears whether it was made.
    pub fn create_canvas(
        &mut self,
        name: String,
        done: impl FnOnce(bool, &mut gpui::App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            done(false, cx);
            return;
        };
        let Some(definition) = (match &self.canvas.list {
            Load::Ready(list) => list.types.first().cloned(),
            _ => None,
        }) else {
            self.toast("The server reported no canvas type.", true);
            done(false, cx);
            return;
        };
        let project = project.to_string();
        self.canvas.busy = true;
        cx.spawn(async move |this, cx| {
            let request = CreateCanvasRequest {
                project_id: project,
                name: name.clone(),
                r#type: definition.r#type.clone(),
                agent_visible: false,
                graph: Some(CanvasGraph::default()),
                layout_json: "{}".into(),
                change: Some(change(
                    &format!("Create canvas: {name}"),
                    "Start a local canvas for visual exploration.",
                )),
                ..CreateCanvasRequest::default()
            };
            let result = cx
                .background_spawn(async move { core.canvas().create_canvas(&request) })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.canvas.busy = false;
                match result.map(|response| response.canvas.unwrap_or_default()) {
                    Ok(canvas) => {
                        state.show_canvas(canvas, cx);
                        state.load_canvases(cx);
                        done(true, cx);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                        done(false, cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Writes `patch` to canvas `id`, then reads the list again; `done` hears whether it held.
    pub fn update_canvas(
        &mut self,
        id: String,
        patch: CanvasPatch,
        done: impl FnOnce(bool, &mut gpui::App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            done(false, cx);
            return;
        };
        let request = update_request(project, &id, patch);
        self.canvas.busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { core.canvas().update_canvas(&request) })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.canvas.busy = false;
                match result {
                    Ok(_) => {
                        state.load_canvases(cx);
                        done(true, cx);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                        done(false, cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The editor's save: writes `patch` to the open canvas of `project`, whatever is open by
    /// the time it lands, and answers with the core's refusal, if any.
    pub fn save_canvas(
        &self,
        project: String,
        id: String,
        patch: CanvasPatch,
        cx: &mut Context<Self>,
    ) -> Task<Result<(), String>> {
        let Some(core) = self.ready_core() else {
            return Task::ready(Err(CoreError::NotRunning.to_string()));
        };
        let request = update_request(&project, &id, patch);
        cx.background_spawn(async move {
            core.canvas()
                .update_canvas(&request)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
    }

    /// Archives a canvas at once; archiving is reversible, so it asks nothing.
    pub fn archive_canvas(&mut self, id: String, name: String, cx: &mut Context<Self>) {
        self.set_canvas_archived(id, name, true, cx);
    }

    /// Restores an archived canvas and says so.
    pub fn restore_canvas_by_id(&mut self, id: String, name: String, cx: &mut Context<Self>) {
        self.set_canvas_archived(id, name, false, cx);
    }

    fn set_canvas_archived(
        &mut self,
        id: String,
        name: String,
        archived: bool,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        self.canvas.busy = true;
        cx.spawn(async move |this, cx| {
            let request = SetCanvasArchivedRequest {
                project_id: project,
                id,
                archived,
                change: Some(change(
                    if archived {
                        "Archive canvas"
                    } else {
                        "Restore canvas"
                    },
                    "",
                )),
            };
            let result = cx
                .background_spawn(async move { core.canvas().set_canvas_archived(&request) })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.canvas.busy = false;
                match result {
                    Ok(_) => {
                        state.load_canvases(cx);
                        if !matches!(state.canvas.archived, Load::Idle) {
                            state.load_archived_canvases(cx);
                        }
                        let verb = if archived { "Archived" } else { "Restored" };
                        state.push_toast(format!("{verb} \u{201c}{name}\u{201d}"), ToastTone::Info);
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

    /// Deletes an archived canvas for good (asked first by the list).
    pub fn delete_canvas(&mut self, id: String, title: String, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        self.canvas.busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.canvas().delete_canvas(&DeleteCanvasRequest {
                        project_id: project,
                        id,
                        change: None,
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.canvas.busy = false;
                match result {
                    Ok(_) => {
                        state.load_archived_canvases(cx);
                        state.push_toast(
                            format!("Deleted \u{201c}{title}\u{201d}"),
                            ToastTone::Info,
                        );
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

    /// Promotes a drawing to a card, a decision, or a Fact through `ApplyPolter`, and answers
    /// with the record created (`canvasPersistence.promote`).
    pub fn promote_canvas_item(
        &self,
        kind: PromotionKind,
        title: String,
        slug: String,
        canvas_name: String,
        cx: &mut Context<Self>,
    ) -> Task<Result<StoreEntity, String>> {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return Task::ready(Err(CoreError::NotRunning.to_string()));
        };
        let project = project.to_string();
        cx.background_spawn(async move {
            let text = promotion_text(kind, &title, &slug, &canvas_name)?;
            let apply = |dry_run: bool| {
                core.work()
                    .apply_polter(&ApplyPolterRequest {
                        project_id: project.clone(),
                        text: text.clone(),
                        dry_run,
                        change: (!dry_run).then(|| {
                            change(
                                &format!("Promote canvas item: {title}"),
                                &promotion_rationale(kind, &canvas_name),
                            )
                        }),
                        session_label: "desktop".into(),
                        ..ApplyPolterRequest::default()
                    })
                    .map_err(|error| error.to_string())
            };
            let errors = |diagnostics: &[crate::contracts::PolterDiagnostic]| {
                diagnostics
                    .iter()
                    .filter(|item| item.severity == "error")
                    .map(|item| item.message.clone())
                    .collect::<Vec<_>>()
            };
            if kind == PromotionKind::Card {
                let cards = core
                    .work()
                    .list_work_cards(&ListWorkCardsRequest {
                        project_id: project.clone(),
                        ..ListWorkCardsRequest::default()
                    })
                    .map_err(|error| error.to_string())?
                    .cards;
                if cards.iter().any(|card| card.slug == slug) {
                    return Err("That Work card slug already exists. Choose another slug.".into());
                }
            }
            if kind != PromotionKind::Decision {
                // Removed cards and Facts still own their slug in Polter.
                let plan = apply(true)?;
                let refused = errors(&plan.diagnostics);
                if !refused.is_empty() {
                    return Err(refused.join("\n"));
                }
                let prefix = match kind {
                    PromotionKind::Card => format!("+ card {slug} "),
                    _ => "+ fact ".into(),
                };
                if !plan.ops.iter().any(|op| op.starts_with(&prefix)) {
                    return Err(if kind == PromotionKind::Card {
                        "That slug already belongs to a Work card, including removed cards. Choose another slug.".into()
                    } else {
                        "That slug already belongs to a Fact. Choose another slug.".into()
                    });
                }
            }
            let applied = apply(false)?;
            let refused = errors(&applied.diagnostics);
            if !refused.is_empty() || !applied.applied {
                return Err(if refused.is_empty() {
                    "ApplyPolter did not apply the promotion.".into()
                } else {
                    refused.join("\n")
                });
            }
            let linked = || -> Result<StoreEntity, String> {
                match kind {
                    PromotionKind::Card => {
                        let cards = core
                            .work()
                            .list_work_cards(&ListWorkCardsRequest {
                                project_id: project.clone(),
                                ..ListWorkCardsRequest::default()
                            })
                            .map_err(|error| error.to_string())?
                            .cards;
                        let card = cards
                            .into_iter()
                            .find(|card| card.slug == slug && !card.id.is_empty())
                            .ok_or("The new Work card was not returned.")?;
                        Ok(StoreEntity {
                            kind: "card".into(),
                            id: card.id,
                            title: card.title,
                            slug: card.slug,
                            ..StoreEntity::default()
                        })
                    }
                    PromotionKind::Fact => {
                        let id = created_fact_id(&applied.ops)
                            .ok_or("ApplyPolter returned no created Fact identity.")?;
                        let facts = core
                            .project_model()
                            .list_facts(&ListFactsRequest {
                                project_id: project.clone(),
                                ..ListFactsRequest::default()
                            })
                            .map_err(|error| error.to_string())?
                            .facts;
                        let fact = facts
                            .into_iter()
                            .find(|fact| fact.id == id)
                            .ok_or("The new Fact was not returned.")?;
                        Ok(StoreEntity {
                            kind: "fact".into(),
                            id,
                            title: fact.statement,
                            slug: slug.clone(),
                            ..StoreEntity::default()
                        })
                    }
                    PromotionKind::Decision => {
                        let title = created_decision_title(&applied.ops)
                            .ok_or("ApplyPolter returned no created Decision identity.")?;
                        let items = core
                            .knowledge()
                            .list_knowledge(&ListKnowledgeRequest {
                                project_id: project.clone(),
                                ..ListKnowledgeRequest::default()
                            })
                            .map_err(|error| error.to_string())?
                            .items;
                        let decision = items
                            .into_iter()
                            .find(|item| {
                                item.kind == "decision" && item.title == title && !item.id.is_empty()
                            })
                            .ok_or("The new Decision was not returned.")?;
                        Ok(StoreEntity {
                            kind: "decision".into(),
                            id: decision.id,
                            title: decision.title,
                            ..StoreEntity::default()
                        })
                    }
                }
            };
            linked().map_err(|error| {
                format!(
                    "The entity was created, but could not be linked. Reload project data and link it using the picker. {error}"
                )
            })
        })
    }

    /// Adds a promoted record to what nodes can cite, until the next read has it.
    pub fn add_canvas_entity(&mut self, entity: StoreEntity) {
        if let Load::Ready(entities) = &mut self.canvas.entities
            && !entities
                .iter()
                .any(|item| item.kind == entity.kind && item.id == entity.id)
        {
            entities.push(entity);
        }
    }

    /// Structure's Draw as canvas: draws the tree on the project's Structure canvas, creating
    /// it the first time, agent-visible, and remembers it as the one Canvas opens on.
    pub fn draw_structure_canvas(&mut self, nodes: Vec<StructureNode>, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        if self.canvas.drawing {
            return;
        }
        self.canvas.drawing = true;
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move {
                    let listed = core
                        .canvas()
                        .list_canvases(&ListCanvasesRequest {
                            project_id: project.clone(),
                            ..ListCanvasesRequest::default()
                        })?
                        .canvases
                        .into_iter()
                        .find(is_structure_canvas);
                    let canvas = match listed {
                        Some(listed) => core
                            .canvas()
                            .get_canvas(&GetCanvasRequest {
                                project_id: project.clone(),
                                id: listed.id,
                            })?
                            .canvas
                            .unwrap_or_default(),
                        None => core
                            .canvas()
                            .create_canvas(&CreateCanvasRequest {
                                project_id: project.clone(),
                                name: STRUCTURE_NAME.into(),
                                r#type: "canvas".into(),
                                agent_visible: false,
                                graph: Some(CanvasGraph::default()),
                                layout_json: "{}".into(),
                                change: Some(change(
                                    &format!("Create canvas: {STRUCTURE_NAME}"),
                                    "Start a local canvas for visual exploration.",
                                )),
                                ..CreateCanvasRequest::default()
                            })?
                            .canvas
                            .unwrap_or_default(),
                    };
                    let (graph, layout_json) = draw_structure(&nodes, Some(&canvas));
                    core.canvas().update_canvas(&update_request(
                        &project,
                        &canvas.id,
                        CanvasPatch {
                            agent_visible: Some(true),
                            graph: Some(graph),
                            layout_json: Some(layout_json),
                            ..CanvasPatch::default()
                        },
                    ))?;
                    Ok::<_, CoreError>(canvas.id)
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.canvas.drawing = false;
                match result {
                    Ok(id) => {
                        if state.project_id() == Some(wanted.as_str()) {
                            state.canvas.remembered.insert(wanted, id);
                            state.canvas.restored = false;
                            state.canvas.list = Load::Idle;
                        }
                        state.push_toast("Structure drawn on its canvas.", ToastTone::Success);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(
                            if message.is_empty() {
                                "Could not draw the canvas.".to_string()
                            } else {
                                message
                            },
                            true,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
