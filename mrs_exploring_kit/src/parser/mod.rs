pub mod hic_parser;

pub mod cool_parser;

pub mod parser_utils;

pub use hic_parser::{HiCConfig, NormalizationMethod, extract_contact_matrix as extract_hic};

pub use cool_parser::*;

pub use parser_utils::*;
