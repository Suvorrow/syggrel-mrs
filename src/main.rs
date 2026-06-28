// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use dioxus::prelude::*;
use dioxus::LaunchBuilder;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use syggrel_mrs::App;
use std::process::ExitCode;

// ---- Constants ----
const APP_TITLE: &str = "Syggrel-MRS";
const WINDOW_WIDTH: f64 = 1536.0;
const WINDOW_HEIGHT: f64 = 864.0;

// ---- Main Entry Point ----
fn main() -> ExitCode {
    // Initialize logging early for debugging and observability.
    // TODO: Build a more configurable logger setup later.
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logger: {e}");
        return ExitCode::FAILURE;
    }

    // Perform application setup, returning a Result.
    match setup() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Application startup failed: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---- Initialization Functions ----
fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: For production build configuring levels, output format, or file targets.
    tracing_subscriber::fmt::init();
    Ok(())
}

// ---- Application Setup ----
fn setup() -> Result<(), Box<dyn std::error::Error>> {
    // Get the embedded CSS content from the ui crate
    let css_content = ui::get_compiled_css();

    // Configure the desktop window
    let window = WindowBuilder::new()
        .with_title(APP_TITLE)
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_decorations(false)
        .with_transparent(true)
        .with_minimizable(true)
        .with_maximizable(true)
        .with_resizable(true);

    let config = Config::default()
        .with_window(window)
        .with_custom_head(format!(
            r#"
        <title>{}</title>
        <style>{}</style>
        "#,
            APP_TITLE, css_content
        ));
    
    LaunchBuilder::new()
        .with_cfg(config)
        .launch(App);

    Ok(())
}
