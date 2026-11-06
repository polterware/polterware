//! A select (`Select` in the design system): a trigger showing the value, which opens a menu
//! of choices under it. The menu hangs over the whole window from where the trigger was last
//! laid out, so a panel's edge or a dialog never clips it.

use std::rc::Rc;

use gpui::{AnyElement, App, AppContext as _, Context, Entity, SharedString, div, prelude::*, px};
use polterware_ui::menu::{Menu, MenuItem};
use polterware_ui::patterns::{AnchorSlot, window_anchor};
use polterware_ui::select::select_trigger;

/// A select's menu, and where its trigger was laid out.
pub(crate) struct Picker {
    menu: Entity<Menu>,
    anchor: AnchorSlot,
}

impl Picker {
    pub(crate) fn new<V: 'static>(cx: &mut Context<V>) -> Self {
        let menu = cx.new(Menu::new);
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        Self {
            menu,
            anchor: AnchorSlot::default(),
        }
    }

    /// The trigger showing `label`, which opens the menu of `items` under it.
    pub(crate) fn trigger(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        width: f32,
        items: Vec<MenuItem>,
        disabled: bool,
        cx: &App,
    ) -> AnyElement {
        let menu = self.menu.clone();
        let trigger = select_trigger(
            id.into(),
            label,
            None,
            Rc::new(move |_, cx: &mut App| {
                if disabled {
                    return;
                }
                let items = items.clone();
                menu.update(cx, |menu, cx| {
                    let open = menu.is_open();
                    menu.set_menu(items);
                    menu.set_width(width);
                    menu.set_open(!open);
                    cx.notify();
                });
            }),
            cx,
        );
        div()
            .relative()
            .w(px(width))
            .when(disabled, |trigger| trigger.opacity(0.5))
            .child(trigger)
            .child(self.anchor.probe())
            .into_any_element()
    }

    /// The open menu, hung under its trigger.
    pub(crate) fn overlay(&self, cx: &App) -> Option<AnyElement> {
        let anchor = self.anchor.get()?;
        self.menu
            .read(cx)
            .is_open()
            .then(|| window_anchor(anchor, self.menu.clone()))
    }

    pub(crate) fn close(&self, cx: &mut App) {
        self.menu.update(cx, |menu, cx| {
            menu.set_open(false);
            cx.notify();
        });
    }
}
