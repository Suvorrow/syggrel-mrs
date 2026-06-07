//! Reusable helpers for parser configuration and logging.
//! 
//! Format-agnostic, Dioxus-agnostic, and testable.
use std::path::Path;
use ndarray::Array2;

#[inline]
#[must_use]
pub fn parse_resolution(s: &str, default: u32) -> u32 {
    s.parse().unwrap_or(default)
}

/// Parse optional chromosome name (empty string -->> None).
#[inline]
#[must_use]
pub fn parse_chromosome(s: Option<&str>) -> Option<String> {
    s.filter(|v| !v.is_empty()).map(|v| v.to_string())
}

/// Parse bin range from two optional strings. Returns None if invalid or start >= end.
#[inline]
#[must_use]
pub fn parse_bin_range(start: Option<&str>, end: Option<&str>) -> Option<(usize, usize)> {
    match (start, end) {
        (Some(s), Some(e)) => {
            s.parse::<usize>().ok().and_then(|start| {
                e.parse::<usize>().ok().and_then(|end| {
                    if start < end { Some((start, end)) } else { None }
                })
            })
        }
        _ => None,
    }
}

/// Parse normalization method (treat "NONE" or empty as None).
#[inline]
#[must_use]
pub fn parse_normalization(s: Option<&str>) -> Option<String> {
    s.filter(|v| *v != "NONE" && !v.is_empty())
        .map(|v| v.to_string())
}

//---------------------------------------------------------------------------------------------
// Logging helpers
//---------------------------------------------------------------------------------------------
/// Returns a user-friendly log message describing the output path intent.
/// 
/// # Arguments
/// * 'output_path' - Optional path where the matrix will be saved.
/// 
/// # Returns
/// A formatted log message string.
#[inline]
#[must_use]
pub fn format_output_intent(output_path: Option<&std::path::Path>) -> String {
    match output_path {
        Some(path) => format!("💾 Will save matrix to: {}", path.display()),
        None => "⚠️ No output path set. Matrix will be kept in memory only.".to_string(),
    }
}

/// Saves a 2D matrix to disk, auto-detecting format from extension.
#[inline]
pub fn save_matrix_to_path(
    rows: usize,
    cols: usize,
    data: &[f64],
    path: &Path,
) -> Result<(), String> {
    let matrix = Array2::from_shape_vec((rows, cols), data.to_vec())
        .map_err(|e| format!("Matrix shape mismatch: {e}"))?;

    crate::serializer::write_matrix(&matrix, path, None)
        .map_err(|e| format!("Serialization failed: {e}"))
}

#[inline]
#[must_use]
pub fn format_save_confirmation(output_path: Option<&std::path::Path>) -> String {
    match output_path {
        Some(path) => format!("✅ Successfully saved to: {}", path.display()),
        None => "💡 Matrix kept in memory (not saved to disk).".to_string(),
    }
}

/// Generates standardized log messages for matrix extraction results.
#[inline]
#[must_use]
pub fn format_matrix_stats(matrix: &Array2<f64>) -> Vec<String> {
    let rows = matrix.nrows();
    let cols = matrix.ncols();
    let total = rows * cols;

    if total == 0 {
        return vec!["📐 Extracted empty matrix (0 × 0)".to_string()];
    }

    let non_zero = matrix.iter().filter(|&&v| v > 0.0).count();
    let density = (non_zero as f64 / total as f64 * 100.0).max(0.0).min(100.0);

    vec![
        format!("📐 Dimensions: {} × {} ({} total entries)", rows, cols, total),
        format!("🔢 Non-zero entries: {}", non_zero),
        format!("📊 Data density: {:.2}% | Sparsity: {:.2}%", density, 100.0 - density),
    ]
}