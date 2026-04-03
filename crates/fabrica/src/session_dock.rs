use gpui::*;
use ui::Styled;
use ui::dock::{DockArea, DockItem};

use crate::layout::Layout;
use crate::panels::test::TestPanel;

struct DockAreaTab {
    id: &'static str,
    version: usize,
}

const SESSION_DOCKAREA: DockAreaTab = DockAreaTab {
    id: "session-dock",
    version: 1,
};

pub(crate) struct SessionDock {
    pub(crate) dock_area: Entity<DockArea>,
}

impl Layout for SessionDock {}

impl SessionDock {
    pub(crate) fn new(window: &mut Window, cx: &mut App) -> Self {
        let dock_area = cx.new(|cx| {
            DockArea::new(
                SESSION_DOCKAREA.id,
                Some(SESSION_DOCKAREA.version),
                window,
                cx,
            )
        });

        let weak_dock_area = dock_area.downgrade();

        let test_panel = cx.new(|cx| TestPanel::new(window, cx));
        let center_item = DockItem::tab(test_panel, &weak_dock_area, window, cx);

        dock_area.update(cx, |dock_area, cx| {
            dock_area.set_center(center_item, window, cx);
            dock_area.set_locked(true, window, cx);
        });

        Self { dock_area }
    }
}

impl Render for SessionDock {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.dock_area.clone())
    }
}
