use gpui::*;
use ui::dock::{Panel, PanelEvent};

pub struct DashboardPanel {
    focus_handle: FocusHandle,
    name: SharedString,
}

impl DashboardPanel {
    pub(crate) fn new(_: &Window, cx: &App) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            focus_handle,
            name: "DashboardPanel".into(),
        }
    }
}

impl Render for DashboardPanel {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
    }
}

impl Focusable for DashboardPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for DashboardPanel {}
impl Panel for DashboardPanel {
    fn panel_name(&self) -> &'static str {
        "DashboardPanel"
    }

    fn closable(&self, _: &App) -> bool {
        false
    }
}
