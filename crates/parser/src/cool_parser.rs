// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

//! COOL format parser for genomic contact matrices.
//!
//! Implements extraction of contact matrices from .cool and .mcool files (HDF5-based).

use std::borrow::Cow;
use std::path::Path;

use hdf5::File;
use hdf5::types::FixedAscii;
use itertools::izip;
use ndarray::{Array1, Array2, Ix1, s};
use thiserror::Error;

use serializer::write_matrix;

//---------------------------------------------------------------------------------------------
// Error types
//---------------------------------------------------------------------------------------------
#[derive(Error, Debug)]
pub enum CoolParserError {
    #[error("HDF5 I/O error: {0}")]
    Hdf5Error(String),

    #[error("Invalid Cooler format: {0}")]
    InvalidFormat(String),

    #[error("Missing required dataset: {0}")]
    MissingDataset(String),

    #[error("Invalid resolution: expected {expected}, found {actual:?}")]
    InvalidResolution { expected: u32, actual: Vec<u32> },

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
    /// Constructs a new 'CoolConfig' with explicitly typed values.
    pub fn new(
        resolution: u32,
        chromosome: Option<String>,
        bin_range: Option<(usize, usize)>,
        normalization: Option<String>,
        group_path: String,
    ) -> Self {
        Self {
            resolution,
            chromosome,
            bin_range,
            normalization,
            group_path,
        }
    }

    /// Parses a 'CoolConfig' from raw string inputs.
    ///
    /// Delegates parsing to 'parser_utils' to ensure consistent default fallbacks
    /// and handling of empty/malformed strings from the frontend.
    pub fn parse_from_strings(
        resolution: &str,
        chromosome: Option<&str>,
        bin_range_start: Option<&str>,
        bin_range_end: Option<&str>,
        normalization: Option<&str>,
        group_path: Option<&str>,
    ) -> Self {
        use crate::parser::parser_utils::{
            parse_bin_range, parse_chromosome, parse_normalization, parse_resolution,
        };

        Self {
            resolution: parse_resolution(resolution, 1_000_000),
            chromosome: parse_chromosome(chromosome),
            bin_range: parse_bin_range(bin_range_start, bin_range_end),
            normalization: parse_normalization(normalization),
            group_path: group_path
                .filter(|s| !s.is_empty())
                .unwrap_or("resolutions")
                .to_owned(),
        }
    }

    /// Returns 'true' if the configuration targets a specific chromosome.
    #[inline]
    pub fn has_chromosome_filter(&self) -> bool {
        self.chromosome.is_some()
    }

    /// Returns 'true' if the configuration targets a specific bin range.
    #[inline]
    pub fn has_bin_range_filter(&self) -> bool {
        self.bin_range.is_some()
    }
}

//---------------------------------------------------------------------------------------------
// Main extraction fnction
//---------------------------------------------------------------------------------------------
/// Extract a contact matrix from a .cool or .mcool file.
///
/// # Notes
/// - For large matrices, consider streaming or sparse output.
/// - Normalization vectors are applied if specified and available.
pub fn extract_contact_matrix(
    input_path: &str,
    config: &CoolConfig,
    output_path: Option<&Path>,
) -> Result<Array2<f64>> {
    let file = File::open(input_path).map_err(|e| CoolParserError::Hdf5Error(e.to_string()))?;

    let res_group_path = if file
        .group(&format!("resolutions/{}", config.resolution))
        .is_ok()
    {
        // Standard .mcool path: data is under /resolutions/{res}
        format!("resolutions/{}", config.resolution)
    } else if file.group("bins").is_ok() {
        // Standard .cool path: data is at the root /
        String::new()
    } else if !config.group_path.is_empty() && file.group(&config.group_path).is_ok() {
        // Fallback: custom group path if it exists
        config.group_path.clone()
    } else {
        return Err(CoolParserError::ExtractionFailed(
            "Invalid file structure. Could not find /resolutions/{res} or /bins. \
             Ensure this is a valid .cool or .mcool file."
                .into(),
        ));
    };

    let bins = load_bins(&file, &res_group_path)?;
    let (start_idx, end_idx) = resolve_target_region(&file, &res_group_path, &bins, config)?;
    let size = end_idx - start_idx;

    if size == 0 {
        return Ok(Array2::zeros((0, 0)));
    }

    let pixels = load_pixels_optimized(&file, &res_group_path, start_idx, end_idx)?;

    let weights = if let Some(ref norm) = config.normalization {
        load_normalization_weights(&file, &res_group_path, norm, start_idx, end_idx)?
    } else {
        None
    };

    let matrix = sparse_to_dense_symmetric(&pixels, &weights, size)?;

    if let Some(out_path) = output_path {
        write_matrix(&matrix, out_path, None)
            .map_err(|e| CoolParserError::ExtractionFailed(e.to_string()))?;
    }

    Ok(matrix)
}

//---------------------------------------------------------------------------------------------
// Validation & Resolution Handling
//---------------------------------------------------------------------------------------------
pub fn list_resolutions(file: &File) -> Result<Vec<u32>> {
    let common_resolutions = [
        500, 1000, 2000, 5000, 10000, 25000, 50000, 100000, 250000, 500000, 1000000, 2500000,
        5000000, 10000000,
    ];

    let mut found = Vec::new();
    for &res in &common_resolutions {
        let res_path = format!("resolutions/{}", res);
        if file.group(&res_path).is_ok() {
            found.push(res);
        }
    }
    Ok(found)
}

//---------------------------------------------------------------------------------------------
// Bin & Region Resolution
//---------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
struct BinInfo {
    chrom_id: u32,
    chrom_name: String,
    #[allow(dead_code)]
    start: u64,
    #[allow(dead_code)]
    end: u64,
}

fn load_bins(file: &File, res_group_path: &str) -> Result<Vec<BinInfo>> {
    let bins_base = if res_group_path == "/" {
        "bins".to_owned()
    } else {
        format!("{}/bins", res_group_path)
    };

    // Read chromosome names
    let chrom_names: Vec<String> = file
        .dataset("chroms/name")
        .map_err(|_| CoolParserError::MissingDataset("/chroms/name".into()))?
        .read::<FixedAscii<64>, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read /chroms/name: {}", e)))?
        .iter()
        .map(|s| s.as_str().to_owned()) // Rust 2024 idiom: prefer to_owned() for &str -> String
        .collect();

    // Read bin columns
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

    let mut bins = Vec::with_capacity(chrom_ids.len());

    // Destructure directly from izip without unused enumerate index
    for (&cid, &s, &e) in izip!(&chrom_ids, &starts, &ends) {
        let name = chrom_names
            .get(cid as usize)
            .cloned()
            .unwrap_or_else(|| format!("chr{}", cid));
        bins.push(BinInfo {
            chrom_id: cid,
            chrom_name: name,
            start: s,
            end: e,
        });
    }

    Ok(bins)
}

fn resolve_target_region(
    file: &File,
    res_group_path: &str,
    bins: &[BinInfo],
    config: &CoolConfig,
) -> Result<(usize, usize)> {
    // If user provided explicit bin range, use it
    if let Some((s, e)) = config.bin_range {
        return Ok((s, e));
    }

    // Otherwise, find the chromosome
    let target_chrom_id = match &config.chromosome {
        Some(name) => {
            // 🚀 OPTIMIZATION: Pre-allocate alternative names to avoid O(N) allocations in the find loop
            let name1 = name.as_str();
            let name2 = if name1.starts_with("chr") {
                Cow::Borrowed(&name1[3..])
            } else {
                Cow::Owned(format!("chr{}", name1))
            };

            let bin = bins
                .iter()
                .find(|b| b.chrom_name == name1 || b.chrom_name == name2.as_ref())
                .ok_or_else(|| {
                    let available: Vec<&str> =
                        bins.iter().map(|b| b.chrom_name.as_str()).take(5).collect();
                    CoolParserError::ChromosomeNotFound(format!(
                        "{} (File contains: {:?}...)",
                        name, available
                    ))
                })?;
            bin.chrom_id
        }
        None => {
            // Whole genome: from 0 to n_bins
            return Ok((0, bins.len()));
        }
    };

    // 🚀 OPTIMIZATION: Use /indexes/chrom_offset if available for O(1) lookup
    let indexes_base = if res_group_path == "/" {
        "indexes".to_owned()
    } else {
        format!("{}/indexes", res_group_path)
    };

    if let Ok(ds) = file.dataset(&format!("{}/chrom_offset", indexes_base)) {
        let offsets: Vec<u64> = ds
            .read::<u64, Ix1>()
            .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read chrom_offset: {}", e)))?
            .to_vec();

        let start_idx = offsets.get(target_chrom_id as usize).copied().unwrap_or(0) as usize;
        let end_idx = offsets
            .get(target_chrom_id as usize + 1)
            .copied()
            .unwrap_or(bins.len() as u64) as usize;
        return Ok((start_idx, end_idx));
    }

    // Fallback: Linear scan (O(N)) - ZERO ALLOCATION
    // Cooler bins are strictly sorted by chrom_id, so we can find the contiguous block
    let start_idx = bins
        .iter()
        .position(|b| b.chrom_id == target_chrom_id)
        .unwrap_or(0);
    let end_idx = bins[start_idx..]
        .iter()
        .position(|b| b.chrom_id != target_chrom_id)
        .map(|p| start_idx + p)
        .unwrap_or(bins.len());

    Ok((start_idx, end_idx))
}

//---------------------------------------------------------------------------------------------
// Pixel Loading (Sparse Data)
//---------------------------------------------------------------------------------------------

fn load_pixels_optimized(
    file: &File,
    res_group_path: &str,
    start_idx: usize,
    end_idx: usize,
) -> Result<Vec<(usize, usize, f64)>> {
    let pixels_base = if res_group_path == "/" {
        "pixels".to_owned()
    } else {
        format!("{}/pixels", res_group_path)
    };
    let indexes_base = if res_group_path == "/" {
        "indexes".to_owned()
    } else {
        format!("{}/indexes", res_group_path)
    };

    if let Ok(ds) = file.dataset(&format!("{}/bin1_offset", indexes_base)) {
        let offsets: Vec<u64> = ds
            .read::<u64, Ix1>()
            .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read bin1_offset: {}", e)))?
            .to_vec();

        if offsets.len() > end_idx {
            let pixel_start = offsets[start_idx] as usize;
            let pixel_end = offsets[end_idx] as usize;

            return read_pixel_slice(
                file,
                &pixels_base,
                pixel_start,
                pixel_end,
                start_idx,
                end_idx,
            );
        }
    }

    read_and_filter_pixels(file, &pixels_base, start_idx, end_idx)
}

fn read_pixel_slice(
    file: &File,
    pixels_base: &str,
    pixel_start: usize,
    pixel_end: usize,
    bin_offset: usize,
    bin_end: usize,
) -> Result<Vec<(usize, usize, f64)>> {
    if pixel_start >= pixel_end {
        return Ok(Vec::new());
    }

    let bin1_ds = file
        .dataset(&format!("{}/bin1_id", pixels_base))
        .map_err(|e| CoolParserError::Hdf5Error(e.to_string()))?;
    let bin2_ds = file
        .dataset(&format!("{}/bin2_id", pixels_base))
        .map_err(|e| CoolParserError::Hdf5Error(e.to_string()))?;
    let count_ds = file
        .dataset(&format!("{}/count", pixels_base))
        .map_err(|e| CoolParserError::Hdf5Error(e.to_string()))?;

    let bin1_raw: Array1<u64> = bin1_ds
        .read_slice(s![pixel_start..pixel_end])
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read bin1_id slice: {}", e)))?;
    let bin2_raw: Array1<u64> = bin2_ds
        .read_slice(s![pixel_start..pixel_end])
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read bin2_id slice: {}", e)))?;
    let counts_raw: Array1<f64> = count_ds
        .read_slice(s![pixel_start..pixel_end])
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read count slice: {}", e)))?;

    build_pixel_vec(&bin1_raw, &bin2_raw, &counts_raw, bin_offset, bin_end)
}

fn read_and_filter_pixels(
    file: &File,
    pixels_base: &str,
    start_idx: usize,
    end_idx: usize,
) -> Result<Vec<(usize, usize, f64)>> {
    let bin1_raw: Array1<u64> = file
        .dataset(&format!("{}/bin1_id", pixels_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/bin1_id", pixels_base)))?
        .read::<u64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read bin1_id: {}", e)))?;

    let bin2_raw: Array1<u64> = file
        .dataset(&format!("{}/bin2_id", pixels_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/bin2_id", pixels_base)))?
        .read::<u64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read bin2_id: {}", e)))?;

    let counts_raw: Array1<f64> = file
        .dataset(&format!("{}/count", pixels_base))
        .map_err(|_| CoolParserError::MissingDataset(format!("{}/count", pixels_base)))?
        .read::<f64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read count: {}", e)))?;

    build_pixel_vec(&bin1_raw, &bin2_raw, &counts_raw, start_idx, end_idx)
}

fn build_pixel_vec(
    bin1_raw: &Array1<u64>,
    bin2_raw: &Array1<u64>,
    counts_raw: &Array1<f64>,
    bin_offset: usize,
    bin_end: usize,
) -> Result<Vec<(usize, usize, f64)>> {
    let len = bin1_raw.len();
    let capacity_hint = (len as f64 * 0.8) as usize;
    let mut pixels = Vec::with_capacity(capacity_hint.min(5_000_000));

    for (&b1, &b2, &count) in izip!(bin1_raw.iter(), bin2_raw.iter(), counts_raw.iter()) {
        let Ok(i) = usize::try_from(b1) else { continue };
        let Ok(j) = usize::try_from(b2) else { continue };

        if i >= bin_offset
            && i < bin_end
            && j >= bin_offset
            && j < bin_end
            && count.is_finite()
            && count >= 0.0
        {
            pixels.push((i - bin_offset, j - bin_offset, count));
        }
    }

    pixels.shrink_to_fit();
    Ok(pixels)
}

//---------------------------------------------------------------------------------------------
// Normalization Weights
//---------------------------------------------------------------------------------------------

fn load_normalization_weights(
    file: &File,
    res_group_path: &str,
    norm_name: &str,
    start_idx: usize,
    end_idx: usize,
) -> Result<Option<Vec<f64>>> {
    // Cooler stores weights in /bins/{name} or /norm/{name}
    let paths_to_try = [
        format!("{}/bins/{}", res_group_path, norm_name),
        format!("norm/{}", norm_name),
        format!("{}/norm/{}", res_group_path, norm_name),
    ];

    let mut dataset = None;
    for path in &paths_to_try {
        if let Ok(ds) = file.dataset(path) {
            dataset = Some(ds);
            break;
        }
    }

    let dataset = match dataset {
        Some(ds) => ds,
        None => return Ok(None), // Normalization not found, return raw counts
    };

    let weights_raw: Array1<f64> = dataset
        .read::<f64, Ix1>()
        .map_err(|e| CoolParserError::Hdf5Error(format!("Failed to read weights: {}", e)))?;

    let weights = weights_raw.to_vec();

    if end_idx > weights.len() {
        return Err(CoolParserError::ExtractionFailed(format!(
            "Weight slice range [{}, {}) exceeds dataset length ({})",
            start_idx,
            end_idx,
            weights.len()
        )));
    }

    let sliced = &weights[start_idx..end_idx];
    Ok(Some(sliced.to_vec()))
}

//---------------------------------------------------------------------------------------------
// Sparse-to-Dense Conversion + Symmetrization
//---------------------------------------------------------------------------------------------

fn sparse_to_dense_symmetric(
    pixels: &[(usize, usize, f64)],
    weights: &Option<Vec<f64>>,
    size: usize,
) -> Result<Array2<f64>> {
    if size == 0 {
        return Ok(Array2::zeros((0, 0)));
    }

    if let Some(w) = weights {
        if w.len() < size {
            return Err(CoolParserError::ExtractionFailed(format!(
                "Normalization weights length ({}) is less than matrix size ({})",
                w.len(),
                size
            )));
        }
    }

    let mut matrix = Array2::zeros((size, size));

    for &(i, j, val) in pixels {
        if i >= size || j >= size {
            continue; // Defensive bounds check
        }

        // Apply normalization (Cooler uses multiplicative weights)
        let normalized = if let Some(w) = weights {
            let wi = w[i];
            let wj = w[j];

            // Skip if weights are invalid (NaN, 0, or negative)
            if !wi.is_finite() || !wj.is_finite() || wi <= 0.0 || wj <= 0.0 {
                continue;
            }
            val * wi * wj
        } else {
            val
        };

        if normalized.is_finite() && normalized >= 0.0 {
            matrix[[i, j]] = normalized;
            matrix[[j, i]] = normalized; // ✅ Symmetrize (Cooler stores upper triangle)
        }
    }

    Ok(matrix)
}
