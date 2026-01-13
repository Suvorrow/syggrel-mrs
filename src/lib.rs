//! Syggrel-MRS: Topologal-Informational Validator for Genomic Instability
//! 
//! Core mathematical framework for assessing gRNA through
//! topological and information-theoretic, and chaos-theoretic biomarkers.

pub mod types;
pub mod errors;
pub mod mrs_algorithms;

// Re-export the algorithms through mrs_algorithms
pub use mrs_algorithms::*;
pub use types::*;
pub use errors::*;