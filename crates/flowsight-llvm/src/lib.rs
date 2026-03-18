#![forbid(unsafe_code)]
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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod ir_parser;
pub mod types;

// Re-export types from flowsight_core
pub use flowsight_core::ExecutionContext;

// Re-export our own types
pub use types::{
    LlvmBasicBlock, LlvmFunction, LlvmInstruction, LlvmIrPageRequest, LlvmIrPageResponse,
    LlvmIrParseResult, LlvmParameter,
};

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

/// Convert internal IrParseResult to frontend-compatible LlvmIrParseResult
pub fn to_frontend_format(result: &IrParseResult) -> LlvmIrParseResult {
    let mut functions: HashMap<String, LlvmFunction> = HashMap::new();

    for (name, ir_func) in &result.functions {
        let blocks: Vec<LlvmBasicBlock> = ir_func
            .blocks
            .iter()
            .map(|block| LlvmBasicBlock {
                name: block.name.clone(),
                instructions: block
                    .instructions
                    .iter()
                    .map(|instr| LlvmInstruction {
                        opcode: instr.opcode.clone(),
                        dest: instr.dest.clone(),
                        type_str: instr.type_str.clone(),
                        operands: instr.operands.clone(),
                        location: instr.location.clone(),
                    })
                    .collect(),
                predecessors: block.predecessors.clone(),
                successors: block.successors.clone(),
                terminator: block.terminator.as_ref().map(|instr| LlvmInstruction {
                    opcode: instr.opcode.clone(),
                    dest: instr.dest.clone(),
                    type_str: instr.type_str.clone(),
                    operands: instr.operands.clone(),
                    location: instr.location.clone(),
                }),
            })
            .collect();

        let parameters: Vec<LlvmParameter> = ir_func
            .parameters
            .iter()
            .map(|p| LlvmParameter {
                name: p.name.clone(),
                type_str: p.type_str.clone(),
            })
            .collect();

        functions.insert(
            name.clone(),
            LlvmFunction {
                name: ir_func.name.clone(),
                return_type: ir_func.return_type.clone(),
                parameters,
                blocks,
                is_callback: ir_func.is_callback,
                callback_context: ir_func.callback_context.clone(),
            },
        );
    }

    LlvmIrParseResult {
        module_name: result.module_name.clone(),
        functions,
    }
}

/// Paginate instructions within a function
pub fn paginate_function(
    result: &LlvmIrParseResult,
    request: &LlvmIrPageRequest,
) -> Option<LlvmIrPageResponse> {
    let func = result.functions.get(&request.function_name)?;

    // Collect all instructions (including terminators)
    let all_instructions: Vec<&LlvmInstruction> = func
        .blocks
        .iter()
        .flat_map(|block| block.instructions.iter().chain(block.terminator.as_ref()))
        .collect();

    let total_instructions = all_instructions.len() as u32;
    let total_pages = (total_instructions as f64 / request.page_size as f64).ceil() as u32;

    let start = (request.page * request.page_size) as usize;
    let end = ((request.page + 1) * request.page_size) as usize;
    let page_instructions: Vec<LlvmInstruction> = all_instructions
        .get(start..end.min(all_instructions.len()))
        .map(|slice| slice.iter().map(|instr| (*instr).clone()).collect())
        .unwrap_or_default();

    Some(LlvmIrPageResponse {
        function_name: request.function_name.clone(),
        page: request.page,
        page_size: request.page_size,
        total_instructions,
        total_pages,
        has_next: request.page + 1 < total_pages,
        has_previous: request.page > 0,
        instructions: page_instructions,
        block_name: None,
    })
}

/// Get function summary (for quick display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSummary {
    pub name: String,
    pub return_type: String,
    pub param_count: usize,
    pub instruction_count: u32,
    pub block_count: usize,
    pub is_callback: bool,
}

/// Get summary of all functions in a parse result
pub fn get_function_summaries(result: &LlvmIrParseResult) -> Vec<FunctionSummary> {
    result
        .functions
        .values()
        .map(|func| {
            let instruction_count: u32 = func
                .blocks
                .iter()
                .map(|block| {
                    block.instructions.len() as u32 + block.terminator.as_ref().map_or(0, |_| 1u32)
                })
                .sum();

            FunctionSummary {
                name: func.name.clone(),
                return_type: func.return_type.clone(),
                param_count: func.parameters.len(),
                instruction_count,
                block_count: func.blocks.len(),
                is_callback: func.is_callback,
            }
        })
        .collect()
}

/// Utility to get call graph from ParseResult
pub fn build_call_graph(result: &IrParseResult) -> std::collections::HashMap<String, Vec<String>> {
    let mut call_graph: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

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

    #[test]
    #[ignore = "Requires /tmp/test_simple.ll - run with: cargo test --package flowsight-llvm -- --ignored"]
    fn test_parse_real_bitcode() {
        // Test parsing a real bitcode file generated by clang-18
        // Note: .bc files require inkwell for parsing, so we test with .ll (text IR)
        // To run: 1. Generate test_simple.ll with clang
        //         2. cargo test --package flowsight-llvm -- --ignored
        let parser = LlvmParser::new();
        let result = parser.parse_file(std::path::Path::new("/tmp/test_simple.ll"));

        match result {
            Ok(parse_result) => {
                // Should find 3 functions: add, process_data, callback_handler
                assert!(
                    parse_result.functions.len() >= 3,
                    "Expected at least 3 functions, found {}",
                    parse_result.functions.len()
                );

                // Verify we can find the main functions
                assert!(
                    parse_result.functions.contains_key("add"),
                    "Should contain 'add' function"
                );
                assert!(
                    parse_result.functions.contains_key("process_data"),
                    "Should contain 'process_data' function"
                );
                assert!(
                    parse_result.functions.contains_key("callback_handler"),
                    "Should contain 'callback_handler' function"
                );

                // Check function return types
                let add_func = parse_result.functions.get("add").unwrap();
                assert_eq!(add_func.return_type, "i32");

                let process_func = parse_result.functions.get("process_data").unwrap();
                assert_eq!(process_func.return_type, "i32");
                assert_eq!(process_func.parameters.len(), 2);

                // Check call graph
                // callback_handler should call the callback function pointer
                let calls: Vec<_> = parse_result
                    .calls
                    .iter()
                    .filter(|c| c.caller_function == "callback_handler")
                    .collect();
                assert!(!calls.is_empty(), "callback_handler should make a call");
            }
            Err(e) => {
                panic!("Failed to parse real bitcode: {:?}", e);
            }
        }
    }
}
