//! Error types for PolarisDB operations.

use thiserror::Error;

/// Result type alias using PolarisDB's Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during PolarisDB operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Vector dimension mismatch between index and input.
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    /// Attempted to insert a duplicate vector ID.
    #[error("duplicate vector id: {0}")]
    DuplicateId(u64),

    /// Vector with the given ID was not found.
    #[error("vector not found: {0}")]
    NotFound(u64),

    /// Invalid filter expression.
    #[error("invalid filter: {0}")]
    InvalidFilter(String),

    /// Payload field access error.
    #[error("payload error: {0}")]
    PayloadError(String),

    /// Empty vector provided where non-empty expected.
    #[error("empty vector not allowed")]
    EmptyVector,

    /// IO error during storage operations.
    #[error("io error: {0}")]
    IoError(String),

    /// WAL corruption detected.
    #[error("WAL corrupted: {0}")]
    WalCorrupted(String),

    /// Collection not found or invalid path.
    #[error("collection error: {0}")]
    CollectionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::DimensionMismatch {
            expected: 384,
            got: 512,
        };
        assert_eq!(err.to_string(), "dimension mismatch: expected 384, got 512");
    }
}
