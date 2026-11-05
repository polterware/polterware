//! The shared Composer (`KnowledgeComposer` in `composers.tsx`, hosted by `App.tsx`): the one
//! composer C, Commander's New card, New decision, and New doc, Milestones' "+", and
//! Knowledge's "+" open over the page, for a card, a decision, or a doc, with what the place it
//! was asked from already knows. The app owns every write.
//!
//! A card (`CardComposer`) takes its title, its description as the intent, and four pills:
//! Stage (Idea unless asked for another), Milestone (the Backlog, or the board it was asked
//! from), Component (shown while the project has components), and Kind (Feature or
//! Architecture change); Create card writes it as a Polter block under a slug free across the
//! project. A decision (`DecisionComposer`) or a doc (`DocComposer`) takes its title, its
//! Context paragraph or Markdown body, and its Context as tokens, which start from the scope
//! filter's domains and components (`defaultContexts`). Each takes an optional reason, recorded
//! in Activity.

use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, Render, SharedString, Window, div,
    prelude::*, px,
};
use polterware_ui::icons::Icon;
use polterware_ui::patterns::{
    Composer, ComposerDraft, ComposerEvent, ComposerSpec, Property, PropertyOption,
};
use polterware_ui::theme::{StatusTone, Theme, tone_color};

use crate::contracts::{Milestone, StructureNode};
use crate::state::{AppState, ComposeRequest, Load, NewRecord};
use crate::views::knowledge::context_property;
use crate::views::milestones::stage_tone;
use crate::work::{self, NewCard, Stage};

/// The reason's placeholder (`WHY`).
const WHY: &str = "Optional. Recorded in Activity";

/// The card's pills while the Composer holds them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CardProps {
    stage: Option<Stage>,
    /// The milestone's id; empty is the Backlog.
    milestone: String,
    /// The component's path; empty is not placed yet.
    under: String,
    architecture: bool,
}

/// A status dot for a Composer option, in `tone`.
pub(crate) fn dot_option(value: &str, label: &str, tone: StatusTone) -> PropertyOption {
    PropertyOption::new(value.to_string(), label.to_string()).mark(move |size, _, cx| {
        let theme = Theme::global(cx);
        div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .size(px(6.0))
                    .rounded_full()
                    .bg(tone_color(theme, tone)),
            )
            .into_any_element()
    })
}

/// `CardComposer`'s pills: Stage, Milestone, Component while there are components, and Kind.
fn card_properties(
    card: &CardProps,
    milestones: &[Milestone],
    nodes: &[StructureNode],
) -> Vec<Property> {
    let stage = card.stage.unwrap_or(Stage::Shape);
    let stages = Stage::ALL
        .iter()
        .map(|stage| dot_option(stage.id(), stage.label(), stage_tone(*stage)))
        .collect();
    let listed = std::iter::once(PropertyOption::new("", "Backlog").icon(Icon::Inbox))
        .chain(
            work::listed_milestones(milestones)
                .into_iter()
                .map(|milestone| {
                    PropertyOption::new(milestone.id.clone(), milestone.name.clone())
                        .icon(Icon::Folder)
                }),
        )
        .collect();
    let mut properties = vec![
        Property::single("stage", "Stage", stages, stage.id()),
        Property::single("milestone", "Milestone", listed, card.milestone.clone())
            .icon(Icon::Folder),
    ];
    let components: Vec<&StructureNode> = nodes
        .iter()
        .filter(|node| node.level == "component")
        .collect();
    if !components.is_empty() {
        let options = std::iter::once(PropertyOption::new("", "No component").icon(Icon::Close))
            .chain(components.iter().map(|node| {
                PropertyOption::new(node.path.clone(), node.path.clone()).icon(Icon::Boxes)
            }))
            .collect();
        properties.push(
            Property::single("under", "Component", options, card.under.clone()).icon(Icon::Boxes),
        );
    }
    properties.push(Property::single(
        "kind",
        "Kind",
        vec![
            PropertyOption::new("feature", "Feature").icon(Icon::FolderKanban),
            PropertyOption::new("architecture", "Architecture change")
                .icon(Icon::ChartRelationshipIcon),
        ],
        if card.architecture {
            "architecture"
        } else {
            "feature"
        },
    ));
    properties
}

pub struct ComposeView {
    state: Entity<AppState>,
    composer: Entity<Composer>,
    /// What the Composer is open for.
    open: Option<ComposeRequest>,
    card: CardProps,
    /// A decision's or a doc's Context.
    contexts: Vec<SharedString>,
}

impl ComposeView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let composer = cx.new(Composer::new);
        cx.subscribe(&composer, Self::composed).detach();
        cx.observe(&composer, |_, _, cx| cx.notify()).detach();
        cx.observe_in(&state, window, |this, state, window, cx| {
            if let Some(request) = state.update(cx, |state, _| state.take_compose_request()) {
                this.open(request, window, cx);
            }
        })
        .detach();
        Self {
            state,
            composer,
            open: None,
            card: CardProps::default(),
            contexts: Vec::new(),
        }
    }

    /// The project's milestones and Structure nodes: Milestones' own while it has read them,
    /// as they follow its writes, else the scope filter's catalog.
    fn catalog(&self, cx: &App) -> (Vec<Milestone>, Vec<StructureNode>) {
        let app = self.state.read(cx);
        match (app.work(), app.catalog()) {
            (Load::Ready(work), _) => (work.milestones.clone(), work.nodes.clone()),
            (_, Load::Ready(catalog)) => (catalog.milestones.clone(), catalog.nodes.clone()),
            _ => (Vec::new(), Vec::new()),
        }
    }

    fn properties(&self, cx: &App) -> Vec<Property> {
        let (milestones, nodes) = self.catalog(cx);
        match &self.open {
            Some(ComposeRequest::Card { .. }) => card_properties(&self.card, &milestones, &nodes),
            Some(ComposeRequest::Decision | ComposeRequest::Doc) if !nodes.is_empty() => {
                let ids: Vec<String> = self.contexts.iter().map(|id| id.to_string()).collect();
                vec![context_property(&ids, &nodes)]
            }
            _ => Vec::new(),
        }
    }

    /// Opens the Composer for `request`.
    pub fn open(&mut self, request: ComposeRequest, window: &mut Window, cx: &mut Context<Self>) {
        let spec = match &request {
            ComposeRequest::Card { stage, milestone } => {
                self.card = CardProps {
                    stage: *stage,
                    milestone: milestone.clone(),
                    ..CardProps::default()
                };
                ComposerSpec::new("Card title", "Create card")
                    .description("What we want and why…")
                    .why(WHY)
            }
            ComposeRequest::Decision => ComposerSpec::new("Decision title", "Create decision")
                .description("Context: what forces this decision…")
                .why(WHY),
            ComposeRequest::Doc => ComposerSpec::new("Doc title", "Create doc")
                .description("Write in Markdown…")
                .why(WHY),
        };
        self.contexts = self
            .state
            .read(cx)
            .scope_contexts()
            .into_iter()
            .map(SharedString::from)
            .collect();
        self.open = Some(request);
        let properties = self.properties(cx);
        self.composer.update(cx, |composer, cx| {
            composer.open(spec, properties, window, cx)
        });
    }

    /// The Composer is open, or opening.
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Closes the Composer, as Escape does.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.composer.update(cx, |composer, cx| composer.close(cx));
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
                match id.as_ref() {
                    "stage" => self.card.stage = Stage::parse(&value),
                    "milestone" => self.card.milestone = value,
                    "under" => self.card.under = value,
                    "kind" => self.card.architecture = value == "architecture",
                    "context" => self.contexts = values.clone(),
                    _ => {}
                }
                let properties = self.properties(cx);
                composer.update(cx, |composer, cx| composer.set_properties(properties, cx));
            }
            ComposerEvent::Action => {}
            ComposerEvent::Closed => self.open = None,
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
        match self.open.clone() {
            Some(ComposeRequest::Card { .. }) => {
                let (milestones, _) = self.catalog(cx);
                let milestone = milestones
                    .iter()
                    .find(|milestone| milestone.id == self.card.milestone)
                    .map(|milestone| milestone.slug.clone())
                    .unwrap_or_default();
                let card = NewCard {
                    title: draft.title,
                    intent: draft.description,
                    stage: self.card.stage,
                    milestone,
                    under: self.card.under.clone(),
                    architecture: self.card.architecture,
                    why: draft.why,
                };
                self.state
                    .update(cx, |state, cx| state.create_card(card, done, cx));
            }
            Some(request @ (ComposeRequest::Decision | ComposeRequest::Doc)) => {
                let record = NewRecord {
                    decision: request == ComposeRequest::Decision,
                    title: draft.title,
                    body: draft.description,
                    nodes: self.contexts.iter().map(|id| id.to_string()).collect(),
                    why: draft.why,
                };
                self.state
                    .update(cx, |state, cx| state.create_knowledge(record, done, cx));
            }
            None => done(false, cx),
        }
    }
}

impl Render for ComposeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.composer.clone())
    }
}
