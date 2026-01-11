//! FlowSight Parser
//!
//! Code parsing using tree-sitter for fast incremental parsing,
//! with optional libclang integration for precise semantic analysis.
//!
//! ## Modules
//!
//! - `treesitter` - Fast incremental parsing using tree-sitter
//! - `preprocessor` - C preprocessor integration using Clang
//! - `ast` - AST types and utilities
//! - `cache` - LRU cache for parsed syntax trees
//! - `parallel` - Parallel file parsing using rayon

pub mod ast;
pub mod cache;
pub mod parallel;
pub mod preprocessor;
pub mod treesitter;

use flowsight_core::{FunctionDef, Result, StructDef};
use std::collections::HashMap;
use std::path::Path;

/// Parse result containing extracted information
#[derive(Debug, Default, Clone)]
pub struct ParseResult {
    /// Functions found in the source
    pub functions: HashMap<String, FunctionDef>,
    /// Structs found in the source
    pub structs: HashMap<String, StructDef>,
    /// Parse errors (non-fatal)
    pub errors: Vec<String>,
}

/// Parser trait for different backends
pub trait Parser: Send + Sync {
    /// Parse source code string
    fn parse(&self, source: &str, filename: &str) -> Result<ParseResult>;

    /// Parse a file
    fn parse_file(&self, path: &Path) -> Result<ParseResult> {
        let source = std::fs::read_to_string(path)?;
        let filename = path.to_string_lossy();
        self.parse(&source, &filename)
    }

    /// Get parser name
    fn name(&self) -> &str;

    /// Check if parser is available
    fn is_available(&self) -> bool;
}

/// Get the best available parser
pub fn get_parser() -> Box<dyn Parser> {
    Box::new(treesitter::TreeSitterParser::new())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_parser_available() {
        let parser = get_parser();
        assert!(parser.is_available());
    }
}
