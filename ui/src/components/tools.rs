use dioxus::prelude::*;

#[component]
pub fn ToolsView() -> Element {
    rsx! {
        div { class: "view-container",
            h1 { "Combine gRNA Risk Analysis Tools" }
            p { "Perform safety assessments here" }
        }
    }
}