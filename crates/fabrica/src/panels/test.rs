use gpui::*;
use ui::dock::{Panel, PanelEvent};

pub(crate) struct TestPanel {
    name: SharedString,
    focus_handle: FocusHandle,
}

impl TestPanel {
    pub(crate) fn new(_: &Window, cx: &mut App) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            name: "".into(),
            focus_handle,
        }
    }
}

impl Render for TestPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().id("test-panel").text_center().text_xl().child("test")
    }
}

impl EventEmitter<PanelEvent> for TestPanel {}

impl Focusable for TestPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for TestPanel {
    fn panel_name(&self) -> &'static str {
        "TestPanel"
    }

    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.name.clone().into_element()
    }

    fn closable(&self, _: &App) -> bool {
        false
    }
}
