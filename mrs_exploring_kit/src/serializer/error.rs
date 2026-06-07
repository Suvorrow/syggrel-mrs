use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializerError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[cfg(feature = "serializer-csv")]
    #[error("CSV serialization error: {0}")]
    CsvError(#[from] csv::Error),

    #[cfg(feature = "serializer-npy")]
    #[error("NPY serialization error: {0}")]
    NpyError(String),

    #[cfg(feature = "serializer-hdf5")]
    #[error("HDF5 serialization error: {0}")]
    Hdf5Error(String),

    #[cfg(feature = "serializer-bincode")]
    #[error("Bincode serialization error: {0}")]
    BincodeError(#[from] bincode::error::EncodeError),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid matrix: {0}")]
    InvalidMatrix(String),
}

pub type Result<T> = std::result::Result<T, SerializerError>;