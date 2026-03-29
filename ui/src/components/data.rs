use dioxus::prelude::*;

#[component]
pub fn DataView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Data Management" }
            p { "View and manage genomic datasets" }
        }
    }
}