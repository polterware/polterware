//! App settings' own pages: the CLI reference (`CliReferenceSettings`: the usage, every
//! command by group, the interactive keys, then each command's usage, options, and example,
//! with Copy full reference as Markdown), Shortcuts (`ShortcutsPage`: each remappable command
//! by group, recorded from the next key press), Appearance (Reduce transparency), Local core
//! (`CoreCard`: its status and lifecycle, its idle timeout, start on open, and what quitting
//! does), Privacy, and the app's Danger zone (Reset local store, after a confirmation).

use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, ClipboardItem, Context, FontWeight, IntoElement, SharedString, Window, div,
    prelude::*, px, relative,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::chip::chip_rounded;
use polterware_ui::icon_tile::IconTile;
use polterware_ui::icons::Icon;
use polterware_ui::kbd::kbd;
use polterware_ui::menu::MenuItem;
use polterware_ui::patterns::{
    scroll_to_settings_section, settings_anchor, settings_group, settings_row,
};
use polterware_ui::theme::{StatusTone, Theme, tw};
use polterware_ui::toast::ToastTone;
use polterware_ui::wrap::wrap_text;

use super::project::CHIP_RADIUS;
use super::{Ask, SettingsView};
use crate::cli_reference::{self, CliCommand, Flag, INTRO};
use crate::prefs::{Idle, QuitMode};
use crate::shortcuts::{self, OwnedChord};
use crate::state::CoreRun;

/// How long a copy button shows its check (`COPIED_HOLD_MS`).
const COPIED_HOLD: Duration = Duration::from_millis(1500);
/// The selects' width on Local core.
const SELECT_WIDTH: f32 = 200.0;

/// `STATE_TONE`.
fn run_tone(run: CoreRun) -> StatusTone {
    match run {
        CoreRun::Running => StatusTone::Ok,
        CoreRun::Stopped => StatusTone::Muted,
        CoreRun::Starting | CoreRun::Stopping => StatusTone::Warn,
        CoreRun::Error => StatusTone::Danger,
    }
}

/// A reference table's column: its title and its share of the width (the rest share what
/// is left).
struct RefColumn {
    title: &'static str,
    share: Option<f32>,
}

impl SettingsView {
    /// `CodeBlock`: the command in mono on the surface, with Copy, which shows its check for a
    /// moment once it copied.
    fn code_block(
        &self,
        key: String,
        command: &str,
        prompt: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let copied = self
            .copied
            .as_ref()
            .is_some_and(|(copied, at)| *copied == key && at.elapsed() < COPIED_HOLD);
        let view = cx.entity().downgrade();
        let text = command.to_string();
        let state = self.state.clone();
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(tw(3.0)))
            .rounded(px(theme.radius_control))
            .border_1()
            .border_color(theme.color_border_subtle)
            .bg(theme.color_surface)
            .py(px(tw(1.0)))
            .pl(px(tw(3.0)))
            .pr(px(tw(1.0)))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .font_family(theme.font_mono)
                    .text_size(px(12.0))
                    .text_color(theme.color_text_primary)
                    .child(wrap_text(if prompt {
                        format!("$ {command}")
                    } else {
                        command.to_string()
                    })),
            )
            .child(
                Button::new(SharedString::from(key.clone()), ButtonKind::Ghost)
                    .size(ButtonSize::Sm)
                    .icon(if copied { Icon::Check } else { Icon::Copy })
                    .label("Copy")
                    .tooltip(format!("Copy {command}"))
                    .on_click(move |_, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                        state.update(cx, |state, cx| {
                            state.push_toast("Command copied.", ToastTone::Success);
                            cx.notify();
                        });
                        let key = key.clone();
                        let _ = view.update(cx, |this, cx| {
                            this.copied = Some((key, Instant::now()));
                            cx.notify();
                            cx.spawn(async move |this, cx| {
                                cx.background_executor().timer(COPIED_HOLD).await;
                                let _ = this.update(cx, |_, cx| cx.notify());
                            })
                            .detach();
                        });
                    }),
            )
            .into_any_element()
    }

    /// `ReferenceTable`: a header of muted labels, then each row's cells, every text wrapping
    /// in its column.
    fn reference_table(columns: &[RefColumn], rows: Vec<Vec<AnyElement>>, cx: &App) -> AnyElement {
        let theme = *Theme::global(cx);
        let cell = |share: Option<f32>, child: AnyElement| {
            let cell = div().min_w(px(0.0)).px(px(tw(2.0))).py(px(tw(2.0)));
            match share {
                Some(share) => cell.flex_none().w(relative(share)),
                None => cell.flex_1(),
            }
            .child(child)
        };
        let header = div()
            .flex()
            .border_b_1()
            .border_color(theme.color_border_subtle)
            .children(columns.iter().map(|column| {
                cell(
                    column.share,
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.color_text_muted)
                        .child(column.title)
                        .into_any_element(),
                )
            }));
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(header)
            .children(rows.into_iter().map(|cells| {
                div()
                    .flex()
                    .items_start()
                    .border_b_1()
                    .border_color(theme.color_border_subtle)
                    .children(
                        cells
                            .into_iter()
                            .zip(columns.iter())
                            .map(|(content, column)| cell(column.share, content)),
                    )
            }))
            .into_any_element()
    }

    fn flag_table(flags: &[Flag], required: bool, cx: &App) -> AnyElement {
        let theme = *Theme::global(cx);
        let mut columns = vec![
            RefColumn {
                title: "Flag",
                share: Some(0.22),
            },
            RefColumn {
                title: "Value",
                share: Some(0.14),
            },
        ];
        if required {
            columns.push(RefColumn {
                title: "Required",
                share: Some(0.12),
            });
        }
        columns.push(RefColumn {
            title: "Description",
            share: None,
        });
        let mono = |text: String, muted: bool| {
            div()
                .font_family(theme.font_mono)
                .text_size(px(12.0))
                .text_color(if muted {
                    theme.color_text_muted
                } else {
                    theme.color_text_primary
                })
                .child(wrap_text(text))
                .into_any_element()
        };
        let rows = flags
            .iter()
            .map(|flag| {
                let mut cells = vec![
                    mono(flag.name.clone(), false),
                    mono(
                        flag.value
                            .as_ref()
                            .map(|value| format!("<{value}>"))
                            .unwrap_or_default(),
                        true,
                    ),
                ];
                if required {
                    cells.push(
                        div()
                            .text_size(px(12.0))
                            .child(if flag.required { "Yes" } else { "" })
                            .into_any_element(),
                    );
                }
                cells.push(
                    div()
                        .text_size(px(12.5))
                        .line_height(relative(1.625))
                        .child(wrap_text(flag.description.clone()))
                        .into_any_element(),
                );
                cells
            })
            .collect();
        Self::reference_table(&columns, rows, cx)
    }

    fn section_heading(text: impl Into<SharedString>, cx: &App) -> AnyElement {
        let theme = Theme::global(cx);
        div()
            .font_family(theme.font_mono)
            .text_size(px(15.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.color_text_primary)
            .child(text.into())
            .into_any_element()
    }

    fn group_heading(text: &str, cx: &App) -> AnyElement {
        div()
            .text_size(px(11.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(Theme::global(cx).color_text_muted)
            .child(SharedString::from(text.to_uppercase()))
            .into_any_element()
    }

    fn prose(text: impl Into<SharedString>, cx: &App) -> AnyElement {
        div()
            .text_size(px(13.0))
            .line_height(relative(1.625))
            .text_color(Theme::global(cx).color_text_secondary)
            .child(wrap_text(text))
            .into_any_element()
    }

    fn command_section(&self, command: &CliCommand, cx: &mut Context<Self>) -> AnyElement {
        let mut section = div()
            .flex()
            .flex_col()
            .gap(px(tw(4.0)))
            .child(Self::section_heading(command.name.clone(), cx))
            .child(Self::prose(command.summary.clone(), cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(tw(2.0)))
                    .child(Self::group_heading("Usage", cx))
                    .child(self.code_block(
                        format!("cli-usage-{}", command.id),
                        &command.synopsis,
                        false,
                        cx,
                    )),
            );
        if !command.flags.is_empty() {
            section = section.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(tw(2.0)))
                    .child(Self::group_heading("Options", cx))
                    .child(Self::flag_table(&command.flags, true, cx)),
            );
        }
        settings_anchor(
            command.id.clone(),
            section.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(tw(2.0)))
                    .child(Self::group_heading("Example", cx))
                    .child(self.code_block(
                        format!("cli-example-{}", command.id),
                        &command.example,
                        true,
                        cx,
                    )),
            ),
        )
        .into_any_element()
    }

    pub(super) fn cli_reference_page(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let reference = cli_reference::reference();
        let state = self.state.clone();
        let top = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(tw(3.0)))
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(theme.color_text_muted)
                    .child("Every polterware command, its options, and the interactive keys."),
            )
            .child(
                Button::new("settings-cli-copy-reference", ButtonKind::Primary)
                    .size(ButtonSize::Sm)
                    .pill()
                    .icon(Icon::Copy)
                    .label("Copy full reference")
                    .on_click(move |_, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(reference.markdown()));
                        state.update(cx, |state, cx| {
                            state.push_toast("Reference copied.", ToastTone::Success);
                            cx.notify();
                        });
                    }),
            );
        let usage = div()
            .flex()
            .flex_col()
            .gap(px(tw(4.0)))
            .child(Self::section_heading("Usage", cx))
            .child(Self::prose(INTRO, cx))
            .child(self.code_block("cli-usage".into(), &reference.usage, false, cx))
            .child(Self::flag_table(&reference.usage_flags, false, cx));
        let index_columns = [
            RefColumn {
                title: "Command",
                share: Some(0.32),
            },
            RefColumn {
                title: "Description",
                share: None,
            },
        ];
        let mono_cell = |text: &str| {
            div()
                .font_family(theme.font_mono)
                .text_size(px(12.0))
                .text_color(theme.color_text_primary)
                .child(SharedString::from(text.to_string()))
                .into_any_element()
        };
        let text_cell = |text: &str| {
            div()
                .text_size(px(12.5))
                .line_height(relative(1.625))
                .child(wrap_text(text.to_string()))
                .into_any_element()
        };
        let commands = div()
            .flex()
            .flex_col()
            .gap(px(tw(6.0)))
            .child(Self::section_heading("All commands", cx))
            .children(reference.groups.iter().map(|group| {
                let rows = reference
                    .grouped(group)
                    .into_iter()
                    .map(|command| {
                        let target = command.id.clone();
                        vec![
                            div()
                                .id(SharedString::from(format!("cli-index-{target}")))
                                .cursor_pointer()
                                .hover(move |row| row.text_color(theme.color_info))
                                .child(mono_cell(&command.name))
                                .on_click(move |_, window, cx| {
                                    scroll_to_settings_section(target.clone(), window, cx)
                                })
                                .into_any_element(),
                            text_cell(&command.summary),
                        ]
                    })
                    .collect();
                div()
                    .flex()
                    .flex_col()
                    .gap(px(tw(2.0)))
                    .child(Self::group_heading(&group.label, cx))
                    .child(Self::reference_table(&index_columns, rows, cx))
            }));
        let key_columns = [
            RefColumn {
                title: "Key",
                share: Some(0.32),
            },
            RefColumn {
                title: "Action",
                share: None,
            },
        ];
        let keys = div()
            .flex()
            .flex_col()
            .gap(px(tw(4.0)))
            .child(Self::section_heading("Interactive keys", cx))
            .child(Self::prose(
                "Keys available in the interactive status view started by running polterware with no command.",
                cx,
            ))
            .child(Self::reference_table(
                &key_columns,
                reference
                    .keys
                    .iter()
                    .map(|key| {
                        vec![
                            div().flex().child(kbd(key.keys.clone(), &theme)).into_any_element(),
                            text_cell(&key.description),
                        ]
                    })
                    .collect(),
                cx,
            ));
        let mut page = div()
            .flex()
            .flex_col()
            .gap(px(tw(12.0)))
            .child(top)
            .child(usage)
            .child(settings_anchor("Commands", commands))
            .child(keys);
        for group in &reference.groups {
            let mut block = div().flex().flex_col().gap(px(tw(12.0))).child(
                div()
                    .pb(px(tw(2.0)))
                    .border_b_1()
                    .border_color(theme.color_border)
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.color_text_muted)
                    .child(SharedString::from(group.label.to_uppercase())),
            );
            for command in reference.grouped(group) {
                block = block.child(self.command_section(command, cx));
            }
            page = page.child(block);
        }
        page.into_any_element()
    }

    pub(super) fn shortcuts_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let overrides = self.state.read(cx).shortcut_overrides().clone();
        let recording = self.recording.as_ref().map(|(id, _)| *id);
        let view = cx.entity().downgrade();
        let mut group = settings_group("Shortcuts").description(
            "Shortcuts apply on this computer. Single-key shortcuts such as [ and N never fire while you type in a field. \u{2318}/ lists every shortcut, including the fixed G sequences. Escape cancels recording and stays reserved for dialogs.",
        );
        for shortcut_group in shortcuts::Group::ALL {
            let mut block = div().flex().flex_col().gap(px(tw(4.0))).child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.color_text_muted)
                    .child(shortcut_group.label()),
            );
            for command in shortcuts::in_group(shortcut_group) {
                let current = shortcuts::resolve(command, &overrides);
                let custom = overrides.contains_key(command.id);
                let is_recording = recording == Some(command.id);
                let id = command.id;
                let mut text = div().flex_1().min_w(px(0.0)).child(
                    div()
                        .text_size(px(12.5))
                        .text_color(theme.color_text_primary)
                        .child(command.label),
                );
                if custom {
                    text = text.child(
                        div()
                            .mt(px(tw(0.5)))
                            .text_size(px(11.0))
                            .text_color(theme.color_text_muted)
                            .child(SharedString::from(format!(
                                "Default {}",
                                OwnedChord::from(command.default).format()
                            ))),
                    );
                }
                let chord = if is_recording {
                    div()
                        .min_w(px(4.5 * tw(4.0)))
                        .text_center()
                        .text_size(px(11.0))
                        .text_color(theme.color_text_muted)
                        .child("Press keys\u{2026}")
                        .into_any_element()
                } else {
                    div()
                        .min_w(px(4.5 * tw(4.0)))
                        .flex()
                        .justify_center()
                        .child(kbd(current.format(), &theme))
                        .into_any_element()
                };
                let mut controls = div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(tw(2.0)))
                    .child(chord);
                if is_recording {
                    let cancel = view.clone();
                    controls = controls.child(
                        Button::new(
                            SharedString::from(format!("shortcut-cancel-{id}")),
                            ButtonKind::Ghost,
                        )
                        .size(ButtonSize::Sm)
                        .label("Cancel")
                        .on_click(move |_, cx| {
                            let _ = cancel.update(cx, |this, cx| {
                                this.recording = None;
                                cx.notify();
                            });
                        }),
                    );
                } else {
                    let edit = view.clone();
                    controls = controls.child(
                        Button::new(
                            SharedString::from(format!("shortcut-edit-{id}")),
                            ButtonKind::Ghost,
                        )
                        .size(ButtonSize::Sm)
                        .label("Edit")
                        .on_click(move |_, cx| {
                            let _ = edit.update(cx, |this, cx| this.record(id, cx));
                        }),
                    );
                    if custom {
                        let reset = self.state.clone();
                        controls = controls.child(
                            Button::new(
                                SharedString::from(format!("shortcut-reset-{id}")),
                                ButtonKind::Ghost,
                            )
                            .size(ButtonSize::Sm)
                            .label("Reset")
                            .on_click(move |_, cx| {
                                reset.update(cx, |state, cx| {
                                    state.reset_shortcut(id);
                                    cx.notify();
                                });
                            }),
                        );
                    }
                }
                block = block.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(tw(3.0)))
                        .child(
                            div()
                                .flex_none()
                                .child(IconTile::new(Icon::Zap).size(24.0, 14.0)),
                        )
                        .child(text)
                        .child(controls),
                );
            }
            group = group.child(block);
        }
        group.into_any_element()
    }

    /// `ShortcutRecorder`: the next key press, chord or Escape, goes to the recording.
    fn record(&mut self, id: &'static str, cx: &mut Context<Self>) {
        let view = cx.entity().downgrade();
        let subscription = cx.intercept_keystrokes(move |event, _, cx| {
            let keystroke = event.keystroke.clone();
            let handled = view
                .update(cx, |this, cx| {
                    let Some((recording, _)) = this.recording.as_ref() else {
                        return false;
                    };
                    let recording = *recording;
                    if keystroke.key == "escape" {
                        this.stop_recording(cx);
                        return true;
                    }
                    let Some(chord) = OwnedChord::from_keystroke(&keystroke) else {
                        return false;
                    };
                    this.stop_recording(cx);
                    this.state.update(cx, |state, cx| {
                        state.set_shortcut(recording, chord);
                        cx.notify();
                    });
                    true
                })
                .unwrap_or(false);
            if handled {
                cx.stop_propagation();
            }
        });
        self.recording = Some((id, subscription));
        cx.notify();
    }

    /// Ends the recording; its interceptor is dropped once the key it heard is done.
    fn stop_recording(&mut self, cx: &mut Context<Self>) {
        if let Some(recording) = self.recording.take() {
            cx.defer(move |_| drop(recording));
        }
        cx.notify();
    }

    pub(super) fn appearance_page(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let reduce = self.state.read(cx).reduce_transparency();
        self.transparency.update(cx, |switch, _| {
            if switch.is_on() != reduce {
                switch.set_on(reduce);
            }
        });
        settings_group("Appearance")
            .description("Choose frosted or opaque macOS window chrome on this computer.")
            .child(
                settings_row("Reduce transparency")
                    .description(
                        "On macOS, the title bar and sidebar can show a frosted glass view of windows behind Polterware. This uses a solid fill instead.",
                    )
                    .child(self.transparency.clone()),
            )
            .into_any_element()
    }

    pub(super) fn core_page(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let app = self.state.read(cx);
        let run = app.core_run();
        let error = app.core_error();
        let prefs = app.core_prefs();
        let applied = app.applied_idle();
        let quit = app.quit_mode();
        let busy = matches!(run, CoreRun::Starting | CoreRun::Stopping);
        self.auto_start.update(cx, |switch, _| {
            if switch.is_on() != prefs.auto_start {
                switch.set_on(prefs.auto_start);
            }
        });
        let state = self.state.clone();
        let action = |id: &'static str,
                      label: &'static str,
                      kind: ButtonKind,
                      disabled: bool,
                      act: fn(
            &mut crate::state::AppState,
            &mut Context<crate::state::AppState>,
        )| {
            let state = state.clone();
            Button::new(id, kind)
                .size(ButtonSize::Sm)
                .label(label)
                .disabled(disabled)
                .on_click(move |_, cx| {
                    state.update(cx, |state, cx| {
                        act(state, cx);
                        cx.notify();
                    });
                })
        };
        let mut status =
            settings_row("Status").child(chip_rounded(run.label(), run_tone(run), CHIP_RADIUS, cx));
        if run == CoreRun::Running {
            status = status
                .child(action(
                    "settings-core-restart",
                    "Restart",
                    ButtonKind::Secondary,
                    busy,
                    |state, cx| state.restart_core(cx),
                ))
                .child(action(
                    "settings-core-stop",
                    "Stop",
                    ButtonKind::Secondary,
                    busy,
                    |state, cx| state.stop_core(cx),
                ));
        } else {
            status = status.child(action(
                "settings-core-start",
                "Start",
                ButtonKind::Secondary,
                busy,
                |state, cx| state.start_core(cx),
            ));
        }
        let logs_state = self.state.clone();
        status = status.child(
            Button::new("settings-core-logs", ButtonKind::Ghost)
                .size(ButtonSize::Sm)
                .label("Show logs")
                .on_click(move |_, cx| logs_state.read(cx).reveal_core_logs(cx)),
        );
        let idle_items = Idle::ALL
            .into_iter()
            .map(|idle| {
                let state = self.state.clone();
                MenuItem::radio(
                    idle.label(),
                    idle == prefs.idle,
                    Rc::new(move |_, cx: &mut App| {
                        state.update(cx, |state, cx| {
                            state.set_core_idle(idle);
                            cx.notify();
                        });
                    }),
                )
            })
            .collect();
        let quit_items = QuitMode::ALL
            .into_iter()
            .map(|mode| {
                let state = self.state.clone();
                MenuItem::radio(
                    mode.label(),
                    mode == quit,
                    Rc::new(move |_, cx: &mut App| {
                        state.update(cx, |state, cx| {
                            state.set_quit_mode(mode);
                            cx.notify();
                        });
                    }),
                )
            })
            .collect();
        let idle_select = self.idle_picker.trigger(
            "settings-core-idle",
            prefs.idle.label(),
            SELECT_WIDTH,
            idle_items,
            false,
            cx,
        );
        let quit_select = self.quit_picker.trigger(
            "settings-core-quit",
            quit.label(),
            SELECT_WIDTH,
            quit_items,
            false,
            cx,
        );
        let mut group = settings_group("Local core")
            .description(
                "The local core serves the Desktop, CLI, and MCP clients. It keeps running after you quit until you stop it.",
            )
            .child(status);
        if let Some(error) = error {
            group = group.child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme.color_danger)
                    .child(SharedString::from(error)),
            );
        }
        let idle_note = (run == CoreRun::Running && prefs.idle != applied)
            .then_some("Applies the next time the core starts. Restart it to apply now.");
        let mut idle_row = settings_row("Stop automatically when idle");
        if let Some(note) = idle_note {
            idle_row = idle_row.description(note);
        }
        group
            .child(idle_row.child(idle_select))
            .child(
                settings_row("Start the core when Polterware opens").child(self.auto_start.clone()),
            )
            .child(settings_row("When quitting Polterware").child(quit_select))
            .into_any_element()
    }

    pub(super) fn privacy_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let paragraph = |text: &'static str| {
            div()
                .text_size(px(12.0))
                .text_color(theme.color_text_muted)
                .child(wrap_text(text))
        };
        settings_group("Privacy")
            .description("Local-first project data and provider credential handling.")
            .child(paragraph(
                "Polterware runs as a local Desktop on your machine. There is no account sign-in and no hosted workspace for day-to-day use.",
            ))
            .child(paragraph(
                "Project data stays on this device. Provider credentials used through Integrations remain in the OS or CLI credential store \u{2014} Polterware does not store provider tokens in the app database.",
            ))
            .into_any_element()
    }

    pub(super) fn app_danger_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let view = cx.entity().downgrade();
        settings_group("Danger zone")
            .description(
                "Reset recreates the demonstration project in the local store. Architecture, resources, and knowledge are replaced.",
            )
            .child(
                settings_row("Reset local store")
                    .description("Delete local data and reseed the demonstration project.")
                    .child(
                        Button::new("settings-reset-store", ButtonKind::Ghost)
                            .size(ButtonSize::Sm)
                            .label("Reset local store")
                            .on_click(move |_, cx| {
                                let _ = view.update(cx, |this, cx| this.ask(Ask::ResetStore, cx));
                            }),
                    ),
            )
            .into_any_element()
    }
}
