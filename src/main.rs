use dioxus::prelude::*;
use dioxus::LaunchBuilder;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use syggrel_mrs::core::routes::{AppRouter, Route};
use std::process::ExitCode;

// ---- Constants ----
const APP_TITLE: &str = "Syggrel-MRS";
const WINDOW_WIDTH: f64 = 1280.0;
const WINDOW_HEIGHT: f64 = 720.0;

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
    // Configure the desktop window
    let window = WindowBuilder::new()
        .with_title(APP_TITLE)
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

    let config = Config::default()
        .with_window(window);

    LaunchBuilder::new()
        .with_cfg(config)
        .launch(AppRouter);

    Ok(())
}

// Define 'app' as a Dioxus component
#[component]
fn app() -> Element {
    rsx! {
        div { class: "app-container",
            NavigationMenu {}
            AppRouter {}
        }
    }
}

#[component]
fn NavigationMenu() -> Element {
    // Get access to the current route signal from the AppRouter component's scope.

    let mut current_route = use_context::<Signal<Route>>();

    rsx! {
        nav { class: "navigation-menu",
            ul { class: "nav-list",
                li {
                    class: if current_route() == Route::Home { "nav-item active" } else { "nav-item" },
                    onclick: move |_| current_route.set(Route::Home),
                    "Home"
                }
                li {
                    class: if current_route() == Route::Analysis { "nav-item active" } else { "nav-item" },
                    onclick: move |_| current_route.set(Route::Analysis),
                    "Analysis"
                }
                li {
                    class: if current_route() == Route::Data { "nav-item active" } else { "nav-item" },
                    onclick: move |_| current_route.set(Route::Data),
                    "Data"
                }
                li {
                    class: if current_route() == Route::Settings { "nav-item active" } else { "nav-item" },
                    onclick: move |_| current_route.set(Route::Settings),
                    "Settings"
                }
            }
        }
    }
}
