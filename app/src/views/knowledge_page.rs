//! A doc's page (`doc-detail.tsx` on `KnowledgeDocumentShell`): a [`DocumentPage`] whose
//! column holds the editable title (24px, Enter moves into the text), the Markdown body in the
//! document style, and the record's Activity (`DocumentActivity`: the last 20 changes that
//! named it, a burst of the same edit folded into its latest line, hidden while there are
//! none). The rail's Properties group holds the doc's Context, and Labels holds the tags; the
//! rail ends with Archive.
//!
//! The title and the body save 800ms after the last keystroke (`createDocumentAutosave`), a
//! tag at once, each under its own summary and no reason; Archive is confirmed, with no reason
//! asked. Context is a `TokenPicker` of the domains and components (`ContextPicker`), written
//! at once with no change (`setScope`).

use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, Focusable, FontWeight,
    IntoElement, Render, Subscription, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::icons::Icon;
use polterware_ui::markdown::{MarkdownEditor, MarkdownEditorEvent};
use polterware_ui::patterns::{
    ConfirmDialog, DocumentPage, PropertyPills, PropertyPillsEvent, RailState, TagPicker,
    TagPickerEvent, document_footer, document_group, document_group_body, document_title,
    modal_layer, rail_properties, rail_property, window_layer,
};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;
use polterware_ui::wrap::{SpanStyle, wrap_text};

use crate::contracts::{KnowledgeItem, ProjectChange, StructureNode};
use crate::format::{document_ago, now_seconds};
use crate::state::{AppState, Load};
use crate::views::activity::{actor_icon, actor_kind};
use crate::views::knowledge::context_property;

/// `createDocumentAutosave`'s debounce.
const AUTOSAVE: Duration = Duration::from_millis(800);
/// `ActorIcon size={20}`.
const ACTOR_SIZE: f32 = 20.0;

/// A question asked before a write.
#[derive(Clone, Debug, PartialEq)]
enum Ask {
    Archive,
}

/// One line of a record's Activity (`DocumentChange`).
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentChange {
    pub change: ProjectChange,
    pub actor: String,
    pub summary: String,
    pub rationale: String,
}

/// `mapDocumentChanges`: the changes newest first, a run of the same actor making the same
/// change with no reason (an autosave burst) folded into its latest line.
pub fn document_changes(changes: &[ProjectChange]) -> Vec<DocumentChange> {
    let mut out: Vec<DocumentChange> = Vec::new();
    for change in changes {
        if change.step_id.is_empty() {
            continue;
        }
        let rationale = serde_json::from_str::<serde_json::Value>(&change.payload_json)
            .ok()
            .and_then(|value| value.get("rationale")?.as_str().map(str::to_string))
            .unwrap_or_default()
            .trim()
            .to_string();
        let actor = if !change.agent_label.is_empty() {
            change.agent_label.clone()
        } else if actor_kind(change) == "human" {
            "Someone".into()
        } else {
            "An agent".into()
        };
        if let Some(previous) = out.last()
            && previous.actor == actor
            && previous.summary == change.summary
            && rationale.is_empty()
        {
            continue;
        }
        out.push(DocumentChange {
            change: change.clone(),
            actor,
            summary: change.summary.clone(),
            rationale,
        });
    }
    out
}

/// The tags docs already use, offered when one is added.
fn tag_suggestions(items: &[KnowledgeItem]) -> Vec<String> {
    let mut tags: Vec<String> = items
        .iter()
        .filter(|item| item.kind == "doc")
        .flat_map(|item| item.tags.iter().cloned())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

pub struct KnowledgePage {
    state: Entity<AppState>,
    /// The record the fields were set for.
    record: Option<String>,
    title: Entity<TextInput>,
    body: Entity<MarkdownEditor>,
    tags: Entity<TagPicker>,
    context: Entity<PropertyPills>,
    /// What the Context tokens were set from: the record, its nodes, and the project's.
    context_for: Option<(String, Vec<String>, Vec<StructureNode>)>,
    ask: Option<Ask>,
    ask_focus: FocusHandle,
    /// Whether the rail shows, kept across pages as the web keeps it, and the narrow page's
    /// drawer.
    rail: Entity<RailState>,
    /// The title and the body waiting out the autosave: the record and the value.
    pending_title: Option<(String, String)>,
    pending_body: Option<(String, String)>,
    /// Bumped by each keystroke, so only the last one's timer saves.
    due: [u64; 2],
    _body_changes: Subscription,
}

impl KnowledgePage {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let page = cx.weak_entity();
        let title = cx.new(|cx| {
            TextInput::new(cx)
                .prose(24.0, 32.0, FontWeight::SEMIBOLD)
                .wrap()
                .placeholder("Untitled")
                // Enter moves into the text, after the field's own update.
                .on_submit(move |_, window, cx| {
                    let page = page.clone();
                    window.defer(cx, move |window, cx| {
                        let _ = page.update(cx, |this, cx| {
                            this.body.update(cx, |body, cx| body.edit_start(window, cx));
                        });
                    });
                })
        });
        cx.observe(&title, |this, _, cx| this.title_typed(cx))
            .detach();
        let body = cx.new(|cx| MarkdownEditor::new("", cx));
        let body_changes = cx.subscribe(&body, Self::body_typed);
        cx.observe(&body, |_, _, cx| cx.notify()).detach();
        let tags = cx.new(|cx| TagPicker::new("knowledge-tags", cx));
        cx.subscribe(&tags, |this, _, event: &TagPickerEvent, cx| {
            let TagPickerEvent::Changed(tags) = event;
            let tags = tags.clone();
            this.save(|item| item.tags = tags, "Update doc tags", cx);
        })
        .detach();
        let context = cx.new(PropertyPills::new);
        cx.observe(&context, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&context, |this, _, event: &PropertyPillsEvent, cx| {
            let PropertyPillsEvent::Changed { values, .. } = event;
            let Some(id) = this.record.clone() else {
                return;
            };
            let nodes = values.iter().map(|value| value.to_string()).collect();
            this.state
                .update(cx, |state, cx| state.set_knowledge_scope(id, nodes, cx));
        })
        .detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            record: None,
            title,
            body,
            tags,
            context,
            context_for: None,
            ask: None,
            ask_focus: cx.focus_handle(),
            rail: cx.new(|_| RailState::new(true)),
            pending_title: None,
            pending_body: None,
            due: [0, 0],
            _body_changes: body_changes,
        }
    }

    fn item(&self, id: &str, cx: &App) -> Option<KnowledgeItem> {
        match self.state.read(cx).knowledge() {
            Load::Ready(knowledge) => knowledge.items.iter().find(|item| item.id == id).cloned(),
            _ => None,
        }
    }

    /// Shows `item`'s fields: all of them for another record (writing what the last one still
    /// held), else only what changed elsewhere and is not being typed.
    fn sync(
        &mut self,
        item: &KnowledgeItem,
        suggestions: Vec<String>,
        nodes: &[StructureNode],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.record.as_deref() != Some(item.id.as_str()) {
            self.flush(0, cx);
            self.flush(1, cx);
            self.record = Some(item.id.clone());
            self.ask = None;
            self.title
                .update(cx, |title, cx| title.set_text(item.title.clone(), cx));
            // A new editor, so nothing of the last record's open block comes along.
            self.body = cx.new(|cx| MarkdownEditor::new(item.body.clone(), cx));
            self._body_changes = cx.subscribe(&self.body, Self::body_typed);
            cx.observe(&self.body, |_, _, cx| cx.notify()).detach();
            let id = item.id.clone();
            self.state
                .update(cx, |state, cx| state.load_document_activity(id, cx));
        } else {
            let typing = self.title.read(cx).focus_handle(cx).is_focused(window);
            if !typing && self.pending_title.is_none() && self.title.read(cx).text() != item.title {
                self.title
                    .update(cx, |title, cx| title.set_text(item.title.clone(), cx));
            }
            if self.pending_body.is_none() {
                self.body
                    .update(cx, |body, cx| body.set_source(item.body.clone(), cx));
            }
        }
        let tags = item.tags.clone();
        self.tags.update(cx, |picker, cx| {
            picker.set_suggestions(suggestions);
            picker.set_tags(tags, cx);
        });
        let shown = matches!(&self.context_for, Some((id, ids, set))
            if *id == item.id && *ids == item.scope_node_ids && set.as_slice() == nodes);
        if !shown {
            let property = context_property(&item.scope_node_ids, nodes);
            self.context
                .update(cx, |pills, cx| pills.set_properties(vec![property], cx));
            self.context_for = Some((item.id.clone(), item.scope_node_ids.clone(), nodes.to_vec()));
        }
    }

    fn title_typed(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.record.clone() else {
            return;
        };
        let text = self.title.read(cx).text().trim().to_string();
        let saved = self.item(&id, cx).map(|item| item.title);
        // A blank title is not a valid record: it stays here until a name is typed.
        if text.is_empty() || saved.as_deref() == Some(text.as_str()) {
            self.pending_title = None;
            return;
        }
        self.pending_title = Some((id, text));
        self.schedule(0, cx);
    }

    fn body_typed(
        &mut self,
        _: Entity<MarkdownEditor>,
        event: &MarkdownEditorEvent,
        cx: &mut Context<Self>,
    ) {
        let MarkdownEditorEvent::Changed(source) = event;
        let Some(id) = self.record.clone() else {
            return;
        };
        if self.item(&id, cx).is_some_and(|item| &item.body == source) {
            self.pending_body = None;
            return;
        }
        self.pending_body = Some((id, source.clone()));
        self.schedule(1, cx);
    }

    /// Saves field `field` (0 the title, 1 the body) once no keystroke came for a while.
    fn schedule(&mut self, field: usize, cx: &mut Context<Self>) {
        self.due[field] += 1;
        let due = self.due[field];
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AUTOSAVE).await;
            let _ = this.update(cx, |this, cx| {
                if this.due[field] == due {
                    this.flush(field, cx);
                }
            });
        })
        .detach();
    }

    /// Writes field `field` now, when it waits.
    fn flush(&mut self, field: usize, cx: &mut Context<Self>) {
        let pending = if field == 0 {
            self.pending_title.take()
        } else {
            self.pending_body.take()
        };
        let Some((id, value)) = pending else {
            return;
        };
        let Some(mut item) = self.item(&id, cx) else {
            return;
        };
        let summary = if field == 0 {
            item.title = value;
            "Rename doc"
        } else {
            item.body = value;
            "Update knowledge body"
        };
        self.state
            .update(cx, |state, cx| state.save_knowledge_item(item, summary, cx));
    }

    /// Changes the open record with `edit` and saves it under `summary`.
    fn save(
        &mut self,
        edit: impl FnOnce(&mut KnowledgeItem),
        summary: &'static str,
        cx: &mut Context<Self>,
    ) {
        let Some(mut item) = self.record.clone().and_then(|id| self.item(&id, cx)) else {
            return;
        };
        edit(&mut item);
        self.state
            .update(cx, |state, cx| state.save_knowledge_item(item, summary, cx));
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        self.flush(0, cx);
        self.flush(1, cx);
        self.state.update(cx, |state, cx| {
            if state.can_go_back() {
                state.go_back(cx);
            } else if let Some((tab, _)) = state.knowledge_route() {
                state.set_knowledge_tab(tab, cx);
            }
            cx.notify();
        });
    }

    fn ask(&mut self, ask: Ask, window: &mut Window, cx: &mut Context<Self>) {
        self.ask = Some(ask);
        self.ask_focus.focus(window);
        cx.notify();
    }

    fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.ask = None;
        cx.notify();
    }

    /// Writes the asked move, with no reason: a person may leave it empty.
    fn answer(&mut self, cx: &mut Context<Self>) {
        let (Some(ask), Some(item)) = (
            self.ask.clone(),
            self.record.clone().and_then(|id| self.item(&id, cx)),
        ) else {
            return;
        };
        self.ask = None;
        let title = item.title.clone();
        self.state.update(cx, |state, cx| match ask {
            Ask::Archive => state.archive_knowledge(item.id.clone(), title, String::new(), cx),
        });
        cx.notify();
    }

    fn context_row(&self, cx: &App) -> AnyElement {
        rail_property("Context", self.context.clone(), false, cx)
    }

    fn activity(&self, subject: &str, cx: &App) -> Option<AnyElement> {
        document_activity(&self.state, subject, cx)
    }

    fn confirm(&self, item: &KnowledgeItem, cx: &mut Context<Self>) -> Option<AnyElement> {
        let ask = self.ask.as_ref()?;
        let theme = *Theme::global(cx);
        let title = &item.title;
        let (heading, description, label) = match ask {
            Ask::Archive => (
                format!("Archive “{title}”?"),
                "It leaves the lists and agent reads. Restore brings it back.".to_string(),
                "Archive",
            ),
        };
        let page = cx.weak_entity();
        let dismiss = page.clone();
        let close: polterware_ui::Handler = Rc::new(move |_, cx: &mut App| {
            let _ = dismiss.update(cx, |this, cx| this.dismiss(cx));
        });
        let dialog = ConfirmDialog::new("knowledge-confirm", heading, description)
            .confirm_label(label)
            .focus(&self.ask_focus)
            .on_cancel(close.clone())
            .on_confirm(Rc::new(move |_, cx: &mut App| {
                let _ = page.update(cx, |this, cx| this.answer(cx));
            }));
        let dialog = dialog.danger();
        Some(window_layer(modal_layer(1.0, true, close, dialog, &theme)))
    }
}

impl Render for KnowledgePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.state.read(cx);
        let Some((_, Some(id))) = app.knowledge_route() else {
            return div().size_full();
        };
        let id = id.to_string();
        let pending = app.knowledge_pending();
        let Load::Ready(knowledge) = app.knowledge() else {
            return div().size_full();
        };
        let knowledge = knowledge.clone();
        let page = cx.weak_entity();
        let back: polterware_ui::Handler = Rc::new(move |_, cx: &mut App| {
            let _ = page.update(cx, |this, cx| this.back(cx));
        });
        let Some(item) = knowledge.items.iter().find(|item| item.id == id).cloned() else {
            // `KnowledgeDocumentNotFound`.
            let kind = "Doc";
            let theme = Theme::global(cx);
            return div().size_full().child(
                DocumentPage::new("knowledge-page")
                    .on_back(back)
                    .child(document_title(format!("{kind} not found"), cx))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(theme.color_text_muted)
                            .child(wrap_text(format!(
                                "This {} is not available in the current project knowledge set. It may have been archived or the link is stale.",
                                kind.to_lowercase()
                            ))),
                    ),
            );
        };
        self.sync(
            &item,
            tag_suggestions(&knowledge.items),
            &knowledge.nodes,
            window,
            cx,
        );
        let properties: Vec<AnyElement> = vec![self.context_row(cx)];
        let mut document = DocumentPage::new("knowledge-page")
            .scroll_locked(self.body.read(cx).slash_open())
            .on_back(back)
            .rail_state(&self.rail)
            .rail(document_group(
                "Properties",
                Icon::InformationCircle,
                rail_properties().children(properties),
                cx,
            ))
            .rail(document_group(
                "Labels",
                Icon::Tag,
                document_group_body(self.tags.clone()),
                cx,
            ))
            .child(
                div()
                    .w_full()
                    .mt(px(32.0))
                    .mb(px(8.0))
                    .child(self.title.clone()),
            )
            .child(self.body.clone())
            .children(self.activity(&item.id, cx));
        {
            let archive = Button::new("knowledge-archive", ButtonKind::Secondary)
                .size(ButtonSize::Sm)
                .label("Archive")
                .disabled(pending)
                .on_click({
                    let page = cx.weak_entity();
                    move |window, cx| {
                        let _ = page.update(cx, |this, cx| this.ask(Ask::Archive, window, cx));
                    }
                })
                .into_any_element();
            document = document.rail(document_footer([archive], cx));
        }
        div()
            .size_full()
            .child(document)
            .children(self.context.read(cx).overlay(cx))
            .children(self.confirm(&item, cx))
    }
}

/// A record page's Activity (`DocumentActivity`): the last 20 changes that named `subject`, a
/// burst of the same edit folded into its latest line, none while there are none.
pub(crate) fn document_activity(
    state: &Entity<AppState>,
    subject: &str,
    cx: &App,
) -> Option<AnyElement> {
    let theme = Theme::global(cx);
    let changes = document_changes(state.read(cx).document_activity(subject));
    if changes.is_empty() {
        return None;
    }
    let now = now_seconds();
    let rows = changes.into_iter().map(|line| {
        let summary = if line.summary.is_empty() {
            "Changed this".to_string()
        } else {
            line.summary.clone()
        };
        let when = document_ago(&line.change.created_at, now);
        // `__when` sits 6px after the summary: an en space at 13px.
        let text = format!(
            "{} {}{}",
            line.actor,
            summary,
            if when.is_empty() {
                String::new()
            } else {
                format!("\u{2002}{when}")
            }
        );
        let name_end = line.actor.len();
        let when_start = text.len() - when.len();
        let mut styles = vec![(
            0..name_end,
            SpanStyle {
                highlight: gpui::HighlightStyle {
                    color: Some(theme.color_text_primary.into()),
                    font_weight: Some(FontWeight::MEDIUM),
                    ..Default::default()
                },
                font_family: None,
            },
        )];
        if !when.is_empty() {
            styles.push((
                when_start..text.len(),
                SpanStyle {
                    highlight: gpui::HighlightStyle {
                        color: Some(theme.color_text_muted.into()),
                        ..Default::default()
                    },
                    font_family: None,
                },
            ));
        }
        div()
            .w_full()
            .flex()
            .items_start()
            .gap(px(10.0))
            .child(
                div()
                    .flex_none()
                    .child(actor_icon(&line.change, ACTOR_SIZE, cx)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .line_height(px(20.0))
                            .text_color(theme.color_text_secondary)
                            .child(wrap_text(text).with_styles(styles)),
                    )
                    .when(!line.rationale.is_empty(), |body| {
                        body.child(
                            div()
                                .text_size(px(12.5))
                                .line_height(px(18.0))
                                .text_color(theme.color_text_muted)
                                .child(wrap_text(line.rationale.clone())),
                        )
                    }),
            )
    });
    Some(
        div()
            .w_full()
            .mt(px(48.0))
            .pt(px(24.0))
            .border_t(px(0.5))
            .border_color(theme.color_border_default)
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .text_size(px(13.0))
                    .line_height(px(20.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.color_text_primary)
                    .child("Activity"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .children(rows),
            )
            .into_any_element(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_autosave_burst_folds_into_its_latest_line() {
        let change = |step: &str, label: &str, summary: &str, payload: &str| ProjectChange {
            step_id: step.into(),
            agent_label: label.into(),
            actor_kind: "human".into(),
            summary: summary.into(),
            payload_json: payload.into(),
            ..ProjectChange::default()
        };
        let lines = document_changes(&[
            change("3", "", "Update knowledge body", "{}"),
            change("2", "", "Update knowledge body", "{}"),
            change("1", "", "Update knowledge body", r#"{"rationale":" Fix "}"#),
            change("", "", "No id", "{}"),
        ]);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].actor, "Someone");
        assert_eq!(lines[1].rationale, "Fix");
    }

    #[test]
    fn context_offers_the_domains_then_the_components_and_a_shared_title_its_path() {
        use crate::views::knowledge::context_options;
        let node = |id: &str, level: &str, title: &str, path: &str| StructureNode {
            id: id.into(),
            level: level.into(),
            title: title.into(),
            path: path.into(),
            ..StructureNode::default()
        };
        let nodes = [
            node("c1", "component", "Api", "core/api"),
            node("f1", "feature", "Login", "core/api/login"),
            node("d1", "domain", "Core", "core"),
            node("c2", "component", "Api", "web/api"),
            node("d2", "domain", "Web", "web"),
        ];
        let options = context_options(&nodes);
        let shown: Vec<(&str, &str)> = options
            .iter()
            .map(|option| (option.value().as_ref(), option.label().as_ref()))
            .collect();
        assert_eq!(
            shown,
            [
                ("d1", "Core"),
                ("d2", "Web"),
                ("c1", "core/api"),
                ("c2", "web/api"),
            ]
        );
        let property = context_property(&["d2".into()], &nodes);
        assert_eq!(property.values(), ["d2"]);
    }
}
