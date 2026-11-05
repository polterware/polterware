//! Polterware in GPUI: the Desktop's frame and surfaces over the local Go core, read through
//! Connect-JSON. The Electron Desktop (`apps/polterware-desktop`) stays the shipped app until
//! this one reaches parity.

pub mod app;
pub mod canvas;
pub mod cli_reference;
pub mod client;
pub mod contracts;
pub mod dispatch;
pub mod format;
pub mod mcp;
pub mod prefs;
pub mod proto_json;
pub mod shortcuts;
pub mod shots;
pub mod state;
pub mod updater;
pub mod views;
pub mod work;

use app::{
    About, CheckForUpdates, Hide, HideOthers, Minimize, Quit, ShowAll, ToggleFullScreen, Zoom,
    register_key_bindings,
};
use gpui::{App, AppContext as _, Application, Menu, MenuItem, OsAction, SystemMenuType, px, size};
use polterware_ui::icons::IconAssets;
use polterware_ui::text_input::{Copy, Cut, Paste, Redo, SelectAll, Undo};
use polterware_ui::theme::{self, Theme};
use polterware_ui::window::vibrancy;

/// The design system's root size in the desktop apps.
const REM: f32 = 13.0;

/// Opens the Polterware window and runs the app until it quits.
pub fn run() {
    Application::new()
        .with_assets(IconAssets)
        .run(|cx: &mut App| {
            cx.activate(true);
            theme::set_root_rem(REM);
            cx.set_global(Theme::dark());
            polterware_ui::init(cx);
            register_key_bindings(cx);
            if updater::start().is_ok() {
                cx.on_action(|_: &CheckForUpdates, _| updater::check());
            }
            // Follows the Electron Desktop's `application-menu.ts` using GPUI's menu API.
            cx.set_menus(vec![
                Menu {
                    name: "Polterware".into(),
                    items: vec![
                        MenuItem::action("About Polterware", About),
                        MenuItem::separator(),
                        MenuItem::action("Check for Updates…", CheckForUpdates),
                        MenuItem::separator(),
                        MenuItem::os_submenu("Services", SystemMenuType::Services),
                        MenuItem::separator(),
                        MenuItem::action("Hide Polterware", Hide),
                        MenuItem::action("Hide Others", HideOthers),
                        MenuItem::action("Show All", ShowAll),
                        MenuItem::separator(),
                        MenuItem::action("Quit", Quit),
                    ],
                },
                Menu {
                    name: "Edit".into(),
                    items: vec![
                        MenuItem::os_action("Undo", Undo, OsAction::Undo),
                        MenuItem::os_action("Redo", Redo, OsAction::Redo),
                        MenuItem::separator(),
                        MenuItem::os_action("Cut", Cut, OsAction::Cut),
                        MenuItem::os_action("Copy", Copy, OsAction::Copy),
                        MenuItem::os_action("Paste", Paste, OsAction::Paste),
                        MenuItem::os_action("Select All", SelectAll, OsAction::SelectAll),
                    ],
                },
                Menu {
                    name: "View".into(),
                    items: vec![MenuItem::action("Toggle Full Screen", ToggleFullScreen)],
                },
                Menu {
                    name: "Window".into(),
                    items: vec![
                        MenuItem::action("Minimize", Minimize),
                        MenuItem::action("Zoom", Zoom),
                    ],
                },
            ]);
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();

            // The Electron Desktop's window: 1440 by 920, at least 1180 by 760.
            let mut options = polterware_ui::window::window_options(
                "Polterware",
                size(px(1440.0), px(920.0)),
                size(px(1180.0), px(760.0)),
                cx,
            );
            // `trafficLightPosition: { x: 14, y: 15 }`, as the Electron window sets them.
            if let Some(titlebar) = options.titlebar.as_mut() {
                titlebar.traffic_light_position = Some(gpui::point(px(14.0), px(15.0)));
            }
            cx.open_window(options, |window, cx| {
                vibrancy::install(window);
                cx.new(|cx| app::AppView::new(window, cx))
            })
            .expect("failed to open GPUI window");
        });
}
