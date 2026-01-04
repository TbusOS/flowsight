//! FlowSight Query Engine
//!
//! High-level query interface for code analysis.

use flowsight_core::{Result, FunctionDef};
use flowsight_index::SymbolIndex;

/// Query engine
pub struct QueryEngine {
    index: SymbolIndex,
}

impl QueryEngine {
    /// Create a new query engine with in-memory index
    pub fn new() -> Self {
        Self {
            index: SymbolIndex::new(),
        }
    }

    /// Get mutable access to index for adding symbols
    pub fn index_mut(&mut self) -> &mut SymbolIndex {
        &mut self.index
    }

    /// Search for functions by pattern (simple substring match)
    pub fn search_functions(&self, pattern: &str) -> Vec<&FunctionDef> {
        self.index
            .functions
            .values()
            .filter(|f| f.name.contains(pattern))
            .collect()
    }

    /// Get function by name
    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.index.get_function(name)
    }

    /// Get all callback functions
    pub fn get_callbacks(&self) -> Vec<&FunctionDef> {
        self.index
            .functions
            .values()
            .filter(|f| f.is_callback)
            .collect()
    }

    /// Get callers of a function
    pub fn get_callers(&self, name: &str) -> Vec<String> {
        self.index
            .functions
            .values()
            .filter(|f| f.calls.contains(&name.to_string()))
            .map(|f| f.name.clone())
            .collect()
    }

    /// Get callees of a function
    pub fn get_callees(&self, name: &str) -> Vec<String> {
        self.index
            .get_function(name)
            .map(|f| f.calls.clone())
            .unwrap_or_default()
    }

    /// Get the index (for direct access)
    pub fn index(&self) -> &SymbolIndex {
        &self.index
    }
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}
