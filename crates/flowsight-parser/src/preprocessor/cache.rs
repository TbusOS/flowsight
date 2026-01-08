//! Preprocessor Cache
//!
//! Caches preprocessed results to avoid redundant preprocessing.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::clang::PreprocessOptions;

/// Cache errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializeError(#[from] serde_json::Error),
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// Hash of the source file content
    source_hash: u64,
    /// Hash of the preprocess options
    options_hash: u64,
    /// Modification time of source file
    mtime: u64,
    /// Path to cached preprocessed file
    cache_file: PathBuf,
}

/// Preprocessor result cache
pub struct PreprocessorCache {
    /// Cache directory
    cache_dir: PathBuf,
    /// In-memory index of cached entries
    index: HashMap<PathBuf, CacheEntry>,
    /// Whether cache is enabled
    enabled: bool,
}

impl PreprocessorCache {
    /// Create a new cache in the specified directory
    pub fn new(cache_dir: PathBuf) -> Result<Self, CacheError> {
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        let mut cache = Self {
            cache_dir,
            index: HashMap::new(),
            enabled: true,
        };

        cache.load_index()?;
        Ok(cache)
    }

    /// Create a disabled cache (no-op)
    pub fn disabled() -> Self {
        Self {
            cache_dir: PathBuf::new(),
            index: HashMap::new(),
            enabled: false,
        }
    }

    /// Check if cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get cached preprocessed code if valid
    pub fn get(&self, source_path: &Path, options: &PreprocessOptions) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let entry = self.index.get(source_path)?;

        // Check if cache is still valid
        if !self.is_valid(source_path, entry, options) {
            return None;
        }

        // Read cached file
        fs::read_to_string(&entry.cache_file).ok()
    }

    /// Store preprocessed code in cache
    pub fn put(
        &mut self,
        source_path: &Path,
        options: &PreprocessOptions,
        preprocessed: &str,
    ) -> Result<(), CacheError> {
        if !self.enabled {
            return Ok(());
        }

        let source_hash = self.hash_file(source_path)?;
        let options_hash = self.hash_options(options);
        let mtime = self.get_mtime(source_path)?;

        // Generate cache file path
        let cache_file = self.cache_file_path(source_path);

        // Ensure parent directory exists
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write preprocessed code
        fs::write(&cache_file, preprocessed)?;

        // Update index
        let entry = CacheEntry {
            source_hash,
            options_hash,
            mtime,
            cache_file,
        };
        self.index.insert(source_path.to_path_buf(), entry);

        // Save index
        self.save_index()?;

        Ok(())
    }

    /// Invalidate cache for a specific file
    pub fn invalidate(&mut self, source_path: &Path) -> Result<(), CacheError> {
        if let Some(entry) = self.index.remove(source_path) {
            if entry.cache_file.exists() {
                fs::remove_file(&entry.cache_file)?;
            }
        }
        self.save_index()?;
        Ok(())
    }

    /// Clear all cached entries
    pub fn clear(&mut self) -> Result<(), CacheError> {
        if !self.enabled {
            return Ok(());
        }

        // Remove all cache files
        for entry in self.index.values() {
            if entry.cache_file.exists() {
                let _ = fs::remove_file(&entry.cache_file);
            }
        }

        self.index.clear();
        self.save_index()?;

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_entries = self.index.len();
        let total_size: u64 = self.index.values()
            .filter_map(|e| fs::metadata(&e.cache_file).ok())
            .map(|m| m.len())
            .sum();

        CacheStats {
            total_entries,
            total_size,
        }
    }

    /// Check if a cache entry is still valid
    fn is_valid(&self, source_path: &Path, entry: &CacheEntry, options: &PreprocessOptions) -> bool {
        // Check if cache file exists
        if !entry.cache_file.exists() {
            return false;
        }

        // Check options hash
        if entry.options_hash != self.hash_options(options) {
            return false;
        }

        // Check file modification time (fast check)
        if let Ok(mtime) = self.get_mtime(source_path) {
            if mtime != entry.mtime {
                return false;
            }
        } else {
            return false;
        }

        // Check content hash (slower but more accurate)
        if let Ok(hash) = self.hash_file(source_path) {
            if hash != entry.source_hash {
                return false;
            }
        } else {
            return false;
        }

        true
    }

    /// Generate cache file path for a source file
    fn cache_file_path(&self, source_path: &Path) -> PathBuf {
        let hash = self.hash_path(source_path);
        self.cache_dir.join(format!("{:016x}.i", hash))
    }

    /// Hash a file's contents
    fn hash_file(&self, path: &Path) -> Result<u64, CacheError> {
        let content = fs::read(path)?;
        Ok(self.hash_bytes(&content))
    }

    /// Hash preprocess options
    fn hash_options(&self, options: &PreprocessOptions) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();

        // Hash target
        options.target.target_triple().hash(&mut hasher);

        // Hash defines
        for def in &options.defines {
            def.name.hash(&mut hasher);
            def.value.hash(&mut hasher);
        }

        // Hash include paths
        for inc in &options.includes {
            inc.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Hash a path
    fn hash_path(&self, path: &Path) -> u64 {
        self.hash_bytes(path.to_string_lossy().as_bytes())
    }

    /// Hash bytes using a simple hash function
    fn hash_bytes(&self, bytes: &[u8]) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        hasher.finish()
    }

    /// Get file modification time as u64
    fn get_mtime(&self, path: &Path) -> Result<u64, CacheError> {
        let metadata = fs::metadata(path)?;
        let mtime = metadata.modified()?;
        Ok(mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs())
    }

    /// Load cache index from disk
    fn load_index(&mut self) -> Result<(), CacheError> {
        let index_path = self.cache_dir.join("index.json");
        if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            self.index = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    /// Save cache index to disk
    fn save_index(&self) -> Result<(), CacheError> {
        let index_path = self.cache_dir.join("index.json");
        let content = serde_json::to_string_pretty(&self.index)?;
        fs::write(&index_path, content)?;
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached entries
    pub total_entries: usize,
    /// Total size of cached files in bytes
    pub total_size: u64,
}

impl CacheStats {
    /// Format total size as human-readable string
    pub fn size_human(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.total_size >= GB {
            format!("{:.2} GB", self.total_size as f64 / GB as f64)
        } else if self.total_size >= MB {
            format!("{:.2} MB", self.total_size as f64 / MB as f64)
        } else if self.total_size >= KB {
            format!("{:.2} KB", self.total_size as f64 / KB as f64)
        } else {
            format!("{} bytes", self.total_size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_put_get() {
        let temp = TempDir::new().unwrap();
        let cache_dir = temp.path().join("cache");
        let mut cache = PreprocessorCache::new(cache_dir).unwrap();

        // Create a test source file
        let source_path = temp.path().join("test.c");
        fs::write(&source_path, "int main() {}").unwrap();

        let options = PreprocessOptions::default();
        let preprocessed = "preprocessed content";

        // Put in cache
        cache.put(&source_path, &options, preprocessed).unwrap();

        // Get from cache
        let cached = cache.get(&source_path, &options);
        assert_eq!(cached, Some(preprocessed.to_string()));
    }

    #[test]
    fn test_cache_invalidation() {
        let temp = TempDir::new().unwrap();
        let cache_dir = temp.path().join("cache");
        let mut cache = PreprocessorCache::new(cache_dir).unwrap();

        let source_path = temp.path().join("test.c");
        fs::write(&source_path, "int main() {}").unwrap();

        let options = PreprocessOptions::default();
        cache.put(&source_path, &options, "content").unwrap();

        // Invalidate
        cache.invalidate(&source_path).unwrap();

        // Should not be in cache
        assert!(cache.get(&source_path, &options).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let temp = TempDir::new().unwrap();
        let cache_dir = temp.path().join("cache");
        let mut cache = PreprocessorCache::new(cache_dir).unwrap();

        let source_path = temp.path().join("test.c");
        fs::write(&source_path, "int main() {}").unwrap();

        let options = PreprocessOptions::default();
        cache.put(&source_path, &options, "preprocessed").unwrap();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
        assert!(stats.total_size > 0);
    }
}
