use gpui::{App, Entity, ParentElement, Render, Window, div};
use ui::{Styled, components::dock::DockArea};

use crate::session_dock::SessionDock;

pub(crate) struct Fabrica {
    dock_area: Entity<DockArea>,
}

impl Fabrica {
    pub(crate) fn new(window: &mut Window, cx: &mut App) -> Self {
        let session_dock = SessionDock::new(window, cx);

        Self {
            dock_area: session_dock.dock_area,
        }
    }
}

impl Render for Fabrica {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().size_full().child(self.dock_area.clone())
    }
}
