// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::chaos_metrics::compute_chaos_metrics;
use crate::chromatic_risk::compute_chromatic_risk;
use crate::info_theory::compute_information_risk;
use crate::topology::compute_topological_risk;
use crate::types::{GRNARiskAssessment, GenomicTarget};

// ---- Core Functionality ----
/// Orchestrates the computation of a complete gRNA safety assessment.
///
/// This function integrates risk metrics from multiple mathematical domains
/// to produce a comprehensive safety profile for a given genomic target
///
/// # Arguments
/// * 'target' - The genomic location and sequence to assess.
///
/// # Returns
/// A 'GRNARiskAssessment' containing individual risk components and an overall score.
pub fn assess_grna_safety(target: &GenomicTarget) -> GRNARiskAssessment {
    // Execute computations in parallel using Rayon's scope mechanism
    let (topo_risk, (info_risk, (chromatic_risk, chaos_risk))) = rayon::join(
        || compute_topological_risk(target),
        || {
            rayon::join(
                || compute_information_risk(target),
                || {
                    rayon::join(
                        || compute_chromatic_risk(target),
                        || {
                            let chaos_metrics = compute_chaos_metrics(target);
                            chaos_metrics.lyapunov_estimate
                        },
                    )
                },
            )
        },
    );

    combine_assessments(topo_risk, info_risk, chromatic_risk, chaos_risk)
}

/// Combines individual risk metrics into a final 'GRNARiskAssessment'.
fn combine_assessments(
    topological_risk: f64,
    information_risk: f64,
    chromatic_risk: f64,
    chaos_risk: f64,
) -> GRNARiskAssessment {
    let overall_score = (topological_risk + information_risk + chromatic_risk + chaos_risk) / 4.0;
    let confidence = 0.85; // Model Certainty parameter: Temporary Placeholder

    GRNARiskAssessment {
        topological_risk,
        information_risk,
        chromatic_risk,
        chaos_risk,
        overall_safety_score: overall_score,
        confidence,
    }
}
