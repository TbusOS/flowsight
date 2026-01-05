//! Persistent storage for symbol index
//!
//! Uses sled for fast key-value storage with automatic persistence.

use crate::{FileVersion, SymbolIndex};
use flowsight_core::{FunctionDef, StructDef};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Persistent index storage using sled
pub struct IndexStorage {
    db: sled::Db,
    functions_tree: sled::Tree,
    structs_tree: sled::Tree,
    files_tree: sled::Tree,
    versions_tree: sled::Tree,
}

/// Serializable wrapper for file-to-functions mapping
#[derive(Serialize, Deserialize)]
struct FileFunctions {
    functions: Vec<String>,
}

impl IndexStorage {
    /// Open or create a storage at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let db = sled::open(path)?;
        let functions_tree = db.open_tree("functions")?;
        let structs_tree = db.open_tree("structs")?;
        let files_tree = db.open_tree("files")?;
        let versions_tree = db.open_tree("versions")?;

        Ok(Self {
            db,
            functions_tree,
            structs_tree,
            files_tree,
            versions_tree,
        })
    }

    /// Create an in-memory storage (for testing)
    pub fn in_memory() -> Result<Self> {
        let db = sled::Config::new().temporary(true).open()?;
        let functions_tree = db.open_tree("functions")?;
        let structs_tree = db.open_tree("structs")?;
        let files_tree = db.open_tree("files")?;
        let versions_tree = db.open_tree("versions")?;

        Ok(Self {
            db,
            functions_tree,
            structs_tree,
            files_tree,
            versions_tree,
        })
    }

    /// Store a function
    pub fn store_function(&self, func: &FunctionDef, file: &Path) -> Result<()> {
        let key = func.name.as_bytes();
        let value = serde_json::to_vec(func)?;
        self.functions_tree.insert(key, value)?;

        // Update file->functions mapping
        let file_key = file.to_string_lossy().into_owned();
        let mut file_funcs = self.get_file_functions(file)?;
        if !file_funcs.contains(&func.name) {
            file_funcs.push(func.name.clone());
            let value = serde_json::to_vec(&FileFunctions {
                functions: file_funcs,
            })?;
            self.files_tree.insert(file_key.as_bytes(), value)?;
        }

        Ok(())
    }

    /// Store a struct
    pub fn store_struct(&self, st: &StructDef) -> Result<()> {
        let key = st.name.as_bytes();
        let value = serde_json::to_vec(st)?;
        self.structs_tree.insert(key, value)?;
        Ok(())
    }

    /// Get a function by name
    pub fn get_function(&self, name: &str) -> Result<Option<FunctionDef>> {
        match self.functions_tree.get(name.as_bytes())? {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Get a struct by name
    pub fn get_struct(&self, name: &str) -> Result<Option<StructDef>> {
        match self.structs_tree.get(name.as_bytes())? {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Get all functions from a file
    fn get_file_functions(&self, file: &Path) -> Result<Vec<String>> {
        let file_key = file.to_string_lossy();
        match self.files_tree.get(file_key.as_bytes())? {
            Some(bytes) => {
                let ff: FileFunctions = serde_json::from_slice(&bytes)?;
                Ok(ff.functions)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Remove all symbols from a file
    pub fn remove_file(&self, file: &Path) -> Result<()> {
        let file_funcs = self.get_file_functions(file)?;
        for func_name in file_funcs {
            self.functions_tree.remove(func_name.as_bytes())?;
        }
        let file_key = file.to_string_lossy();
        self.files_tree.remove(file_key.as_bytes())?;
        self.versions_tree.remove(file_key.as_bytes())?;
        Ok(())
    }

    /// Store file version
    pub fn store_file_version(&self, version: &FileVersion) -> Result<()> {
        let key = version.path.to_string_lossy();
        let value = serde_json::to_vec(version)?;
        self.versions_tree.insert(key.as_bytes(), value)?;
        Ok(())
    }

    /// Get file version
    pub fn get_file_version(&self, path: &Path) -> Result<Option<FileVersion>> {
        let key = path.to_string_lossy();
        match self.versions_tree.get(key.as_bytes())? {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Search functions by name pattern
    pub fn search_functions(&self, pattern: &str) -> Result<Vec<FunctionDef>> {
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();

        for item in self.functions_tree.iter() {
            let (key, value) = item?;
            let name = String::from_utf8_lossy(&key);
            if name.to_lowercase().contains(&pattern_lower) {
                let func: FunctionDef = serde_json::from_slice(&value)?;
                results.push(func);
            }
        }

        Ok(results)
    }

    /// Search structs by name pattern
    pub fn search_structs(&self, pattern: &str) -> Result<Vec<StructDef>> {
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();

        for item in self.structs_tree.iter() {
            let (key, value) = item?;
            let name = String::from_utf8_lossy(&key);
            if name.to_lowercase().contains(&pattern_lower) {
                let st: StructDef = serde_json::from_slice(&value)?;
                results.push(st);
            }
        }

        Ok(results)
    }

    /// Get all function names
    pub fn all_function_names(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for item in self.functions_tree.iter() {
            let (key, _) = item?;
            names.push(String::from_utf8_lossy(&key).into_owned());
        }
        Ok(names)
    }

    /// Load entire index into memory
    pub fn load_index(&self) -> Result<SymbolIndex> {
        let mut index = SymbolIndex::new();

        // Load all functions
        for item in self.functions_tree.iter() {
            let (_, value) = item?;
            let func: FunctionDef = serde_json::from_slice(&value)?;
            index.functions.insert(func.name.clone(), func);
        }

        // Load all structs
        for item in self.structs_tree.iter() {
            let (_, value) = item?;
            let st: StructDef = serde_json::from_slice(&value)?;
            index.structs.insert(st.name.clone(), st);
        }

        // Load file mappings
        for item in self.files_tree.iter() {
            let (key, value) = item?;
            let path = PathBuf::from(String::from_utf8_lossy(&key).into_owned());
            let ff: FileFunctions = serde_json::from_slice(&value)?;
            index.functions_by_file.insert(path, ff.functions);
        }

        // Load file versions
        for item in self.versions_tree.iter() {
            let (_, value) = item?;
            let version: FileVersion = serde_json::from_slice(&value)?;
            index.file_versions.insert(version.path.clone(), version);
        }

        Ok(index)
    }

    /// Save entire index to storage
    pub fn save_index(&self, index: &SymbolIndex) -> Result<()> {
        // Clear existing data
        self.functions_tree.clear()?;
        self.structs_tree.clear()?;
        self.files_tree.clear()?;
        self.versions_tree.clear()?;

        // Store functions
        for func in index.functions.values() {
            let key = func.name.as_bytes();
            let value = serde_json::to_vec(func)?;
            self.functions_tree.insert(key, value)?;
        }

        // Store structs
        for st in index.structs.values() {
            let key = st.name.as_bytes();
            let value = serde_json::to_vec(st)?;
            self.structs_tree.insert(key, value)?;
        }

        // Store file mappings
        for (path, funcs) in &index.functions_by_file {
            let key = path.to_string_lossy();
            let value = serde_json::to_vec(&FileFunctions {
                functions: funcs.clone(),
            })?;
            self.files_tree.insert(key.as_bytes(), value)?;
        }

        // Store file versions
        for version in index.file_versions.values() {
            let key = version.path.to_string_lossy();
            let value = serde_json::to_vec(version)?;
            self.versions_tree.insert(key.as_bytes(), value)?;
        }

        // Flush to disk
        self.db.flush()?;

        Ok(())
    }

    /// Flush changes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> Result<StorageStats> {
        Ok(StorageStats {
            functions_count: self.functions_tree.len(),
            structs_count: self.structs_tree.len(),
            files_count: self.files_tree.len(),
        })
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub functions_count: usize,
    pub structs_count: usize,
    pub files_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowsight_core::Location;

    #[test]
    fn test_storage_basic() {
        let storage = IndexStorage::in_memory().unwrap();

        let func = FunctionDef {
            name: "test_func".into(),
            return_type: "int".into(),
            params: vec![],
            location: Some(Location::new("test.c", 1, 0)),
            calls: vec![],
            called_by: vec![],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        };

        storage.store_function(&func, Path::new("test.c")).unwrap();

        let loaded = storage.get_function("test_func").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "test_func");
    }

    #[test]
    fn test_search() {
        let storage = IndexStorage::in_memory().unwrap();

        for name in &["init_driver", "exit_driver", "probe_device"] {
            let func = FunctionDef {
                name: (*name).into(),
                return_type: "int".into(),
                params: vec![],
                location: Some(Location::new("test.c", 1, 0)),
                calls: vec![],
                called_by: vec![],
                is_callback: false,
                callback_context: None,
                attributes: vec![],
            };
            storage.store_function(&func, Path::new("test.c")).unwrap();
        }

        let results = storage.search_functions("driver").unwrap();
        assert_eq!(results.len(), 2);
    }
}
