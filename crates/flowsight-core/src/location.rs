//! Source code location types

use serde::{Deserialize, Serialize};

/// Represents a location in source code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Location {
    /// File path
    pub file: String,
    /// Start line (1-based)
    pub line: u32,
    /// Start column (0-based)
    pub column: u32,
    /// End line (1-based)
    pub end_line: u32,
    /// End column (0-based)
    pub end_column: u32,
}

impl Location {
    /// Create a new location
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            end_line: line,
            end_column: column,
        }
    }

    /// Create a location with range
    pub fn with_range(
        file: impl Into<String>,
        line: u32,
        column: u32,
        end_line: u32,
        end_column: u32,
    ) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            end_line,
            end_column,
        }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}
