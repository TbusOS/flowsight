//! User-Assisted Learning System
//!
//! When static analysis cannot determine function pointer targets with certainty,
//! this module provides functionality to:
//! 1. Query the user for help
//! 2. Store user annotations persistently
//! 3. Auto-apply annotations on subsequent analyses

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::classification::UserAnnotation;
use crate::types::{FuncPtrType, TypeDatabase};

/// Error type for learning operations
#[derive(Debug, Error)]
pub enum LearningError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// A query asking the user to help resolve an uncertain call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningQuery {
    /// Unique identifier for this query (e.g., "file.c:100:fp")
    pub id: String,
    /// The call site location (file:line)
    pub location: String,
    /// The expression being called (e.g., "dev->ops->callback")
    pub call_expr: String,
    /// Expected function signature (if known)
    pub expected_signature: Option<String>,
    /// Candidate functions that match the signature
    pub candidates: Vec<Candidate>,
    /// Why we're asking (what analysis couldn't determine)
    pub reason: String,
}

/// A candidate function that might be the call target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    /// Function name
    pub name: String,
    /// Function signature
    pub signature: String,
    /// File where the function is defined
    pub defined_in: String,
    /// Confidence score (0-100) based on type matching
    pub score: u32,
}

impl Candidate {
    pub fn new(name: &str, signature: &str, defined_in: &str, score: u32) -> Self {
        Self {
            name: name.to_string(),
            signature: signature.to_string(),
            defined_in: defined_in.to_string(),
            score,
        }
    }
}

/// Persistent store for user annotations and learned knowledge
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserLearningStore {
    /// User annotations indexed by call site ID
    pub annotations: HashMap<String, UserAnnotation>,
    /// Pending queries that haven't been answered yet
    pub pending_queries: Vec<LearningQuery>,
    /// Statistics
    pub stats: LearningStats,
}

/// Statistics about learning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningStats {
    /// Total number of queries generated
    pub total_queries: u32,
    /// Number of queries answered by user
    pub answered_queries: u32,
    /// Number of annotations auto-applied
    pub auto_applied: u32,
}

impl UserLearningStore {
    /// Create a new empty store
    pub fn new() -> Self {
        Self::default()
    }

    /// Load store from a JSON file
    pub fn load(path: &Path) -> Result<Self, LearningError> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Save store to a JSON file
    pub fn save(&self, path: &Path) -> Result<(), LearningError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a user annotation
    pub fn add_annotation(&mut self, id: &str, annotation: UserAnnotation) {
        self.annotations.insert(id.to_string(), annotation);
        self.stats.answered_queries += 1;
    }

    /// Get annotation for a call site
    pub fn get_annotation(&self, id: &str) -> Option<&UserAnnotation> {
        self.annotations.get(id)
    }

    /// Check if we have an annotation for this call site
    pub fn has_annotation(&self, id: &str) -> bool {
        self.annotations.contains_key(id)
    }

    /// Add a pending query
    pub fn add_query(&mut self, query: LearningQuery) {
        // Don't add duplicate queries
        if !self.pending_queries.iter().any(|q| q.id == query.id) {
            self.pending_queries.push(query);
            self.stats.total_queries += 1;
        }
    }

    /// Get pending queries
    pub fn get_pending_queries(&self) -> &[LearningQuery] {
        &self.pending_queries
    }

    /// Answer a query with user selection
    pub fn answer_query(&mut self, query_id: &str, targets: Vec<String>, note: Option<String>) {
        // Remove from pending
        self.pending_queries.retain(|q| q.id != query_id);

        // Create annotation
        let annotation = UserAnnotation {
            call_site: query_id.to_string(),
            targets,
            note,
        };
        self.add_annotation(query_id, annotation);
    }

    /// Clear all pending queries
    pub fn clear_pending(&mut self) {
        self.pending_queries.clear();
    }

    /// Merge another store into this one
    pub fn merge(&mut self, other: UserLearningStore) {
        for (id, annotation) in other.annotations {
            if !self.annotations.contains_key(&id) {
                self.annotations.insert(id, annotation);
            }
        }
        for query in other.pending_queries {
            self.add_query(query);
        }
    }
}

/// Finds candidate functions based on type matching
pub struct CandidateFinder<'a> {
    /// Type database for signature matching
    #[allow(dead_code)]
    type_db: &'a TypeDatabase,
    /// All known functions with their signatures
    functions: HashMap<String, FunctionInfo>,
}

/// Information about a function
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub return_type: String,
    pub params: Vec<String>,
    pub file: String,
}

impl<'a> CandidateFinder<'a> {
    pub fn new(type_db: &'a TypeDatabase) -> Self {
        Self {
            type_db,
            functions: HashMap::new(),
        }
    }

    /// Register a function
    pub fn add_function(&mut self, info: FunctionInfo) {
        self.functions.insert(info.name.clone(), info);
    }

    /// Find candidates that match a function pointer type
    pub fn find_candidates(&self, funcptr_type: &FuncPtrType) -> Vec<Candidate> {
        let mut candidates = Vec::new();

        for (name, info) in &self.functions {
            let score = self.compute_match_score(funcptr_type, info);
            if score > 0 {
                candidates.push(Candidate {
                    name: name.clone(),
                    signature: self.format_signature(info),
                    defined_in: info.file.clone(),
                    score,
                });
            }
        }

        // Sort by score descending
        candidates.sort_by(|a, b| b.score.cmp(&a.score));
        candidates
    }

    /// Find candidates by signature string
    pub fn find_by_signature(&self, signature: &str) -> Vec<Candidate> {
        let mut candidates = Vec::new();

        // Parse expected return type and params from signature
        let expected = self.parse_signature(signature);

        for (name, info) in &self.functions {
            let score = self.compute_signature_match(&expected, info);
            if score > 0 {
                candidates.push(Candidate {
                    name: name.clone(),
                    signature: self.format_signature(info),
                    defined_in: info.file.clone(),
                    score,
                });
            }
        }

        candidates.sort_by(|a, b| b.score.cmp(&a.score));
        candidates
    }

    /// Compute match score between function pointer type and function
    fn compute_match_score(&self, funcptr: &FuncPtrType, func: &FunctionInfo) -> u32 {
        let mut score = 0;

        // Return type match
        if self.types_compatible(&funcptr.return_type, &func.return_type) {
            score += 40;
        }

        // Parameter count match
        if funcptr.param_types.len() == func.params.len() {
            score += 30;

            // Parameter type match
            let mut param_score = 0;
            for (expected, actual) in funcptr.param_types.iter().zip(func.params.iter()) {
                if self.types_compatible(expected, actual) {
                    param_score += 30 / funcptr.param_types.len().max(1) as u32;
                }
            }
            score += param_score;
        }

        score
    }

    fn compute_signature_match(&self, expected: &(String, Vec<String>), func: &FunctionInfo) -> u32 {
        let mut score = 0;

        // Return type match
        if self.types_compatible(&expected.0, &func.return_type) {
            score += 40;
        }

        // Parameter count match
        if expected.1.len() == func.params.len() {
            score += 30;

            // Parameter type match
            for (exp, act) in expected.1.iter().zip(func.params.iter()) {
                if self.types_compatible(exp, act) {
                    score += 30 / expected.1.len().max(1) as u32;
                }
            }
        }

        score
    }

    fn types_compatible(&self, a: &str, b: &str) -> bool {
        let a = a.trim();
        let b = b.trim();

        // Exact match
        if a == b {
            return true;
        }

        // Normalize void *
        if (a == "void *" || a == "void*") && (b == "void *" || b == "void*") {
            return true;
        }

        // struct pointers are compatible with void*
        if (a.contains("struct") && a.contains('*') && (b == "void *" || b == "void*"))
            || (b.contains("struct") && b.contains('*') && (a == "void *" || a == "void*"))
        {
            return true;
        }

        // int/long compatibility
        if (a == "int" || a == "long") && (b == "int" || b == "long") {
            return true;
        }

        false
    }

    fn format_signature(&self, info: &FunctionInfo) -> String {
        format!(
            "{} {}({})",
            info.return_type,
            info.name,
            info.params.join(", ")
        )
    }

    fn parse_signature(&self, sig: &str) -> (String, Vec<String>) {
        // Simple parsing: "return_type (param1, param2)"
        let sig = sig.trim();

        if let Some(paren_pos) = sig.find('(') {
            let ret = sig[..paren_pos].trim().to_string();
            let params_str = &sig[paren_pos + 1..sig.len() - 1];
            let params: Vec<String> = params_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            (ret, params)
        } else {
            (sig.to_string(), Vec::new())
        }
    }
}

/// Builder for creating learning queries
pub struct QueryBuilder {
    location: String,
    call_expr: String,
    expected_signature: Option<String>,
    candidates: Vec<Candidate>,
    reason: String,
}

impl QueryBuilder {
    pub fn new(location: &str, call_expr: &str) -> Self {
        Self {
            location: location.to_string(),
            call_expr: call_expr.to_string(),
            expected_signature: None,
            candidates: Vec::new(),
            reason: "Unknown function pointer target".to_string(),
        }
    }

    pub fn with_signature(mut self, sig: &str) -> Self {
        self.expected_signature = Some(sig.to_string());
        self
    }

    pub fn with_candidates(mut self, candidates: Vec<Candidate>) -> Self {
        self.candidates = candidates;
        self
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = reason.to_string();
        self
    }

    pub fn build(self) -> LearningQuery {
        let id = format!("{}:{}", self.location, self.call_expr);
        LearningQuery {
            id,
            location: self.location,
            call_expr: self.call_expr,
            expected_signature: self.expected_signature,
            candidates: self.candidates,
            reason: self.reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_store_basic() {
        let mut store = UserLearningStore::new();

        assert!(!store.has_annotation("test:100"));

        store.add_annotation(
            "test:100",
            UserAnnotation {
                call_site: "test:100".to_string(),
                targets: vec!["handler_func".to_string()],
                note: None,
            },
        );

        assert!(store.has_annotation("test:100"));
        assert_eq!(
            store.get_annotation("test:100").unwrap().targets,
            vec!["handler_func"]
        );
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new("file.c:100", "dev->callback")
            .with_signature("int (void *)")
            .with_reason("Cannot resolve ops table callback")
            .build();

        assert_eq!(query.location, "file.c:100");
        assert_eq!(query.call_expr, "dev->callback");
        assert_eq!(query.expected_signature, Some("int (void *)".to_string()));
    }

    #[test]
    fn test_candidate_finder() {
        use crate::types::TypeDatabase;

        let type_db = TypeDatabase::new();
        let mut finder = CandidateFinder::new(&type_db);

        // Add some functions
        finder.add_function(FunctionInfo {
            name: "my_read".to_string(),
            return_type: "int".to_string(),
            params: vec!["void *".to_string(), "int".to_string()],
            file: "driver.c".to_string(),
        });

        finder.add_function(FunctionInfo {
            name: "my_write".to_string(),
            return_type: "int".to_string(),
            params: vec!["void *".to_string(), "int".to_string()],
            file: "driver.c".to_string(),
        });

        finder.add_function(FunctionInfo {
            name: "unrelated".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            file: "other.c".to_string(),
        });

        // Create a function pointer type to match against
        let funcptr = FuncPtrType {
            name: "read_func_t".to_string(),
            return_type: "int".to_string(),
            param_types: vec!["void *".to_string(), "int".to_string()],
            location: None,
            definition_kind: crate::types::FuncPtrDefKind::Typedef,
        };

        let candidates = finder.find_candidates(&funcptr);

        // Should find my_read and my_write but not unrelated
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().any(|c| c.name == "my_read"));
        assert!(candidates.iter().any(|c| c.name == "my_write"));
    }

    #[test]
    fn test_store_persistence() {
        let mut store = UserLearningStore::new();

        store.add_annotation(
            "test:50",
            UserAnnotation {
                call_site: "test:50".to_string(),
                targets: vec!["func_a".to_string(), "func_b".to_string()],
                note: Some("User identified these".to_string()),
            },
        );

        let query = QueryBuilder::new("test:100", "fp()")
            .with_reason("Cannot determine target")
            .build();
        store.add_query(query);

        // Serialize and deserialize
        let json = serde_json::to_string(&store).unwrap();
        let loaded: UserLearningStore = serde_json::from_str(&json).unwrap();

        assert!(loaded.has_annotation("test:50"));
        assert_eq!(loaded.pending_queries.len(), 1);
    }

    #[test]
    fn test_answer_query() {
        let mut store = UserLearningStore::new();

        let query = QueryBuilder::new("file.c:200", "callback()")
            .with_reason("Unknown target")
            .build();

        let query_id = query.id.clone();
        store.add_query(query);

        assert_eq!(store.pending_queries.len(), 1);

        // Answer the query
        store.answer_query(
            &query_id,
            vec!["my_handler".to_string()],
            Some("Confirmed by reading documentation".to_string()),
        );

        // Query should be removed from pending
        assert_eq!(store.pending_queries.len(), 0);

        // Annotation should be added
        assert!(store.has_annotation(&query_id));
    }
}
