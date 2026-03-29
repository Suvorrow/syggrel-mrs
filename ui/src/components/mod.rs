pub mod home;
pub mod tools;
pub mod data;
pub mod settings;
pub mod parser;

// Re-export components for easier access from other modules
pub use home::HomeView;
pub use tools::ToolsView;
pub use data::DataView;
pub use settings::SettingsView;
pub use parser::ParserToolView;
