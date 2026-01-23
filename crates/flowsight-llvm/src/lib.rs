//! FlowSight LLVM IR Parser
//!
//! Parses LLVM IR (.bc and .ll files) to extract:
//! - Function definitions
//! - Function calls
//! - Type information
//! - Basic blocks and branch information
//!
//! This crate provides accurate type and pointer information that complements
//! the Tree-sitter-based source code analysis.

pub mod ir_parser;
pub mod types;

// Re-export types from flowsight_core
pub use flowsight_core::ExecutionContext;

// Re-export our own types
pub use types::{
    IrBasicBlock, IrCall, IrFunction, IrInstruction, IrParameter, IrParseResult, IrType,
};

/// Parsing errors
#[derive(Debug, thiserror::Error)]
pub enum LlvmError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid IR format: {0}")]
    InvalidFormat(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("LLVM error: {0}")]
    LlvmError(String),
}

/// Main parser interface
#[derive(Debug, Default)]
pub struct LlvmParser {
    /// Knowledge base for context annotation
    knowledge_base: Option<ir_parser::KnowledgeBase>,
}

impl LlvmParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            knowledge_base: Some(ir_parser::KnowledgeBase::default()),
        }
    }

    /// Create a parser with custom knowledge base
    pub fn with_knowledge_base(kb: ir_parser::KnowledgeBase) -> Self {
        Self {
            knowledge_base: Some(kb),
        }
    }

    /// Parse an LLVM IR file (.bc or .ll)
    pub fn parse_file(&self, path: &std::path::Path) -> Result<IrParseResult, LlvmError> {
        ir_parser::parse_llvm_ir(path, self.knowledge_base.as_ref())
    }

    /// Parse LLVM IR from a string
    pub fn parse_ir(&self, ir_content: &str) -> Result<IrParseResult, LlvmError> {
        let kb = self.knowledge_base.as_ref().unwrap_or_else(|| {
            static DEFAULT_KB: std::sync::LazyLock<ir_parser::KnowledgeBase> =
                std::sync::LazyLock::new(ir_parser::KnowledgeBase::default);
            &DEFAULT_KB
        });
        ir_parser::parse_llvm_ir_from_str(ir_content, Some(kb))
    }

    /// Get functions by execution context
    pub fn filter_by_context<'a>(
        &'a self,
        result: &'a IrParseResult,
        context: ExecutionContext,
    ) -> Vec<&'a IrFunction> {
        result
            .functions
            .values()
            .filter(|f| f.execution_context == context)
            .collect()
    }

    /// Get callback functions
    pub fn get_callbacks<'a>(&'a self, result: &'a IrParseResult) -> Vec<&'a IrFunction> {
        result
            .functions
            .values()
            .filter(|f| f.is_callback)
            .collect()
    }
}

/// Utility to extract function from ParseResult by name
pub fn find_function<'a>(result: &'a IrParseResult, name: &str) -> Option<&'a IrFunction> {
    result.functions.get(name).or_else(|| {
        // Try to find by suffix (mangled names)
        result.functions.values().find(|f| f.name.ends_with(name))
    })
}

/// Utility to get call graph from ParseResult
pub fn build_call_graph(result: &IrParseResult) -> std::collections::HashMap<String, Vec<String>> {
    let mut call_graph: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    // Initialize all functions
    for func_name in result.functions.keys() {
        call_graph.insert(func_name.clone(), Vec::new());
    }

    // Add call edges
    for call in &result.calls {
        if let Some(callees) = call_graph.get_mut(&call.caller_function) {
            if !callees.contains(&call.callee) {
                callees.push(call.callee.clone());
            }
        }
    }

    call_graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_result_structure() {
        let result = IrParseResult {
            functions: std::collections::HashMap::new(),
            calls: Vec::new(),
            types: std::collections::HashMap::new(),
            module_name: "test".to_string(),
            source_files: Vec::new(),
        };

        assert_eq!(result.module_name, "test");
    }

    #[test]
    fn test_function_structure() {
        let func = IrFunction {
            name: "test_func".to_string(),
            return_type: "i32".to_string(),
            parameters: vec![IrParameter {
                name: "x".to_string(),
                type_str: "i32".to_string(),
            }],
            blocks: Vec::new(),
            is_callback: false,
            callback_context: None,
            execution_context: ExecutionContext::Process,
            source_file: None,
            source_line: None,
        };

        assert_eq!(func.name, "test_func");
        assert!(!func.is_callback);
    }
}
