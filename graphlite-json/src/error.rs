//! Error types for the GraphLite JSON layer

use thiserror::Error;

/// Errors that can occur in the JSON layer
#[derive(Debug, Error)]
pub enum JsonLayerError {
    #[error("GraphLite error: {0}")]
    GraphLite(String),

    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    #[error("JTD validation failed: {0}")]
    ValidationFailed(String),

    #[error("JDT transform failed: {0}")]
    TransformFailed(String),

    #[error("JSON document not found")]
    NotFound,

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
}

/// Result type for JSON layer operations
pub type Result<T> = std::result::Result<T, JsonLayerError>;
