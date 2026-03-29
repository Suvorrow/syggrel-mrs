use dioxus::prelude::*;

#[component]
pub fn HomeView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Welcome to Syggrel-MRS" }
            p { "Advanced morphodynamic risks sentinel platform" }
        }
    }
}