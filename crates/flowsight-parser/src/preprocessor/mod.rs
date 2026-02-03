//! C Preprocessor Integration
//!
//! This module provides integration with Clang preprocessor for accurate
//! C code analysis, handling macros, conditional compilation, and header files.

pub mod cache;
pub mod clang;
pub mod config;
pub mod headers;

pub use cache::PreprocessorCache;
pub use clang::{ClangPreprocessor, PreprocessOptions, PreprocessResult};
pub use config::{Architecture, ConfigExtractor, MacroDefinition};
pub use headers::HeaderResolver;
