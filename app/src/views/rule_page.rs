//! A Rule's page, on the same `KnowledgeDocumentShell` a doc uses: a [`DocumentPage`] whose
//! column holds the editable statement (24px, Enter moves into the why),
//! the optional why as Markdown, and the Rule's Activity (`DocumentActivity`: the last 20
//! changes that named it, a burst of the same edit folded into its latest line, hidden while
//! there are none). The rail's Properties group holds the Status (its chip and the move it
//! allows: Activate, Mark under review, Reactivate), Replaced by, and the Context; the rail
//! ends with Replace and Remove.
//!
//! The statement and the why save 800ms after the last keystroke, Context at once, each under
//! its own summary and no reason. Mark under review, Reactivate, Replace, and Remove are
//! confirmed, with no reason asked: the app never asks a person for a reason. Every write is a
//! Polter rule block through `WriteRule` (`rule_text`), and removing one goes back to the list.

use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, Focusable, FontWeight,
    IntoElement, Render, Subscription, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::chip::chip_rounded;
use polterware_ui::icons::Icon;
use polterware_ui::list_popover::{ListPopover, ListRow};
use polterware_ui::markdown::{MarkdownEditor, MarkdownEditorEvent};
use polterware_ui::patterns::{
    AnchorSlot, ConfirmDialog, DocumentPage, PropertyPills, PropertyPillsEvent, RailState,
    document_footer, document_group, document_group_with, document_title, modal_layer,
    rail_properties, rail_property, rail_select, window_anchor, window_layer,
};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;
use polterware_ui::wrap::wrap_text;

use crate::contracts::{Rule, StructureNode};
use crate::state::{AppState, KnowledgeTab, Load};
use crate::views::knowledge::{
    context_property, rule_handle, rule_status_label, rule_status_tone, rule_text,
};
use crate::views::knowledge_page::document_activity;

/// `createDocumentAutosave`'s debounce.
const AUTOSAVE: Duration = Duration::from_millis(800);
/// The StatusChip's radius on the web.
const CHIP_RADIUS: f32 = 6.0;
/// The Replace list's least width, wider than its trigger.
const SELECT_WIDTH: f32 = 260.0;
/// What an empty why reads.
const WHY_HINT: &str = "Add a why…";

/// A question asked before a write.
#[derive(Clone, Debug, PartialEq)]
enum Ask {
    Activate,
    Review,
    Remove,
    Replace { by: String, by_statement: String },
}

/// The why's editor: the Rule's Markdown, or the hint an empty one reads.
fn why_editor(source: impl Into<String>, cx: &mut Context<RulePage>) -> Entity<MarkdownEditor> {
    cx.new(|cx| {
        let mut why = MarkdownEditor::new(source, cx);
        why.set_placeholder(WHY_HINT);
        why
    })
}

pub struct RulePage {
    state: Entity<AppState>,
    /// The Rule the fields were set for.
    record: Option<String>,
    statement: Entity<TextInput>,
    why: Entity<MarkdownEditor>,
    context: Entity<PropertyPills>,
    /// What the Context tokens were set from: the Rule, its nodes, and the project's.
    context_for: Option<(String, Vec<String>, Vec<StructureNode>)>,
    ask: Option<Ask>,
    ask_focus: FocusHandle,
    /// Whether the rail shows, kept across pages as the web keeps it.
    rail: Entity<RailState>,
    replace: Entity<ListPopover>,
    replace_anchor: AnchorSlot,
    /// The statement and the why waiting out the autosave: the Rule and the value.
    pending_statement: Option<(String, String)>,
    pending_why: Option<(String, String)>,
    /// Bumped by each keystroke, so only the last one's timer saves.
    due: [u64; 2],
    _why_changes: Subscription,
}

impl RulePage {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let page = cx.weak_entity();
        let statement = cx.new(|cx| {
            TextInput::new(cx)
                .prose(24.0, 32.0, FontWeight::SEMIBOLD)
                .wrap()
                .placeholder("Untitled")
                // Enter moves into the why, after the field's own update.
                .on_submit(move |_, window, cx| {
                    let page = page.clone();
                    window.defer(cx, move |window, cx| {
                        let _ = page.update(cx, |this, cx| {
                            this.why.update(cx, |why, cx| why.edit_start(window, cx));
                        });
                    });
                })
        });
        cx.observe(&statement, |this, _, cx| this.statement_typed(cx))
            .detach();
        let why = why_editor("", cx);
        let why_changes = cx.subscribe(&why, Self::why_typed);
        cx.observe(&why, |_, _, cx| cx.notify()).detach();
        let context = cx.new(PropertyPills::new);
        cx.observe(&context, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&context, |this, _, event: &PropertyPillsEvent, cx| {
            let PropertyPillsEvent::Changed { values, .. } = event;
            this.context_changed(values, cx);
        })
        .detach();
        let replace = cx.new(ListPopover::new);
        cx.observe(&replace, |_, _, cx| cx.notify()).detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            record: None,
            statement,
            why,
            context,
            context_for: None,
            ask: None,
            ask_focus: cx.focus_handle(),
            rail: cx.new(|_| RailState::new(true)),
            replace,
            replace_anchor: AnchorSlot::default(),
            pending_statement: None,
            pending_why: None,
            due: [0, 0],
            _why_changes: why_changes,
        }
    }

    fn rule(&self, id: &str, cx: &App) -> Option<Rule> {
        match self.state.read(cx).knowledge() {
            Load::Ready(knowledge) => knowledge.rules.iter().find(|rule| rule.id == id).cloned(),
            _ => None,
        }
    }

    fn nodes(&self, cx: &App) -> Vec<StructureNode> {
        match self.state.read(cx).knowledge() {
            Load::Ready(knowledge) => knowledge.nodes.clone(),
            _ => Vec::new(),
        }
    }

    fn statement_of(&self, rule: &Rule) -> String {
        if rule.statement.is_empty() {
            rule_handle(rule).to_string()
        } else {
            rule.statement.clone()
        }
    }

    /// Shows `rule`'s fields: all of them for another Rule (writing what the last one still
    /// held), else only what changed elsewhere and is not being typed.
    fn sync(
        &mut self,
        rule: &Rule,
        nodes: &[StructureNode],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.record.as_deref() != Some(rule.id.as_str()) {
            self.flush(0, cx);
            self.flush(1, cx);
            self.record = Some(rule.id.clone());
            self.ask = None;
            self.statement.update(cx, |statement, cx| {
                statement.set_text(rule.statement.clone(), cx)
            });
            // A new editor, so nothing of the last Rule's open block comes along.
            self.why = why_editor(rule.why.clone(), cx);
            self._why_changes = cx.subscribe(&self.why, Self::why_typed);
            cx.observe(&self.why, |_, _, cx| cx.notify()).detach();
            let id = rule.id.clone();
            self.state
                .update(cx, |state, cx| state.load_document_activity(id, cx));
        } else {
            let typing = self.statement.read(cx).focus_handle(cx).is_focused(window);
            if !typing
                && self.pending_statement.is_none()
                && self.statement.read(cx).text() != rule.statement
            {
                self.statement.update(cx, |statement, cx| {
                    statement.set_text(rule.statement.clone(), cx)
                });
            }
            if self.pending_why.is_none() {
                self.why
                    .update(cx, |why, cx| why.set_source(rule.why.clone(), cx));
            }
        }
        let shown = matches!(&self.context_for, Some((id, ids, set))
            if *id == rule.id && *ids == rule.scope_node_ids && set.as_slice() == nodes);
        if !shown {
            let property = context_property(&rule.scope_node_ids, nodes);
            self.context
                .update(cx, |pills, cx| pills.set_properties(vec![property], cx));
            self.context_for = Some((rule.id.clone(), rule.scope_node_ids.clone(), nodes.to_vec()));
        }
    }

    fn statement_typed(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.record.clone() else {
            return;
        };
        let text = self.statement.read(cx).text().trim().to_string();
        let saved = self.rule(&id, cx).map(|rule| rule.statement);
        // A blank statement is not a valid Rule: it stays here until one is typed.
        if text.is_empty() || saved.as_deref() == Some(text.as_str()) {
            self.pending_statement = None;
            return;
        }
        self.pending_statement = Some((id, text));
        self.schedule(0, cx);
    }

    fn why_typed(
        &mut self,
        _: Entity<MarkdownEditor>,
        event: &MarkdownEditorEvent,
        cx: &mut Context<Self>,
    ) {
        let MarkdownEditorEvent::Changed(source) = event;
        let Some(id) = self.record.clone() else {
            return;
        };
        if self.rule(&id, cx).is_some_and(|rule| &rule.why == source) {
            self.pending_why = None;
            return;
        }
        self.pending_why = Some((id, source.clone()));
        self.schedule(1, cx);
    }

    /// Saves field `field` (0 the statement, 1 the why) once no keystroke came for a while.
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
            self.pending_statement.take()
        } else {
            self.pending_why.take()
        };
        let Some((id, value)) = pending else {
            return;
        };
        let Some(rule) = self.rule(&id, cx) else {
            return;
        };
        let handle = rule_handle(&rule).to_string();
        let (text, summary) = if field == 0 {
            let text = rule_text(&handle, &value, None, None, None, None);
            (text, summary_line("Update rule statement", &value))
        } else {
            let text = rule_text(&handle, "", None, Some(&value), None, None);
            (
                text,
                summary_line("Update rule why", &self.statement_of(&rule)),
            )
        };
        self.write(text, summary, "Rule updated.", false, cx);
    }

    /// Sets the Rule's scope to the nodes the Context holds, at once and with no question.
    fn context_changed(&mut self, values: &[gpui::SharedString], cx: &mut Context<Self>) {
        let Some(id) = self.record.clone() else {
            return;
        };
        let Some(rule) = self.rule(&id, cx) else {
            return;
        };
        let nodes = self.nodes(cx);
        let mut paths: Vec<String> = nodes
            .iter()
            .filter(|node| {
                values
                    .iter()
                    .any(|value| value.as_ref() == node.id.as_str())
            })
            .map(|node| node.path.clone())
            .collect();
        paths.sort();
        let mut was = rule.scope_paths.clone();
        was.sort();
        if was == paths {
            return;
        }
        let text = rule_text(rule_handle(&rule), "", Some(&paths), None, None, None);
        let summary = summary_line("Update rule context", &self.statement_of(&rule));
        self.write(text, summary, "Rule updated.", false, cx);
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        self.flush(0, cx);
        self.flush(1, cx);
        self.state.update(cx, |state, cx| {
            if state.can_go_back() {
                state.go_back(cx);
            } else {
                state.set_knowledge_tab(KnowledgeTab::Rules, cx);
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
        let (Some(ask), Some(rule)) = (
            self.ask.clone(),
            self.record.clone().and_then(|id| self.rule(&id, cx)),
        ) else {
            return;
        };
        self.ask = None;
        let handle = rule_handle(&rule).to_string();
        let statement = self.statement_of(&rule);
        let (text, summary, written, close) = match ask {
            Ask::Activate => (
                rule_text(&handle, "", None, None, Some("active"), None),
                summary_line("Activate rule", &statement),
                "Rule active.",
                false,
            ),
            Ask::Review => (
                rule_text(&handle, "", None, None, Some("under_review"), None),
                summary_line("Mark rule under review", &statement),
                "Rule under review.",
                false,
            ),
            Ask::Remove => (
                rule_text(&handle, "", None, None, Some("removed"), None),
                summary_line("Remove rule", &statement),
                "Rule removed.",
                true,
            ),
            Ask::Replace { by, by_statement } => (
                rule_text(&handle, "", None, None, None, Some(&by)),
                summary_line("Replace rule", &format!("{statement} with {by_statement}")),
                "Rule replaced.",
                false,
            ),
        };
        self.write(text, summary, written, close, cx);
    }

    /// Writes a Rule's Polter block under `summary`; `close` goes back to the list once it
    /// lands.
    fn write(
        &mut self,
        text: String,
        summary: String,
        written: &'static str,
        close: bool,
        cx: &mut Context<Self>,
    ) {
        let app_state = self.state.downgrade();
        self.state.update(cx, |state, cx| {
            state.write_rule(
                text,
                summary,
                String::new(),
                written,
                move |ok, cx| {
                    if ok
                        && close
                        && let Some(state) = app_state.upgrade()
                    {
                        state.update(cx, |state, cx| {
                            state.set_knowledge_tab(KnowledgeTab::Rules, cx)
                        });
                    }
                },
                cx,
            );
        });
        cx.notify();
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
            .map(|(id, statement)| {
                let page = page.clone();
                let by_statement = statement.clone();
                ListRow::new(
                    statement,
                    Rc::new(move |window: &mut Window, cx: &mut App| {
                        let ask = Ask::Replace {
                            by: id.clone(),
                            by_statement: by_statement.clone(),
                        };
                        let _ = page.update(cx, |this, cx| this.ask(ask, window, cx));
                    }),
                )
                .icon(Icon::ShieldCheck)
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

    fn status_row(&self, rule: &Rule, pending: bool, cx: &mut Context<Self>) -> AnyElement {
        let status = rule.status.as_str();
        let action = match status {
            "under_review" => Some(("Reactivate", Ask::Activate, ButtonKind::Secondary)),
            "replaced" | "removed" => Some(("Reactivate", Ask::Activate, ButtonKind::Secondary)),
            "proposed" => Some(("Activate", Ask::Activate, ButtonKind::Secondary)),
            _ => Some(("Mark under review", Ask::Review, ButtonKind::Ghost)),
        };
        let value = div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(6.0))
            .child(chip_rounded(
                rule_status_label(status),
                rule_status_tone(status),
                CHIP_RADIUS,
                cx,
            ))
            .children(action.map(|(label, ask, kind)| {
                let page = cx.weak_entity();
                Button::new("rule-status-move", kind)
                    .size(ButtonSize::Sm)
                    .label(label)
                    .disabled(pending)
                    .on_click(move |window, cx| {
                        let ask = ask.clone();
                        let _ = page.update(cx, |this, cx| this.ask(ask, window, cx));
                    })
            }));
        rail_property("Status", value, false, cx)
    }

    fn replaced_by_row(&self, rule: &Rule, cx: &App) -> Option<AnyElement> {
        if rule.replaced_by.is_empty() {
            return None;
        }
        let title = match self.state.read(cx).knowledge() {
            Load::Ready(knowledge) => knowledge
                .rules
                .iter()
                .find(|other| other.id == rule.replaced_by)
                .map(|other| self.statement_of(other))
                .unwrap_or_else(|| rule.replaced_by.clone()),
            _ => rule.replaced_by.clone(),
        };
        Some(rail_property(
            "Replaced by",
            div().min_w(px(0.0)).child(wrap_text(title)),
            false,
            cx,
        ))
    }

    fn context_row(&self, cx: &App) -> AnyElement {
        rail_property("Context", self.context.clone(), false, cx)
    }

    fn confirm(&self, rule: &Rule, cx: &mut Context<Self>) -> Option<AnyElement> {
        let ask = self.ask.as_ref()?;
        let theme = *Theme::global(cx);
        let statement = self.statement_of(rule);
        let (heading, description, label) = match ask {
            Ask::Activate => (
                format!("Reactivate “{statement}”?"),
                "It binds agents again.".to_string(),
                "Reactivate",
            ),
            Ask::Review => (
                format!("Mark “{statement}” under review?"),
                "It keeps binding until someone reactivates or replaces it.".to_string(),
                "Mark under review",
            ),
            Ask::Remove => (
                format!("Remove “{statement}”?"),
                "It leaves the lists and agent reads. Its history stays in Activity.".to_string(),
                "Remove",
            ),
            Ask::Replace { by_statement, .. } => (
                format!("Replace “{statement}”?"),
                format!("{by_statement} becomes the Rule in force."),
                "Replace",
            ),
        };
        let page = cx.weak_entity();
        let dismiss = page.clone();
        let close: polterware_ui::Handler = Rc::new(move |_, cx: &mut App| {
            let _ = dismiss.update(cx, |this, cx| this.dismiss(cx));
        });
        let dialog = ConfirmDialog::new("rule-confirm", heading, description)
            .confirm_label(label)
            .focus(&self.ask_focus)
            .on_cancel(close.clone())
            .on_confirm(Rc::new(move |_, cx: &mut App| {
                let _ = page.update(cx, |this, cx| this.answer(cx));
            }));
        let dialog = if matches!(ask, Ask::Remove) {
            dialog.danger()
        } else {
            dialog
        };
        Some(window_layer(modal_layer(1.0, true, close, dialog, &theme)))
    }
}

impl Render for RulePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.state.read(cx);
        let Some((KnowledgeTab::Rules, Some(id))) = app.knowledge_route() else {
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
        let Some(rule) = knowledge.rules.iter().find(|rule| rule.id == id).cloned() else {
            let theme = Theme::global(cx);
            return div().size_full().child(
                DocumentPage::new("rule-page")
                    .on_back(back)
                    .child(document_title("Rule not found", cx))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(theme.color_text_muted)
                            .child(wrap_text(
                                "This Rule is not available in the current project knowledge set. It may have been removed or the link is stale.",
                            )),
                    ),
            );
        };
        self.sync(&rule, &knowledge.nodes, window, cx);
        let mut properties: Vec<AnyElement> = vec![self.status_row(&rule, pending, cx)];
        properties.extend(self.replaced_by_row(&rule, cx));
        properties.push(self.context_row(cx));
        // A Rule may be replaced by any live one but itself.
        let successors: Vec<(String, String)> = knowledge
            .rules
            .iter()
            .filter(|other| other.id != rule.id && other.status != "removed")
            .map(|other| (other.id.clone(), self.statement_of(other)))
            .collect();
        let can_replace =
            !matches!(rule.status.as_str(), "removed" | "replaced") && !successors.is_empty();
        if can_replace {
            let successors = successors.clone();
            let page = cx.weak_entity();
            let trigger = rail_select(
                "rule-replace",
                "Choose a rule…",
                Rc::new(move |window: &mut Window, cx: &mut App| {
                    let successors = successors.clone();
                    let _ = page.update(cx, |this, cx| this.open_replace(successors, window, cx));
                }),
                cx,
            )
            .child(self.replace_anchor.probe());
            properties.push(rail_property("Replace with", trigger, false, cx));
        }
        let mut document = DocumentPage::new("rule-page")
            .scroll_locked(self.why.read(cx).slash_open())
            .on_back(back)
            .rail_state(&self.rail)
            .rail(document_group(
                "Properties",
                Icon::InformationCircle,
                rail_properties().children(properties),
                cx,
            ))
            .child(
                div()
                    .w_full()
                    .mt(px(32.0))
                    .mb(px(8.0))
                    .child(self.statement.clone()),
            )
            // The why, the statement's reason in Markdown, under its own group as a card's
            // sections are.
            .child(div().w_full().mt(px(24.0)).child(document_group_with(
                "Why",
                Icon::HelpCircle,
                None,
                true,
                self.why.clone(),
                cx,
            )))
            .children(document_activity(&self.state, &rule.id, cx));
        let remove = Button::new("rule-remove", ButtonKind::Danger)
            .size(ButtonSize::Sm)
            .label("Remove")
            .disabled(pending || rule.status == "removed")
            .on_click({
                let page = cx.weak_entity();
                move |window, cx| {
                    let _ = page.update(cx, |this, cx| this.ask(Ask::Remove, window, cx));
                }
            })
            .into_any_element();
        document = document.rail(document_footer([remove], cx));
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
            .children(self.confirm(&rule, cx))
    }
}

/// A write's summary naming the Rule it touches, cut to 120 characters.
fn summary_line(action: &str, text: &str) -> String {
    let line: String = text.chars().take(120).collect();
    format!("{action}: {line}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_summary_names_the_rule_and_stops_at_120_characters() {
        assert_eq!(
            summary_line("Remove rule", "Never call the network"),
            "Remove rule: Never call the network"
        );
        let long = "a".repeat(200);
        assert_eq!(summary_line("Remove rule", &long).chars().count(), 133);
    }

    #[test]
    fn status_labels_and_tones_read_as_the_list_shows_them() {
        use polterware_ui::theme::StatusTone;
        assert_eq!(rule_status_label("under_review"), "Under review");
        assert_eq!(rule_status_tone("active"), StatusTone::Ok);
        assert_eq!(rule_status_tone("proposed"), StatusTone::Muted);
    }
}
