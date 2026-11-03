//! The root view: the title bar over the sidebar and the routed page on the main surface, with
//! Commander and toasts over them. The Projects screen and App settings have no sidebar.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, IntoElement, KeyBinding, Render, Window,
    actions, div, prelude::*, px,
};
use polterware_ui::patterns::{main_surface, modal_layer};
use polterware_ui::theme::Theme;
use polterware_ui::toast::toast_stack;

use crate::state::{AppState, Route, Section};
use crate::views::commander::CommanderView;
use crate::views::{inbox, placeholder, projects, sidebar, titlebar};

actions!(
    polterware_gpui,
    [
        Quit,
        CloseWindow,
        ToggleCommander,
        OpenAppSettings,
        CloseOverlay
    ]
);

pub fn register_key_bindings(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-w", CloseWindow, None),
        KeyBinding::new("cmd-k", ToggleCommander, None),
        KeyBinding::new("cmd-,", OpenAppSettings, None),
        KeyBinding::new("escape", CloseOverlay, None),
    ]);
}

pub struct AppView {
    focus_handle: FocusHandle,
    state: Entity<AppState>,
    commander: Entity<CommanderView>,
    commander_was_open: bool,
}

impl AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let state = cx.new(AppState::new);
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        let commander = cx.new(|cx| CommanderView::new(state.clone(), cx));
        focus_handle.focus(window);
        if let Some(dir) = crate::shots::shots_dir() {
            crate::shots::start_tour(dir, state.clone(), window, cx);
        }
        Self {
            focus_handle,
            state,
            commander,
            commander_was_open: false,
        }
    }

    pub fn state(&self) -> &Entity<AppState> {
        &self.state
    }

    fn page(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.state.read(cx).route() {
            Route::Projects => projects::projects_page(&self.state, cx),
            Route::AppSettings { .. } => placeholder::app_settings_page(cx),
            Route::Project { section, .. } => match section {
                Section::Inbox => inbox::inbox_page(&self.state, cx),
                section => placeholder::section_page(*section, cx),
            },
        }
    }

    fn update_state(&self, cx: &mut Context<Self>, f: impl FnOnce(&mut AppState)) {
        self.state.update(cx, |state, cx| {
            f(state);
            cx.notify();
        });
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        let page = self.page(cx);
        let app = self.state.read(cx);
        let toasts = app.toasts().to_vec();
        let in_project = matches!(app.route(), Route::Project { .. });
        let commander_open = app.commander_open();
        if commander_open && !self.commander_was_open {
            self.commander
                .update(cx, |commander, cx| commander.open(window, cx));
        } else if !commander_open && self.commander_was_open {
            self.focus_handle.focus(window);
        }
        self.commander_was_open = commander_open;

        let dismiss_state = self.state.clone();
        let dismiss = Rc::new(move |id, _: &mut Window, cx: &mut App| {
            dismiss_state.update(cx, |state, cx| {
                state.dismiss_toast(id);
                cx.notify();
            });
        });

        // `.pw-shell-main-frame`: padding 0 8px 8px 0 under the title bar, 8px on the left
        // too when there is no sidebar.
        let frame = div()
            .flex_1()
            .h_full()
            .min_w(px(0.0))
            .pr(px(theme.space_2))
            .pb(px(theme.space_2))
            .when(!in_project, |frame| frame.pl(px(theme.space_2)))
            .child(main_surface(page, &theme));
        let mut body = div().flex_1().min_h(px(0.0)).w_full().flex().flex_row();
        if in_project {
            body = body.child(sidebar::sidebar(&self.state, cx));
        }
        body = body.child(frame);

        let mut root = div()
            .id("polterware-root")
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.color_shell_glass)
            .text_color(theme.color_text_primary)
            .font_family(theme.font_sans)
            .text_size(px(theme.font_size_md))
            .on_action(cx.listener(|_, _: &CloseWindow, window, _| window.remove_window()))
            .on_action(cx.listener(|this, _: &ToggleCommander, _, cx| {
                this.update_state(cx, |state| {
                    let open = state.commander_open();
                    state.set_commander(!open);
                });
            }))
            .on_action(cx.listener(|this, _: &OpenAppSettings, _, cx| {
                this.update_state(cx, |state| state.open_app_settings());
            }))
            .on_action(cx.listener(|this, _: &CloseOverlay, _, cx| {
                this.update_state(cx, |state| {
                    if state.commander_open() {
                        state.set_commander(false);
                    } else {
                        state.close_app_settings();
                    }
                });
            }))
            .child(titlebar::titlebar(&self.state, cx))
            .child(body);
        if commander_open {
            let close = self.state.clone();
            root = root.child(modal_layer(
                1.0,
                true,
                Rc::new(move |_, cx: &mut App| {
                    close.update(cx, |state, cx| {
                        state.set_commander(false);
                        cx.notify();
                    });
                }),
                self.commander.clone(),
                &theme,
            ));
        }
        root.child(toast_stack(&toasts, dismiss, cx))
    }
}
