use std::path::PathBuf;

use gpui::*;
use ui::Styled;
use ui::dock::{DockArea, DockItem};

use crate::dashboard::Dashboard;
use crate::panels::test::TestPanel;
use crate::recent_projects::RecentProjects;

pub(crate) struct Fabrica {
    dock_area: Entity<DockArea>,
    dashboard: Entity<Dashboard>,
    recent_projects: Entity<RecentProjects>,
    project_open: bool,
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

        let data_dir = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".local/share/fabrica"))
            .unwrap_or_else(|_| PathBuf::from(".fabrica"));
        let recent_path = data_dir.join("recent.json");
        let recent_projects = cx.new(|_cx| {
            let mut rp = RecentProjects::load(&recent_path);
            rp.prune();
            rp
        });
        let dashboard = cx.new(|cx| Dashboard::new(window, cx, recent_projects.clone()));

        let fabrica_weak = cx.entity().downgrade();
        dashboard.update(cx, |dashboard, _| {
            dashboard.set_fabrica(fabrica_weak);
        });

        Self {
            dock_area,
            dashboard,
            recent_projects,
            project_open: false,
        }
    }

    pub(crate) fn open_project(
        &mut self,
        path: PathBuf,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !path.exists() || !path.is_dir() {
            return;
        }
        self.recent_projects.update(cx, |rp, _| {
            rp.add(&path);
        });
        self.project_open = true;
        cx.notify();
    }

    pub(crate) fn close_project(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.project_open = false;
        self.dashboard.update(cx, |_, _| {});
        window.focus(&self.dashboard.read(cx).focus_handle(cx));
        cx.notify();
    }
}

impl Render for Fabrica {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.project_open {
            div()
                .id("workspace-container")
                .size_full()
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key.clone() == "escape" {
                        this.close_project(window, cx);
                    }
                }))
                .child(self.dock_area.clone())
                .into_any_element()
        } else {
            self.dashboard.clone().into_any_element()
        }
    }
}
