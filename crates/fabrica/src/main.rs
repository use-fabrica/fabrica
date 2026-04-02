use gpui::{
    AppContext, Application, ParentElement, Render, SharedString, Styled, WindowOptions, div, px,
    rgb,
};
use gpui_component_assets::Assets;
use ui::Root;

struct Fabrica(SharedString);

impl Render for Fabrica {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x000000))
            .size(px(500.00))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000fff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello 2, {}", &self.0))
    }
}

fn main() {
    Application::new().with_assets(Assets).run(move |cx| {
        ui::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| Fabrica("Fabrica".into()));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window")
        })
        .detach();
    })
}
