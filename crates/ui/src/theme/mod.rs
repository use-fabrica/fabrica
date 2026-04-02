use gpui::App;
use gpui_component::{Theme, ThemeConfig};
use std::rc::Rc;

const FABRICA_LIGHT: &str = include_str!("../../themes/fabrica-light.json");
const FABRICA_DARK: &str = include_str!("../../themes/fabrica-dark.json");

pub fn init(cx: &mut App) {
    gpui_component::init(cx);

    let light_config = parse_first_theme(FABRICA_LIGHT);
    let dark_config = parse_first_theme(FABRICA_DARK);

    {
        let theme = Theme::global_mut(cx);
        theme.light_theme = Rc::new(light_config);
        theme.dark_theme = Rc::new(dark_config);
    }

    let mode = Theme::global(cx).mode;
    Theme::change(mode, None, cx);
}

fn parse_first_theme(json: &str) -> ThemeConfig {
    let set: gpui_component::ThemeSet =
        serde_json::from_str(json).expect("Failed to parse Fabrica theme JSON");
    set.themes
        .into_iter()
        .next()
        .expect("Theme JSON must contain at least one theme")
}
