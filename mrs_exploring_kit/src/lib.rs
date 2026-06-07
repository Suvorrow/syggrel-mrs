pub mod parser;
pub mod serializer;

pub use parser::hic_parser::{HiCConfig, NormalizationMethod, extract_contact_matrix};
pub use serializer::{MatrixWriter, WriterKind, write_matrix, SerializerError};