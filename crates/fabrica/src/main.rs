pub(crate) mod app;
pub(crate) mod dashboard;
mod layout;
pub(crate) mod panels;
pub(crate) mod recent_projects;
pub(crate) mod recent_projects_list;
pub(crate) mod time_utils;

use std::path::PathBuf;

use gpui::{AppContext, Application, WindowOptions};
use gpui_component_assets::Assets;
use ui::Root;

use crate::app::Fabrica;

fn main() {
    // Parse optional project path from CLI args — only accept if it exists and is a directory
    let cli_path: Option<PathBuf> = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .filter(|p| p.exists() && p.is_dir());

    Application::new().with_assets(Assets).run(move |cx| {
        ui::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| Fabrica::new(window, cx));

                // If a valid project path was provided via CLI, open it immediately (skip dashboard)
                if let Some(path) = cli_path {
                    view.update(cx, |fabrica, cx| {
                        fabrica.open_project(path, cx);
                    });
                }

                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window")
        })
        .detach();
    })
}
