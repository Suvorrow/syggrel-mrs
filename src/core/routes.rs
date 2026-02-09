use dioxus::prelude::*;

/// Defines the possible "routes" or views in the application.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Route {
    #[default]
    Home,
    Analysis,
    Data,
    Settings,
}

/// The main router component for the desktop app.
/// It manages the current route state and renders the appropriate component.
#[component]
pub fn AppRouter() -> Element {
    // Use a Signal to track the current route.
    let current_route = use_signal(Route::default);

    use_context_provider(|| current_route);

    rsx! {
        div { class: "app-router-container",
            // Render the component based on the current route.
            match current_route() {
                Route::Home => rsx! {
                    Home {}
                },
                Route::Analysis => rsx! {
                    Analysis {}
                },
                Route::Data => rsx! {
                    Data {}
                },
                Route::Settings => rsx! {
                    Settings {}
                },
            }
        }
    }
}

/// Component for the Home view.
#[component]
fn Home() -> Element {
    rsx! {
        h1 { "Welcome to Syggrel-MRS" }
        p { "The home page." }
    }
}

/// Component for the Analysis view.
#[component]
fn Analysis() -> Element {
    rsx! {
        h1 { "Analysis View" }
        p { "Perform gRNA risk analysis here." }
        // Place for the MainContent component integration.
    }
}

/// Component for the Data view.
#[component]
fn Data() -> Element {
    rsx! {
        h1 { "Data View" }
        p { "Manage and view Hi-C data." }
    }
}

/// Component for the Settings view.
#[component]
fn Settings() -> Element {
    rsx! {
        h1 { "Settings" }
        p { "Configure application settings." }
    }
}
