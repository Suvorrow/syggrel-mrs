#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_router::{Routable, Router, Outlet};
use dioxus::desktop;
use ui::components::{HomeView, DataView, SettingsView, ToolsView,};
use ui::components::parser::ParserToolView;
use crate::core::state::{UiState, DropdownType, ActiveToolType};

#[component]
pub fn AppLayout() -> Element {
    let mut ui_state = use_signal(|| UiState::None);
    let current_route = use_route::<Route>();
    let navigator = use_navigator();

    let mut is_maximized = use_signal(|| false);

    let mut dropdown_hovered = use_signal(|| false);

    // Handle clicks outside to close dropdown
    let handle_tools_click = move |_| {
        match ui_state() {
            UiState::DropdownOpen(DropdownType::Tools) => ui_state.set(UiState::None),
            _ => ui_state.set(UiState::DropdownOpen(DropdownType::Tools)),
        }
    };

    // Define the click handler for the container
    let handle_container_click = move |_| {
    };

    rsx! {
        div { class: "app-container", onclick: handle_container_click,
            // Сustom navigation bar
            div { class: "top-nav-bar",
                div { class: "nav-spacer" }
                div { class: "button-container",
                    button {
                        id: "home-button",
                        class: if matches!(current_route, Route::Home {}) { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| {
                            navigator.push(Route::Home {});
                        },
                        "Home"
                    }
                    button {
                        id: "tools-button",
                        class: if matches!(current_route, Route::Tools {}) { "nav-btn active" } else { "nav-btn" },
                        onclick: handle_tools_click,
                        onmouseenter: move |_| {},
                        "Tools"
                    }
                    button {
                        id: "data-button",
                        class: if matches!(current_route, Route::Data {}) { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| {
                            navigator.push(Route::Data {});
                        },
                        "Data"
                    }
                    button {
                        id: "settings-button",
                        class: if matches!(current_route, Route::Settings {}) { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| {
                            navigator.push(Route::Settings {});
                        },
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
                        onclick: move |_| desktop::window().set_minimized(true),
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
                        onclick: move |_| desktop::window().close(),
                        "✕"
                    }
                }
            }
            // Content area - Render based on the overall UI state (ui_state)
            div { class: "content-area",
                // Check the overall UI state first
                match ui_state() {
                    UiState::ActiveTool(ActiveToolType::Parser) => rsx! {
                        ParserToolView { close_tool: move || ui_state.set(UiState::None) }
                    },
                    UiState::DropdownOpen(DropdownType::Tools) => rsx! {
                        div {
                            class: "tools-dropdown-panel",
                            tabindex: "-1",
                            role: "menu",
                            // Capture hover state on the dropdown itself
                            onmouseenter: move |_| dropdown_hovered.set(true),
                            onmouseleave: move |_| dropdown_hovered.set(false),

                            button {
                                class: "dropdown-item nav-btn",
                                role: "menuitem",
                                onclick: move |_| {
                                    ui_state.set(UiState::None);
                                    ui_state.set(UiState::ActiveTool(ActiveToolType::Parser))
                                },
                                "Parser"
                            }
                            button {
                                class: "dropdown-item nav-btn",
                                role: "menuitem",
                                onclick: move |_| {
                                    ui_state.set(UiState::None);
                                },
                                "Other Tool N2"
                                // Add other items specific to the Tools dropdown here
                            }
                        }
                    },
                    _ => rsx! {
                        // When no special UI state is active, render the standard view
                        // based on the current_view signal
                        Outlet::<Route> {}
                    },
                }
            }
        }
    }
}

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        Home {},

        #[route("/tools")]
        Tools {},

        #[route("/data")]
        Data {},

        #[route("/settings")]
        Settings {},
}

#[component] fn Home() -> Element { rsx! {
    HomeView {}

} }
#[component] fn Tools() -> Element { rsx! {
    ToolsView {}
} }
#[component] fn Data() -> Element { rsx! {
    DataView {}
} }
#[component] fn Settings() -> Element { rsx! {
    SettingsView {}
} }

#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}