//! Platforms (`IntegrationsView` embedded in Project settings, `integration-inline-detail.tsx`,
//! `connect-provider-dialog.tsx`): every platform CLI Polterware knows, in one table, whether
//! its CLI is installed here, whether it is connected to this project, and when its metadata
//! was last refreshed; connected ones first, then the installed, then the rest, each in the
//! catalog's order. A search narrows it.
//!
//! A row opens the provider's sign-in: a dialog that probes the CLI's session
//! (`ProbeCliProvider`), and on Log in runs the provider's login command in a terminal on the
//! core (`TerminalService`, read every 50ms as the Electron emulator does), probing again every
//! two seconds for five minutes until a session shows; Done saves the project's connection. A
//! connected row's metadata opens its detail in place of the table: the connection, its
//! imported resources, and its sync runs, with Refresh metadata.

use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, FontWeight, IntoElement,
    SharedString, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::card::card;
use polterware_ui::chip::chip_rounded;
use polterware_ui::data_table::{Column, DataTable, primary_cell, sans, table_height};
use polterware_ui::filter::{FilterOption, FilterSpec};
use polterware_ui::filter_popover::{FilterField, FilterPopover};
use polterware_ui::icon_tile::{IconTile, TileTone};
use polterware_ui::icons::Icon;
use polterware_ui::patterns::{
    DialogSize, MascotMood, dialog, modal_layer, page_empty, settings_group_gap, window_layer,
};
use polterware_ui::service_mark::{service_mark, service_mark_tile};
use polterware_ui::skeleton::{SkeletonLayout, page_skeleton};
use polterware_ui::terminal::{TerminalEvent, TerminalPalette, TerminalView};
use polterware_ui::theme::{StatusTone, Theme, tw};
use polterware_ui::toast::ToastTone;

use super::SettingsView;
use super::project::CHIP_RADIUS;
use crate::client::Core;
use crate::contracts::{
    BeginCliAuthRequest, CliProvider, CloseTerminalSessionRequest, CreateTerminalSessionRequest,
    IntegrationConnection, ProbeCliProviderRequest, ReadTerminalOutputRequest,
    ResizeTerminalSessionRequest, WriteTerminalSessionRequest,
};
use crate::format::format_instant;
use crate::state::{AppState, CoreState, Load, Platforms};

/// `AUTH_PROBE_INTERVAL_MS`, `AUTH_PROBE_TIMEOUT_MS`, `AUTH_COMMAND_DELAY_MS`.
const PROBE_EVERY: Duration = Duration::from_secs(2);
const PROBE_FOR: Duration = Duration::from_secs(5 * 60);
const COMMAND_DELAY: Duration = Duration::from_millis(300);
/// The terminal emulator's read interval.
const READ_EVERY: Duration = Duration::from_millis(50);
/// `AUTH_DEFAULT_COLS`, `AUTH_DEFAULT_ROWS`.
const COLS: i32 = 100;
const ROWS: i32 = 28;
/// The login terminal's height (`h-[240px]`).
const TERMINAL_HEIGHT: f32 = 240.0;
/// The table's rows.
const ROW: f32 = 40.0;

/// Each sign-in dialog opened, so the tasks of a closed one stop.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// `CliConnectionState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CliState {
    Unavailable,
    Ready,
    Authenticating,
    Connected,
    Failed,
}

impl CliState {
    fn parse(status: &str) -> Self {
        match status {
            "cli-unavailable" => CliState::Unavailable,
            "connected" => CliState::Connected,
            "failed" => CliState::Failed,
            "authenticating" => CliState::Authenticating,
            _ => CliState::Ready,
        }
    }

    /// `cliStateLabel`.
    fn label(self) -> &'static str {
        match self {
            CliState::Unavailable => "CLI unavailable",
            CliState::Ready => "Ready",
            CliState::Authenticating => "Authenticating",
            CliState::Connected => "Connected",
            CliState::Failed => "Failed",
        }
    }

    /// `cliStateTone`.
    fn tone(self) -> StatusTone {
        match self {
            CliState::Unavailable => StatusTone::Unknown,
            CliState::Ready => StatusTone::Muted,
            CliState::Authenticating => StatusTone::Syncing,
            CliState::Connected => StatusTone::Healthy,
            CliState::Failed => StatusTone::Danger,
        }
    }
}

/// The sign-in dialog of one provider.
pub(super) struct Connect {
    provider: CliProvider,
    state: CliState,
    lines: Vec<String>,
    busy: bool,
    account: String,
    docs_url: String,
    session: Option<String>,
    terminal: Option<Entity<TerminalView>>,
    generation: u64,
}

/// A platform row's status (`PlatformRow["status"]`), in the table's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RowStatus {
    Failed,
    Connected,
    SigningIn,
    NotConnected,
}

impl RowStatus {
    /// The Status filter's value (`PlatformRow["status"]`).
    fn id(self) -> &'static str {
        match self {
            RowStatus::Failed => "failed",
            RowStatus::Connected => "connected",
            RowStatus::SigningIn => "signing-in",
            RowStatus::NotConnected => "not-connected",
        }
    }

    fn label(self) -> &'static str {
        match self {
            RowStatus::Failed => "Failed",
            RowStatus::Connected => "Connected",
            RowStatus::SigningIn => "Signing in",
            RowStatus::NotConnected => "Not connected",
        }
    }

    fn tone(self) -> StatusTone {
        match self {
            RowStatus::Failed => StatusTone::Danger,
            RowStatus::Connected => StatusTone::Healthy,
            RowStatus::SigningIn => StatusTone::Syncing,
            RowStatus::NotConnected => StatusTone::Muted,
        }
    }
}

/// A catalog platform with this project's connection to it, when there is one.
#[derive(Clone, Debug)]
struct PlatformRow {
    provider: CliProvider,
    connection: Option<IntegrationConnection>,
    status: RowStatus,
}

/// `platformStatus`: a connection in error failed; any other connection is connected.
fn rows(platforms: &Platforms, signing_in: Option<&str>) -> Vec<PlatformRow> {
    let mut rows: Vec<(usize, PlatformRow)> = platforms
        .providers
        .iter()
        .enumerate()
        .map(|(index, provider)| {
            let connection = platforms
                .connections
                .iter()
                .find(|connection| connection.provider_id == provider.id)
                .cloned();
            let status = if signing_in == Some(provider.id.as_str()) {
                RowStatus::SigningIn
            } else {
                match &connection {
                    Some(connection) if connection.status == "error" => RowStatus::Failed,
                    Some(_) => RowStatus::Connected,
                    None => RowStatus::NotConnected,
                }
            };
            (
                index,
                PlatformRow {
                    provider: provider.clone(),
                    connection,
                    status,
                },
            )
        })
        .collect();
    rows.sort_by_key(|(index, row)| (row.status, *index));
    rows.into_iter().map(|(_, row)| row).collect()
}

/// The environment a connection names, or All.
fn environment_name(platforms: &Platforms, id: &str) -> String {
    platforms
        .environments
        .iter()
        .find(|environment| environment.id == id)
        .map(|environment| environment.name.clone())
        .unwrap_or_else(|| "All".into())
}

/// `syncTone` for a sync run's result.
fn run_tone(result: &str) -> StatusTone {
    match result {
        "success" => StatusTone::Healthy,
        "partial" => StatusTone::Warn,
        "failed" => StatusTone::Danger,
        _ => StatusTone::Syncing,
    }
}

fn ready_core(state: &Entity<AppState>, cx: &App) -> Option<Core> {
    match state.read(cx).core() {
        CoreState::Ready(core) => Some(core.clone()),
        _ => None,
    }
}

impl SettingsView {
    fn platforms(&self, cx: &App) -> Option<Platforms> {
        match self.state.read(cx).platforms() {
            Load::Ready(platforms) => Some(platforms.clone()),
            _ => None,
        }
    }

    /// `valueField` for Category, CLI, and Status: each over the distinct values of the rows
    /// in their order. Every listed CLI reads as installed, as Electron's catalog maps them.
    fn sync_platform_fields(&mut self, rows: &[PlatformRow], cx: &mut Context<Self>) {
        fn distinct(values: impl Iterator<Item = (String, String)>) -> Vec<FilterOption> {
            let mut seen = std::collections::HashSet::new();
            values
                .filter(|(id, _)| !id.is_empty() && seen.insert(id.clone()))
                .map(|(id, label)| FilterOption::new(id, label))
                .collect()
        }
        let category = distinct(
            rows.iter()
                .map(|row| (row.provider.category.clone(), row.provider.category.clone())),
        );
        let status = distinct(
            rows.iter()
                .map(|row| (row.status.id().to_string(), row.status.label().to_string())),
        );
        let cli = distinct(
            rows.iter()
                .map(|_| ("installed".to_string(), "Installed".to_string())),
        );
        let key = format!("{category:?}{status:?}{cli:?}");
        if key == self.platform_fields_key {
            return;
        }
        self.platform_fields_key = key;
        let fields = vec![
            FilterField::new(FilterSpec::multi("category", "Category", category), None),
            FilterField::new(FilterSpec::multi("cli", "CLI", cli), None),
            FilterField::new(FilterSpec::multi("status", "Status", status), None),
        ];
        self.platform_filters
            .update(cx, |popover, _| popover.set_fields(fields));
    }

    pub(super) fn platforms_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        // Commander's Connect integration opens the sign-in on the first platform.
        if let Some(platforms) = self.platforms(cx)
            && let Some(provider) = platforms.providers.first().cloned()
            && self
                .state
                .update(cx, |state, _| state.take_connect_request())
        {
            self.open_connect(provider, window, cx);
        }
        let Some(platforms) = self.platforms(cx) else {
            return match self.state.read(cx).platforms() {
                Load::Failed(error) => super::project::muted_line(error.clone(), cx),
                _ => page_skeleton(SkeletonLayout::Table)
                    .without_toolbar()
                    .into_any_element(),
            };
        };
        if let Some(id) = self.platform_detail.clone()
            && let Some(connection) = platforms
                .connections
                .iter()
                .find(|connection| connection.id == id)
                .cloned()
        {
            return self.platform_detail_page(&platforms, &connection, cx);
        }
        let view = cx.entity().downgrade();
        let signing_in = self
            .connect
            .as_ref()
            .filter(|connect| connect.state == CliState::Authenticating)
            .map(|connect| connect.provider.id.clone());
        let needle = self.platform_search.read(cx).text().trim().to_lowercase();
        let visible: Vec<PlatformRow> = rows(&platforms, signing_in.as_deref())
            .into_iter()
            .filter(|row| {
                needle.is_empty()
                    || [
                        row.provider.name.clone(),
                        row.provider.description.clone(),
                        row.provider.category.clone(),
                        row.provider.binary.clone(),
                        row.connection
                            .as_ref()
                            .map(|connection| {
                                environment_name(&platforms, &connection.environment_id)
                            })
                            .unwrap_or_default(),
                    ]
                    .iter()
                    .any(|value| value.to_lowercase().contains(&needle))
            })
            .collect();
        self.sync_platform_fields(&visible, cx);
        let filters = self.platform_filters.read(cx).filters().clone();
        let visible: Vec<PlatformRow> = visible
            .into_iter()
            .filter(|row| {
                filters.matches(|field| match field {
                    "category" => Some(vec![row.provider.category.as_str()]),
                    "cli" => Some(vec!["installed"]),
                    "status" => Some(vec![row.status.id()]),
                    _ => None,
                })
            })
            .collect();
        let connect_view = view.clone();
        let first = platforms.providers.first().cloned();
        let tools = div()
            .flex()
            .items_center()
            .justify_end()
            .gap(px(tw(2.0)))
            .child(div().w(px(220.0)).child(self.platform_search.clone()))
            .child(self.platform_filters.clone())
            .child(
                Button::new("settings-connect-platform", ButtonKind::Secondary)
                    .size(ButtonSize::Icon)
                    .icon(Icon::Plug)
                    .tooltip("Connect platform")
                    .disabled(first.is_none())
                    .on_click(move |window, cx| {
                        let Some(provider) = first.clone() else {
                            return;
                        };
                        let _ = connect_view
                            .update(cx, |this, cx| this.open_connect(provider, window, cx));
                    }),
            );
        let body = if visible.is_empty() {
            div()
                .h(px(220.0))
                .child(page_empty("No matches", MascotMood::Lost, cx))
                .into_any_element()
        } else {
            let unit = tw(4.0);
            let columns = vec![
                Column::new("Platform").min_width(160.0),
                Column::new("CLI").width(7.5 * unit),
                Column::new("Status").width(8.0 * unit),
                Column::new("Last refresh").width(7.5 * unit),
                Column::new("").width(3.0 * unit).fixed(),
            ];
            let count = visible.len();
            let cells = Rc::new(visible);
            let activate_rows = cells.clone();
            let activate_view = view.clone();
            let detail_view = view.clone();
            let table = DataTable::new("settings-platforms", columns, count, move |row, _, cx| {
                let theme = *Theme::global(cx);
                let platform = &cells[row.index];
                let detail = platform.connection.as_ref().map(|connection| {
                    let id = connection.id.clone();
                    let view = detail_view.clone();
                    Button::new(
                        SharedString::from(format!(
                            "settings-platform-detail-{}",
                            platform.provider.id
                        )),
                        ButtonKind::Ghost,
                    )
                    .size(ButtonSize::Icon)
                    .icon(Icon::FileText)
                    .tooltip(format!("View {} metadata", platform.provider.name))
                    .on_click(move |_, cx| {
                        let id = id.clone();
                        let _ = view.update(cx, |this, cx| {
                            this.platform_detail = Some(id);
                            cx.notify();
                        });
                    })
                    .into_any_element()
                });
                vec![
                    primary_cell(
                        service_mark_tile(&platform.provider.id, 24.0, 14.0, cx),
                        platform.provider.name.clone(),
                        sans(&theme, FontWeight::MEDIUM),
                        None,
                        cx,
                    ),
                    chip_rounded("Installed", StatusTone::Healthy, CHIP_RADIUS, cx)
                        .into_any_element(),
                    chip_rounded(
                        platform.status.label(),
                        platform.status.tone(),
                        CHIP_RADIUS,
                        cx,
                    )
                    .into_any_element(),
                    div()
                        .text_size(px(12.5))
                        .text_color(theme.color_text_muted)
                        .child(SharedString::from(
                            platform
                                .connection
                                .as_ref()
                                .map(|connection| format_instant(&connection.last_sync_at))
                                .filter(|text| !text.is_empty())
                                .unwrap_or_else(|| "\u{2014}".into()),
                        ))
                        .into_any_element(),
                    div().children(detail).into_any_element(),
                ]
            })
            .gutters(0.0)
            .on_activate(move |index, window, cx| {
                let Some(row) = activate_rows.get(index) else {
                    return;
                };
                let provider = row.provider.clone();
                let _ =
                    activate_view.update(cx, |this, cx| this.open_connect(provider, window, cx));
            });
            div()
                .h(px(table_height(count, ROW)))
                .child(table)
                .into_any_element()
        };
        let _ = theme;
        div()
            .flex()
            .flex_col()
            .gap(px(tw(4.0)))
            .child(tools)
            .children(FilterPopover::bar(&self.platform_filters, window, cx))
            .child(body)
            .into_any_element()
    }

    /// One connection's detail (`IntegrationInlineDetail`).
    fn platform_detail_page(
        &mut self,
        platforms: &Platforms,
        connection: &IntegrationConnection,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let provider = platforms
            .providers
            .iter()
            .find(|provider| provider.id == connection.provider_id)
            .cloned();
        let name = provider
            .as_ref()
            .map(|provider| provider.name.clone())
            .unwrap_or_else(|| connection.provider_id.clone());
        let back_view = view.clone();
        let sync_view = view.clone();
        let flow_view = view.clone();
        let connection_id = connection.id.clone();
        let flow_provider = provider.clone();
        let head = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(tw(3.0)))
            .child(
                div()
                    .flex()
                    .min_w(px(0.0))
                    .items_center()
                    .gap(px(tw(2.0)))
                    .child(
                        Button::new("settings-platform-back", ButtonKind::Ghost)
                            .size(ButtonSize::Sm)
                            .label("Back")
                            .on_click(move |_, cx| {
                                let _ = back_view.update(cx, |this, cx| {
                                    this.platform_detail = None;
                                    cx.notify();
                                });
                            }),
                    )
                    .child(service_mark(&connection.provider_id, 16.0, cx))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.color_text_primary)
                            .child(SharedString::from(name.clone())),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(tw(2.0)))
                    .child(
                        Button::new("settings-platform-sync", ButtonKind::Secondary)
                            .size(ButtonSize::Sm)
                            .icon(Icon::RefreshCw)
                            .label("Refresh metadata")
                            .on_click(move |_, cx| {
                                let id = connection_id.clone();
                                let _ = sync_view.update(cx, |this, cx| {
                                    this.state
                                        .update(cx, |state, cx| state.sync_platform(id, cx));
                                });
                            }),
                    )
                    .child(
                        Button::new("settings-platform-flow", ButtonKind::Ghost)
                            .size(ButtonSize::Sm)
                            .icon(Icon::Plug)
                            .label("Open CLI flow")
                            .on_click(move |window, cx| {
                                let Some(provider) = flow_provider.clone() else {
                                    return;
                                };
                                let _ = flow_view
                                    .update(cx, |this, cx| this.open_connect(provider, window, cx));
                            }),
                    ),
            );
        let section = |title: &str, body: AnyElement| {
            div()
                .flex()
                .flex_col()
                .gap(px(tw(2.0)))
                .child(
                    div()
                        .px(px(tw(1.0)))
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.color_text_muted)
                        .child(SharedString::from(title.to_string())),
                )
                .child(
                    card(cx)
                        .p(px(tw(4.0)))
                        .flex()
                        .flex_col()
                        .gap(px(tw(3.0)))
                        .child(body),
                )
        };
        let mut facts = vec![
            ("Status", connection.status.clone()),
            ("Connection", connection.label.clone()),
            (
                "Environment",
                environment_name(platforms, &connection.environment_id),
            ),
            ("Last sync", format_instant(&connection.last_sync_at)),
        ];
        if provider.is_some() {
            facts.push(("Provider", name.clone()));
        }
        let terms = div()
            .flex()
            .flex_col()
            .gap(px(tw(1.5)))
            .children(facts.iter().map(|(term, value)| {
                div()
                    .flex()
                    .gap(px(tw(6.0)))
                    .child(
                        div()
                            .w(px(90.0))
                            .flex_none()
                            .text_color(theme.color_text_muted)
                            .child(SharedString::from(term.to_string())),
                    )
                    .child(div().child(SharedString::from(value.clone())))
            }));
        let mut connection_body = div().flex().flex_col().gap(px(tw(3.0))).child(terms);
        if !connection.summary.is_empty() {
            connection_body = connection_body.child(
                div()
                    .text_color(theme.color_text_secondary)
                    .child(SharedString::from(connection.summary.clone())),
            );
        }
        let resources: Vec<_> = platforms
            .resources
            .iter()
            .filter(|resource| resource.connection_id == connection.id)
            .cloned()
            .collect();
        let resources_body = if resources.is_empty() {
            div()
                .text_color(theme.color_text_muted)
                .child("No imported resources yet. Refresh metadata imports a bounded list from the local CLI; link them to components in the Structure.")
                .into_any_element()
        } else {
            let count = resources.len();
            let names: Vec<(String, String, String, String)> = resources
                .iter()
                .map(|resource| {
                    (
                        resource.name.clone(),
                        resource.kind.clone(),
                        environment_name(platforms, &resource.environment_id),
                        resource.status.clone(),
                    )
                })
                .collect();
            let names = Rc::new(names);
            let table = DataTable::new(
                SharedString::from(format!("settings-resources-{}", connection.id)),
                vec![
                    Column::new("Resource").min_width(140.0),
                    Column::new("Kind"),
                    Column::new("Environment"),
                    Column::new("Status"),
                ],
                count,
                move |row, _, cx| {
                    let theme = *Theme::global(cx);
                    let (name, kind, environment, status) = &names[row.index];
                    let text = |value: &String| {
                        div()
                            .text_size(px(12.5))
                            .child(SharedString::from(value.clone()))
                            .into_any_element()
                    };
                    vec![
                        primary_cell(
                            IconTile::new(Icon::Plug)
                                .size(24.0, 14.0)
                                .tone(TileTone::Integration),
                            name.clone(),
                            sans(&theme, FontWeight::MEDIUM),
                            None,
                            cx,
                        ),
                        text(kind),
                        text(environment),
                        chip_rounded(status.clone(), StatusTone::Healthy, CHIP_RADIUS, cx)
                            .into_any_element(),
                    ]
                },
            );
            div()
                .h(px(table_height(count, ROW)))
                .child(table)
                .into_any_element()
        };
        let runs: Vec<_> = platforms
            .sync_runs
            .iter()
            .filter(|run| run.connection_id == connection.id)
            .cloned()
            .collect();
        let runs_body = if runs.is_empty() {
            div()
                .text_color(theme.color_text_muted)
                .child("No sync runs yet. Refresh metadata pulls inventory through the authenticated local CLI.")
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(tw(3.0)))
                .children(runs.iter().map(|run| {
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_between()
                        .gap(px(tw(3.0)))
                        .child(
                            div()
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .child(SharedString::from(run.message.clone())),
                                )
                                .child(
                                    div()
                                        .mt(px(tw(1.0)))
                                        .text_size(px(11.0))
                                        .text_color(theme.color_text_muted)
                                        .child(SharedString::from(format!(
                                            "{} / {} items",
                                            format_instant(&run.started_at),
                                            run.imported
                                        ))),
                                ),
                        )
                        .child(chip_rounded(
                            run.result.clone(),
                            run_tone(&run.result),
                            CHIP_RADIUS,
                            cx,
                        ))
                }))
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .gap(px(settings_group_gap()))
            .text_size(px(12.5))
            .child(head)
            .child(section("Connection", connection_body.into_any_element()))
            .child(section("Imported resources", resources_body))
            .child(section("Sync history", runs_body))
            .into_any_element()
    }

    /// Opens `provider`'s sign-in and probes its CLI.
    fn open_connect(&mut self, provider: CliProvider, window: &mut Window, cx: &mut Context<Self>) {
        self.close_connect(cx);
        self.connect_leaving = None;
        let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
        self.connect = Some(Connect {
            provider,
            state: CliState::Ready,
            lines: Vec::new(),
            busy: false,
            account: String::new(),
            docs_url: String::new(),
            session: None,
            terminal: None,
            generation,
        });
        let _ = window;
        self.probe(cx);
        cx.notify();
    }

    fn current(&self, generation: u64) -> bool {
        self.connect
            .as_ref()
            .is_some_and(|connect| connect.generation == generation)
    }

    /// Closes the sign-in and its terminal session.
    fn close_connect(&mut self, cx: &mut Context<Self>) {
        if let Some(mut connect) = self.connect.take() {
            let session = connect.session.take();
            self.close_session(session, cx);
            // Drawn as it was while the dialog leaves.
            self.connect_leaving = Some(connect);
        }
    }

    fn close_session(&self, session: Option<String>, cx: &mut Context<Self>) {
        let (Some(id), Some(core)) = (session, ready_core(&self.state, cx)) else {
            return;
        };
        cx.background_spawn(async move {
            let _ = core
                .terminal()
                .close_terminal_session(&CloseTerminalSessionRequest { id });
        })
        .detach();
    }

    /// `probeCurrent`: the CLI's session, and the install docs when it is not on PATH.
    fn probe(&mut self, cx: &mut Context<Self>) {
        let (Some(connect), Some(core)) = (self.connect.as_mut(), ready_core(&self.state, cx))
        else {
            return;
        };
        connect.busy = true;
        let generation = connect.generation;
        let provider_id = connect.provider.id.clone();
        let wants_docs = connect.docs_url.is_empty();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let probe =
                        core.integration()
                            .probe_cli_provider(&ProbeCliProviderRequest {
                                provider_id: provider_id.clone(),
                            })?;
                    let docs = if wants_docs && probe.status == "cli-unavailable" {
                        core.integration()
                            .begin_cli_auth(&BeginCliAuthRequest { provider_id })
                            .map(|plan| plan.docs_url)
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    Ok::<_, crate::client::CoreError>((probe, docs))
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if !this.current(generation) {
                    return;
                }
                let Some(connect) = this.connect.as_mut() else {
                    return;
                };
                connect.busy = false;
                match result {
                    Ok((probe, docs)) => {
                        connect.account = probe.account_label;
                        connect.state = CliState::parse(&probe.status);
                        connect.lines.clear();
                        if !docs.is_empty() {
                            connect.docs_url = docs;
                        }
                    }
                    Err(error) => {
                        connect.state = CliState::Failed;
                        connect.lines = vec![
                            format!("Could not probe {}.", connect.provider.name),
                            error.to_string(),
                        ];
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// `startAuthentication`: runs the login command in a terminal on the core, then probes
    /// until a session shows.
    fn start_login(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let project_id = self.state.read(cx).open_project_id().unwrap_or_default();
        let Some(core) = ready_core(&self.state, cx) else {
            return;
        };
        if project_id.trim().is_empty() {
            self.state.update(cx, |state, _| {
                state.toast("Open a local project before starting CLI login.", true)
            });
            return;
        }
        let Some(connect) = self.connect.as_mut() else {
            return;
        };
        let old = connect.session.take();
        connect.terminal = None;
        connect.busy = true;
        connect.state = CliState::Authenticating;
        let generation = connect.generation;
        let provider = connect.provider.clone();
        self.close_session(old, cx);
        let palette = {
            let theme = Theme::global(cx);
            TerminalPalette {
                background: gpui::rgb(0x111111),
                foreground: gpui::rgb(0xe8e8e8),
                cursor: gpui::rgb(0xe8e8e8),
                cursor_accent: gpui::rgb(0x111111),
                ..TerminalPalette::from_theme(theme)
            }
        };
        cx.spawn_in(window, async move |this, cx| {
            let plan_core = core.clone();
            let plan_provider = provider.clone();
            let started = cx
                .background_spawn(async move {
                    let plan = plan_core
                        .integration()
                        .begin_cli_auth(&BeginCliAuthRequest {
                            provider_id: plan_provider.id.clone(),
                        })?;
                    let session = plan_core
                        .terminal()
                        .create_terminal_session(&CreateTerminalSessionRequest {
                            project_id,
                            target_id: format!("auth-{}", plan_provider.id),
                            cols: COLS,
                            rows: ROWS,
                            ..CreateTerminalSessionRequest::default()
                        })?
                        .session
                        .map(|session| session.id)
                        .filter(|id| !id.is_empty())
                        .ok_or_else(|| {
                            crate::client::CoreError::Rpc("Terminal session id missing.".into())
                        })?;
                    Ok::<_, crate::client::CoreError>((plan, session))
                })
                .await;
            let session = this
                .update_in(cx, |this, window, cx| {
                    let (plan, session) = match started {
                        Ok(started) => started,
                        Err(error) => {
                            if let Some(connect) = this.connect.as_mut()
                                && connect.generation == generation
                            {
                                connect.busy = false;
                                connect.state = CliState::Failed;
                                connect.lines = vec![
                                    format!(
                                        "Could not start authentication for {}.",
                                        provider.name
                                    ),
                                    error.to_string(),
                                ];
                            }
                            let message = error.to_string();
                            this.state.update(cx, |state, _| {
                                state.toast(
                                    if message.is_empty() {
                                        "Could not start CLI login.".to_string()
                                    } else {
                                        message
                                    },
                                    true,
                                )
                            });
                            cx.notify();
                            return None;
                        }
                    };
                    if !this.current(generation) {
                        this.close_session(Some(session), cx);
                        return None;
                    }
                    let terminal = cx.new(|cx| {
                        TerminalView::new(window, cx)
                            .font_size(12.0)
                            .palette(palette)
                    });
                    this.watch_terminal(&terminal, session.clone(), cx);
                    terminal.read(cx).focus(window);
                    let command = if plan.auth_command.is_empty() {
                        provider
                            .auth_commands
                            .first()
                            .cloned()
                            .unwrap_or_else(|| provider.binary.clone())
                    } else {
                        plan.auth_command.clone()
                    };
                    if let Some(connect) = this.connect.as_mut() {
                        connect.docs_url = plan.docs_url;
                        connect.lines.clear();
                        connect.session = Some(session.clone());
                        connect.terminal = Some(terminal);
                        connect.busy = false;
                    }
                    this.state.update(cx, |state, _| {
                        state.push_toast(
                            format!("Complete {} login in the terminal below.", provider.name),
                            ToastTone::Info,
                        )
                    });
                    cx.notify();
                    Some((session, command))
                })
                .ok()
                .flatten();
            let Some((session, command)) = session else {
                return;
            };
            this.update(cx, |this, cx| {
                this.read_output(generation, session.clone(), cx)
            })
            .ok();
            cx.background_executor().timer(COMMAND_DELAY).await;
            let command = command.trim().to_string();
            if !command.is_empty() {
                let write_core = core.clone();
                let id = session.clone();
                cx.background_spawn(async move {
                    let _ = write_core.terminal().write_terminal_session(
                        &WriteTerminalSessionRequest {
                            id,
                            data: format!("{command}\n").into_bytes(),
                        },
                    );
                })
                .await;
            }
            this.update(cx, |this, cx| this.probe_until_signed_in(generation, cx))
                .ok();
        })
        .detach();
    }

    /// Sends the terminal's input and size to the session.
    fn watch_terminal(
        &self,
        terminal: &Entity<TerminalView>,
        session: String,
        cx: &mut Context<Self>,
    ) {
        let state = self.state.clone();
        cx.subscribe(terminal, move |_, _, event: &TerminalEvent, cx| {
            let Some(core) = ready_core(&state, cx) else {
                return;
            };
            let id = session.clone();
            match event.clone() {
                TerminalEvent::Input(data) => {
                    cx.background_spawn(async move {
                        let _ = core
                            .terminal()
                            .write_terminal_session(&WriteTerminalSessionRequest { id, data });
                    })
                    .detach();
                }
                TerminalEvent::Resize(size) => {
                    if size.cols < 2 || size.rows < 2 {
                        return;
                    }
                    cx.background_spawn(async move {
                        let _ = core.terminal().resize_terminal_session(
                            &ResizeTerminalSessionRequest {
                                id,
                                cols: size.cols as i32,
                                rows: size.rows as i32,
                            },
                        );
                    })
                    .detach();
                }
                TerminalEvent::Title(_) => {}
            }
        })
        .detach();
    }

    /// Reads the session's output every 50ms into the terminal until it closes.
    fn read_output(&self, generation: u64, session: String, cx: &mut Context<Self>) {
        let Some(core) = ready_core(&self.state, cx) else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let mut offset = 0i64;
            loop {
                let read_core = core.clone();
                let id = session.clone();
                let chunk = cx
                    .background_spawn(async move {
                        read_core
                            .terminal()
                            .read_terminal_output(&ReadTerminalOutputRequest { id, offset })
                    })
                    .await;
                let go_on = this
                    .update(cx, |this, cx| {
                        let Some(terminal) = this
                            .connect
                            .as_ref()
                            .filter(|connect| {
                                connect.generation == generation
                                    && connect.session.as_deref() == Some(session.as_str())
                            })
                            .and_then(|connect| connect.terminal.clone())
                        else {
                            return false;
                        };
                        match chunk {
                            Ok(chunk) => {
                                if !chunk.data.is_empty() {
                                    offset = chunk.next_offset;
                                    terminal
                                        .update(cx, |terminal, cx| terminal.feed(&chunk.data, cx));
                                }
                                if chunk.closed {
                                    terminal.update(cx, |terminal, cx| {
                                        terminal.feed(b"\r\n[session closed]\r\n", cx)
                                    });
                                    return false;
                                }
                                true
                            }
                            Err(error) => {
                                let gone = error.to_string().to_lowercase().contains("not found");
                                if gone {
                                    terminal.update(cx, |terminal, cx| {
                                        terminal.feed(b"\r\n[session closed]\r\n", cx)
                                    });
                                }
                                !gone
                            }
                        }
                    })
                    .unwrap_or(false);
                if !go_on {
                    return;
                }
                cx.background_executor().timer(READ_EVERY).await;
            }
        })
        .detach();
    }

    /// `startAuthProbeLoop`: every two seconds, until a session, no CLI, or five minutes.
    fn probe_until_signed_in(&self, generation: u64, cx: &mut Context<Self>) {
        let Some(core) = ready_core(&self.state, cx) else {
            return;
        };
        let Some(provider) = self
            .connect
            .as_ref()
            .map(|connect| connect.provider.clone())
        else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let started = Instant::now();
            loop {
                if started.elapsed() >= PROBE_FOR {
                    let _ = this.update(cx, |this, cx| {
                        if !this.current(generation) {
                            return;
                        }
                        if let Some(connect) = this.connect.as_mut() {
                            connect.state = CliState::Failed;
                            connect.busy = false;
                            let session = connect.session.take();
                            connect.terminal = None;
                            this.close_session(session, cx);
                        }
                        this.state.update(cx, |state, _| {
                            state.toast(
                                format!(
                                    "{} login timed out. Try again or refresh status.",
                                    provider.name
                                ),
                                true,
                            )
                        });
                        cx.notify();
                    });
                    return;
                }
                let probe_core = core.clone();
                let provider_id = provider.id.clone();
                let probe = cx
                    .background_spawn(async move {
                        probe_core
                            .integration()
                            .probe_cli_provider(&ProbeCliProviderRequest { provider_id })
                    })
                    .await;
                let done = this
                    .update(cx, |this, cx| {
                        if !this.current(generation) {
                            return true;
                        }
                        let Ok(probe) = probe else {
                            return false;
                        };
                        let Some(connect) = this.connect.as_mut() else {
                            return true;
                        };
                        if !probe.account_label.is_empty() {
                            connect.account = probe.account_label.clone();
                        }
                        match CliState::parse(&probe.status) {
                            CliState::Connected => {
                                connect.state = CliState::Connected;
                                connect.busy = false;
                                if connect.account.is_empty() {
                                    connect.account = provider.name.clone();
                                }
                                this.state.update(cx, |state, _| {
                                    state.push_toast(
                                        format!("{} CLI session detected.", provider.name),
                                        ToastTone::Success,
                                    )
                                });
                                cx.notify();
                                true
                            }
                            CliState::Unavailable => {
                                connect.state = CliState::Unavailable;
                                connect.busy = false;
                                let message = if probe.message.is_empty() {
                                    format!("{} is not available on PATH.", provider.binary)
                                } else {
                                    probe.message
                                };
                                this.state.update(cx, |state, _| state.toast(message, true));
                                cx.notify();
                                true
                            }
                            _ => false,
                        }
                    })
                    .unwrap_or(true);
                if done {
                    return;
                }
                cx.background_executor().timer(PROBE_EVERY).await;
            }
        })
        .detach();
    }

    /// `refreshAndConnect` (Done): a session shows, so the project's connection is saved.
    fn finish_connect(&mut self, cx: &mut Context<Self>) {
        let (Some(connect), Some(core)) = (self.connect.as_mut(), ready_core(&self.state, cx))
        else {
            return;
        };
        connect.busy = true;
        let generation = connect.generation;
        let provider = connect.provider.clone();
        cx.spawn(async move |this, cx| {
            let provider_id = provider.id.clone();
            let probe = cx
                .background_spawn(async move {
                    core.integration()
                        .probe_cli_provider(&ProbeCliProviderRequest { provider_id })
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if !this.current(generation) {
                    return;
                }
                let Some(connect) = this.connect.as_mut() else {
                    return;
                };
                connect.busy = false;
                match probe {
                    Ok(probe) => {
                        connect.account = probe.account_label.clone();
                        match CliState::parse(&probe.status) {
                            CliState::Unavailable => {
                                connect.state = CliState::Unavailable;
                                let message = format!("{} is not available on PATH.", provider.binary);
                                this.state.update(cx, |state, _| state.toast(message, true));
                            }
                            CliState::Connected => {
                                let label = if probe.account_label.is_empty() {
                                    provider.name.clone()
                                } else {
                                    probe.account_label
                                };
                                this.close_connect(cx);
                                this.state.update(cx, |state, cx| {
                                    state.push_toast(
                                        format!("{} connection ready.", provider.name),
                                        ToastTone::Success,
                                    );
                                    state.connect_platform(provider.clone(), label, cx);
                                });
                            }
                            _ => {
                                if connect.state != CliState::Authenticating {
                                    connect.state = CliState::Ready;
                                }
                                this.state.update(cx, |state, _| {
                                    state.toast(
                                        "CLI is available, but no authenticated session was detected yet.",
                                        false,
                                    )
                                });
                            }
                        }
                    }
                    Err(error) => {
                        connect.state = CliState::Failed;
                        let message = error.to_string();
                        this.state.update(cx, |state, _| {
                            state.toast(
                                if message.is_empty() {
                                    "Could not refresh connection status.".to_string()
                                } else {
                                    message
                                },
                                true,
                            )
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The sign-in dialog over the settings, while one is open.
    pub(super) fn connect_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        self.connect_presence.set_open(self.connect.is_some());
        if !self.connect_presence.is_present() {
            self.connect_leaving = None;
            return None;
        }
        let pose = self.connect_presence.frame(window);
        let open = self.connect.is_some();
        let connect = self.connect.as_ref().or(self.connect_leaving.as_ref())?;
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let provider = connect.provider.clone();
        let state = connect.state;
        let busy = connect.busy;
        let has_project = self.state.read(cx).open_project_id().is_some();
        let close: polterware_ui::Handler = {
            let view = view.clone();
            Rc::new(move |_, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    this.close_connect(cx);
                    cx.notify();
                });
            })
        };
        let refresh_view = view.clone();
        let refresh = Button::new("settings-connect-refresh", ButtonKind::Ghost)
            .size(ButtonSize::Sm)
            .icon(Icon::RefreshCw)
            .label("Refresh")
            .tooltip("Refresh status")
            .disabled(busy)
            .on_click(move |_, cx| {
                let _ = refresh_view.update(cx, |this, cx| this.probe(cx));
            })
            .into_any_element();
        let primary_view = view.clone();
        let primary = match state {
            CliState::Connected => Button::new("settings-connect-done", ButtonKind::Inverse)
                .size(ButtonSize::Sm)
                .label("Done")
                .disabled(busy)
                .on_click(move |_, cx| {
                    let _ = primary_view.update(cx, |this, cx| this.finish_connect(cx));
                }),
            CliState::Unavailable => {
                let docs = connect.docs_url.clone();
                Button::new("settings-connect-install", ButtonKind::Inverse)
                    .size(ButtonSize::Sm)
                    .label(format!("Install {}", provider.binary))
                    .disabled(busy)
                    .on_click(move |_, cx| {
                        if !docs.is_empty() {
                            cx.open_url(&docs);
                        }
                    })
            }
            _ => Button::new("settings-connect-login", ButtonKind::Inverse)
                .size(ButtonSize::Sm)
                .label(if connect.session.is_some() {
                    "Restart login"
                } else {
                    "Log in"
                })
                .disabled(busy || !has_project)
                .on_click(move |window, cx| {
                    let _ = primary_view.update(cx, |this, cx| this.start_login(window, cx));
                }),
        }
        .into_any_element();
        let command = provider
            .auth_commands
            .first()
            .cloned()
            .unwrap_or_else(|| provider.binary.clone());
        let copy_state = self.state.clone();
        let copy_command = command.clone();
        let mut body = div().flex().flex_col().gap(px(tw(3.0))).child(
            div()
                .flex()
                .items_center()
                .gap(px(tw(2.0)))
                .child(chip_rounded(state.label(), state.tone(), CHIP_RADIUS, cx))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .font_family(theme.font_mono)
                        .text_size(px(12.0))
                        .text_color(theme.color_text_muted)
                        .child(SharedString::from(if connect.account.is_empty() {
                            command.clone()
                        } else {
                            connect.account.clone()
                        })),
                )
                .child(
                    Button::new("settings-connect-copy", ButtonKind::Ghost)
                        .size(ButtonSize::Icon)
                        .icon(Icon::Copy)
                        .tooltip("Copy login command")
                        .on_click(move |_, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(copy_command.clone()));
                            copy_state.update(cx, |state, cx| {
                                state.push_toast("Login command copied.", ToastTone::Success);
                                cx.notify();
                            });
                        }),
                ),
        );
        let prose = |text: String| {
            div()
                .text_size(px(13.0))
                .text_color(theme.color_text_secondary)
                .child(SharedString::from(text))
        };
        if state == CliState::Unavailable {
            body = body.child(prose(format!(
                "{} was not found on your PATH. Install it, then refresh.",
                provider.binary
            )));
        }
        if state == CliState::Failed && !connect.lines.is_empty() {
            body = body.child(prose(connect.lines.join(" ")));
        }
        if let Some(terminal) = connect.terminal.clone() {
            body = body.child(
                div()
                    .h(px(TERMINAL_HEIGHT))
                    .overflow_hidden()
                    .rounded(px(theme.radius_card))
                    .bg(gpui::rgb(0x111111))
                    .child(terminal),
            );
        }
        let dialog = dialog(format!("Connect {}", provider.name), 1.0)
            .size(DialogSize::Lg)
            .title_leading(service_mark_tile(&provider.id, 28.0, 16.0, cx))
            .on_close(close.clone())
            .pose(pose)
            .footer([refresh, primary])
            .child(body);
        Some(window_layer(modal_layer(
            pose.opacity,
            open,
            close,
            dialog,
            &theme,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(id: &str) -> CliProvider {
        CliProvider {
            id: id.into(),
            name: id.to_uppercase(),
            ..CliProvider::default()
        }
    }

    #[test]
    fn connected_platforms_lead_the_table_in_catalog_order() {
        let platforms = Platforms {
            providers: vec![
                provider("aws"),
                provider("vercel"),
                provider("github"),
                provider("fly"),
            ],
            connections: vec![
                IntegrationConnection {
                    id: "int-1".into(),
                    provider_id: "github".into(),
                    status: "connected".into(),
                    ..IntegrationConnection::default()
                },
                IntegrationConnection {
                    id: "int-2".into(),
                    provider_id: "fly".into(),
                    status: "error".into(),
                    ..IntegrationConnection::default()
                },
            ],
            ..Platforms::default()
        };
        let order: Vec<(String, RowStatus)> = rows(&platforms, Some("vercel"))
            .into_iter()
            .map(|row| (row.provider.id, row.status))
            .collect();
        assert_eq!(
            order,
            [
                ("fly".to_string(), RowStatus::Failed),
                ("github".to_string(), RowStatus::Connected),
                ("vercel".to_string(), RowStatus::SigningIn),
                ("aws".to_string(), RowStatus::NotConnected),
            ]
        );
    }

    #[test]
    fn a_probe_status_maps_to_its_state() {
        assert_eq!(
            CliState::parse("cli-unavailable").label(),
            "CLI unavailable"
        );
        assert_eq!(CliState::parse("whatever"), CliState::Ready);
        assert_eq!(CliState::Authenticating.tone(), StatusTone::Syncing);
    }
}
