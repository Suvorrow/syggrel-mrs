use ndarray::Array2;
use std::process::Command;
use std::path::Path;

// Use type aliases for domain-specific clarity
type ContactFrequency = f64;
type GenomicBinIndex = usize;
type ContactMatrix = Array2<ContactFrequency>;

/// Represents the size of a square Hi-C contact matrix in terms of genomic bins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MatrixSize(GenomicBinIndex);

impl MatrixSize {
    fn new(bins: GenomicBinIndex) -> Result<Self, &'static str> {
        if bins == 0 {
            Err("Matrix size must be greater than zero")
        } else {
            Ok(MatrixSize(bins))
        }
    }

    fn value(&self) -> GenomicBinIndex {
        self.0
    }

    /// Creates a square contact matrix initialized with zeros.
    fn create_zeroed_matrix(&self) -> ContactMatrix {
        ContactMatrix::zeros((self.0, self.0))
    }
}

/// Configuration for Hi-C data extraction
#[derive(Debug, Clone)]
pub struct HiCConfig {
    pub normalization: NormalizationMethod,
    pub resolution: u32,
    pub chromosome: String,
    pub start_bin: Option<GenomicBinIndex>,
    pub end_bin: Option<GenomicBinIndex>,
}

#[derive(Debug, Clone)]
pub enum NormalizationMethod {
    NONE,
    VC,
    VCSQRT,
    KR,
}

impl std::fmt::Display for NormalizationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NormalizationMethod::NONE => write!(f, "NONE"),
            NormalizationMethod::VC => write!(f, "VC"),
            NormalizationMethod::VCSQRT => write!(f, "VC_SQRT"),
            NormalizationMethod::KR => write!(f, "KR"),
        }
    }
}

/// Extracts Hi-C contact matrix from .hic file using hictk
pub fn extract_contact_matrix<P: AsRef<Path>>(
    hic_file_path: P,
    config: &HiCConfig,
) -> Result<ContactMatrix, Box<dyn std::error::Error>> {
    let hic_path = hic_file_path.as_ref();

    // Validate input file exists
    if !hic_path.exists() {
        return Err(format!("Hi-C file not found: {}", hic_path.display()).into());
    }

    // Build hictk command
    let output = if let (Some(start), Some(end)) = (config.start_bin, config.end_bin) {
        // Extract specific region
        Command::new("hictk")
            .arg("dump")
            .arg("--resolution")
            .arg(config.resolution.to_string())
            .arg("--normalization")
            .arg(config.normalization.to_string())
            .arg("--region")
            .arg(format!("{}:{}-{}", config.chromosome, start, end))
            .arg("--region2")
            .arg(format!("{}:{}-{}", config.chromosome, start, end))
            .arg("--out-format")
            .arg("coo")    // Coordinate list format
            .arg(hic_path)
            .output()?
    } else {
        // Extract whole chromosome
        Command::new("hictk")
            .arg("dump")
            .arg("--resolution")
            .arg(config.resolution.to_string())
            .arg("--normalization")
            .arg(config.normalization.to_string())
            .arg("--region")
            .arg(&config.chromosome)
            .arg("--region2")
            .arg(&config.chromosome)
            .arg("--out-format")
            .arg("coo")
            .arg(hic_path)
            .output()?
    };

    if !output.status.success() {
        return Err(format!(
            "hictk command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    // Parse the output into a matrix
    parse_coo_output(&output.stdout, config)
}

/// Parses coordinate list output from hictk into a dense matrix
fn parse_coo_output(
    coo_data: &[u8],
    config: &HiCConfig,
) -> Result<ContactMatrix, Box<dyn std::error::Error>> {
    let content = String::from_utf8(coo_data.to_vec())?;

    // Determine the actual maximum bin index from the data
    let mut max_bin_index = 0;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Ok(bin1_str), Ok(bin2_str)) = (
                parts[0].parse::<GenomicBinIndex>(),
                parts[1].parse::<GenomicBinIndex>(),
            ) {
                max_bin_index = max_bin_index.max(bin1_str).max(bin2_str);
            }
        }
    }

    // Calculate matrix size based on actual data or region config
    let matrix_size = MatrixSize::new(if max_bin_index > 0 {
        max_bin_index + 1    // +1 because indices are 0-based
    } else {
        match (config.start_bin, config.end_bin) {
            (Some(start), Some(end)) if end >= start => end - start + 1,
            _ => 1000,    // Default fallback
        }
    })?;

    let mut contact_matrix = matrix_size.create_zeroed_matrix();

    // Parse COO format: bin1\tbin2\tcount\n
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Ok(bin1_str), Ok(bin2_str), Ok(count_str)) = (
                parts[0].parse::<GenomicBinIndex>(),
                parts[1].parse::<GenomicBinIndex>(),
                parts[2].parse::<ContactFrequency>(),
            ) {
                // Only fill matrix if within bounds
                if bin1_str < matrix_size.value() && bin2_str < matrix_size.value() {
                    contact_matrix[[bin1_str, bin2_str]] = count_str;
                    contact_matrix[[bin2_str, bin1_str]] = count_str;    // Ensure symmetry
                }
            }
        }
    }

    Ok(contact_matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_display() {
        let config = HiCConfig {
            normalization: NormalizationMethod::KR,
            resolution: 10000,
            chromosome: "chr1".to_string(),
            start_bin: Some(0),
            end_bin: Some(100),
        };
        assert_eq!(config.normalization.to_string(), "KR");
    }

    #[test]
    fn test_matrix_size_validation() {
        assert!(MatrixSize::new(0).is_err());
        assert!(MatrixSize::new(1).is_ok());
    }
}