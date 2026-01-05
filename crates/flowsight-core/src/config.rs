//! Configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// FlowSight configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project root path
    pub project_root: PathBuf,

    /// Parser configuration
    pub parser: ParserConfig,

    /// Index configuration
    pub index: IndexConfig,

    /// Analysis configuration
    pub analysis: AnalysisConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_root: PathBuf::from("."),
            parser: ParserConfig::default(),
            index: IndexConfig::default(),
            analysis: AnalysisConfig::default(),
        }
    }
}

/// Parser configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Use libclang for precise parsing (requires compile_commands.json)
    pub use_clang: bool,

    /// Path to compile_commands.json
    pub compile_commands: Option<PathBuf>,

    /// File extensions to parse
    pub extensions: Vec<String>,

    /// Directories to exclude
    pub exclude_dirs: Vec<String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            use_clang: false,
            compile_commands: None,
            extensions: vec!["c".into(), "h".into()],
            exclude_dirs: vec![".git".into(), "build".into(), "node_modules".into()],
        }
    }
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Index storage path
    pub storage_path: Option<PathBuf>,

    /// Enable incremental indexing
    pub incremental: bool,

    /// Maximum memory usage for indexing (MB)
    pub max_memory_mb: usize,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            storage_path: None,
            incremental: true,
            max_memory_mb: 2048,
        }
    }
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Enable async mechanism tracking
    pub track_async: bool,

    /// Enable function pointer resolution
    pub resolve_func_ptrs: bool,

    /// Maximum call depth for analysis
    pub max_call_depth: usize,

    /// Custom knowledge base paths
    pub knowledge_paths: Vec<PathBuf>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            track_async: true,
            resolve_func_ptrs: true,
            max_call_depth: 20,
            knowledge_paths: vec![],
        }
    }
}
