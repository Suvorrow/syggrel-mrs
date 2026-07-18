// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Syggrel-MRS: Topologal-Informational Validator for Genomic Instability
//!
//! Core mathematical framework for assessing gRNA safety through
//! topological and information-theoretic, and chaos-theoretic biomarkers.
pub mod core;
pub mod errors;
pub mod types;

use crate::core::routes::Route;
use dioxus::prelude::*;
use dioxus_router::Router;

// Define the root router
#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

// Re-export the algorithms through mrs_algorithms
pub use errors::*;
pub use types::*;
