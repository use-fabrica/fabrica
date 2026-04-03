use gpui::*;
use ui::Styled;
use ui::dock::{DockArea, DockItem};

use crate::panels::test::TestPanel;

pub(crate) struct Fabrica {
    dock_area: Entity<DockArea>,
}

impl Fabrica {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let dock_area = cx.new(|cx| DockArea::new("main-dock", None, window, cx));

        let weak_dock_area = dock_area.downgrade();

        let test_panel = cx.new(|cx| TestPanel::new(window, cx));
        let center_item = DockItem::tab(test_panel, &weak_dock_area, window, cx);

        dock_area.update(cx, |dock_area: &mut DockArea, cx| {
            dock_area.set_center(center_item, window, cx);
            dock_area.set_locked(true, window, cx);
        });

        Self { dock_area }
    }
}

impl Render for Fabrica {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.dock_area.clone())
    }
}
