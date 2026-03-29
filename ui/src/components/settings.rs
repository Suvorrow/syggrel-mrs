use dioxus::prelude::*;

#[component]
pub fn SettingsView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Configure application settings." }
            p { "Configure your preferences" }
        }
    }
}