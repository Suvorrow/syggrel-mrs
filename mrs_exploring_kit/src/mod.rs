//! Core mathematical algorithms for Syggrel-MRS gRNA safety assessment.
//! 
//! This module provides the main computational engines that transform genomic targets 
//! into multi-dimensional risk metrics using topological, information-theoretic, and 
//! chaos-theoretic approaches.
#[allow(unused_imports)]
use crate::types::{GenomicTarget, GRNARiskAssessment};

// ---- Module Declarations ----
pub mod topology;
pub mod chaos_metrics; 
pub mod info_theory;
pub mod chromatic_risk;
pub mod data_integration;
pub mod combine_analysis;
pub mod parser;

// ---- Public API Re-exports ----
pub use topology::compute_topological_risk;
pub use chaos_metrics::compute_chaos_metrics;
pub use info_theory::compute_information_risk;
pub use chromatic_risk::compute_chromatic_risk;
pub use data_integration::load_contact_matrix;
pub use combine_analysis::assess_grna_safety;
