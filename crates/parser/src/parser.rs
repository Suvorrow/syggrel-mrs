// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

pub mod hic_parser;

pub mod cool_parser;

pub mod parser_utils;

pub use hic_parser::{HiCConfig, NormalizationMethod, extract_contact_matrix as extract_hic};

pub use cool_parser::*;

pub use parser_utils::*;
