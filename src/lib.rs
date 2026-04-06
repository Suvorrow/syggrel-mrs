//! Syggrel-MRS: Topologal-Informational Validator for Genomic Instability
//! 
//! Core mathematical framework for assessing gRNA safety through
//! topological and information-theoretic, and chaos-theoretic biomarkers.
pub mod core;
pub mod types;
pub mod errors;

use dioxus::prelude::*;
use dioxus_router::Router;
use crate::core::routes::Route;

// Define the root router
#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

// Re-export the algorithms through mrs_algorithms
pub use types::*;
pub use errors::*;

