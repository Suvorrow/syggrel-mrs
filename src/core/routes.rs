#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus::desktop;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum View {
    #[default]
    Home,
    Tools,
    Data,
    Settings,
}

#[component]
pub fn App() -> Element {
    let mut current_view = use_signal(View::default);
    let mut is_maximized = use_signal(|| false);

    rsx! {
        div { class: "app-container",
            // Your custom navigation bar - exactly where you want it
            div { class: "top-nav-bar",
                div { class: "nav-spacer" }
                div { class: "button-container",
                    button {
                        class: if current_view() == View::Home { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| current_view.set(View::Home),
                        "Home"
                    }
                    button {
                        class: if current_view() == View::Tools { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| current_view.set(View::Tools),
                        "Tools"
                    }
                    button {
                        class: if current_view() == View::Data { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| current_view.set(View::Data),
                        "Data"
                    }
                    button {
                        class: if current_view() == View::Settings { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| current_view.set(View::Settings),
                        "Settings"
                    }
                    button { class: "nav-btn", onclick: move |_| {}, "Go" }
                    button { class: "nav-btn", onclick: move |_| {}, "Run" }
                    button { class: "nav-btn", onclick: move |_| {}, "Terminal" }
                    button { class: "nav-btn", onclick: move |_| {}, "Help" }
                }
                // Right side - window controls
                div { class: "window-controls",
                    button {
                        class: "window-btn minimize-btn",
                        onclick: move |_| {
                            desktop::window().set_minimized(true);
                        },
                        "▭"
                    }
                    // Maximize/Restore button
                    button {
                        class: if is_maximized() { "window-btn maximize-btn maximized" } else { "window-btn maximize-btn" },
                        onclick: move |_| {
                            is_maximized.set(!is_maximized());
                            desktop::window().toggle_maximized();
                        },
                        "△"
                    }
                    // Close button
                    button {
                        class: "window-btn close-btn",
                        onclick: move |_| {
                            desktop::window().close();
                        },
                        "✕"
                    }
                }
            }
            // Content area below the navigation
            div { class: "content-area",
                match current_view() {
                    View::Home => rsx! {
                        HomeView {}
                    },
                    View::Tools => rsx! {
                        ToolsView {}
                    },
                    View::Data => rsx! {
                        DataView {}
                    },
                    View::Settings => rsx! {
                        SettingsView {}
                    },
                }
            }
        }
    }
}

        
#[component]
fn HomeView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Welcome to Syggrel-MRS" }
            p { "Advanced morphodynamic risks sentinel platform" }
        }
    }
}

#[component]
fn ToolsView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Combine gRNA Risk Analysis Tools" }
            p { "Perform safety assessments here" }
        }
    }
}

#[component]
fn DataView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Data Management" }
            p { "View and manage genomic datasets" }
        }
    }
}

#[component]
fn SettingsView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Configure application settings." }
            p { "Configure your preferences" }
        }
    }
}


/*
use dioxus::prelude::*;

/// Defines the possible "routes" or views in the application.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum View {
    #[default]
    Home,
    Analysis,
    Data,
    Settings,
}

/// The main router component for the desktop app.
/// It manages the current route state and renders the appropriate component.
#[component]
pub fn App() -> Element {
    // Use a Signal to track the current route.
    let mut current_view = use_signal(View::default);

    // use_context_provider(|| current_route);

    rsx! {
        div { class: "app-container",
            header { class: "app-header",
                i { class: "material-icons icon-menu", "menu" }
                h1 { "Syggrel-MRS: {view_name(current_view())}" }
                span {}
                nav { class: "navigation-menu",
                    ul { class: "nav-list",
                        li {
                            class: if current_view() == View::Home { "nav-item active" } else { "nav-item" },
                            onclick: move |_| current_view.set(View::Home),
                            "Home"
                        }
                        li {
                            class: if current_view() == View::Analysis { "nav-item active" } else { "nav-item" },
                            onclick: move |_| current_view.set(View::Analysis),
                            "Analysis"
                        }
                        li {
                            class: if current_view() == View::Data { "nav-item active" } else { "nav-item" },
                            onclick: move |_| current_view.set(View::Data),
                            "Data"
                        }
                        li {
                            class: if current_view() == View::Settings { "nav-item active" } else { "nav-item" },
                            onclick: move |_| current_view.set(View::Settings),
                            "Settings"
                        }
                    }
                }
            }
            main { class: "app-main",
                match current_view() {
                    View::Home => rsx! {
                        HomeView {}
                    },
                    View::Analysis => rsx! {
                        AnalysisView {}
                    },
                    View::Data => rsx! {
                        DataView {}
                    },
                    View::Settings => rsx! {
                        SettingsView {}
                    },
                }
            }
        }
    }
}

fn view_name(view: View) -> String {
    match view {
        View::Home => "Home".to_string(),
        View::Analysis => "Analysis".to_string(),
        View::Data => "Data".to_string(),
        View::Settings => "Settings".to_string(),
    }
}

#[component]
fn HomeView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Welcome to Syggrel-MRS" }
            p { "Advanced morphodynamic risks sentinel platform" }
        }
    }
}

#[component]
fn AnalysisView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Combine gRNA Risk Analysis" }
            p { "Perform safety assessments here" }
        }
    }
}

#[component]
fn DataView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Data Management" }
            p { "View and manage genomic datasets" }
        }
    }
}

#[component]
fn SettingsView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Configure application settings." }
            p { "Configure your preferences" }
        }
    }
}
*/

