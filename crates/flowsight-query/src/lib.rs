//! FlowSight Query Engine
//!
//! High-level query interface for code analysis.

use flowsight_core::{Result, FunctionDef, FlowNode};
use flowsight_index::SymbolIndex;

/// Query engine
pub struct QueryEngine {
    index: SymbolIndex,
}

impl QueryEngine {
    /// Create a new query engine with in-memory index
    pub fn new() -> Result<Self> {
        Ok(Self {
            index: SymbolIndex::new_memory()?,
        })
    }

    /// Search for functions
    pub fn search_functions(&self, pattern: &str) -> Result<Vec<FunctionDef>> {
        self.index.search_functions(pattern)
    }

    /// Get function by name
    pub fn get_function(&self, name: &str) -> Result<Option<FunctionDef>> {
        self.index.get_function(name)
    }

    /// Get all callback functions
    pub fn get_callbacks(&self) -> Result<Vec<FunctionDef>> {
        self.index.get_callbacks()
    }

    /// Get callers of a function
    pub fn get_callers(&self, name: &str) -> Result<Vec<String>> {
        self.index.get_callers(name)
    }

    /// Get callees of a function
    pub fn get_callees(&self, name: &str) -> Result<Vec<String>> {
        self.index.get_callees(name)
    }

    /// Get the index (for direct access)
    pub fn index(&self) -> &SymbolIndex {
        &self.index
    }
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create query engine")
    }
}

