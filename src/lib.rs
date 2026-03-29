//! Syggrel-MRS: Topologal-Informational Validator for Genomic Instability
//! 
//! Core mathematical framework for assessing gRNA safety through
//! topological and information-theoretic, and chaos-theoretic biomarkers.

pub mod core;
pub mod types;
pub mod errors;

// Re-export the algorithms through mrs_algorithms
pub use core::routes::App;
pub use types::*;
pub use errors::*;
