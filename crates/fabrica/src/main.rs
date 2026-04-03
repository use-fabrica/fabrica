mod app;
mod layout;
mod panels;
mod session_dock;

use gpui::{AppContext, Application, WindowOptions};
use gpui_component_assets::Assets;
use ui::Root;

use crate::app::Fabrica;

fn main() {
    Application::new().with_assets(Assets).run(move |cx| {
        ui::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| Fabrica::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window")
        })
        .detach();
    })
}
