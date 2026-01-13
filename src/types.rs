use serde::{Deserialize, Serialize};

/// Genomic target for gRNA safety assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomicTarget {
    pub chromosome: String,
    pub position: u64,
    pub sequence: String,
}

/// Raw computedchaos theory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosMetrics {
    pub lyapunov_estimate: f64,
    pub feigenbaum_ratio: Option<f64>,
    pub sample_entropy: f64,
}

/// Multi-dimensional gRNA safety assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GRNARiskAssessment {
    pub topological_risk: f64,    // Persistent homology stability
    pub information_risk: f64,    // Regulatory coupling strength (mutual information)
    pub chromatic_risk: f64,    // Chromatin planarity violation
    pub chaos_risk: f64,    // Expression dynamics instability (Lyapunov exponent)
    pub overall_safety_score: f64,    // Weighted composite safety score
    pub confidence: f64,    // Algorithmic certainity in assessment
}