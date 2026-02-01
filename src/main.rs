use dioxus::prelude::*;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder, launch_cfg};
use syggrel_mrs::{GenomicTarget, mrs_algorithms::compute_topological_risk};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    launch_cfg(
        app,
        Config::new().with_window(
            WindowBuilder::new()
                .with_inner_size(LogicalSize::new(1280.0, 720.0))
                .with_title("Syggrel-MRS")
        ),
    );
}

fn app() -> Element {
    rsx! {
        div { class: "app-container",
            NavigationMenu {}
            MainContent {}
        }
    }
}

#[component]
fn NavigationMenu() -> Element {
    rsx! {
        nav { class: "navigation-menu",
            ul { class: "nav-list",
                li { class: "nav-item", "Home" }
                li { class: "nav-item", "Analysis" }
                li { class: "nav-item", "Data" }
                li { class: "nav-item", "Settings" }
            }
        }
    }
}
