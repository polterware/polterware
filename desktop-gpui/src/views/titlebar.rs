//! The title bar (`title-bar.tsx`): past the traffic lights, Go back and Go forward in one
//! texture pill and Home beside it, ending on the sidebar's right edge in a project (just
//! after the traffic lights on the Projects screen); then the breadcrumb, `/` between its
//! steps: the project, whose crumb opens Switch project (`ProjectSwitcherPopover`: a search,
//! each project on its folder tile with a check on the open one, and New project), and where
//! you are in it; drag space;
//! and Commander and App settings at the end. The empty bar drags the window.

use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AnyElement, App, Entity, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::icons::{Icon, icon_rotated};
use polterware_ui::list_popover::{ListPopover, ListRow};
use polterware_ui::patterns::AnchorSlot;
use polterware_ui::pointer::{track, use_pointer};
use polterware_ui::text_input::TextInput;
use polterware_ui::texture_group::{TextureButtonGroup, TextureGroupItem};
use polterware_ui::theme::{Theme, tw};
use polterware_ui::window::window_drag::WindowDragRegion;
use polterware_ui::{IconTile, TileTone};

use crate::state::{AppState, Load, Route, Section, SettingsScope, SettingsTab};

/// `--titlebar-traffic-inset`: the room the traffic lights keep at the bar's start.
const TRAFFIC_INSET: f32 = 78.0;
/// The breadcrumb's `margin-left: 8px`.
const CRUMBS_GAP: f32 = 8.0;
/// `.pw-shell-titlebar`'s gap between its slots.
const SLOT_GAP: f32 = 6.0;

/// The bar's own controls the app keeps between frames: Switch project and where it hangs.
pub struct TitlebarParts {
    pub switcher: Entity<ListPopover>,
    pub switcher_search: Entity<TextInput>,
    pub switcher_anchor: AnchorSlot,
}

/// `SELECTION_ACK_MS`: how long a picked project stays checked before the panel closes.
const SELECTION_HOLD: Duration = Duration::from_millis(300);

pub fn titlebar(
    state: &Entity<AppState>,
    parts: &TitlebarParts,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let theme = *Theme::global(cx);
    let app = state.read(cx);
    let in_project = matches!(app.route(), Route::Project { .. });
    let settings_open = app
        .settings_tab()
        .is_some_and(|tab| tab.scope() == SettingsScope::App);

    let back_state = state.clone();
    let forward_state = state.clone();
    let history = TextureButtonGroup::new("titlebar-history")
        .item(
            TextureGroupItem::new("Go back", Icon::ChevronLeft)
                .disabled(!app.can_go_back())
                .on_click(move |_, cx| {
                    back_state.update(cx, |state, cx| {
                        state.go_back(cx);
                        cx.notify();
                    });
                }),
        )
        .item(
            TextureGroupItem::new("Go forward", Icon::ChevronRight)
                .disabled(!app.can_go_forward())
                .on_click(move |_, cx| {
                    forward_state.update(cx, |state, cx| {
                        state.go_forward(cx);
                        cx.notify();
                    });
                }),
        );
    let home_state = state.clone();
    let home = Button::new("titlebar-home", ButtonKind::Secondary)
        .size(ButtonSize::Icon)
        .icon(Icon::Home01Icon)
        .tooltip("Projects")
        .on_click(move |_, cx| {
            home_state.update(cx, |state, cx| {
                state.go_home(cx);
                cx.notify();
            });
        });
    // `ml-2` and `gap-1` on the 13px root. In a project, the group ends on the sidebar's
    // right edge.
    let leading_width = theme.sidebar_width - TRAFFIC_INSET - tw(2.0);
    let leading = div()
        .flex_none()
        .ml(px(tw(2.0)))
        .when(in_project, |leading| leading.min_w(px(leading_width)))
        .flex()
        .items_center()
        .justify_end()
        .gap(px(tw(1.0)))
        .child(history)
        .child(home);

    let route = app.route().clone();
    let project_name: SharedString = app
        .project()
        .map(|project| project.name.clone())
        .unwrap_or_else(|| "Project".into())
        .into();
    let mut trail: Vec<AnyElement> = Vec::new();
    match route {
        Route::Projects => trail.push(current_crumb("Projects", &theme)),
        Route::Project {
            section: Section::Knowledge,
            tab,
            page,
            ..
        } => {
            // The tab names the list; an open record follows it, and the tab's crumb goes
            // back to the list.
            let record = page
                .as_ref()
                .and_then(|id| match state.read(cx).knowledge() {
                    Load::Ready(knowledge) => knowledge
                        .items
                        .iter()
                        .find(|item| &item.id == id)
                        .map(|item| item.title.clone()),
                    _ => None,
                });
            trail.push(project_crumb(state, project_name, parts, window, cx));
            trail.push(separator(&theme));
            match record {
                Some(title) => {
                    let back = state.clone();
                    trail.push(link_crumb(
                        "titlebar-list",
                        tab.label(),
                        move |cx| {
                            back.update(cx, |state, cx| {
                                state.set_knowledge_tab(tab, cx);
                                cx.notify();
                            })
                        },
                        &theme,
                    ));
                    trail.push(separator(&theme));
                    trail.push(current_crumb(title, &theme));
                }
                None => trail.push(current_crumb(tab.label(), &theme)),
            }
        }
        Route::Project {
            section: Section::Milestones,
            board,
            page,
            ..
        } => {
            // Milestones names the section; the trail names the board, then the open card,
            // whose board crumb closes it back to the board (`breadcrumbTrail`).
            let (name, title) = match (state.read(cx).work(), board.view.as_deref()) {
                (Load::Ready(work), Some(view)) => (
                    Some(work.board_name(view)),
                    page.as_deref()
                        .and_then(|slug| work.card(slug))
                        .map(|card| card.title.clone()),
                ),
                _ => (None, None),
            };
            trail.push(project_crumb(state, project_name, parts, window, cx));
            trail.push(separator(&theme));
            let mut steps: Vec<(SharedString, Option<Option<String>>)> =
                vec![(Section::Milestones.label().into(), Some(None))];
            if let Some(name) = name {
                steps.push((name.into(), title.is_some().then(|| board.view.clone())));
            }
            if let Some(title) = title {
                steps.push((title.into(), None));
            }
            let last = steps.len() - 1;
            for (index, (label, to)) in steps.into_iter().enumerate() {
                if index > 0 {
                    trail.push(separator(&theme));
                }
                match to.filter(|_| index < last) {
                    Some(view) => {
                        let back = state.clone();
                        trail.push(link_crumb(
                            if index == 0 {
                                "titlebar-milestones"
                            } else {
                                "titlebar-board"
                            },
                            label,
                            move |cx| {
                                back.update(cx, |state, cx| {
                                    state.open_board(view.clone(), cx);
                                    cx.notify();
                                })
                            },
                            &theme,
                        ));
                    }
                    None => trail.push(current_crumb(label, &theme)),
                }
            }
        }
        Route::Project {
            section: Section::Canvas,
            ..
        } if state.read(cx).open_canvas_name().is_some() => {
            // An open canvas follows the section, whose crumb goes back to the list.
            let name = state
                .read(cx)
                .open_canvas_name()
                .unwrap_or_default()
                .to_string();
            trail.push(project_crumb(state, project_name, parts, window, cx));
            trail.push(separator(&theme));
            let back = state.clone();
            trail.push(link_crumb(
                "titlebar-canvas",
                Section::Canvas.label(),
                move |cx| {
                    back.update(cx, |state, cx| {
                        state.go_section(Section::Canvas, cx);
                        cx.notify();
                    })
                },
                &theme,
            ));
            trail.push(separator(&theme));
            trail.push(current_crumb(name, &theme));
        }
        Route::Project { section, .. } => {
            trail.push(project_crumb(state, project_name, parts, window, cx));
            trail.push(separator(&theme));
            trail.push(current_crumb(section.label(), &theme));
        }
    }
    let crumbs = div()
        .flex_1()
        .min_w(px(0.0))
        .ml(px(CRUMBS_GAP))
        .flex()
        .items_center()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(2.0))
                .min_w(px(0.0))
                .children(trail),
        )
        .child(div().flex_1().h_full().window_drag_region());

    let commander_state = state.clone();
    let commander = Button::new("titlebar-commander", ButtonKind::Secondary)
        .size(ButtonSize::Icon)
        .icon(Icon::Search)
        .tooltip("Commander (\u{2318}K)")
        .on_click(move |_, cx| {
            commander_state.update(cx, |state, cx| {
                let open = state.commander_open();
                state.set_commander(!open);
                cx.notify();
            });
        });
    let settings_state = state.clone();
    let settings = Button::new("titlebar-settings", ButtonKind::Secondary)
        .size(ButtonSize::Icon)
        .icon(Icon::Settings)
        .tooltip("App settings")
        .selected(settings_open)
        .on_click(move |_, cx| {
            settings_state.update(cx, |state, cx| {
                state.open_settings(SettingsTab::AgentsMcp, cx);
                cx.notify();
            });
        });

    div()
        .id("titlebar")
        .h(px(theme.titlebar_height))
        .w_full()
        .flex_none()
        .flex()
        .items_center()
        .gap(px(SLOT_GAP))
        .pl(px(TRAFFIC_INSET))
        .pr(px(theme.space_2))
        .window_drag_region()
        .child(leading)
        .child(crumbs)
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(tw(1.0)))
                .child(commander)
                .child(settings),
        )
}

/// `.pw-titlebar-crumbs__current`: where you are, in 12.5px/500 primary.
fn current_crumb(label: impl Into<SharedString>, theme: &Theme) -> AnyElement {
    crumb_box(theme)
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.color_text_primary)
        .child(label.into())
        .into_any_element()
}

/// `.pw-titlebar-crumbs__link`: a step back, muted, primary on a hover fill.
fn link_crumb(
    id: &'static str,
    label: impl Into<SharedString>,
    on_click: impl Fn(&mut App) + 'static,
    theme: &Theme,
) -> AnyElement {
    let hover = theme.color_surface_hover;
    let lit = theme.color_text_primary;
    crumb_box(theme)
        .id(id)
        .window_drag_control()
        .text_color(theme.color_text_muted)
        .cursor_pointer()
        .hover(move |crumb| crumb.bg(hover).text_color(lit))
        .on_click(move |_, _, cx| on_click(cx))
        .child(label.into())
        .into_any_element()
}

/// `.pw-titlebar-crumbs__sep`: a `/` in the disabled ink, 2px each side.
fn separator(theme: &Theme) -> AnyElement {
    div()
        .flex_none()
        .px(px(2.0))
        .text_size(px(12.5))
        .text_color(theme.color_text_disabled)
        .child("/")
        .into_any_element()
}

fn crumb_box(theme: &Theme) -> gpui::Div {
    div()
        .flex_none()
        .max_w(px(280.0))
        .px(px(6.0))
        .py(px(3.0))
        .rounded(px(theme.radius_item))
        .text_size(px(12.5))
        .whitespace_nowrap()
        .overflow_hidden()
        .text_ellipsis()
}

/// The project's crumb (`.pw-titlebar-crumbs__project`): a muted link, primary on a hover
/// fill, whose 11px chevron shows under the pointer and while Switch project is open, turned
/// over then. It opens Switch project: the projects, the open one picked.
fn project_crumb(
    state: &Entity<AppState>,
    name: SharedString,
    parts: &TitlebarParts,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let theme = *Theme::global(cx);
    let pointer = use_pointer("titlebar-project", window, cx);
    let hovered = pointer.read(cx).hovered();
    let open = parts.switcher.read(cx).is_open();
    let lit = hovered || open;
    let ink = if lit {
        theme.color_text_primary
    } else {
        theme.color_text_muted
    };
    let switcher = parts.switcher.clone();
    let switch_state = state.clone();
    let crumb = crumb_box(&theme)
        .id("titlebar-project")
        .window_drag_control()
        .relative()
        .flex()
        .items_center()
        .text_color(ink)
        .when(lit, |crumb| crumb.bg(theme.color_surface_hover))
        .cursor_pointer()
        .child(div().min_w(px(0.0)).overflow_hidden().child(name))
        .when(lit, |crumb| {
            crumb.child(div().ml(px(4.0)).flex_none().child(icon_rotated(
                Icon::ChevronDown,
                11.0,
                ink,
                if open { std::f32::consts::PI } else { 0.0 },
            )))
        })
        .child(parts.switcher_anchor.probe())
        .on_click({
            let search = parts.switcher_search.clone();
            move |_, window, cx| open_switcher(&switcher, &search, &switch_state, window, cx)
        });
    track(crumb, &pointer, true).into_any_element()
}

/// Opens or closes Switch project, its search cleared.
pub fn open_switcher(
    switcher: &Entity<ListPopover>,
    search: &Entity<TextInput>,
    state: &Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) {
    search.update(cx, |input, cx| input.set_text("", cx));
    let rows = switcher_rows(state, "", cx);
    switcher.update(cx, |popover, cx| {
        let open = !popover.is_open();
        popover.set_width(300.0);
        popover.set_text_size(13.5);
        popover.set_trigger_height(24.5);
        popover.set_hold(Some(SELECTION_HOLD));
        popover.set_empty("No matching projects.");
        popover.set_search(Some(search.clone()));
        popover.set_rows(rows);
        popover.set_footer(vec![new_project_row(state)]);
        popover.set_open(open);
        cx.notify();
    });
    window.refresh();
}

/// The projects whose name holds `query`, archived ones left out, the open one checked.
pub fn switcher_rows(state: &Entity<AppState>, query: &str, cx: &App) -> Vec<ListRow> {
    let app = state.read(cx);
    let current = app.project().map(|project| project.id.clone());
    let Load::Ready(projects) = app.projects() else {
        return Vec::new();
    };
    let query = query.trim().to_lowercase();
    projects
        .iter()
        .filter(|project| project.status != "archived")
        .filter(|project| query.is_empty() || project.name.to_lowercase().contains(&query))
        .map(|project| {
            let id = project.id.clone();
            let open_state = state.clone();
            let is_current = current.as_deref() == Some(project.id.as_str());
            ListRow::new(
                project.name.clone(),
                Rc::new(move |_, cx: &mut App| {
                    if is_current {
                        return;
                    }
                    let id = id.clone();
                    open_state.update(cx, |state, cx| {
                        state.open_project(id, cx);
                        cx.notify();
                    });
                }),
            )
            .leading(|_| {
                IconTile::new(Icon::Folder)
                    .size(20.0, 12.0)
                    .tone(TileTone::Architecture)
                    .into_any_element()
            })
            .selected(is_current)
        })
        .collect()
}

/// The footer's New project.
fn new_project_row(state: &Entity<AppState>) -> ListRow {
    let state = state.clone();
    ListRow::new(
        "New project",
        Rc::new(move |_, cx: &mut App| {
            state.update(cx, |state, cx| {
                state.set_new_project(true);
                cx.notify();
            });
        }),
    )
    .icon(Icon::Plus)
}
