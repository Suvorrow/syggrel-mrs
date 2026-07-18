// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Modular matrix serialization layer.
//! 
//! Provides a trait-based API for writing 'ndarray::Array2<f64> matrices
//! to various formats, with easy GUI integration via 'WriterKind'.

use std::path::Path;
use ndarray::Array2;

use crate::serializer::error::{Result, SerializerError};

//---------------------------------------------------------------------------------------------
// Core trait: all writers implement this
//---------------------------------------------------------------------------------------------
pub trait MatrixWriter: Send + Sync {
    /// Write a matrix to the given path.
    fn write(&self, matrix: &Array2<f64>, path: &Path) -> Result<()>;

    /// Get the file extension this writer produses.
    fn extension(&self) -> &'static str;

    /// Human-readable description for UI display.
    fn description(&self) -> &'static str;
}

//---------------------------------------------------------------------------------------------
// Enum for GUI selection
//---------------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WriterKind {
    Csv,
    #[cfg(feature = "serializer-npy")]
    Npy,
    #[cfg(feature = "serializer-hdf5")]
    Hdf5,
    #[cfg(feature = "serializer-bincode")]
    Bincode,
}

impl WriterKind {
    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            WriterKind::Csv => "csv",
            #[cfg(feature = "serializer-npy")]
            WriterKind::Npy => "npy",
            #[cfg(feature = "serializer-hdf5")]
            WriterKind::Hdf5 => "h5",
            #[cfg(feature = "serializer-bincode")]
            WriterKind::Bincode => "bin",
        }
    }

    /// Create a writer instance for this format.
    pub fn create_writer(&self) -> Result<Box<dyn MatrixWriter>> {
        match self {
            WriterKind::Csv => Ok(Box::new(CsvWriter)),
            #[cfg(feature = "serializer-npy")]
            WriterKind::Npy => Ok(Box::new(NpyWriter)),
            #[cfg(feature = "serializer-hdf5")]
            WriterKind::Hdf5 => Ok(Box::new(Hdf5Writer)),
            #[cfg(feature = "serializer-bincode")]
            WriterKind::Bincode => Ok(Box::new(BincodeWriter)), 
        }
    }

    /// Parse from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "csv" => Some(WriterKind::Csv),
            #[cfg(feature = "serializer-npy")]
            "npy" => Some(WriterKind::Npy),
            #[cfg(feature = "serializer-hdf5")]
            "h5" | "hdf5" => Some(WriterKind::Hdf5),
            #[cfg(feature = "serializer-bincode")]
            "bin" | "bincode" => Some(WriterKind::Bincode),
            _ => None,
        }
    }

    /// List all available formats at compile time (for GUI population).
    pub fn available_formats() -> Vec<Self> {
        #[allow(unused_mut)]
        let mut formats = vec![WriterKind::Csv];
        #[cfg(feature = "serializer-npy")]
        formats.push(WriterKind::Npy);
        #[cfg(feature = "serializer-hdf5")]
        formats.push(WriterKind::Hdf5);
        #[cfg(feature = "serializer-bincode")]
        formats.push(WriterKind::Bincode);
        formats
    }
}

//---------------------------------------------------------------------------------------------
// CSV Writer: universal, human-readable
//---------------------------------------------------------------------------------------------
#[cfg(feature = "serializer-csv")]
#[derive(Debug, Default, Clone, Copy)]
pub struct CsvWriter;

#[cfg(feature = "serializer-csv")]
impl MatrixWriter for CsvWriter {
    fn write(&self, matrix: &Array2<f64>, path: &Path) -> Result<()> {
        use csv::WriterBuilder;

        let mut wtr = WriterBuilder::new()
            .has_headers(false)
            .from_path(path)
            .map_err(SerializerError::CsvError)?;

        for row in matrix.rows() {
            // Convert f64 -->> String with full precision
            let str_row: Vec<String> = row.iter().map(|v| format!("{:.17}", v)).collect();
            let str_refs: Vec<&str> = str_row.iter().map(|s| s.as_str()).collect();
            wtr.write_record(str_refs).map_err(SerializerError::CsvError)?;
        }

        wtr.flush().map_err(SerializerError::IoError)?;
        Ok(())
    }

    fn extension(&self) -> &'static str { "csv" }
    fn description(&self) -> &'static str { "CSV (comma-separated, human-readable)" }
}

//---------------------------------------------------------------------------------------------
// NPY Writer: Python/NumPy interoperability
//---------------------------------------------------------------------------------------------
#[cfg(feature = "serializer-npy")]
#[derive(Debug, Default, Clone, Copy)]
pub struct NpyWriter;

#[cfg(feature = "serializer-npy")]
impl MatrixWriter for NpyWriter {
    fn write(&self, matrix: &Array2<f64>, path: &Path) -> Result<()> {
        // ndarray-npy expects &str path, not &Path
        let path_str = path.to_str().ok_or_else(|| {
            SerializerError::NpyError(format!("Invalid UTF-8 in path: {:?}", path))
        })?;

        ndarray_npy::write_npy(path_str, matrix)
            .map_err(|e| SerializerError::NpyError(e.to_string()))
    }

    fn extension(&self) -> &'static str { "npy" }
    fn description(&self) -> &'static str { "NPY (NumPy binary format, Python-compatible)" }
}

//---------------------------------------------------------------------------------------------
// HDF5 Writer: rich metadata
//---------------------------------------------------------------------------------------------
#[cfg(feature = "serializer-hdf5")]
#[derive(Debug, Default, Clone, Copy)]
pub struct Hdf5Writer;

#[cfg(feature = "serializer-hdf5")]
impl MatrixWriter for Hdf5Writer {
    fn write(&self, matrix: &Array2<f64>, path: &Path) -> Result<()> {
        use hdf5::{File, DatasetCreator};

        // Create or overwrite the file
        let file = File::create(path)
            .map_err(|e| SerializerError::Hdf5Error(e.to_string()))?;

        let shape = matrix.shape();
        let dataset = file
            .new_dataset::<f64>()
            .shape(shape)
            .create("matrix")
            .map_err(|e| SerializerError::Hdf5Error(e.to_string()))?;

        // Write the matrix data (row-major order)
        dataset.write(matrix)
            .map_err(|e| SerializerError::Hdf5Error(e.to_string()))?;

        // Optional: add metadata attributes
        dataset.attrs()
            .create("rows")
            .of_type::<usize>()
            .shape(1)
            .create()
            .and_then(|attr| attr.write(&[shape[0]]))
            .map_err(|e| SerializerError::Hdf5Error(e.to_string()))?;

        dataset.attrs()
            .create("cols")
            .of_type::<usize>()
            .shape(1)
            .create()
            .and_then(|attr| attr.write(&[shape[1]]))
            .map_err(|e| SerializerError::Hdf5Error(e.to_string()))?;

        Ok(())
    }

    fn extension(&self) -> &'static str { "h5" }
    fn description(&self) -> &'static str { "HDF5 (rich metadata, hierarchical, your input format)" }
}

//---------------------------------------------------------------------------------------------
// Bincode Writer
//---------------------------------------------------------------------------------------------
#[cfg(feature = "serializer-bincode")]
#[derive(Debug, Default, Clone, Copy)]
pub struct BincodeWriter;

#[cfg(feature = "serializer-bincode")]
impl MatrixWriter for BincodeWriter {
    fn write(&self, matrix: &Array2<f64>, path: &Path) -> Result<()> {
        use std::fs::File;
        use std::io::BufWriter;
        use bincode::config;

        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let config = config::standard();

        bincode::encode_into_std_write(matrix, writer, config)
            .map_err(SerializerError::BincodeError)
    }

    fn extension(&self) -> &'static str { "bin" }
    fn description(&self) -> &'static str { "Bincode (Rust-native, fast, compact)" }
}

//---------------------------------------------------------------------------------------------
// Convenience function: write matrix by format string
//---------------------------------------------------------------------------------------------
/// Write a matrix to a file, auto-detecting format from extension.
/// 
/// # Arguments
/// * 'matrix' - The matrix to write.
/// * 'path' - Output file path (extension determines format).
/// * 'format_hint' - Optional override (e.g., "csv" if path has no extension).
/// 
/// # Returns
/// * 'Ok(())' on success.
/// * 'Err(SerializerError)' on failure.
pub fn write_matrix(
    matrix: &Array2<f64>,
    path: &Path,
    format_hint: Option<&str>,
) -> Result<()> {
    // Determine format: extension first, then hint, then default to CSV
    let raw_ext: &str = path
        .extension()
        .and_then(|e| e.to_str())
        .or(format_hint)
        .unwrap_or("csv");

    let ext = raw_ext.to_ascii_lowercase();

    let writer_kind = WriterKind::from_str(&ext)
        .ok_or_else(|| SerializerError::UnsupportedFormat(ext))?;

    let writer = writer_kind.create_writer()?;
    writer.write(matrix, path)
}
