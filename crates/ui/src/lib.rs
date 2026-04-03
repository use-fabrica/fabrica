mod theme;
pub mod tokens;

pub use gpui::Styled;
pub use gpui_component::button::{Button, ButtonVariants};
pub use gpui_component::checkbox::Checkbox;
pub use gpui_component::dock;
pub use gpui_component::input::{Input, InputState};
pub use gpui_component::label::Label;
pub use gpui_component::progress::Progress;
pub use gpui_component::switch::Switch;
pub use gpui_component::{
    ActiveTheme, Colorize, Disableable, Icon, IconName, InteractiveElementExt, Sizable, Size,
    StyleSized, StyledExt,
};
pub use gpui_component::{Root, WindowExt, h_flex, v_flex};
pub use theme::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dock_api_surface() {
        fn _dock_area(_: &dock::DockArea) {}
        fn _dock_item(_: &dock::DockItem) {}
        fn _panel<T: dock::Panel>(_: &T) {}
        fn _panel_event(_: &dock::PanelEvent) {}
        fn _dock_placement(_: &dock::DockPlacement) {}
        fn _dock_area_state(_: &dock::DockAreaState) {}
    }

    #[test]
    fn framework_types_accessible() {
        fn _root(_: &Root) {}
        fn _window_ext<T: WindowExt>(_: &T) {}
    }

    #[test]
    fn component_types_accessible() {
        fn _button(_: &Button) {}
        fn _checkbox(_: &Checkbox) {}
        fn _input(_: &Input) {}
        fn _label(_: &Label) {}
        fn _progress(_: &Progress) {}
        fn _switch(_: &Switch) {}
    }
}
