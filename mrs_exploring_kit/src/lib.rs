// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

pub mod parser;
pub mod serializer;

pub use parser::hic_parser::{HiCConfig, NormalizationMethod, extract_contact_matrix};
pub use serializer::{MatrixWriter, WriterKind, write_matrix, SerializerError};
