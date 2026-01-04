//! Batch Indexer
//!
//! Efficient batch indexing with progress reporting.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;

/// Progress event for indexing operations
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub phase: IndexPhase,
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub current_file: Option<PathBuf>,
}

/// Indexing phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexPhase {
    Scanning,
    Parsing,
    Analyzing,
    Indexing,
    BuildingGraph,
    Complete,
}

impl std::fmt::Display for IndexPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexPhase::Scanning => write!(f, "扫描文件"),
            IndexPhase::Parsing => write!(f, "解析代码"),
            IndexPhase::Analyzing => write!(f, "分析模式"),
            IndexPhase::Indexing => write!(f, "构建索引"),
            IndexPhase::BuildingGraph => write!(f, "构建调用图"),
            IndexPhase::Complete => write!(f, "完成"),
        }
    }
}

/// Configuration for batch indexing
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum files to process in parallel
    pub parallelism: usize,
    /// Whether to skip already indexed files
    pub incremental: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            include_patterns: vec!["*.c".into(), "*.h".into()],
            exclude_patterns: vec![],
            parallelism: num_cpus::get().max(1),
            incremental: true,
        }
    }
}

/// Batch indexer for processing multiple files
pub struct BatchIndexer {
    config: BatchConfig,
    progress_sender: Option<Sender<ProgressEvent>>,
}

impl BatchIndexer {
    /// Create a new batch indexer
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            progress_sender: None,
        }
    }

    /// Create with progress channel
    pub fn with_progress(config: BatchConfig) -> (Self, Receiver<ProgressEvent>) {
        let (tx, rx) = channel();
        (
            Self {
                config,
                progress_sender: Some(tx),
            },
            rx,
        )
    }

    /// Send progress event
    fn send_progress(&self, event: ProgressEvent) {
        if let Some(ref sender) = self.progress_sender {
            let _ = sender.send(event);
        }
    }

    /// Scan directory for source files
    pub fn scan_directory(&self, root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        self.send_progress(ProgressEvent {
            phase: IndexPhase::Scanning,
            current: 0,
            total: 0,
            message: format!("扫描目录: {}", root.display()),
            current_file: None,
        });

        for entry in walkdir::WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if self.should_include(path) {
                    files.push(path.to_path_buf());
                }
            }
        }

        self.send_progress(ProgressEvent {
            phase: IndexPhase::Scanning,
            current: files.len(),
            total: files.len(),
            message: format!("找到 {} 个源文件", files.len()),
            current_file: None,
        });

        files
    }

    /// Check if a file should be included based on patterns
    fn should_include(&self, path: &Path) -> bool {
        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        // Check exclude patterns first
        for pattern in &self.config.exclude_patterns {
            if Self::matches_pattern(filename, pattern) {
                return false;
            }
        }

        // Check include patterns
        for pattern in &self.config.include_patterns {
            if Self::matches_pattern(filename, pattern) {
                return true;
            }
        }

        false
    }

    /// Simple glob pattern matching
    fn matches_pattern(filename: &str, pattern: &str) -> bool {
        if pattern.starts_with('*') {
            filename.ends_with(&pattern[1..])
        } else if pattern.ends_with('*') {
            filename.starts_with(&pattern[..pattern.len()-1])
        } else {
            filename == pattern
        }
    }

    /// Get configuration
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }
}

impl Default for BatchIndexer {
    fn default() -> Self {
        Self::new(BatchConfig::default())
    }
}

// Add num_cpus as a simple function if the crate isn't available
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        assert!(BatchIndexer::matches_pattern("test.c", "*.c"));
        assert!(BatchIndexer::matches_pattern("test.h", "*.h"));
        assert!(!BatchIndexer::matches_pattern("test.cpp", "*.c"));
        assert!(BatchIndexer::matches_pattern("Makefile", "Makefile"));
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert!(config.include_patterns.contains(&"*.c".to_string()));
        assert!(config.incremental);
    }
}

