pub mod components;

const COMPILED_CSS: &str = include_str!("../assets/compiled_styles.css");

pub fn get_compiled_css() -> &'static str {
    COMPILED_CSS
}

pub use components::{HomeView, DataView, SettingsView, ToolsView};
pub use components::parser::ParserToolView;
