//! Core mathematical algorithms for Syggrel-MRS gRNA safety assessment.
//! 
//! This module provides the main computational engines that transform genomic targets 
//! into multi-dimensional risk metrics using topological, information-theoretic, and 
//! chaos-theoretic approaches.

pub mod topology;
pub mod chaos_metrics; 
pub mod info_theory;
pub mod chromatic_risk;
pub mod data_integration;

// Re-export the main computation functions for convenient access
pub use topology::compute_topological_risk;
pub use chaos_metrics::compute_chaos_metrics;
pub use info_theory::compute_information_risk;
pub use chromatic_risk::compute_chromatic_risk;
pub use data_integration::load_contact_matrix;

/// Computes a complete gRNARiskAssessment for a given genomic target
/// by orchestrating all mathematical dimensions.
pub fn assess_grna_safety(target: &crate::types::GenomicTarget) -> crate::types::GRNARiskAssessment {
    let topo_risk = compute_topological_risk(target);
    let info_risk = compute_information_risk(target);
    let chromatic_risk = compute_chromatic_risk(target);
    let chaos_metrics = compute_chaos_metrics(target);
    let chaos_risk = chaos_metrics.lyapunov_estimate;    // or some combined chaos metric

    // TODO: Implement proper weighting and confidence calculation
    crate::types::GRNARiskAssessment {
        topological_risk: topo_risk,
        information_risk: info_risk,
        chromatic_risk: chromatic_risk,
        chaos_risk: chaos_risk,
        overall_safety_score: 0.0,    // Placeholder
        confidence: 0.0,    // Placeholder
    }
}