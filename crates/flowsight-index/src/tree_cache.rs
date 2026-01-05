//! Tree Cache
//!
//! LRU cache for parsed syntax trees to support incremental parsing.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Cached tree entry
#[derive(Clone)]
pub struct CachedTree {
    /// The raw tree bytes (serialized)
    pub tree_data: Arc<Vec<u8>>,
    /// Source hash for validation
    pub source_hash: u64,
    /// Last access time (for LRU)
    pub last_access: std::time::Instant,
}

/// LRU cache for syntax trees
pub struct TreeCache {
    /// Maximum number of trees to cache
    max_size: usize,
    /// Cached trees by file path
    cache: HashMap<PathBuf, CachedTree>,
    /// Access order for LRU eviction
    access_order: Vec<PathBuf>,
}

impl TreeCache {
    /// Create a new tree cache with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            cache: HashMap::new(),
            access_order: Vec::new(),
        }
    }

    /// Get a cached tree if available and source hasn't changed
    pub fn get(&mut self, path: &PathBuf, current_hash: u64) -> Option<&CachedTree> {
        if let Some(entry) = self.cache.get_mut(path) {
            if entry.source_hash == current_hash {
                // Update access time and order
                entry.last_access = std::time::Instant::now();
                self.update_access_order(path.clone());
                return self.cache.get(path);
            }
        }
        None
    }

    /// Insert a tree into the cache
    pub fn insert(&mut self, path: PathBuf, tree_data: Vec<u8>, source_hash: u64) {
        // Evict if necessary
        while self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        let entry = CachedTree {
            tree_data: Arc::new(tree_data),
            source_hash,
            last_access: std::time::Instant::now(),
        };

        self.cache.insert(path.clone(), entry);
        self.update_access_order(path);
    }

    /// Remove a tree from the cache
    pub fn remove(&mut self, path: &PathBuf) {
        self.cache.remove(path);
        self.access_order.retain(|p| p != path);
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            max_size: self.max_size,
            memory_bytes: self.cache.values().map(|e| e.tree_data.len()).sum(),
        }
    }

    fn update_access_order(&mut self, path: PathBuf) {
        self.access_order.retain(|p| p != &path);
        self.access_order.push(path);
    }

    fn evict_lru(&mut self) {
        if let Some(path) = self.access_order.first().cloned() {
            self.cache.remove(&path);
            self.access_order.remove(0);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub max_size: usize,
    pub memory_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_cache() {
        let mut cache = TreeCache::new(2);

        let path1 = PathBuf::from("file1.c");
        let path2 = PathBuf::from("file2.c");
        let path3 = PathBuf::from("file3.c");

        cache.insert(path1.clone(), vec![1, 2, 3], 100);
        cache.insert(path2.clone(), vec![4, 5, 6], 200);

        assert!(cache.get(&path1, 100).is_some());
        assert!(cache.get(&path2, 200).is_some());

        // Inserting third should evict path2 (LRU after path1 was accessed)
        cache.insert(path3.clone(), vec![7, 8, 9], 300);

        assert_eq!(cache.stats().entries, 2);
    }

    #[test]
    fn test_hash_invalidation() {
        let mut cache = TreeCache::new(10);
        let path = PathBuf::from("test.c");

        cache.insert(path.clone(), vec![1, 2, 3], 100);

        // Same hash should return cached
        assert!(cache.get(&path, 100).is_some());

        // Different hash should return None
        assert!(cache.get(&path, 200).is_none());
    }
}
