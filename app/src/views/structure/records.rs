//! Electron `RecordEditor` (`knowledge/src/record-editor.tsx`): cited records edit in
//! their Composer first. Expand alone navigates; a Rule has no page and no Expand.
use super::*;
use crate::views::{
    compose::{CardProps, card_properties},
    knowledge::{context_property, rule_save},
};
use crate::work::{NewCard, Stage};
use polterware_ui::patterns::ComposerDraft;

impl StructureView {
    fn record_properties(&self, cx: &App) -> Vec<Property> {
        let Some(structure) = self.structure(cx) else {
            return Vec::new();
        };
        if matches!(self.editing, Some((RecordKind::Card, _))) {
            let milestones = match self.state.read(cx).catalog() {
                Load::Ready(c) => c.milestones.as_slice(),
                _ => &[],
            };
            card_properties(&self.record_card, milestones, &structure.nodes)
        } else if structure.nodes.is_empty() {
            Vec::new()
        } else {
            vec![context_property(&self.record_contexts, &structure.nodes)]
        }
    }

    pub(super) fn open_record_editor(
        &mut self,
        kind: RecordKind,
        id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(structure) = self.structure(cx) else {
            return;
        };
        let (title, description, contexts, placeholder, submit, label) = match kind {
            RecordKind::Card => {
                let Some(card) = structure.cards.iter().find(|card| card.slug == id) else {
                    return;
                };
                let result = (
                    card.title.clone(),
                    card.intent.clone(),
                    Vec::new(),
                    "Card title",
                    "Save card",
                    "Edit card",
                );
                self.record_card = CardProps {
                    stage: Some(Stage::of(card)),
                    milestone: card.milestone_id.clone(),
                    under: card.under.clone(),
                    architecture: card.kind == "architecture",
                };
                result
            }
            RecordKind::Rule => {
                let Some(rule) = structure.rules.iter().find(|rule| rule.id == id) else {
                    return;
                };
                (
                    rule.statement.clone(),
                    String::new(),
                    rule.scope_node_ids.clone(),
                    "The API returns only fields declared in the contract",
                    "Save rule",
                    "Edit rule",
                )
            }
            RecordKind::Decision | RecordKind::Doc => {
                let Some(item) = structure.items.iter().find(|item| item.id == id) else {
                    return;
                };
                let decision = kind == RecordKind::Decision;
                (
                    item.title.clone(),
                    item.body.clone(),
                    item.scope_node_ids.clone(),
                    if decision {
                        "Decision title"
                    } else {
                        "Doc title"
                    },
                    if decision {
                        "Save decision"
                    } else {
                        "Save doc"
                    },
                    if decision {
                        "Edit decision"
                    } else {
                        "Edit doc"
                    },
                )
            }
        };
        self.editing = Some((kind, id));
        self.record_contexts = contexts;
        let mut spec = ComposerSpec::new(placeholder, submit)
            .edit()
            .why(WHY)
            .crumbs([
                if kind == RecordKind::Rule {
                    "Rules".into()
                } else {
                    self.project_name(cx)
                },
                label.into(),
            ])
            .initial(ComposerDraft {
                title,
                description,
                ..Default::default()
            });
        if kind != RecordKind::Rule {
            spec = spec.description(match kind {
                RecordKind::Card => "What we want and why…",
                RecordKind::Decision => "Context, decision, consequences…",
                _ => "Write in Markdown…",
            });
        }
        let expand: Option<polterware_ui::Handler> = (kind != RecordKind::Rule).then(|| {
            let view = cx.entity().downgrade();
            Rc::new(move |_: &mut Window, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    if let Some((kind, id)) = this.editing.clone() {
                        this.record_editor.update(cx, |editor, cx| editor.close(cx));
                        this.state
                            .update(cx, |state, cx| state.open_structure_record(kind, id, cx));
                    }
                });
            }) as polterware_ui::Handler
        });
        let properties = self.record_properties(cx);
        self.record_editor.update(cx, |editor, cx| {
            editor.set_on_expand(expand);
            editor.open(spec, properties, window, cx);
        });
    }

    pub(super) fn record_composed(
        &mut self,
        composer: Entity<Composer>,
        event: &ComposerEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ComposerEvent::Closed => self.editing = None,
            ComposerEvent::Property { id, values } => {
                let value = values.first().map(ToString::to_string).unwrap_or_default();
                match id.as_ref() {
                    "stage" => self.record_card.stage = Stage::parse(&value),
                    "milestone" => self.record_card.milestone = value,
                    "under" => self.record_card.under = value,
                    "kind" => self.record_card.architecture = value == "architecture",
                    "context" => {
                        self.record_contexts = values.iter().map(ToString::to_string).collect()
                    }
                    _ => {}
                }
                let properties = self.record_properties(cx);
                composer.update(cx, |composer, cx| composer.set_properties(properties, cx));
            }
            ComposerEvent::Submit(draft) => {
                let editor = composer.downgrade();
                let done = move |ok, cx: &mut App| {
                    let _ = editor.update(cx, |editor, cx| editor.finish(ok, cx));
                };
                let Some((kind, id)) = self.editing.clone() else {
                    return done(false, cx);
                };
                let Some(structure) = self.structure(cx).cloned() else {
                    return done(false, cx);
                };
                match kind {
                    RecordKind::Decision | RecordKind::Doc => {
                        let Some(mut item) =
                            structure.items.iter().find(|item| item.id == id).cloned()
                        else {
                            return done(false, cx);
                        };
                        item.title = draft.title.clone();
                        item.body = draft.description.clone();
                        item.scope_node_ids = self.record_contexts.clone();
                        self.state.update(cx, |state, cx| {
                            state.save_structure_knowledge(item, draft.why.clone(), done, cx)
                        });
                    }
                    RecordKind::Rule => {
                        let Some(rule) = structure.rules.iter().find(|rule| rule.id == id) else {
                            return done(false, cx);
                        };
                        let text = rule_save(
                            Some(rule),
                            &structure.rules,
                            &draft.title,
                            &self.record_contexts,
                            &structure.nodes,
                        );
                        self.state.update(cx, |state, cx| {
                            state.write_structure(
                                text,
                                format!("Update rule: {}", draft.title),
                                draft.why.clone(),
                                "Could not save rule.",
                                done,
                                cx,
                            )
                        });
                    }
                    RecordKind::Card => {
                        let milestone = if self.record_card.milestone.is_empty() {
                            String::new()
                        } else {
                            let slug = match self.state.read(cx).catalog() {
                                Load::Ready(catalog) => catalog
                                    .milestones
                                    .iter()
                                    .find(|m| m.id == self.record_card.milestone)
                                    .map(|m| m.slug.clone()),
                                _ => None,
                            };
                            let Some(slug) = slug else {
                                return done(false, cx);
                            };
                            slug
                        };
                        let card = NewCard {
                            title: draft.title.clone(),
                            intent: draft.description.clone(),
                            stage: self.record_card.stage,
                            milestone,
                            under: self.record_card.under.clone(),
                            architecture: self.record_card.architecture,
                            why: draft.why.clone(),
                        };
                        let text = crate::work::polter_edit_card(&id, &card);
                        self.state.update(cx, |state, cx| {
                            state.write_structure(
                                text,
                                format!("Update card: {}", draft.title),
                                draft.why.clone(),
                                "Could not save card.",
                                done,
                                cx,
                            )
                        });
                    }
                }
            }
            ComposerEvent::Action => {}
        }
    }
}
