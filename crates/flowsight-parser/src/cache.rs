//! LRU Cache for parsed syntax trees
//!
//! Provides memory-efficient caching of parsed trees with automatic eviction.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Cache entry with metadata
#[derive(Clone)]
pub struct CacheEntry {
    /// File content hash for change detection
    pub content_hash: u64,
    /// File modification time
    pub mtime: u64,
    /// Cached parse result (serialized to reduce memory)
    pub result: Arc<crate::ParseResult>,
}

/// LRU cache for parsed files
pub struct ParseCache {
    /// Maximum number of entries
    capacity: usize,
    /// Cache entries
    entries: RwLock<HashMap<PathBuf, CacheEntry>>,
    /// Access order for LRU eviction
    access_order: RwLock<Vec<PathBuf>>,
}

impl ParseCache {
    /// Create a new cache with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: RwLock::new(HashMap::with_capacity(capacity)),
            access_order: RwLock::new(Vec::with_capacity(capacity)),
        }
    }

    /// Get a cached entry if valid
    pub fn get(&self, path: &PathBuf, content_hash: u64) -> Option<Arc<crate::ParseResult>> {
        let entries = self.entries.read().ok()?;
        if let Some(entry) = entries.get(path) {
            if entry.content_hash == content_hash {
                let result = entry.result.clone();
                // Update access order after releasing read lock
                drop(entries);
                self.touch(path);
                return Some(result);
            }
        }
        None
    }

    /// Insert or update a cache entry
    pub fn insert(&self, path: PathBuf, content_hash: u64, mtime: u64, result: crate::ParseResult) {
        let mut entries = match self.entries.write() {
            Ok(e) => e,
            Err(_) => return,
        };

        // Evict if at capacity
        if entries.len() >= self.capacity && !entries.contains_key(&path) {
            self.evict_lru(&mut entries);
        }

        let entry = CacheEntry {
            content_hash,
            mtime,
            result: Arc::new(result),
        };
        entries.insert(path.clone(), entry);

        // Update access order
        drop(entries);
        self.touch(&path);
    }

    /// Remove an entry from cache
    pub fn invalidate(&self, path: &PathBuf) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(path);
        }
        if let Ok(mut order) = self.access_order.write() {
            order.retain(|p| p != path);
        }
    }

    /// Clear all entries
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
        if let Ok(mut order) = self.access_order.write() {
            order.clear();
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.read().map(|e| e.len()).unwrap_or(0);
        CacheStats {
            entries,
            capacity: self.capacity,
        }
    }

    fn touch(&self, path: &PathBuf) {
        if let Ok(mut order) = self.access_order.write() {
            order.retain(|p| p != path);
            order.push(path.clone());
        }
    }

    fn evict_lru(&self, entries: &mut HashMap<PathBuf, CacheEntry>) {
        if let Ok(mut order) = self.access_order.write() {
            if let Some(oldest) = order.first().cloned() {
                entries.remove(&oldest);
                order.remove(0);
            }
        }
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new(100) // Default: cache 100 files
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub capacity: usize,
}

/// Simple hash function for file content
pub fn hash_content(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_get() {
        let cache = ParseCache::new(10);
        let path = PathBuf::from("/test/file.c");
        let result = crate::ParseResult::default();
        let hash = 12345u64;

        cache.insert(path.clone(), hash, 0, result);

        assert!(cache.get(&path, hash).is_some());
        assert!(cache.get(&path, 99999).is_none()); // Wrong hash
    }

    #[test]
    fn test_cache_eviction() {
        let cache = ParseCache::new(2);

        for i in 0..3 {
            let path = PathBuf::from(format!("/test/file{}.c", i));
            cache.insert(path, i as u64, 0, crate::ParseResult::default());
        }

        // First entry should be evicted
        let path0 = PathBuf::from("/test/file0.c");
        assert!(cache.get(&path0, 0).is_none());

        // Later entries should still exist
        let path2 = PathBuf::from("/test/file2.c");
        assert!(cache.get(&path2, 2).is_some());
    }
}
