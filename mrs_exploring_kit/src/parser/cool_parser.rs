//! COOL format parser for genomic contact matrices.
//! 
//! Implements extraction of contact matrices from .cool files (HDF5-based).

use std::path::Path;

use hdf5::File;
use hdf5::types::FixedAscii;
use itertools::izip;
use thiserror::Error;
use ndarray::{Array1, Array2, Ix1};

use crate::serializer::write_matrix;

//---------------------------------------------------------------------------------------------
// Error types
//---------------------------------------------------------------------------------------------
#[derive(Error, Debug)]
pub enum CoolParserError {
    #[error("HDF5 I/O error: {0}")]
    Hdf5Error(String),

    #[error("Missing required dataset: {0}")]
    MissingDataset(String),

    #[error("Invalid resolution: expected {expected}, found {actual}")]
    InvalidResolution { expected: u32, actual: u32 },

    #[error("Chromosome not found: {0}")]
    ChromosomeNotFound(String),

    #[error("Matrix extraction failed: {0}")]
    ExtractionFailed(String),
}

pub type Result<T> = std::result::Result<T, CoolParserError>;

/// Configuration for COOL matrix extraction.
#[derive(Clone, Debug, Default)]
pub struct CoolConfig {
    /// Resolution bin size in base pairs (must match a resolution in the file).
    pub resolution: u32,
    /// Chromosome name (e.g. "chr1", "1"). If None, extract whole-genome.
    pub chromosome: Option<String>,
    /// Bin range (inclusive start, exclusive end). None = full chromosome.
    pub bin_range: Option<(usize, usize)>,
    /// Normalization method name (e.g., "KR", "VC"). None = raw counts.
    pub normalization: Option<String>,
    /// HDF5 group path inside the .cool file (default: "resolutions").
    pub group_path: String,
}

impl CoolConfig {
    pub fn parse_from_strings(
        resolution: &str,
        chromosome: Option<&str>,
        bin_range_start: Option<&str>,
        bin_range_end: Option<&str>,
        normalization: Option<&str>,
        group_path: Option<&str>,
    ) -> Self {
        use crate::parser::parser_utils::{
            parse_resolution, parse_chromosome, parse_bin_range,
            parse_normalization,
        };

        Self {
            resolution: parse_resolution(resolution, 1_000_000),
            chromosome: parse_chromosome(chromosome),
            bin_range: parse_bin_range(bin_range_start, bin_range_end),
            normalization: parse_normalization(normalization),
            group_path: group_path
                .filter(|s| !s.is_empty())
                .unwrap_or("resolutions")
                .to_string(),
        }
    }
}

//---------------------------------------------------------------------------------------------
// Main extraction fnction
//---------------------------------------------------------------------------------------------
/// Extract a contact matrix from a .cool file.
/// 
/// # Arguments
/// * 'input_path' - Path to the .cool file.
/// * 'config' - Extraction configuration.
/// * 'output_path' - Optional path to write extracted matrix (NPZ/CSV).
/// 
/// # Returns
/// * 'Ok(ndarray::Array2<f64>)' - Extracted contact matrix (dense).
/// * 'Err(CoolParserError)' - Parsing or I/O failure.
/// 
/// # Notes
/// - For large matrices, consider streaming or sparse output.
/// - Normalization vectors are applied if specified and available.
pub fn extract_contact_matrix(
    input_path: &str,
    config: &CoolConfig,
    output_path: Option<&Path>,
) -> Result<Array2<f64>> {
    // Open HDF5 file
    let file = File::open(input_path)
        .map_err(|e| CoolParserError::Hdf5Error(e.to_string()))?;
    
    // Validate resolution exists & determine correct group path
    let res_group = if file.group(&format!("{}/{}", config.group_path, config.resolution)).is_ok() {
        format!("{}/{}", config.group_path, config.resolution)
    } else if file.group(&config.resolution.to_string()).is_ok() {
        // Fallback 1: Single-res file with numeric folder (e.g., "1000000/")
        config.resolution.to_string()
    } else if file.group("/bins").is_ok() {
        // Fallback 2: Standard single-res .cool (data at root)
        "/".to_string()
    } else {
        // Fallback 3: List available to give a helpful error
        let available = list_resolutions(&file, &config.group_path)?;
        return if available.is_empty() {
            Err(CoolParserError::ExtractionFailed(
                "No resolutions found. Ensure the file is valid .cool/.mcool file.".into()
            ))
        } else {
            Err(CoolParserError::InvalidResolution {
                expected: config.resolution,
                actual: *available.first().unwrap(),
            })
        };
    };

    // Load chromosome/bin metadata
    let bins = load_bins(&file, &res_group)?;
    let chrom_id = match &config.chromosome {
        Some(name) => {
            // Robust matching: automatically handles "chr1" vs "1" naming conventions
            let find_id = bins.iter().position(|b| {
                let b_name = b.chrom_name.as_str();
                b_name == name ||
                (name.starts_with("chr") && b_name == &name[3..]) ||
                (!name.starts_with("chr") && b_name == &format!("chr{}", name))
            });

            find_id.ok_or_else(|| {
                // Show available chromosomes to help debug mismatched names
                let available: Vec<&str> = bins.iter()
                    .map(|b| b.chrom_name.as_str())
                    .take(5)
                    .collect();
                CoolParserError::ChromosomeNotFound(format!(
                    "{} (File contains: {}...)",
                    name,
                    available.join(", ")
                ))
            })?
        }
        None => 0,    // COOL spec: bins sorted by (chrom_id, start); 0 = first
    };

    // Determine bin indices for extraction
    let (start_idx, end_idx) = match config.bin_range {
        Some((s, e)) => (s, e),
        None => {
            let chrom_bins: Vec<_> = bins.iter()
                .enumerate()
                .filter(|(_, b)| b.chrom_id == chrom_id)
                .map(|(i, _)| i)
                .collect();
            (*chrom_bins.first().unwrap_or(&0),
            chrom_bins.last().map(|i| i + 1).unwrap_or(0))
        }
    };


    // Load sparse matrix data (pixels)
    let pixels = load_pixels(&file, &res_group, chrom_id, start_idx, end_idx)?;

    // Apply normalization if requested
    let weights = if let Some(ref norm) = config.normalization {
        load_normalization_weights(&file, &res_group, norm, start_idx, end_idx)?
    } else {
        None
    };

    // Convert sparse - dense matrix
    let matrix = sparse_to_dense(&pixels, &weights, end_idx - start_idx)?;

    // Optional: write to output file
    if let Some(out_path) = output_path {
        write_matrix(&matrix, out_path, None)
            .map_err(|e| CoolParserError::ExtractionFailed(e.to_string()))?;
    }

    Ok(matrix)
}

/// List available resolutions in a .cool file.
pub fn list_resolutions(file: &File, group_path: &str) -> Result<Vec<u32>> {
    
    let common_resolutions = [
        1000, 2500, 5000, 10000, 25000, 50000,
        100000, 250000, 500000, 1000000, 2500000, 5000000
    ];

    let mut found = Vec::new();

    for &res in &common_resolutions {
        let res_path = format!("{}/{}", group_path, res);
        // Try to open the group - if it succeeds, this resolution exists
        if file.group(&res_path).is_ok() {
            found.push(res);
        }
    }

    Ok(found)
}

/*
Alternative: Use Lower-Level HDF5 API (If You Really Need Dynamic Listing)
If absolutely need to discover any resolution dynamically, you can drop to the hdf5_sys FFI layer:

// ⚠️ Advanced: Only use if you truly need dynamic discovery
use hdf5_sys::h5g;
use std::ffi::CString;

pub fn list_resolutions_dynamic(file: &File, group_path: &str) -> Result<Vec<u32>> {
    let c_path = CString::new(group_path).map_err(|_| {
        CoolParserError::ExtractionFailed("Invalid group path".into())
    })?;
    
    let group_id = file.id(); // Get underlying HDF5 hid_t
    
    // Use H5Giterate to walk members (complex, unsafe)
    // ... implementation omitted for brevity ...
    
    // For now, prefer the simple "try known values" approach above
}

Stick with the "try known values" approach unless you have a specific file that requires dynamic discovery.
 */

// --------------------------------------------------------------------------------------------
// Internal helpers (private to this module)
// --------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct BinInfo {
    chrom_id: usize,
    chrom_name: String,
    #[allow(dead_code)]
    start: u64,
    #[allow(dead_code)]
    end: u64,
}

fn load_bins(file: &File, res_group: &str) -> Result<Vec<BinInfo>> {
    //-----------------------------------------------------------------------------------------
    // Step 1: Load chromosome names from /chroms/name
    //-----------------------------------------------------------------------------------------
    let chrom_names: Vec<String> = file
        .dataset("/chroms/name")
        .map_err(|_| CoolParserError::MissingDataset("/chroms/name".into()))?
        .read::<FixedAscii<32>, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read /chroms/name: {}", e)))?
        .iter()
        .map(|s| s.as_str().to_string())
        .collect();

    //-----------------------------------------------------------------------------------------
    // Step 2: Read bin columns directly by full path
    //-----------------------------------------------------------------------------------------
    let bins_base = format!("{}/bins", res_group);

    let chrom_ids: Vec<u32> = file
        .dataset(&format!("{}/chrom", bins_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/chrom", bins_base)))?
        .read::<u32, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read bins/chrom: {}", e)))?
        .to_vec();

    let starts: Vec<u64> = file
        .dataset(&format!("{}/start", bins_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/start", bins_base)))?
        .read::<u64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read bins/start: {}", e)))?
        .to_vec();

    let ends: Vec<u64> = file
        .dataset(&format!("{}/end", bins_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/end", bins_base)))?
        .read::<u64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read bins/end: {}", e)))?
        .to_vec();

    if chrom_ids.len() != starts.len() || chrom_ids.len() != ends.len() {
        return Err(CoolParserError::ExtractionFailed(
            "Bin table columns have mismatched lengths".into(),
        ));
    }

    //-----------------------------------------------------------------------------------------
    // Step 3: Build Vec<BinInfo>
    //-----------------------------------------------------------------------------------------
    let mut bins = Vec::with_capacity(chrom_ids.len());
    
    for (idx, (&chrom_id, &start, &end)) in izip!(chrom_ids.iter(), starts.iter(), ends.iter()).enumerate() {
        let chrom_name = chrom_names
            .get(chrom_id as usize)
            .cloned()
            .unwrap_or_else(|| format!("chr{}", chrom_id));

        bins.push(BinInfo {
            chrom_id: idx,
            chrom_name,
            start,
            end,
        });
    }

    Ok(bins)
}

fn load_pixels(
    file: &File,
    res_group: &str,
    _chrom_id: usize,
    start_idx: usize,
    end_idx: usize,
) -> Result<Vec<(usize, usize, f64)>> {
    //-----------------------------------------------------------------------------------------
    // Step 1: Open the pixels dataset
    //-----------------------------------------------------------------------------------------
    let pixels_base = format!("{}/pixels", res_group);

    let bin1_raw: Array1<u64> = file
        .dataset(&format!("{}/bin1_id", pixels_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/bin1_id", pixels_base)))?
        .read::<u64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read pixels/bin1_id: {}", e)))?;

    let bin2_raw: Array1<u64> = file
        .dataset(&format!("{}/bin2_id", pixels_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/bin2_id", pixels_base)))?
        .read::<u64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read pixels/bin2_id: {}", e)))?;

    let counts_raw: Array1<f64> = file
        .dataset(&format!("{}/count", pixels_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/count", pixels_base)))?
        .read::<f64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read pixels/count: {}", e)))?;

    // Validate column length match
    let n = bin1_raw.len();
    if bin2_raw.len() != n || counts_raw.len() != n {
        return Err(CoolParserError::ExtractionFailed(
            "Pixels table columns have mismatched lengths".into(),
        ));
    }

    // Convert ndarray -->> Vec
    let bin1 = bin1_raw.to_vec();
    let bin2 = bin2_raw.to_vec();
    let counts = counts_raw.to_vec();

    // Filter & transform in single pass
    let capacity_hint = (n as f64 * 0.1) as usize;
    let mut pixels = Vec::with_capacity(capacity_hint.min(1_000_000));

    // Flat destructuring for izip!
    for (&b1, &b2, &count) in izip!(bin1.iter(), bin2.iter(), counts.iter()) {
        if let (Ok(i), Ok(j)) = (usize::try_from(b1), usize::try_from(b2)) {
            if (start_idx..end_idx).contains(&i) && (start_idx..end_idx).contains(&j) {
                if count.is_finite() && count >= 0.0 {
                    pixels.push((i - start_idx, j - start_idx, count));
                }
            }
        }
    }

    // Shrink capacity to fit - avoids holding unused memory
    pixels.shrink_to_fit();

    Ok(pixels)
}

fn load_normalization_weights(
    file: &File,
    res_group: &str,
    norm_name: &str,
    start_idx: usize,
    end_idx: usize,
) -> Result<Option<Vec<f64>>> {
    //-----------------------------------------------------------------------------------------
    // Step 1: Check if weights dataset exists
    //-----------------------------------------------------------------------------------------
    let weights_path = format!("{}/weights/{}", res_group, norm_name);

    // dataset_exists() returns Result<bool, Error> - handle both cases
    let dataset = match file.dataset(&weights_path) {
        Ok(ds) => ds,
        Err(_) => return Ok(None),
    };

    //-----------------------------------------------------------------------------------------
    // Step 2: Read the full weights vector as 1D array of f64
    //-----------------------------------------------------------------------------------------
    let weights_raw: Array1<f64> = dataset
        .read::<f64, Ix1>()    // Element type (f64) + dimension type (Ix1)
        .map_err(|e| CoolParserError::Hdf5Error(format!(
            "Failed to read {}: {}", weights_path, e
        )))?;
    
    let weights = weights_raw.to_vec();

    //-----------------------------------------------------------------------------------------
    // Step 3: Validate slice bounds
    //-----------------------------------------------------------------------------------------
    if start_idx > end_idx {
        return Err(CoolParserError::ExtractionFailed(format!(
            "Invalid weight slice range: start ({}) > end ({})", start_idx, end_idx
        )));
    }

    if end_idx > weights.len() {
        return Err(CoolParserError::ExtractionFailed(format!(
            "Weight slice range [{}, {}) exceeds dataset length ({})",
            start_idx, end_idx, weights.len()
        )));
    }

    //-----------------------------------------------------------------------------------------
    // Step 4: Extract and validate the requested slice
    //-----------------------------------------------------------------------------------------
    let sliced = &weights[start_idx..end_idx];

    // Validate all weights are finite and positive (required for normalization)
    for (idx, &w) in sliced.iter().enumerate() {
        if !w.is_finite() {
            return Err(CoolParserError::ExtractionFailed(format!(
                "Normalization weight at index {} is NaN or Inf", start_idx + idx
            )));
        }
        if w <= 0.0 {
            return Err(CoolParserError::ExtractionFailed(format!(
                "Normalization weight at index {} is non-positive: {}", start_idx + idx, w
            )));
        }
    }

    // Return a cloned slice (Vec<f64>)
    Ok(Some(sliced.to_vec()))
}

fn sparse_to_dense(
    pixels: &[(usize, usize, f64)],
    weights: &Option<Vec<f64>>,
    size: usize,
) -> Result<Array2<f64>> {
    //-----------------------------------------------------------------------------------------
    // Edge case: empty matrix
    //-----------------------------------------------------------------------------------------
    if size == 0 {
        return Ok(Array2::zeros((0, 0)));
    }

    //-----------------------------------------------------------------------------------------
    // Pre-validate weights length (fail fast, not per-pixel)
    //-----------------------------------------------------------------------------------------
    if let Some(w) = weights {
        if w.len() < size {
            return Err(CoolParserError::ExtractionFailed(format!(
                "Normalization weights length ({}) is less than matrix size ({})",
                w.len(), size
            )));
        }
    }

    //-----------------------------------------------------------------------------------------
    // Allocate output matrix (zero-initialized)
    //-----------------------------------------------------------------------------------------
    let mut matrix = Array2::zeros((size, size));

    //-----------------------------------------------------------------------------------------
    // Process each sparse pixel entry
    //-----------------------------------------------------------------------------------------
    for &(i, j, val) in pixels {
        // Skip entries outside the requested matrix bounds (defensive)
        if i >= size || j >= size {
            continue;
        }

        // Apply normalization weights if provided
        let normalized = if let Some(w) = weights {
            let divisor = w[i] * w[j];

            // Guard against invalid normalization factors
            if divisor.is_finite() && divisor > 0.0 {
                val / divisor
            } else {
                // Skip this entry - don't corrupt the matrix with NaN/Inf
                continue;
            }
        } else {
            val
        };

        // Only write valid, non-negative values (Hi-C counts are non-negative)
        if normalized.is_finite() && normalized >= 0.0 {
            matrix[[i, j]] = normalized;
        }
    }

    Ok(matrix)
}
