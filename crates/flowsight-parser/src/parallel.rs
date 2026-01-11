//! Parallel file parsing using rayon
//!
//! Provides efficient multi-file parsing with progress reporting.

use crate::cache::{hash_content, ParseCache};
use crate::treesitter::TreeSitterParser;
use crate::ParseResult;
use flowsight_core::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, info};
use walkdir::WalkDir;

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(ProgressEvent) + Send + Sync>;

/// Progress event for tracking parsing progress
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub phase: ProgressPhase,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

/// Parsing phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressPhase {
    Scanning,
    Parsing,
    Indexing,
    Complete,
}

/// Parallel parser with caching support
pub struct ParallelParser {
    cache: Arc<ParseCache>,
    progress_callback: Option<Arc<ProgressCallback>>,
}

impl ParallelParser {
    /// Create a new parallel parser
    pub fn new() -> Self {
        Self {
            cache: Arc::new(ParseCache::default()),
            progress_callback: None,
        }
    }

    /// Create with custom cache capacity
    pub fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            cache: Arc::new(ParseCache::new(capacity)),
            progress_callback: None,
        }
    }

    /// Set progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(ProgressEvent) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(Box::new(callback)));
        self
    }

    /// Parse multiple files in parallel
    pub fn parse_files(&self, paths: &[PathBuf]) -> Vec<(PathBuf, Result<ParseResult>)> {
        let total = paths.len();
        let processed = AtomicUsize::new(0);

        self.emit_progress(ProgressPhase::Parsing, 0, total, "Starting parallel parse...");

        let results: Vec<_> = paths
            .par_iter()
            .map(|path| {
                let result = self.parse_file_cached(path);

                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                if current % 10 == 0 || current == total {
                    self.emit_progress(
                        ProgressPhase::Parsing,
                        current,
                        total,
                        format!("Parsed {}/{} files", current, total),
                    );
                }

                (path.clone(), result)
            })
            .collect();

        self.emit_progress(ProgressPhase::Complete, total, total, "Parsing complete");
        results
    }

    /// Parse a directory recursively
    pub fn parse_directory(&self, dir: &Path, extensions: &[&str]) -> Vec<(PathBuf, Result<ParseResult>)> {
        // Scan phase
        self.emit_progress(ProgressPhase::Scanning, 0, 0, "Scanning directory...");

        let paths: Vec<PathBuf> = WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| extensions.contains(&ext))
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        info!("Found {} files to parse", paths.len());
        self.emit_progress(
            ProgressPhase::Scanning,
            paths.len(),
            paths.len(),
            format!("Found {} files", paths.len()),
        );

        self.parse_files(&paths)
    }

    /// Parse a single file with caching
    pub fn parse_file_cached(&self, path: &Path) -> Result<ParseResult> {
        let content = std::fs::read_to_string(path)?;
        let hash = hash_content(&content);
        let path_buf = path.to_path_buf();

        // Check cache
        if let Some(cached) = self.cache.get(&path_buf, hash) {
            debug!("Cache hit for {:?}", path);
            return Ok((*cached).clone());
        }

        // Parse
        debug!("Parsing {:?}", path);
        let mut parser = TreeSitterParser::new();
        let filename = path.to_string_lossy();
        let result = parser.parse_source(&content, &filename)?;

        // Cache result
        let mtime = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0);
        self.cache.insert(path_buf, hash, mtime, result.clone());

        Ok(result)
    }

    /// Invalidate cache for a file
    pub fn invalidate(&self, path: &Path) {
        self.cache.invalidate(&path.to_path_buf());
    }

    /// Clear all cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> crate::cache::CacheStats {
        self.cache.stats()
    }

    fn emit_progress<S: Into<String>>(&self, phase: ProgressPhase, current: usize, total: usize, message: S) {
        if let Some(ref callback) = self.progress_callback {
            callback(ProgressEvent {
                phase,
                current,
                total,
                message: message.into(),
            });
        }
    }
}

impl Default for ParallelParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge multiple parse results into one
pub fn merge_results(results: Vec<ParseResult>) -> ParseResult {
    let mut merged = ParseResult::default();

    for result in results {
        merged.functions.extend(result.functions);
        merged.structs.extend(result.structs);
        merged.errors.extend(result.errors);
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parallel_parse() {
        let dir = TempDir::new().unwrap();

        // Create test files
        for i in 0..5 {
            let path = dir.path().join(format!("test{}.c", i));
            let mut file = std::fs::File::create(&path).unwrap();
            writeln!(file, "void func{}(void) {{}}", i).unwrap();
        }

        let parser = ParallelParser::new();
        let results = parser.parse_directory(dir.path(), &["c"]);

        assert_eq!(results.len(), 5);
        for (_, result) in results {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_cache_hit() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.c");
        std::fs::write(&path, "void test(void) {}").unwrap();

        let parser = ParallelParser::new();

        // First parse
        let _ = parser.parse_file_cached(&path);
        let stats1 = parser.cache_stats();
        assert_eq!(stats1.entries, 1);

        // Second parse should hit cache
        let _ = parser.parse_file_cached(&path);
        let stats2 = parser.cache_stats();
        assert_eq!(stats2.entries, 1); // Still 1, cache hit
    }
}
