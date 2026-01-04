//! File Version Tracker
//!
//! Tracks file modifications to determine which files need reindexing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// File version information
#[derive(Debug, Clone)]
pub struct TrackedFile {
    pub path: PathBuf,
    pub mtime: SystemTime,
    pub size: u64,
    pub content_hash: Option<u64>,
}

impl TrackedFile {
    /// Check if file has changed compared to disk
    pub fn has_changed(&self) -> bool {
        if let Ok(metadata) = std::fs::metadata(&self.path) {
            if let Ok(mtime) = metadata.modified() {
                return mtime != self.mtime || metadata.len() != self.size;
            }
        }
        true
    }
}

/// Tracks file versions for incremental indexing
pub struct FileVersionTracker {
    files: HashMap<PathBuf, TrackedFile>,
}

impl FileVersionTracker {
    /// Create a new file tracker
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Track a file
    pub fn track(&mut self, path: &Path) -> std::io::Result<()> {
        let metadata = std::fs::metadata(path)?;
        let mtime = metadata.modified()?;
        
        self.files.insert(
            path.to_path_buf(),
            TrackedFile {
                path: path.to_path_buf(),
                mtime,
                size: metadata.len(),
                content_hash: None,
            },
        );
        
        Ok(())
    }

    /// Track a file with content hash
    pub fn track_with_hash(&mut self, path: &Path, content: &str) -> std::io::Result<()> {
        let metadata = std::fs::metadata(path)?;
        let mtime = metadata.modified()?;
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();
        
        self.files.insert(
            path.to_path_buf(),
            TrackedFile {
                path: path.to_path_buf(),
                mtime,
                size: metadata.len(),
                content_hash: Some(hash),
            },
        );
        
        Ok(())
    }

    /// Check if a file needs reindexing
    pub fn needs_reindex(&self, path: &Path) -> bool {
        match self.files.get(path) {
            Some(tracked) => tracked.has_changed(),
            None => true,
        }
    }

    /// Get all files that need reindexing
    pub fn get_changed_files(&self) -> Vec<PathBuf> {
        self.files
            .values()
            .filter(|f| f.has_changed())
            .map(|f| f.path.clone())
            .collect()
    }

    /// Remove a file from tracking
    pub fn untrack(&mut self, path: &Path) {
        self.files.remove(path);
    }

    /// Get all tracked files
    pub fn tracked_files(&self) -> impl Iterator<Item = &TrackedFile> {
        self.files.values()
    }

    /// Get number of tracked files
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if tracker is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Clear all tracked files
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// Compute hash of file content
    pub fn hash_content(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for FileVersionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_tracker() {
        let mut tracker = FileVersionTracker::new();
        
        // Create a temp file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "test content").unwrap();
        
        // Track it
        tracker.track(file.path()).unwrap();
        
        // Should not need reindex immediately
        assert!(!tracker.needs_reindex(file.path()));
        
        // Unknown file should need reindex
        assert!(tracker.needs_reindex(Path::new("/nonexistent")));
    }
}

