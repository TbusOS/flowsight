//! Error types for FlowSight

use thiserror::Error;

/// FlowSight error type
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Index error: {0}")]
    Index(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("{0}")]
    Other(String),
}

/// Result type alias for FlowSight
pub type Result<T> = std::result::Result<T, Error>;

