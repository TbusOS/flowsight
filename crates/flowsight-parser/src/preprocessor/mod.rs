//! C Preprocessor Integration
//!
//! This module provides integration with Clang preprocessor for accurate
//! C code analysis, handling macros, conditional compilation, and header files.

pub mod config;
pub mod clang;
pub mod headers;
pub mod cache;

pub use config::{ConfigExtractor, MacroDefinition, Architecture};
pub use clang::{ClangPreprocessor, PreprocessOptions, PreprocessResult};
pub use headers::HeaderResolver;
pub use cache::PreprocessorCache;
