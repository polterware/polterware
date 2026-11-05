//! A decision's or a doc's page (`decision-detail.tsx`, `doc-detail.tsx` on
//! `KnowledgeDocumentShell`): a [`DocumentPage`] whose column holds the editable title (24px,
//! Enter moves into the text), the Markdown body in the document style, and the record's
//! Activity (`DocumentActivity`: the last 20 changes that named it, a burst of the same edit
//! folded into its latest line, hidden while there are none). The rail's Properties group
//! holds a decision's Status (its chip and the move it allows: Activate, Open review, Keep
//! active), Reason, Review when, Context, Governs, Replaces, Replaced by, and Replace with;
//! a doc's Context. Labels holds the tags, and a doc's rail ends with Archive.
//!
//! The title and the body save 800ms after the last keystroke (`createDocumentAutosave`),
//! a tag at once, each under its own summary and no reason; Open review, Replace, and
//! Archive change intent, so they ask why first. Context is a `TokenPicker` of the domains and
//! components (`ContextPicker`), written at once with no change (`setScope`).

use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, Focusable, FontWeight, IntoElement, Render,
    Subscription, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::chip::chip_rounded;
use polterware_ui::icons::Icon;
use polterware_ui::list_popover::{ListPopover, ListRow};
use polterware_ui::markdown::{MarkdownEditor, MarkdownEditorEvent};
use polterware_ui::patterns::{
    AnchorSlot, ConfirmDialog, DocumentPage, PropertyPills, PropertyPillsEvent, RAIL_BUTTON_RADIUS,
    RailState, TagPicker, TagPickerEvent, document_footer, document_group, document_group_body,
    document_title, modal_layer, rail_properties, rail_property, rail_select, window_anchor,
    window_layer,
};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;
use polterware_ui::wrap::{SpanStyle, wrap_text};

use crate::contracts::{KnowledgeItem, ProjectChange, StructureNode};
use crate::format::{document_ago, now_seconds};
use crate::state::{AppState, Knowledge, KnowledgeTab, Load};
use crate::views::activity::{actor_icon, actor_kind};
use crate::views::knowledge::{
    context_property, decision_state, decision_state_label, decision_state_tone,
};

/// `createDocumentAutosave`'s debounce.
const AUTOSAVE: Duration = Duration::from_millis(800);
/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
/// The Replace with list's least width (`max-w-[20rem]` on the 13px root), wider than its pill.
const SELECT_WIDTH: f32 = 260.0;
/// `ActorIcon size={20}`.
const ACTOR_SIZE: f32 = 20.0;

/// A move that changes intent, waiting for its reason.
#[derive(Clone, Debug, PartialEq)]
enum Ask {
    Review,
    Replace { by: String, by_title: String },
    Archive,
}

/// What a decision's page shows of it (`mapDecisions`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecisionFacts {
    pub state: &'static str,
    pub reason: String,
    pub review_when: String,
    /// The Structure paths whose `decisions:` cite it.
    pub governs: Vec<String>,
    /// The decisions it replaced, by title.
    pub replaces: Vec<String>,
    /// The decision that replaced it, by title.
    pub replaced_by: String,
    /// The decisions that may replace it: the others still in force.
    pub successors: Vec<(String, String)>,
}

fn extra(item: &KnowledgeItem, key: &str) -> String {
    serde_json::from_str::<serde_json::Value>(&item.extra_json)
        .ok()
        .and_then(|value| value.get(key)?.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// `decisionNumber`: the number a title or a reference names, as `Decision-0012`,
/// `ADR 12:`, or `Verdict_0012.`.
fn decision_number(text: &str) -> Option<u64> {
    let text = text.trim_start().to_lowercase();
    let rest = ["decision", "verdict", "adr"]
        .iter()
        .find_map(|prefix| text.strip_prefix(prefix))?;
    let rest = rest.trim_start_matches([' ', '-', '_', '\t']);
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    let after = rest[digits.len()..].trim_start();
    if !(after.is_empty() || after.starts_with([':', '.', '-', '–', '—'])) {
        return None;
    }
    digits.parse().ok()
}

/// `citesDecision`: a `decisions:` entry names the decision by its id or its number.
pub(crate) fn cites(reference: &str, id: &str, title: &str) -> bool {
    if reference.trim().eq_ignore_ascii_case(id) {
        return true;
    }
    decision_number(reference).is_some_and(|wanted| Some(wanted) == decision_number(title))
}

pub fn decision_facts(item: &KnowledgeItem, knowledge: &Knowledge) -> DecisionFacts {
    let decisions: Vec<&KnowledgeItem> = knowledge
        .items
        .iter()
        .filter(|other| other.kind == "decision")
        .collect();
    let title_of = |id: &str| {
        decisions
            .iter()
            .find(|other| other.id == id)
            .map(|other| other.title.clone())
            .unwrap_or_default()
    };
    DecisionFacts {
        state: decision_state(&item.status),
        reason: extra(item, "statusReason"),
        review_when: extra(item, "reviewTrigger"),
        governs: knowledge
            .nodes
            .iter()
            .filter(|node| node.status != "removed")
            .filter(|node| {
                node.decisions
                    .iter()
                    .any(|reference| cites(reference, &item.id, &item.title))
            })
            .map(|node| node.path.clone())
            .collect(),
        replaces: decisions
            .iter()
            .filter(|other| extra(other, "supersededBy") == item.id)
            .map(|other| other.title.clone())
            .collect(),
        replaced_by: title_of(&extra(item, "supersededBy")),
        successors: decisions
            .iter()
            .filter(|other| other.id != item.id && decision_state(&other.status) != "superseded")
            .map(|other| (other.id.clone(), other.title.clone()))
            .collect(),
    }
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

/// `polterValue`: a Polter metadata value on one line, without the separators a line splits
/// on.
fn polter_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" · ", ", ")
        .replace(" • ", ", ")
        .replace(" - ", " – ")
}

/// The tags docs and decisions already use, offered when one is added.
fn tag_suggestions(items: &[KnowledgeItem]) -> Vec<String> {
    let mut tags: Vec<String> = items
        .iter()
        .filter(|item| item.kind == "doc" || item.kind == "decision")
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
    reason: Entity<TextInput>,
    ask: Option<Ask>,
    /// Whether the rail shows, kept across pages as the web keeps it, and the narrow page's
    /// drawer.
    rail: Entity<RailState>,
    replace: Entity<ListPopover>,
    replace_anchor: AnchorSlot,
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
        let confirm = cx.weak_entity();
        let cancel = cx.weak_entity();
        let reason = cx.new(|cx| {
            TextInput::new(cx)
                .wrap()
                .on_submit(move |_, window, cx| {
                    let confirm = confirm.clone();
                    window.defer(cx, move |_, cx| {
                        let _ = confirm.update(cx, |this, cx| this.answer(cx));
                    });
                })
                .on_cancel(move |window, cx| {
                    let cancel = cancel.clone();
                    window.defer(cx, move |_, cx| {
                        let _ = cancel.update(cx, |this, cx| this.dismiss(cx));
                    });
                })
        });
        cx.observe(&reason, |_, _, cx| cx.notify()).detach();
        let replace = cx.new(ListPopover::new);
        cx.observe(&replace, |_, _, cx| cx.notify()).detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            record: None,
            title,
            body,
            tags,
            context,
            context_for: None,
            reason,
            ask: None,
            rail: cx.new(|_| RailState::new(true)),
            replace,
            replace_anchor: AnchorSlot::default(),
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
        self.reason.update(cx, |field, cx| field.set_text("", cx));
        self.reason.read(cx).focus_handle(cx).focus(window);
        cx.notify();
    }

    fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.ask = None;
        cx.notify();
    }

    /// Writes the asked move with its reason; a blank reason writes nothing.
    fn answer(&mut self, cx: &mut Context<Self>) {
        let reason = self.reason.read(cx).text().trim().to_string();
        let (Some(ask), Some(item)) = (
            self.ask.clone(),
            self.record.clone().and_then(|id| self.item(&id, cx)),
        ) else {
            return;
        };
        if reason.is_empty() {
            return;
        }
        self.ask = None;
        let title = item.title.clone();
        self.state.update(cx, |state, cx| match ask {
            Ask::Review => state.move_decision(
                format!(
                    "### {}\nstatus: challenged · reason: {}",
                    item.id,
                    polter_value(&reason)
                ),
                format!("Open review of decision: {title}"),
                reason,
                cx,
            ),
            Ask::Replace { by, by_title } => state.move_decision(
                format!(
                    "### {by}\nsupersedes: {} · reason: {}",
                    item.id,
                    polter_value(&reason)
                ),
                format!("Replace {title} with {by_title}"),
                reason,
                cx,
            ),
            Ask::Archive => state.archive_knowledge(item.id.clone(), title, reason, cx),
        });
        cx.notify();
    }

    /// A move that needs no reason: Activate, or Keep active after a review.
    fn activate(&mut self, item: &KnowledgeItem, keep: bool, cx: &mut Context<Self>) {
        let summary = if keep {
            format!("Keep decision active after review: {}", item.title)
        } else {
            format!("Activate decision: {}", item.title)
        };
        let block = format!("### {}\nstatus: active", item.id);
        self.state.update(cx, |state, cx| {
            state.move_decision(block, summary, String::new(), cx)
        });
    }

    fn open_replace(
        &mut self,
        successors: Vec<(String, String)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let page = cx.weak_entity();
        let rows = successors
            .into_iter()
            .map(|(id, title)| {
                let page = page.clone();
                let by_title = title.clone();
                ListRow::new(
                    title,
                    Rc::new(move |window: &mut Window, cx: &mut App| {
                        let ask = Ask::Replace {
                            by: id.clone(),
                            by_title: by_title.clone(),
                        };
                        let _ = page.update(cx, |this, cx| this.ask(ask, window, cx));
                    }),
                )
                .icon(Icon::Scale)
            })
            .collect();
        let width = self.replace_anchor.get().map_or(SELECT_WIDTH, |bounds| {
            f32::from(bounds.size.width).max(SELECT_WIDTH)
        });
        self.replace.update(cx, |popover, cx| {
            let open = !popover.is_open();
            popover.set_width(width);
            popover.set_text_size(12.0);
            popover.set_trigger_height(32.0);
            popover.set_rows(rows);
            popover.set_open(open);
            cx.notify();
        });
        window.refresh();
    }

    fn status_row(
        &self,
        item: &KnowledgeItem,
        facts: &DecisionFacts,
        pending: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = facts.state;
        let action = match state {
            "proposed" => Some(("Activate", ButtonKind::Secondary)),
            "active" => Some(("Open review", ButtonKind::Ghost)),
            "challenged" => Some(("Keep active", ButtonKind::Ghost)),
            _ => None,
        };
        let value = div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(6.0))
            .child(chip_rounded(
                decision_state_label(state),
                decision_state_tone(state),
                CHIP_RADIUS,
                cx,
            ))
            .children(action.map(|(label, kind)| {
                let item = item.clone();
                let page = cx.weak_entity();
                Button::new("knowledge-decision-move", kind)
                    .size(ButtonSize::Sm)
                    .label(label)
                    .disabled(pending)
                    .on_click(move |window, cx| {
                        let _ = page.update(cx, |this, cx| match state {
                            "active" => this.ask(Ask::Review, window, cx),
                            _ => this.activate(&item, state == "challenged", cx),
                        });
                    })
            }));
        rail_property("Status", value, false, cx)
    }

    fn text_row(label: &str, value: &str, mono: bool, cx: &App) -> Option<AnyElement> {
        (!value.is_empty()).then(|| {
            rail_property(
                label,
                div().min_w(px(0.0)).child(wrap_text(value.to_string())),
                mono,
                cx,
            )
        })
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
        let (heading, description, label, placeholder) = match ask {
            Ask::Review => (
                format!("Open a review of “{title}”?"),
                "It stays under review until someone keeps or replaces it. Say what is wrong with it.".to_string(),
                "Open review",
                "What should the review look at?",
            ),
            Ask::Replace { by_title, .. } => (
                format!("Replace “{title}”?"),
                format!("{by_title} replaces it. Say why."),
                "Replace",
                "Why replace it?",
            ),
            Ask::Archive => (
                format!("Archive “{title}”?"),
                "Say why, so Activity and agents know it was dropped on purpose. Restore brings it back.".to_string(),
                "Archive",
                "Why archive it?",
            ),
        };
        self.reason
            .update(cx, |field, _| field.set_placeholder(placeholder));
        let page = cx.weak_entity();
        let dismiss = page.clone();
        let close: polterware_ui::Handler = Rc::new(move |_, cx: &mut App| {
            let _ = dismiss.update(cx, |this, cx| this.dismiss(cx));
        });
        let dialog = ConfirmDialog::new("knowledge-reason", heading, description)
            .confirm_label(label)
            .reason(&self.reason)
            .on_cancel(close.clone())
            .on_confirm(Rc::new(move |_, cx: &mut App| {
                let _ = page.update(cx, |this, cx| this.answer(cx));
            }));
        Some(window_layer(modal_layer(1.0, true, close, dialog, &theme)))
    }
}

impl Render for KnowledgePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.state.read(cx);
        let Some((tab, Some(id))) = app.knowledge_route() else {
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
            let kind = if tab == KnowledgeTab::Docs {
                "Doc"
            } else {
                "Decision"
            };
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
        let decision = item.kind == "decision";
        let mut properties: Vec<AnyElement> = Vec::new();
        if decision {
            let facts = decision_facts(&item, &knowledge);
            properties.push(self.status_row(&item, &facts, pending, cx));
            properties.extend(Self::text_row("Reason", &facts.reason, false, cx));
            properties.extend(Self::text_row("Review when", &facts.review_when, false, cx));
            properties.push(self.context_row(cx));
            properties.extend(Self::text_row(
                "Governs",
                &facts.governs.join(", "),
                true,
                cx,
            ));
            properties.extend(Self::text_row(
                "Replaces",
                &facts.replaces.join(", "),
                false,
                cx,
            ));
            properties.extend(Self::text_row("Replaced by", &facts.replaced_by, false, cx));
            let can_replace =
                matches!(facts.state, "active" | "challenged") && !facts.successors.is_empty();
            if can_replace {
                let successors = facts.successors.clone();
                let page = cx.weak_entity();
                let trigger = rail_select(
                    "knowledge-replace",
                    "Choose a decision…",
                    Rc::new(move |window: &mut Window, cx: &mut App| {
                        let successors = successors.clone();
                        let _ =
                            page.update(cx, |this, cx| this.open_replace(successors, window, cx));
                    }),
                    cx,
                )
                .child(self.replace_anchor.probe());
                properties.push(rail_property("Replace with", trigger, false, cx));
            }
        } else {
            properties.push(self.context_row(cx));
        }
        let mut document = DocumentPage::new("knowledge-page")
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
        if !decision {
            let archive = Button::new("knowledge-archive", ButtonKind::Secondary)
                .size(ButtonSize::Sm)
                .radius(RAIL_BUTTON_RADIUS)
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
        let popover = self
            .replace
            .read(cx)
            .is_open()
            .then(|| self.replace_anchor.get())
            .flatten()
            .map(|anchor| window_anchor(anchor, self.replace.clone()));
        div()
            .size_full()
            .child(document)
            .children(popover)
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

    fn decision(id: &str, title: &str, status: &str, extra: &str) -> KnowledgeItem {
        KnowledgeItem {
            id: id.into(),
            title: title.into(),
            kind: "decision".into(),
            status: status.into(),
            extra_json: extra.into(),
            ..KnowledgeItem::default()
        }
    }

    #[test]
    fn a_decision_number_reads_every_prefix() {
        assert_eq!(decision_number("Decision 0012: Cache"), Some(12));
        assert_eq!(decision_number("ADR-7"), Some(7));
        assert_eq!(decision_number("verdict_0003."), Some(3));
        assert_eq!(decision_number("Decision 12a"), None);
        assert_eq!(decision_number("Cache 12"), None);
        assert!(cites("decision-0012", "x", "Decision 0012: Cache"));
        assert!(cites("DEC-ID", "dec-id", "Anything"));
    }

    #[test]
    fn a_decision_knows_what_it_replaced_and_who_may_replace_it() {
        let knowledge = Knowledge {
            items: vec![
                decision(
                    "a",
                    "Decision 0001: Old",
                    "superseded",
                    r#"{"supersededBy":"b"}"#,
                ),
                decision(
                    "b",
                    "Decision 0002: New",
                    "active",
                    r#"{"statusReason":"Faster"}"#,
                ),
                decision("c", "Decision 0003: Other", "proposed", "{}"),
            ],
            rules: Vec::new(),
            nodes: vec![StructureNode {
                id: "n".into(),
                path: "app/web".into(),
                decisions: vec!["Decision-0002".into()],
                ..StructureNode::default()
            }],
        };
        let facts = decision_facts(&knowledge.items[1], &knowledge);
        assert_eq!(facts.state, "active");
        assert_eq!(facts.reason, "Faster");
        assert_eq!(facts.replaces, ["Decision 0001: Old"]);
        assert_eq!(facts.governs, ["app/web"]);
        assert_eq!(
            facts.successors,
            [("c".to_string(), "Decision 0003: Other".to_string())]
        );
        assert_eq!(
            decision_facts(&knowledge.items[0], &knowledge).replaced_by,
            "Decision 0002: New"
        );
    }

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
    fn a_reason_stays_on_its_polter_line() {
        assert_eq!(
            polter_value("Too  slow\n- see · notes"),
            "Too slow – see, notes"
        );
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
