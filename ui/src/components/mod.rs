// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

pub mod home;
pub mod tools;
pub mod data;
pub mod dropdown;
pub mod parser;
pub mod settings;


// Re-export components for access from other modules
pub use home::HomeView;
pub use tools::ToolsView;
pub use data::DataView;
pub use dropdown::{dropdown_header, dropdown_item, DropdownItem, DropdownPanel};
pub use settings::SettingsView;
pub use parser::ParserToolView;
