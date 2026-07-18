// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only
//! Core mathematical algorithms for Syggrel-MRS gRNA safety assessment.
//!
//! This module provides the main computational engines that transform genomic targets
//! into multi-dimensional risk metrics using topological, information-theoretic, and
//! chaos-theoretic approaches.
#[allow(unused_imports)]
use crate::types::{GRNARiskAssessment, GenomicTarget};

// ---- Module Declarations ----
pub mod chaos_metrics;
pub mod chromatic_risk;
pub mod combine_analysis;
pub mod data_integration;
pub mod info_theory;
pub mod topology;

// ---- Public API Re-exports ----
pub use chaos_metrics::compute_chaos_metrics;
pub use chromatic_risk::compute_chromatic_risk;
pub use combine_analysis::assess_grna_safety;
pub use data_integration::load_contact_matrix;
pub use info_theory::compute_information_risk;
pub use topology::compute_topological_risk;
