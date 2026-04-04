use std::path::PathBuf;

use gpui::*;
use ui::tokens::Spacing;
use ui::{ActiveTheme, Button, ButtonVariants, Styled, h_flex, v_flex};

use crate::recent_projects::{RecentProjects, format_relative_time};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DashboardEvent {
    OpenProject { path: PathBuf },
    DeleteRecent { path: PathBuf },
    PickFolder,
}

pub(crate) struct Dashboard {
    focus_handle: FocusHandle,
    recent_projects: Entity<RecentProjects>,
    focused_index: Option<usize>,
}

impl EventEmitter<DashboardEvent> for Dashboard {}

impl Dashboard {
    pub(crate) fn new(cx: &mut Context<Self>, recent_projects: Entity<RecentProjects>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            recent_projects,
            focused_index: None,
        }
    }

    pub(crate) fn open_focused(&mut self, cx: &mut Context<Self>) {
        if let Some(i) = self.focused_index {
            let projects = self.recent_projects.read(cx);
            if let Some(entry) = projects.projects.get(i) {
                cx.emit(DashboardEvent::OpenProject {
                    path: entry.path.clone(),
                });
            }
        }
    }

    pub(crate) fn delete_focused(&mut self, cx: &mut Context<Self>) {
        if let Some(i) = self.focused_index {
            let path = self
                .recent_projects
                .read(cx)
                .projects
                .get(i)
                .map(|p| p.path.clone());
            if let Some(path) = path {
                let new_len = self
                    .recent_projects
                    .read(cx)
                    .projects
                    .len()
                    .saturating_sub(1);
                if new_len == 0 {
                    self.focused_index = None;
                } else if i >= new_len {
                    self.focused_index = Some(new_len - 1);
                }
                cx.emit(DashboardEvent::DeleteRecent { path });
                cx.notify();
            }
        }
    }

    pub(crate) fn pick_folder(&mut self, cx: &mut Context<Self>) {
        cx.emit(DashboardEvent::PickFolder);
    }

    fn render_left_column(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let projects = &self.recent_projects.read(cx).projects;
        let radius = theme.radius;

        let mut col = v_flex()
            .w(relative(0.65))
            .p(Spacing::px_6())
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.colors.foreground)
                    .child("Fabrica"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.colors.muted_foreground)
                    .child(env!("CARGO_PKG_VERSION")),
            );

        if projects.is_empty() {
            col = col.child(
                div()
                    .mt(Spacing::px_6())
                    .text_sm()
                    .text_color(theme.colors.muted_foreground)
                    .child("No projects yet"),
            );
        } else {
            let mut list = v_flex().mt(Spacing::px_6()).gap(Spacing::px_0_5());
            for (i, entry) in projects.iter().enumerate() {
                let name = entry
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let rel_time = format_relative_time(&entry.last_opened);
                let is_focused = self.focused_index == Some(i);

                let mut row = h_flex()
                    .id(ElementId::Name(format!("project-{i}").into()))
                    .px(Spacing::px_2())
                    .py(Spacing::px_1())
                    .rounded(radius)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        let path = this.recent_projects.read(cx).projects[i].path.clone();
                        cx.emit(DashboardEvent::OpenProject { path });
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.colors.foreground)
                            .child(name),
                    )
                    .child(
                        div()
                            .ml(Spacing::px_2())
                            .text_sm()
                            .text_color(theme.colors.muted_foreground)
                            .child(rel_time),
                    );

                if is_focused {
                    row = row
                        .bg(theme.colors.list_active)
                        .border_color(theme.colors.list_active_border)
                        .border_1();
                }

                list = list.child(row);
            }
            col = col.child(list);
        }

        col = col.child(
            Button::new("open-project-btn")
                .label("Open Project")
                .primary()
                .mt(Spacing::px_4())
                .on_click(cx.listener(move |this, _event, _window, cx| {
                    this.pick_folder(cx);
                })),
        );

        col
    }

    fn render_right_column(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w(px(320.))
            .p(Spacing::px_6())
            .gap(Spacing::px_6())
            .child(Self::render_shortcuts(cx))
            .child(self.render_telemetry(cx))
    }

    fn render_shortcuts(cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let shortcuts = [
            ("↑/↓", "Navigate"),
            ("Enter", "Open"),
            ("Alt+O", "Browse"),
            ("Delete", "Remove"),
            ("Ctrl+Q", "Quit"),
        ];

        let mut container = v_flex().gap(Spacing::px_3()).child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.foreground)
                .child("Shortcuts"),
        );

        for (key, desc) in shortcuts {
            container = container.child(
                h_flex()
                    .gap(Spacing::px_2())
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.colors.muted_foreground)
                            .child(key),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.colors.muted_foreground)
                            .child(desc),
                    ),
            );
        }

        container
    }

    fn render_telemetry(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let projects = &self.recent_projects.read(cx).projects;

        if projects.is_empty() {
            div()
                .text_sm()
                .text_color(theme.colors.muted_foreground)
                .child("No projects opened yet")
        } else {
            let count = projects.len();
            let count_str = if count == 1 {
                "1 project".to_string()
            } else {
                format!("{} projects", count)
            };
            let last_name = projects[0]
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            let last_rel = format_relative_time(&projects[0].last_opened);

            v_flex()
                .gap(Spacing::px_1())
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.colors.muted_foreground)
                        .child(count_str),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.colors.muted_foreground)
                        .child(format!("Last: {}, {}", last_name, last_rel)),
                )
        }
    }
}

impl Render for Dashboard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id("dashboard")
            .track_focus(&self.focus_handle)
            .tab_index(0)
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(theme.colors.background)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let len = this.recent_projects.read(cx).projects.len();
                match event.keystroke.key.as_ref() {
                    "j" | "down" => {
                        if len == 0 {
                            return;
                        }
                        this.focused_index = Some(match this.focused_index {
                            None => 0,
                            Some(n) => (n + 1).min(len - 1),
                        });
                        cx.notify();
                    }
                    "k" | "up" => {
                        if len == 0 {
                            return;
                        }
                        this.focused_index = match this.focused_index {
                            None => None,
                            Some(0) => Some(0),
                            Some(n) => {
                                let new = n.saturating_sub(1);
                                this.focused_index = Some(new);
                                cx.notify();
                                return;
                            }
                        };
                        cx.notify();
                    }
                    "enter" => {
                        this.open_focused(cx);
                    }
                    "o" if event.keystroke.modifiers.alt => {
                        this.pick_folder(cx);
                    }
                    "delete" => {
                        this.delete_focused(cx);
                    }
                    _ => {}
                }
            }))
            .child(
                h_flex()
                    .max_w(px(1024.))
                    .mx_auto()
                    .w_full()
                    .child(self.render_left_column(cx))
                    .child(self.render_right_column(cx)),
            )
    }
}

impl Focusable for Dashboard {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
