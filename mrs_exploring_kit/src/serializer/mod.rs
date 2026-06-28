// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Modular matrix serialization layer.
//! 
//! Provides trait-based writers for multiple formats with easy GUI integration.

pub mod error;
pub mod matrix_writer;

// Re-export main types for convenience
pub use error::{SerializerError, Result};
pub use matrix_writer::{MatrixWriter, WriterKind, write_matrix};

// Conditionally re-export writer structs (for advanced usage)
#[cfg(feature = "serializer-csv")]
pub use matrix_writer::CsvWriter;

#[cfg(feature = "serializer-npy")]
pub use matrix_writer::NpyWriter;

#[cfg(feature = "serializer-hdf5")]
pub use matrix_writer::Hdf5Writer;

#[cfg(feature = "serializer-bincode")]
pub use matrix_writer::BincodeWriter;
