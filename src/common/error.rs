use pyo3::PyErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrustyError {
    #[error("Hamming distance requires equal length strings")]
    LengthMismatch,
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Failed to extract string: {0}")]
    StringExtraction(String),
    #[error("Failed to downcast to sequence: {0}")]
    SequenceDowncast(String),
    #[error("Failed to get sequence length: {0}")]
    SequenceLength(String),
    #[error("Failed to get sequence item: {0}")]
    SequenceItem(String),
    #[error("Failed to hash element: {0}")]
    Hashing(String),
    #[error("Failed to convert array to bytes: {0}")]
    ArrayConversion(String),
}

impl From<PyErr> for ConversionError {
    fn from(err: PyErr) -> Self {
        ConversionError::StringExtraction(err.to_string())
    }
}
