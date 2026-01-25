use ndarray::Array2;
use crate::types::GenomicTarget;

pub fn load_contact_matrix(target: &GenomicTarget) -> Array2<f64> {
    // MVP: synthetic data
    // Production: load Hi-C around target coordinates
    Array2::from_shape_fn((100, 100), |(i, j)| {
        let distance = (i as f64 - j as f64).abs();
        1.0 / (distance + 1.0).powf(0.5)
    })
}
