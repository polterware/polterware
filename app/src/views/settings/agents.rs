//! App settings' Agents pages (`agents-settings-panel.tsx`): the MCP server (`McpStatusCard`:
//! the local helper and its process, the database it serves, each MCP client this computer
//! has and whether it points here, and the command and Cursor config a client pastes), Agent
//! setup (`AgentSetupCard`: the setup doctor's checks, each with the command that fixes it),
//! Hooks (the plugin's hook switches), and the AI skill (`SkillCard`: the instructions for an
//! agent without the plugin, to copy or save).

use gpui::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, IntoElement, SharedString,
    Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::chip::chip_rounded;
use polterware_ui::icon_tile::{IconTile, TileTone};
use polterware_ui::icons::Icon;
use polterware_ui::patterns::{settings_group, settings_row};
use polterware_ui::skeleton::text_skeleton;
use polterware_ui::switch::Switch;
use polterware_ui::theme::{StatusTone, Theme};
use polterware_ui::toast::ToastTone;

use super::SettingsView;
use super::project::{CHIP_RADIUS, muted_line};
use crate::contracts::AgentSetup;
use crate::mcp::McpClient;
use crate::state::{AppState, Load};

/// The skill's file name (`POLTERWARE_AI_SKILL_FILENAME`).
const SKILL_FILE: &str = "polterware-ai-skill.md";

/// `POLTERWARE_AI_SKILL_MARKDOWN`: the skill for agents without the plugin, pointing at the
/// commands that describe themselves so it never drifts from the binary.
pub const SKILL_MARKDOWN: &str = r#"# Polterware project context

This project is tracked by Polterware, a local store of the project's Work cards, Structure, Decisions, Facts, and change history. Read it through the `polterware` CLI (or the Polterware MCP server) instead of guessing.

1. Start a session with `polterware status`.
2. Before you work on a card, a Structure node, or a file, read `polterware brief <slug|path|file>`: the Rules you must follow, open tasks and questions, the decisions that govern it, scope facts, and the milestone, in one read.
3. Find anything with `polterware search <terms>`.
4. Write Work cards, Decisions, and Facts in Polter format: `polterware format` prints the core rules (topics: `structure`, `decisions`, `facts`, `feature`); then `polterware apply <file|-> --dry-run` and apply. Every write needs a summary and a rationale.
5. End with `polterware handoff --card <slug> --note "..."`.

Over MCP the server has four tools: `polter.query` for the same reads ("brief …", "search …", "show <id>", "commits <slug>"), `polter.format`, `polter.apply` for Polter writes, and `doc.upsert` for docs with a Markdown body. `polterware --help` lists every command.

`polterware setup claude` or `polterware setup codex` installs the full plugin: this skill with the format embedded, the hooks, and the MCP server.

Polterware never stores credentials; do not ask it for secrets.
"#;

/// One row of the setup doctor (`setupChecks`).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Check {
    label: &'static str,
    ok: bool,
    state: String,
    hint: Option<String>,
}

/// `setupChecks`: what is ready, and the command that fixes what is not.
fn setup_checks(setup: &AgentSetup) -> Vec<Check> {
    let check = |label, ok, state: &str, hint: Option<&str>| Check {
        label,
        ok,
        state: state.to_string(),
        hint: hint.map(str::to_string),
    };
    let mut checks = vec![if setup.polterware_path.is_empty() {
        check(
            "polterware on PATH",
            false,
            "Missing",
            Some("Hooks skip themselves until it is on PATH."),
        )
    } else {
        check("polterware on PATH", true, "Found", None)
    }];
    checks.push(if setup.claude_path.is_empty() {
        check("Claude Code", false, "Not detected", None)
    } else if !setup.problem.is_empty() {
        check("Claude plugin", false, "Unreadable", Some(&setup.problem))
    } else if !setup.plugin_installed {
        check(
            "Claude plugin",
            false,
            "Not installed",
            Some("polterware setup claude"),
        )
    } else if setup.plugin_version != setup.embedded_version {
        check(
            "Claude plugin",
            false,
            &format!("{} behind {}", setup.plugin_version, setup.embedded_version),
            Some("polterware setup claude"),
        )
    } else if setup.plugin_enabled {
        check(
            "Claude plugin",
            true,
            &format!("{} enabled", setup.plugin_version),
            None,
        )
    } else {
        check(
            "Claude plugin",
            false,
            &format!("{} disabled", setup.plugin_version),
            Some("polterware setup claude --enable"),
        )
    });
    checks.push(if setup.skill_ready {
        check("Codex skill", true, "Installed", None)
    } else {
        check(
            "Codex skill",
            false,
            "Not installed",
            Some("polterware setup codex"),
        )
    });
    checks
}

/// A hint that copies on click is a command (`looksLikeCommand`).
fn is_command(hint: &str) -> bool {
    hint.starts_with("polterware ")
}

/// `truncatePath`: the path's end, at most `max` characters.
fn truncate_path(path: &str, max: usize) -> String {
    if path.is_empty() {
        return "\u{2014}".into();
    }
    let count = path.chars().count();
    if count <= max {
        return path.to_string();
    }
    let tail: String = path.chars().skip(count - (max - 1)).collect();
    format!("\u{2026}{tail}")
}

/// `CLIENT_STATE`'s tone.
fn client_tone(client: &McpClient) -> StatusTone {
    match client.status.as_str() {
        "configured" => StatusTone::Ok,
        "configured_other" => StatusTone::Warn,
        _ => StatusTone::Muted,
    }
}

/// Copies `text` and says what was copied.
fn copy(state: &Entity<AppState>, what: &str, text: String, cx: &mut App) {
    cx.write_to_clipboard(ClipboardItem::new_string(text));
    let message = format!("{what} copied.");
    state.update(cx, |state, cx| {
        state.push_toast(message, ToastTone::Success);
        cx.notify();
    });
}

fn refresh_button(
    id: &'static str,
    disabled: bool,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    Button::new(id, ButtonKind::Ghost)
        .size(ButtonSize::Icon)
        .icon(Icon::RefreshCw)
        .tooltip("Refresh")
        .disabled(disabled)
        .on_click(on_click)
        .into_any_element()
}

impl SettingsView {
    pub(super) fn mcp_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let app = self.state.read(cx);
        let status = app.mcp_status().cloned();
        let clients = app
            .mcp_clients()
            .map(<[McpClient]>::to_vec)
            .unwrap_or_default();
        let busy = app.mcp_busy();
        let running = status.as_ref().is_some_and(|status| status.running);
        let state = self.state.clone();
        let refresh_state = state.clone();
        let mono = |text: String| {
            div()
                .font_family(theme.font_mono)
                .text_size(px(12.0))
                .text_color(theme.color_text_secondary)
                .child(SharedString::from(text))
        };
        let lifecycle_state = state.clone();
        let lifecycle = if running {
            Button::new("settings-mcp-stop", ButtonKind::Secondary)
                .size(ButtonSize::Sm)
                .label("Stop")
                .disabled(busy)
                .on_click(move |_, cx| {
                    lifecycle_state.update(cx, |state, cx| state.stop_mcp(cx));
                })
        } else {
            Button::new("settings-mcp-start", ButtonKind::Primary)
                .size(ButtonSize::Sm)
                .label("Start")
                .disabled(busy || status.is_none())
                .on_click(move |_, cx| {
                    lifecycle_state.update(cx, |state, cx| state.start_mcp(cx));
                })
        };
        let chip = match &status {
            None => chip_rounded("Checking", StatusTone::Muted, CHIP_RADIUS, cx),
            Some(_) if running => chip_rounded("Running", StatusTone::Ok, CHIP_RADIUS, cx),
            Some(_) => chip_rounded("Stopped", StatusTone::Muted, CHIP_RADIUS, cx),
        };
        let mut group = settings_group("MCP server")
            .description(
                "A local helper that lets MCP clients read and write this project through the core.",
            )
            .action(refresh_button("settings-mcp-refresh", busy, move |_, cx| {
                refresh_state.update(cx, |state, cx| state.refresh_mcp(cx));
            }))
            .child(
                settings_row("Local MCP helper")
                    .icon(Icon::Plug, TileTone::Integration)
                    .child(chip)
                    .child(lifecycle),
            )
            .child(
                settings_row("Process").icon(Icon::Cpu, TileTone::Neutral).child(mono(
                    status
                        .as_ref()
                        .and_then(|status| status.pid)
                        .map(|pid| format!("PID {pid}"))
                        .unwrap_or_else(|| "\u{2014}".into()),
                )),
            )
            .child(
                settings_row("Database").icon(Icon::Database, TileTone::Data).child(
                    div().max_w(px(260.0)).overflow_hidden().child(mono(truncate_path(
                        status
                            .as_ref()
                            .map(|status| status.db_path.as_str())
                            .unwrap_or_default(),
                        56,
                    ))),
                ),
            );
        for client in &clients {
            let set_state = state.clone();
            let id = client.id.clone();
            let mut row = settings_row(client.name.clone())
                .description(client.error.clone())
                .icon(Icon::Bot, TileTone::Execution)
                .child(chip_rounded(
                    client.label(),
                    client_tone(client),
                    CHIP_RADIUS,
                    cx,
                ));
            if client.status != "not_installed" {
                let configured = client.status == "configured";
                row = row.child(
                    Button::new(
                        SharedString::from(format!("settings-mcp-client-{}", client.id)),
                        if configured {
                            ButtonKind::Ghost
                        } else {
                            ButtonKind::Secondary
                        },
                    )
                    .size(ButtonSize::Sm)
                    .label(if configured { "Remove" } else { "Set up" })
                    .disabled(busy)
                    .on_click(move |_, cx| {
                        let id = id.clone();
                        set_state.update(cx, |state, cx| state.set_mcp_client(id, !configured, cx));
                    }),
                );
            }
            group = group.child(row);
        }
        let command = status
            .as_ref()
            .map(|status| status.command.clone())
            .unwrap_or_default();
        let config = status
            .as_ref()
            .map(|status| status.cursor_config.clone())
            .unwrap_or_default();
        let (command_state, config_state) = (state.clone(), state.clone());
        group
            .child(
                settings_row("Connect a client")
                    .description("Paste into your agent's MCP settings.")
                    .icon(Icon::SourceCodeIcon, TileTone::Neutral)
                    .child(
                        Button::new("settings-mcp-copy-command", ButtonKind::Ghost)
                            .size(ButtonSize::Sm)
                            .label("Copy command")
                            .disabled(command.is_empty())
                            .on_click(move |_, cx| {
                                copy(&command_state, "Command", command.clone(), cx)
                            }),
                    )
                    .child(
                        Button::new("settings-mcp-copy-config", ButtonKind::Ghost)
                            .size(ButtonSize::Sm)
                            .label("Copy Cursor config")
                            .disabled(config.is_empty())
                            .on_click(move |_, cx| {
                                copy(&config_state, "Cursor config", config.clone(), cx)
                            }),
                    ),
            )
            .into_any_element()
    }

    fn setup(&self, cx: &App) -> (Option<AgentSetup>, bool) {
        match self.state.read(cx).agent_setup() {
            Load::Ready(setup) => (Some(setup.clone()), true),
            Load::Failed(_) => (None, true),
            _ => (None, false),
        }
    }

    pub(super) fn setup_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let (setup, loaded) = self.setup(cx);
        let refresh_state = self.state.clone();
        let mut group = settings_group("Agent setup")
            .description(
                "What Claude Code and Codex need to read this project. Copy a fix and run it in your terminal.",
            )
            .action(refresh_button("settings-setup-refresh", false, move |_, cx| {
                refresh_state.update(cx, |state, cx| state.load_agent_setup(cx));
            }));
        match setup {
            None if !loaded => {
                group = group.child(text_skeleton(3, 12.0, window, cx));
            }
            None => group = group.child(muted_line("Setup state is unavailable.", cx)),
            Some(setup) => {
                for (index, check) in setup_checks(&setup).into_iter().enumerate() {
                    let command = check.hint.clone().filter(|hint| is_command(hint));
                    let mut row = settings_row(check.label).icon(Icon::Bot, TileTone::Identity);
                    if let Some(hint) = &check.hint {
                        row = row.description(hint.clone());
                    }
                    if let Some(command) = command {
                        let state = self.state.clone();
                        row = row.child(
                            Button::new(
                                SharedString::from(format!("settings-setup-fix-{index}")),
                                ButtonKind::Ghost,
                            )
                            .size(ButtonSize::Sm)
                            .icon(Icon::Copy)
                            .label("Copy fix")
                            .tooltip(format!("Copy {command}"))
                            .on_click(move |_, cx| copy(&state, "Command", command.clone(), cx)),
                        );
                    }
                    row = row.child(chip_rounded(
                        check.state.clone(),
                        if check.ok {
                            StatusTone::Ok
                        } else {
                            StatusTone::Warn
                        },
                        CHIP_RADIUS,
                        cx,
                    ));
                    group = group.child(row);
                }
            }
        }
        let _ = theme;
        group.into_any_element()
    }

    pub(super) fn hooks_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let (setup, loaded) = self.setup(cx);
        let refresh_state = self.state.clone();
        let group = settings_group("Hooks")
            .description("Claude Code hooks shipped with the Polterware plugin.")
            .action(refresh_button(
                "settings-hooks-refresh",
                false,
                move |_, cx| {
                    refresh_state.update(cx, |state, cx| state.load_agent_setup(cx));
                },
            ));
        let hooks = setup
            .as_ref()
            .map(|setup| setup.hooks.clone())
            .unwrap_or_default();
        if hooks.is_empty() {
            let text = if !loaded {
                "Checking hooks\u{2026}"
            } else if setup.is_some() {
                "No hooks available."
            } else {
                "Hook state is unavailable."
            };
            return group.child(muted_line(text, cx)).into_any_element();
        }
        let theme = *Theme::global(cx);
        let mut rows = div()
            .flex()
            .flex_col()
            .gap(px(polterware_ui::theme::tw(1.0)));
        for hook in hooks {
            let switch = self.hook_switch(&hook.name, hook.enabled, cx);
            let label = div()
                .flex()
                .items_center()
                .gap(px(polterware_ui::theme::tw(3.0)))
                .child(
                    IconTile::new(Icon::Webhook)
                        .size(24.0, 14.0)
                        .tone(TileTone::Execution),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(theme.color_text_primary)
                                .child(hook.label),
                        )
                        .when(!hook.note.is_empty(), |label| {
                            label.child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(theme.color_text_muted)
                                    .child(hook.note),
                            )
                        }),
                );
            rows = rows.child(polterware_ui::switch::switch_row_content(
                SharedString::from(format!("settings-hook-{}", hook.name)),
                label,
                &switch,
                window,
                cx,
            ));
        }
        group.child(rows).into_any_element()
    }

    /// A hook's switch, made the first time the hook is listed and kept in step with it.
    fn hook_switch(&mut self, name: &str, enabled: bool, cx: &mut Context<Self>) -> Entity<Switch> {
        if let Some(switch) = self.hooks.get(name).cloned() {
            switch.update(cx, |switch, _| {
                if switch.is_on() != enabled {
                    switch.set_on(enabled);
                }
            });
            return switch;
        }
        let state = self.state.clone();
        let hook = name.to_string();
        let switch = cx.new(|cx| {
            Switch::new(enabled, cx).on_change(move |on, _, cx| {
                let hook = hook.clone();
                state.update(cx, |state, cx| state.set_hook(hook, on, cx));
            })
        });
        self.hooks.insert(name.to_string(), switch.clone());
        switch
    }

    pub(super) fn skill_page(&self) -> AnyElement {
        let (copy_state, save_state) = (self.state.clone(), self.state.clone());
        settings_group("AI skill")
            .description("Copy or download the Polterware instructions for coding agents.")
            .child(
                settings_row("Polterware AI skill")
                    .description(
                        "Short Markdown that points any agent at polterware status, brief, search, and format.",
                    )
                    .icon(Icon::BookOpen, TileTone::Knowledge)
                    .child(
                        Button::new("settings-skill-copy", ButtonKind::Primary)
                            .size(ButtonSize::Sm)
                            .icon(Icon::Copy)
                            .label("Copy")
                            .on_click(move |_, cx| {
                                copy(&copy_state, "Skill", SKILL_MARKDOWN.to_string(), cx)
                            }),
                    )
                    .child(
                        Button::new("settings-skill-download", ButtonKind::Ghost)
                            .size(ButtonSize::Sm)
                            .icon(Icon::Download)
                            .label("Download .md")
                            .on_click(move |_, cx| save_skill(&save_state, cx)),
                    ),
            )
            .into_any_element()
    }
}

/// The Electron download's save panel: cancellation writes nothing.
fn save_skill(state: &Entity<AppState>, cx: &mut App) {
    let directory = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default();
    let chosen = cx.prompt_for_new_path(&directory, Some(SKILL_FILE));
    let state = state.clone();
    cx.spawn(async move |cx| {
        let path = match chosen.await {
            Ok(Ok(Some(path))) => path,
            Ok(Ok(None)) | Err(_) => return,
            Ok(Err(error)) => {
                let _ = cx.update(|cx| {
                    state.update(cx, |state, cx| {
                        state.toast(format!("Could not save the skill: {error}"), true);
                        cx.notify();
                    })
                });
                return;
            }
        };
        let written = cx
            .background_spawn(async move { std::fs::write(&path, SKILL_MARKDOWN) })
            .await;
        let _ = cx.update(|cx| {
            state.update(cx, |state, cx| {
                match written {
                    Ok(()) => state.push_toast(format!("Saved {SKILL_FILE}."), ToastTone::Success),
                    Err(error) => state.toast(format!("Could not save the skill: {error}"), true),
                }
                cx.notify();
            });
        });
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_doctor_names_the_fix_for_what_is_missing() {
        let setup = AgentSetup {
            polterware_path: "/usr/local/bin/polterware".into(),
            claude_path: "/usr/local/bin/claude".into(),
            plugin_installed: true,
            plugin_version: "0.9.0".into(),
            embedded_version: "1.0.0".into(),
            ..AgentSetup::default()
        };
        let checks = setup_checks(&setup);
        assert_eq!(checks.len(), 3);
        assert!(checks[0].ok);
        assert_eq!(checks[1].state, "0.9.0 behind 1.0.0");
        assert_eq!(checks[1].hint.as_deref(), Some("polterware setup claude"));
        assert_eq!(checks[2].hint.as_deref(), Some("polterware setup codex"));
        assert!(is_command("polterware setup codex"));
        assert!(!is_command("Hooks skip themselves until it is on PATH."));
    }

    #[test]
    fn a_long_path_keeps_its_end() {
        assert_eq!(truncate_path("", 56), "\u{2014}");
        assert_eq!(truncate_path("/a/b", 56), "/a/b");
        let long = format!("/{}", "x".repeat(80));
        let short = truncate_path(&long, 56);
        assert_eq!(short.chars().count(), 56);
        assert!(short.starts_with('\u{2026}'));
    }
}
