//! The Projects screen (`projects-screen.tsx`), reached through Home: a 44px header with the
//! title, Search projects, the Status filter, and New project; then every local project in
//! one table, Name (the project's tile, its name, and its description), Status, Environments,
//! and Updated, each sorting, and the row's actions under the pointer. A row enters the
//! project on its Inbox.

use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FontWeight, IntoElement, Render,
    SharedString, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::data_table::{Column, DataTable, SortDirection, next_sort, sans, text_width};
use polterware_ui::ellipsis::fade;
use polterware_ui::filter::{FilterOption, FilterSpec};
use polterware_ui::filter_popover::{FilterField, FilterPopover, FiltersChanged};
use polterware_ui::icon_tile::IconTile;
use polterware_ui::icons::Icon;
use polterware_ui::menu::MenuItem;
use polterware_ui::patterns::{ContextMenu, MascotMood, page_empty, page_empty_with};
use polterware_ui::status_dot::status_dot_label;
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::{StatusTone, Theme};
use polterware_ui::window::window_drag::WindowDragRegion;

use crate::state::{AppState, Load, ProjectRow};
use crate::views::core_status::core_status;

const COLUMNS: [&str; 5] = ["Name", "Status", "Environments", "Updated", ""];
const SORTABLE: [bool; 5] = [true, true, true, true, false];
/// `STATUS_FILTERS`: each status, its label, and its dot's tone.
const STATUSES: [(&str, &str, StatusTone); 3] = [
    ("active", "Active", StatusTone::Ok),
    ("paused", "Paused", StatusTone::Warn),
    ("archived", "Archived", StatusTone::Muted),
];

fn status_of(project: &ProjectRow) -> (&'static str, &'static str, StatusTone) {
    STATUSES
        .iter()
        .copied()
        .find(|(id, ..)| *id == project.status)
        .unwrap_or(STATUSES[0])
}

pub struct ProjectsView {
    state: Entity<AppState>,
    search: Entity<TextInput>,
    filters: Entity<FilterPopover>,
    menu: Entity<ContextMenu>,
    query: String,
    sort: Option<(usize, SortDirection)>,
    fields_key: String,
}

impl ProjectsView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new(cx).placeholder("Search projects").pill());
        cx.observe(&search, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.query {
                this.query = text;
                cx.notify();
            }
        })
        .detach();
        let filters =
            cx.new(|cx| FilterPopover::new("project-filters", "Project filters", true, cx));
        cx.subscribe(&filters, |_, _, _: &FiltersChanged, cx| cx.notify())
            .detach();
        cx.observe(&filters, |_, _, cx| cx.notify()).detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        let menu = cx.new(ContextMenu::new);
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            search,
            filters,
            menu,
            query: String::new(),
            sort: None,
            fields_key: String::new(),
        }
    }

    pub fn filters(&self) -> &Entity<FilterPopover> {
        &self.filters
    }

    /// The Status field, its options counting the projects in each.
    fn sync_fields(&mut self, projects: &[ProjectRow], cx: &mut Context<Self>) {
        let counts: Vec<usize> = STATUSES
            .iter()
            .map(|(id, ..)| projects.iter().filter(|p| status_of(p).0 == *id).count())
            .collect();
        let key = format!("{counts:?}");
        if key == self.fields_key {
            return;
        }
        self.fields_key = key;
        let options = STATUSES
            .iter()
            .zip(&counts)
            .map(|((id, label, _), count)| {
                let mut option = FilterOption::new(*id, *label);
                option.trailing = Some(count.to_string());
                option
            })
            .collect();
        let mut spec = FilterSpec::multi("status", "Status", options);
        spec.plural_label = Some("statuses".into());
        self.filters.update(cx, |popover, _| {
            popover.set_fields(vec![FilterField::new(spec, None)]);
        });
    }

    fn visible(&self, projects: &[ProjectRow], cx: &App) -> Vec<ProjectRow> {
        let needle = self.query.trim().to_lowercase();
        let filters = self.filters.read(cx).filters();
        let mut rows: Vec<ProjectRow> = projects
            .iter()
            .filter(|project| needle.is_empty() || project.name.to_lowercase().contains(&needle))
            .filter(|project| {
                filters.matches(|field| (field == "status").then(|| vec![status_of(project).0]))
            })
            .cloned()
            .collect();
        if let Some((column, direction)) = self.sort {
            rows.sort_by(|a, b| {
                let order = match column {
                    0 => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    1 => status_of(a).1.cmp(status_of(b).1),
                    2 => a.environments.cmp(&b.environments),
                    3 => a.updated_at.cmp(&b.updated_at),
                    _ => Ordering::Equal,
                };
                match direction {
                    SortDirection::Ascending => order,
                    SortDirection::Descending => order.reverse(),
                }
            });
        }
        rows
    }

    fn open_menu(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        let items = project_actions(&self.state, id);
        let at = window.mouse_position();
        self.menu
            .update(cx, |menu, cx| menu.open(at, items, window, cx));
    }
}

/// `projectActions`: Open and Project settings, for a row's "…" and its right click.
fn project_actions(state: &Entity<AppState>, id: String) -> Vec<MenuItem> {
    let open_state = state.clone();
    let open_id = id.clone();
    let settings_state = state.clone();
    vec![
        MenuItem::action(
            "Open",
            Some(Icon::ExternalLink),
            Rc::new(move |_, cx: &mut App| {
                let id = open_id.clone();
                open_state.update(cx, |state, cx| {
                    state.open_project(id, cx);
                    cx.notify();
                });
            }),
        ),
        MenuItem::action(
            "Project settings",
            Some(Icon::Settings),
            Rc::new(move |_, cx: &mut App| {
                let id = id.clone();
                settings_state.update(cx, |state, cx| {
                    state.open_project_settings(id, cx);
                    cx.notify();
                });
            }),
        ),
    ]
}

impl Render for ProjectsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        if let Some(page) = core_status(&self.state, cx) {
            return div().size_full().child(page);
        }
        let projects = match self.state.read(cx).projects() {
            Load::Ready(projects) => Some(projects.clone()),
            _ => None,
        };
        if let Some(projects) = &projects {
            self.sync_fields(projects, cx);
        }

        let create_state = self.state.clone();
        let actions = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(div().w(px(200.0)).child(self.search.clone()))
            .child(self.filters.clone())
            .child(
                Button::new("projects-new", ButtonKind::Primary)
                    .size(ButtonSize::Icon)
                    .icon(Icon::Plus)
                    .tooltip("New project")
                    .on_click(move |_, cx| {
                        create_state.update(cx, |state, cx| {
                            state.set_new_project(true);
                            cx.notify();
                        });
                    }),
            );
        let header = div()
            .flex_none()
            .min_h(px(44.0))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .pl(px(24.0))
            .pr(px(12.0))
            .child(
                div()
                    .flex_none()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.color_text_primary)
                    .whitespace_nowrap()
                    .child("Projects"),
            )
            .child(div().flex_1().h(px(44.0)).window_drag_region())
            .child(actions);
        let bar = FilterPopover::bar(&self.filters, window, cx).map(|bar| {
            div()
                .flex_none()
                .pl(px(18.0))
                .pr(px(12.0))
                .pb(px(8.0))
                .child(bar)
        });

        let content: AnyElement = match &projects {
            None => match self.state.read(cx).projects() {
                Load::Failed(message) => page_empty_with(
                    Some("Could not read the projects"),
                    MascotMood::Sad,
                    Some(
                        div()
                            .text_size(px(theme.font_size_sm))
                            .text_color(theme.color_text_muted)
                            .child(SharedString::from(message.clone()))
                            .into_any_element(),
                    ),
                    cx,
                )
                .into_any_element(),
                _ => div().into_any_element(),
            },
            Some(projects) if projects.is_empty() => {
                page_empty("No projects yet", MascotMood::Sleeping, cx).into_any_element()
            }
            Some(projects) => {
                let visible = self.visible(projects, cx);
                if visible.is_empty() {
                    page_empty("No matches", MascotMood::Lost, cx).into_any_element()
                } else {
                    // `.pw-data-table-scroll`'s 12px inline padding sits inside the scroll,
                    // whose width the spare room is shared from.
                    div()
                        .size_full()
                        .child(
                            projects_table(Rc::new(visible), self, cx)
                                .gutters(12.0)
                                .measure_every_row()
                                .freeze_once(),
                        )
                        .into_any_element()
                }
            }
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(header)
            .children(bar)
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .flex_col()
                    .child(content),
            )
            .child(self.menu.clone())
    }
}

fn muted_cell(value: impl Into<SharedString>, theme: &Theme) -> AnyElement {
    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.color_text_muted)
        .whitespace_nowrap()
        .child(value.into())
        .into_any_element()
}

fn projects_table(
    rows: Rc<Vec<ProjectRow>>,
    view: &ProjectsView,
    cx: &mut Context<ProjectsView>,
) -> DataTable {
    let columns = COLUMNS
        .iter()
        .enumerate()
        .map(|(index, title)| {
            let mut column = Column::new(*title);
            if SORTABLE[index] {
                column = column.sortable();
            }
            if index == 0 {
                column.max_width = 960.0;
                column = column.shrinks();
            }
            if index == 4 {
                column = column.width(44.0).end();
                column.resizable = false;
            }
            column
        })
        .collect();
    let entity = cx.entity();
    let cells = rows.clone();
    let menu_entity = entity.clone();
    let open_state = view.state.clone();
    let opened = rows.clone();
    let sorted = entity.clone();
    let measured = rows.clone();
    DataTable::new("projects", columns, rows.len(), move |row, _window, cx| {
        let theme = *Theme::global(cx);
        let project = &cells[row.index];
        let (_, label, tone) = status_of(project);
        let name = div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(IconTile::new(Icon::FolderKanban).size(24.0, 14.0))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.color_text_primary)
                            .child(SharedString::from(project.name.clone())),
                    )
                    .when(!project.description.trim().is_empty(), |line| {
                        line.child(
                            div()
                                .ml(px(8.0))
                                .min_w(px(0.0))
                                .text_size(px(13.0))
                                .text_color(theme.color_text_muted)
                                .child(fade(SharedString::from(
                                    project.description.trim().to_string(),
                                ))),
                        )
                    }),
            )
            .into_any_element();
        let id = project.id.clone();
        let menu = menu_entity.clone();
        let more = div()
            .when(row.hovered, |slot| {
                slot.child(
                    Button::new(("project-more", row.index), ButtonKind::Ghost)
                        .size(ButtonSize::Icon)
                        .icon(Icon::MoreHorizontal)
                        .tooltip(format!("Actions for {}", project.name))
                        .on_click(move |window, cx| {
                            let id = id.clone();
                            menu.update(cx, |view, cx| view.open_menu(id, window, cx));
                        }),
                )
            })
            .into_any_element();
        vec![
            name,
            status_dot_label(tone, label, cx).into_any_element(),
            muted_cell(project.environments.clone(), &theme),
            muted_cell(project.updated.clone(), &theme),
            more,
        ]
    })
    .measure(move |range, window, cx| {
        let theme = *Theme::global(cx);
        let medium = sans(&theme, FontWeight::MEDIUM);
        let normal = sans(&theme, FontWeight::NORMAL);
        let mut widest = [0.0_f32; 5];
        for project in measured.get(range).unwrap_or_default() {
            let description = project.description.trim();
            let name = 24.0
                + 8.0
                + text_width(&project.name, medium.clone(), 13.0, window)
                + if description.is_empty() {
                    0.0
                } else {
                    8.0 + text_width(description, normal.clone(), 13.0, window)
                };
            let (_, label, _) = status_of(project);
            // The dot and its gap, `h-1.5` and `gap-1.5` on the 13px root, then `text-xs`.
            let status = 2.0 * 4.875 + text_width(label, normal.clone(), 9.75, window);
            let environments = text_width(&project.environments, medium.clone(), 12.0, window);
            let updated = text_width(&project.updated, medium.clone(), 12.0, window);
            for (slot, width) in widest
                .iter_mut()
                .zip([name, status, environments, updated, 0.0])
            {
                *slot = slot.max(width);
            }
        }
        widest.to_vec()
    })
    .row_height(48.0)
    .header(32.0, 12.0)
    .sort(view.sort, move |column, _, cx| {
        sorted.update(cx, |view, cx| {
            if SORTABLE.get(column).copied().unwrap_or(false) {
                view.sort = next_sort(view.sort, column);
                cx.notify();
            }
        });
    })
    .on_activate(move |index, _window, cx| {
        let id = opened[index].id.clone();
        open_state.update(cx, |state, cx| {
            state.open_project(id, cx);
            cx.notify();
        });
    })
    .on_context(move |index, at, window, cx| {
        let id = rows[index].id.clone();
        entity.update(cx, |view, cx| {
            let items = project_actions(&view.state, id);
            view.menu
                .update(cx, |menu, cx| menu.open(at, items, window, cx));
        });
    })
}
