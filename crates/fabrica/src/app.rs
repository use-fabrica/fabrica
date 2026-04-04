use std::path::PathBuf;

use gpui::*;
use sqlx::SqlitePool;
use ui::Styled;
use ui::dock::{DockArea, DockItem};

use crate::dashboard::{Dashboard, DashboardEvent};
use crate::db;
use crate::panels::test::TestPanel;
use crate::recent_projects::RecentProjects;

pub(crate) struct Fabrica {
    dock_area: Entity<DockArea>,
    dashboard: Entity<Dashboard>,
    recent_projects: Entity<RecentProjects>,
    project_open: bool,
    db_pool: Option<SqlitePool>,
    _subscriptions: Vec<Subscription>,
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

        let recent_projects = cx.new(|_cx| {
            let mut rp = RecentProjects::load();
            rp.prune();
            rp
        });
        let dashboard = cx.new(|cx| Dashboard::new(cx, recent_projects.clone()));

        let subscription = cx.subscribe(&dashboard, |fabrica, _dashboard, event, cx| match event {
            DashboardEvent::OpenProject { path } => {
                fabrica.open_project(path.clone(), cx);
            }
            DashboardEvent::DeleteRecent { path } => {
                fabrica.recent_projects.update(cx, |rp, _| rp.remove(path));
            }
            DashboardEvent::PickFolder => {
                let rx = cx.prompt_for_paths(PathPromptOptions {
                    files: false,
                    directories: true,
                    multiple: false,
                    prompt: Some("Select Project Folder".into()),
                });
                cx.spawn(async move |this, cx| {
                    if let Ok(Ok(Some(paths))) = rx.await
                        && let Some(path) = paths.first()
                    {
                        let path = path.clone();
                        let _ = this.update(cx, |fabrica, cx| {
                            fabrica.open_project(path, cx);
                        });
                    }
                })
                .detach();
            }
        });

        Self {
            dock_area,
            dashboard,
            recent_projects,
            project_open: false,
            db_pool: None,
            _subscriptions: vec![subscription],
        }
    }

    pub(crate) fn open_project(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if !path.exists() || !path.is_dir() {
            return;
        }
        self.recent_projects.update(cx, |rp, _| {
            rp.add(&path);
        });
        self.project_open = true;
        cx.notify();
        cx.spawn(async move |this, cx| match db::init_db(&path).await {
            Ok(pool) => {
                let _ = this.update(cx, |fabrica, _cx| {
                    fabrica.db_pool = Some(pool);
                });
            }
            Err(e) => {
                eprintln!("Warning: DB init failed: {}", e);
            }
        })
        .detach();
    }

    pub(crate) fn close_project(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.db_pool = None;
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
