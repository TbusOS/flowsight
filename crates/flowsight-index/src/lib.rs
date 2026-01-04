//! FlowSight Index
//!
//! Provides persistent indexing for code symbols and call graphs.
//! Supports incremental updates for large codebases.

use flowsight_core::{FunctionDef, StructDef};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

mod file_tracker;
mod tree_cache;
mod batch_indexer;

pub use file_tracker::FileVersionTracker;
pub use tree_cache::TreeCache;
pub use batch_indexer::BatchIndexer;

/// File version information for incremental indexing
#[derive(Debug, Clone)]
pub struct FileVersion {
    pub path: PathBuf,
    pub hash: u64,
    pub mtime: SystemTime,
    pub indexed_at: SystemTime,
}

/// Symbol index containing all indexed information
#[derive(Debug, Default)]
pub struct SymbolIndex {
    /// All functions indexed by name
    pub functions: HashMap<String, FunctionDef>,
    /// All structs indexed by name
    pub structs: HashMap<String, StructDef>,
    /// Functions indexed by file
    pub functions_by_file: HashMap<PathBuf, Vec<String>>,
    /// File versions for incremental updates
    pub file_versions: HashMap<PathBuf, FileVersion>,
}

impl SymbolIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a function to the index
    pub fn add_function(&mut self, func: FunctionDef, file: &Path) {
        let name = func.name.clone();
        self.functions.insert(name.clone(), func);
        
        self.functions_by_file
            .entry(file.to_path_buf())
            .or_default()
            .push(name);
    }

    /// Add a struct to the index
    pub fn add_struct(&mut self, st: StructDef) {
        self.structs.insert(st.name.clone(), st);
    }

    /// Remove all symbols from a file
    pub fn remove_file(&mut self, file: &Path) {
        if let Some(func_names) = self.functions_by_file.remove(file) {
            for name in func_names {
                self.functions.remove(&name);
            }
        }
        self.file_versions.remove(file);
    }

    /// Get function by name
    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.get(name)
    }

    /// Get struct by name
    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.structs.get(name)
    }

    /// Get all functions in a file
    pub fn get_functions_in_file(&self, file: &Path) -> Vec<&FunctionDef> {
        self.functions_by_file
            .get(file)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|n| self.functions.get(n))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a file needs reindexing
    pub fn needs_reindex(&self, file: &Path, current_mtime: SystemTime) -> bool {
        match self.file_versions.get(file) {
            Some(version) => version.mtime != current_mtime,
            None => true,
        }
    }

    /// Update file version
    pub fn update_file_version(&mut self, file: &Path, hash: u64, mtime: SystemTime) {
        self.file_versions.insert(
            file.to_path_buf(),
            FileVersion {
                path: file.to_path_buf(),
                hash,
                mtime,
                indexed_at: SystemTime::now(),
            },
        );
    }

    /// Get statistics
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            total_functions: self.functions.len(),
            total_structs: self.structs.len(),
            total_files: self.functions_by_file.len(),
        }
    }
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_functions: usize,
    pub total_structs: usize,
    pub total_files: usize,
}

/// Index manager for handling indexing operations
pub struct IndexManager {
    index: SymbolIndex,
    tree_cache: TreeCache,
    file_tracker: FileVersionTracker,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new() -> Self {
        Self {
            index: SymbolIndex::new(),
            tree_cache: TreeCache::new(100), // Cache up to 100 trees
            file_tracker: FileVersionTracker::new(),
        }
    }

    /// Get the symbol index
    pub fn index(&self) -> &SymbolIndex {
        &self.index
    }

    /// Get mutable symbol index
    pub fn index_mut(&mut self) -> &mut SymbolIndex {
        &mut self.index
    }

    /// Get tree cache
    pub fn tree_cache(&self) -> &TreeCache {
        &self.tree_cache
    }

    /// Get mutable tree cache
    pub fn tree_cache_mut(&mut self) -> &mut TreeCache {
        &mut self.tree_cache
    }

    /// Check which files need reindexing
    pub fn get_files_to_index(&self, files: &[PathBuf]) -> Vec<PathBuf> {
        files
            .iter()
            .filter(|f| {
                if let Ok(metadata) = std::fs::metadata(f) {
                    if let Ok(mtime) = metadata.modified() {
                        return self.index.needs_reindex(f, mtime);
                    }
                }
                true
            })
            .cloned()
            .collect()
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowsight_core::Location;

    #[test]
    fn test_symbol_index() {
        let mut index = SymbolIndex::new();
        
        let func = FunctionDef {
            name: "my_func".into(),
            return_type: "int".into(),
            params: vec![],
            location: Some(Location::new("test.c", 10, 0)),
            calls: vec![],
            called_by: vec![],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        };
        
        index.add_function(func, Path::new("test.c"));
        
        assert!(index.get_function("my_func").is_some());
        assert_eq!(index.stats().total_functions, 1);
    }
}
