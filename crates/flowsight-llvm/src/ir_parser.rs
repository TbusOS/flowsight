//! LLVM IR Parser Implementation
//!
//! This module provides LLVM IR parsing capabilities.
//! For full functionality, the inkwell crate can be enabled with the `llvm*-0` features.

use crate::types::{IrType, StructType};
use crate::{
    IrBasicBlock, IrCall, IrFunction, IrInstruction, IrParameter, IrParseResult, LlvmError,
};
use flowsight_core::ExecutionContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Knowledge base for context annotation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeBase {
    /// Known callback patterns
    pub callback_patterns: Vec<CallbackPattern>,
    /// Known execution contexts
    pub context_patterns: Vec<ContextPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackPattern {
    /// Pattern name
    pub name: String,
    /// Regex pattern for detection
    pub pattern: String,
    /// Callback type
    pub callback_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPattern {
    /// Pattern name
    pub name: String,
    /// Regex pattern
    pub pattern: String,
    /// Execution context
    pub context: ExecutionContext,
}

impl KnowledgeBase {
    /// Get default knowledge base
    pub fn default() -> Self {
        Self {
            callback_patterns: vec![
                CallbackPattern {
                    name: "workqueue_handler".to_string(),
                    pattern: r"@\.work\.".to_string(),
                    callback_type: "workqueue".to_string(),
                },
                CallbackPattern {
                    name: "irq_handler".to_string(),
                    pattern: r"@\.irq\.".to_string(),
                    callback_type: "irq".to_string(),
                },
                CallbackPattern {
                    name: "timer_handler".to_string(),
                    pattern: r"@\.timer\.".to_string(),
                    callback_type: "timer".to_string(),
                },
            ],
            context_patterns: vec![
                ContextPattern {
                    name: "process_context".to_string(),
                    pattern: r"(call|invoke).*@.*workqueue.*".to_string(),
                    context: ExecutionContext::Process,
                },
                ContextPattern {
                    name: "softirq_context".to_string(),
                    pattern: r"(call|invoke).*@.*softirq.*".to_string(),
                    context: ExecutionContext::SoftIrq,
                },
            ],
        }
    }
}

/// Parse LLVM IR from a file
pub fn parse_llvm_ir(
    path: &Path,
    knowledge_base: Option<&KnowledgeBase>,
) -> Result<IrParseResult, LlvmError> {
    let content = std::fs::read_to_string(path).map_err(|e| LlvmError::IoError(e))?;
    parse_llvm_ir_from_str(&content, knowledge_base)
}

/// Parse LLVM IR from a string
pub fn parse_llvm_ir_from_str(
    ir_content: &str,
    knowledge_base: Option<&KnowledgeBase>,
) -> Result<IrParseResult, LlvmError> {
    let kb = knowledge_base.unwrap_or_else(|| {
        static DEFAULT_KB: std::sync::LazyLock<KnowledgeBase> =
            std::sync::LazyLock::new(KnowledgeBase::default);
        &DEFAULT_KB
    });

    // Detect IR format (bitcode vs text)
    if ir_content.starts_with("BC") || ir_content.starts_with("\u{00}BC") {
        // Bitcode format - requires LLVM library
        // For now, return an error with helpful message
        return Err(LlvmError::LlvmError(
            "Bitcode parsing requires inkwell LLVM bindings. \
             Please compile with --features llvm17 or use .ll text format."
                .to_string(),
        ));
    }

    // Parse as text IR
    parse_ll_text(ir_content, kb)
}

/// Parse LLVM IR text format (.ll)
fn parse_ll_text(content: &str, kb: &KnowledgeBase) -> Result<IrParseResult, LlvmError> {
    // Extract module name
    let module_name = extract_module_name(content);

    // Extract source files
    let source_files = extract_source_files(content);

    // Extract type definitions
    let types = extract_types(content);

    // Extract functions
    let func_sections = split_into_functions(content);
    let mut functions: HashMap<String, IrFunction> = HashMap::new();

    for func_section in func_sections {
        if let Some(func) = parse_function_section(&func_section, kb) {
            functions.insert(func.name.clone(), func);
        }
    }

    // Extract calls
    let calls = extract_calls(content, &functions);

    Ok(IrParseResult {
        functions,
        calls,
        types,
        module_name,
        source_files,
    })
}

/// Extract module name from IR
fn extract_module_name(content: &str) -> String {
    // Look for ; ModuleID = "name"
    for line in content.lines() {
        if line.trim_start().starts_with("; ModuleID") {
            if let Some(name) = line.split('"').nth(1) {
                return name.to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Extract source file references
fn extract_source_files(content: &str) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();

    for line in content.lines() {
        if line.trim_start().starts_with("source_filename") {
            if let Some(name) = line.split('"').nth(1) {
                if !files.contains(&name.to_string()) {
                    files.push(name.to_string());
                }
            }
        }
    }

    files
}

/// Extract type definitions
fn extract_types(content: &str) -> HashMap<String, IrType> {
    let mut types: HashMap<String, IrType> = HashMap::new();

    // Simple pattern matching for named types
    let type_re = regex::Regex::new(r"@(\w+)\s*=\s*(distinct\s+)?(struct|type)\s*\{([^}]+)\}").unwrap();

    for cap in type_re.captures_iter(content) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let fields_str = cap.get(4).unwrap().as_str();

        let fields: Vec<IrType> = fields_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(|s| IrType::parse_from_str(s))
            .collect();

        types.insert(
            name.clone(),
            IrType::Struct(StructType {
                name: Some(name),
                elements: fields,
                is_packed: false,
            }),
        );
    }

    types
}

/// Split content into individual function sections
fn split_into_functions(content: &str) -> Vec<String> {
    let mut functions: Vec<String> = Vec::new();
    let mut current_func = String::new();
    let mut in_function = false;
    let mut brace_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and metadata
        if trimmed.starts_with(';') || trimmed.starts_with('!') {
            continue;
        }

        // Check for function definition
        if trimmed.starts_with("define") {
            in_function = true;
            brace_count = 0;
        }

        if in_function {
            current_func.push_str(line);
            current_func.push('\n');

            brace_count += line.matches('{').count();
            brace_count = brace_count.saturating_sub(line.matches('}').count());

            if brace_count == 0 && !current_func.trim_end().ends_with('{') {
                // End of function
                functions.push(current_func.clone());
                current_func.clear();
                in_function = false;
            }
        }
    }

    functions
}

/// Parse a single function section
fn parse_function_section(section: &str, kb: &KnowledgeBase) -> Option<IrFunction> {
    // Parse function header: define ret_type @func_name(params) ...
    let header_re = regex::Regex::new(
        r"define\s+(?:hidden\s+)?(?P<ret>[^@]+)@(?P<name>[\w\.]+)\s*\((?P<params>[^)]*)\)",
    )
    .unwrap();

    let header_cap = header_re.captures(section)?;

    let name = header_cap.name("name")?.as_str().to_string();
    let return_type = header_cap.name("ret")?.as_str().trim().to_string();
    let params_str = header_cap.name("params")?.as_str();

    // Parse parameters
    let parameters = parse_parameters(params_str);

    // Extract basic blocks
    let blocks = extract_basic_blocks(section);

    // Determine if this is a callback
    let (is_callback, callback_context) = detect_callback(&name, &parameters, kb);

    // Determine execution context
    let execution_context = detect_context(&name, &parameters, kb);

    Some(IrFunction {
        name,
        return_type,
        parameters,
        blocks,
        is_callback,
        callback_context,
        execution_context,
        source_file: None,
        source_line: None,
    })
}

/// Parse function parameters
fn parse_parameters(params_str: &str) -> Vec<IrParameter> {
    let mut params: Vec<IrParameter> = Vec::new();

    // Split by comma, but handle nested structures
    let parts = split_params(params_str);

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parameter format: type name or type %name
        let parts: Vec<&str> = trimmed.rsplitn(2, ' ').collect();
        if parts.len() == 2 {
            let name = parts[0].trim().trim_start_matches('%').to_string();
            let type_str = parts[1].trim().to_string();
            params.push(IrParameter { name, type_str });
        } else {
            // Maybe just type
            params.push(IrParameter {
                name: String::new(),
                type_str: trimmed.to_string(),
            });
        }
    }

    params
}

/// Split parameters handling nested structures
fn split_params(s: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;

    for c in s.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(c);
            }
            ',' if depth == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

/// Extract basic blocks from function
fn extract_basic_blocks(func_body: &str) -> Vec<IrBasicBlock> {
    let mut blocks: Vec<IrBasicBlock> = Vec::new();

    // Split by labels (lines ending with :)
    let block_re = regex::Regex::new(r"(?m)^(\w+):\s*(?:;.*)?$").unwrap();

    let labels: Vec<(String, usize)> = block_re
        .captures_iter(func_body)
        .map(|cap| {
            let name = cap.get(1).unwrap().as_str().to_string();
            let pos = cap.get(0).unwrap().start();
            (name, pos)
        })
        .collect();

    // Extract instructions between labels
    for (i, (name, _)) in labels.iter().enumerate() {
        let start = labels[i].1;
        let end = labels.get(i + 1).map(|(_, p)| *p).unwrap_or(func_body.len());

        let block_content = &func_body[start..end];
        let instructions = extract_instructions(block_content);

        // Extract terminator before moving instructions
        let terminator = instructions.last().filter(|i| is_terminator(&i.opcode)).cloned();

        // Find successors (branches)
        let successors = find_successors(block_content);

        // Find predecessors (from other blocks)
        let predecessors = find_predecessors(func_body, name);

        blocks.push(IrBasicBlock {
            name: name.clone(),
            instructions,
            terminator,
            predecessors,
            successors,
        });
    }

    blocks
}

/// Extract instructions from a block
fn extract_instructions(block_content: &str) -> Vec<IrInstruction> {
    let mut instructions: Vec<IrInstruction> = Vec::new();

    // Match LLVM instructions (exclude labels)
    let instr_re = regex::Regex::new(
        r"(?m)^\s+(?:%)?(\w+)\s*=\s*([\w.]+)([^;]*?)(;.*)?$",
    ).unwrap();

    for cap in instr_re.captures_iter(block_content) {
        let dest = cap.get(1).map(|m| m.as_str().to_string());
        let opcode = cap.get(2).unwrap().as_str().to_string();
        let operands = cap
            .get(3)
            .unwrap()
            .as_str()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        instructions.push(IrInstruction {
            opcode,
            dest,
            type_str: String::new(),
            operands,
            location: None,
        });
    }

    instructions
}

/// Check if instruction is a terminator
fn is_terminator(opcode: &str) -> bool {
    matches!(
        opcode,
        "ret" | "br" | "switch" | "invoke" | "resume" | "unreachable"
    )
}

/// Find successor blocks
fn find_successors(block_content: &str) -> Vec<String> {
    let mut successors: Vec<String> = Vec::new();

    // Look for br and switch instructions
    let br_re = regex::Regex::new(r"br\s+(?:i1\s+\w+\s*,?\s*)?label\s+%" ).unwrap();
    let switch_re = regex::Regex::new(r"switch\s+.*label\s+%" ).unwrap();

    for cap in br_re.captures_iter(block_content) {
        let rest = cap.get(0).unwrap().as_str();
        if let Some(label) = rest.split("label %").nth(1) {
            let name = label.split_whitespace().next().unwrap_or("").to_string();
            if !name.is_empty() {
                successors.push(name);
            }
        }
    }

    for cap in switch_re.captures_iter(block_content) {
        let rest = cap.get(0).unwrap().as_str();
        let parts: Vec<&str> = rest.split("label %").collect();
        for part in parts.iter().skip(1) {
            let name = part.split_whitespace().next().unwrap_or("").to_string();
            if !name.is_empty() && !successors.contains(&name) {
                successors.push(name);
            }
        }
    }

    successors
}

/// Find predecessor blocks
fn find_predecessors(func_body: &str, target: &str) -> Vec<String> {
    let mut predecessors: Vec<String> = Vec::new();

    // Look for branches to target
    let br_re = regex::Regex::new(&format!(r"br\s+.*label\s+%{}", regex::escape(target))).unwrap();

    for cap in br_re.captures_iter(func_body) {
        // Find which block this branch is in
        let match_pos = cap.get(0).unwrap().start();
        let before = &func_body[..match_pos];
        let block_re = regex::Regex::new(r"(?m)^(\w+):").unwrap();

        if let Some(cap) = block_re.captures_iter(before).last() {
            let pred_name = cap.get(1).unwrap().as_str().to_string();
            if !predecessors.contains(&pred_name) {
                predecessors.push(pred_name);
            }
        }
    }

    predecessors
}

/// Extract function calls
fn extract_calls(content: &str, _functions: &HashMap<String, IrFunction>) -> Vec<IrCall> {
    let mut calls: Vec<IrCall> = Vec::new();

    // Match call and invoke instructions
    let call_re = regex::Regex::new(
        r"(?m)^\s+(?:%)?\w+\s*=\s*(call|invoke)\s+(?:cold\s+)?(?:<[^>]*>\s*)?(?P<attrs>[^@]*?)@(?P<func>[\w\.]+)\s*\((?P<args>[^)]*)\)",
    ).unwrap();

    // Build function-to-block mapping
    let mut func_of_block: HashMap<String, String> = HashMap::new();
    let define_re = regex::Regex::new(r"define\s+[^\@]+@(\w+)\s*\(").unwrap();
    let block_re = regex::Regex::new(r"(?m)^(\w+):").unwrap();

    // Track current function as we iterate through blocks
    let mut current_func: Option<String> = None;
    for line in content.lines() {
        // Check if we're entering a new function
        if let Some(cap) = define_re.captures(line) {
            current_func = Some(cap.get(1).unwrap().as_str().to_string());
        }
        // Check for blocks
        if line.trim_start().ends_with(':') && !line.trim_start().starts_with(';') {
            if let Some(cap) = block_re.captures(line) {
                let block_name = cap.get(1).unwrap().as_str().to_string();
                if let Some(ref func) = current_func {
                    func_of_block.insert(block_name.clone(), func.clone());
                }
            }
        }
        // Check for end of function (next define or end of content)
        if line.trim_start().starts_with("define") && current_func.is_some() {
            // This is a new function, reset
        }
    }

    for cap in call_re.captures_iter(content) {
        let _call_type = cap.get(1).unwrap().as_str();
        let callee = cap.get(3).unwrap().as_str().to_string();
        let args_str = cap.get(4).unwrap().as_str();

        // Skip intrinsics and standard library
        if callee.starts_with("llvm.") || callee.starts_with("@llvm") {
            continue;
        }

        // Determine caller block
        let call_pos = cap.get(0).unwrap().start();
        let block_positions: Vec<(String, usize)> = block_re
            .captures_iter(content)
            .filter_map(|cap| {
                let name = cap.get(1).unwrap().as_str().to_string();
                let pos = cap.get(0).unwrap().start();
                if pos < call_pos {
                    Some((name, pos))
                } else {
                    None
                }
            })
            .collect();

        let caller_block = block_positions
            .iter()
            .max_by_key(|(_, p)| *p)
            .map(|(n, _)| n.clone())
            .unwrap_or_else(|| String::from("unknown"));

        // Determine caller function from block
        let caller_function = func_of_block
            .get(&caller_block)
            .cloned()
            .unwrap_or_else(|| String::from("unknown"));

        // Parse arguments
        let arguments: Vec<String> = args_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Check if indirect call
        let is_indirect = callee.contains('$') || callee.contains("ptr");

        calls.push(IrCall {
            caller_block,
            caller_function,
            callee,
            is_indirect,
            arguments,
        });
    }

    calls
}

/// Detect if a function is a callback
fn detect_callback(name: &str, _params: &[IrParameter], kb: &KnowledgeBase) -> (bool, Option<String>) {
    for pattern in &kb.callback_patterns {
        let regex = regex::RegexBuilder::new(&pattern.pattern)
            .case_insensitive(true)
            .build()
            .ok();

        if let Some(re) = regex {
            if re.is_match(name) {
                return (true, Some(pattern.callback_type.clone()));
            }
        }
    }

    (false, None)
}

/// Detect execution context
fn detect_context(name: &str, params: &[IrParameter], kb: &KnowledgeBase) -> ExecutionContext {
    for pattern in &kb.context_patterns {
        let regex = regex::RegexBuilder::new(&pattern.pattern)
            .case_insensitive(true)
            .build()
            .ok();

        if let Some(re) = regex {
            if re.is_match(name) {
                return pattern.context.clone();
            }
        }
    }

    // Check parameter types for context hints
    for param in params {
        if param.type_str.contains("work_struct") {
            return ExecutionContext::Process;
        }
        if param.type_str.contains("timer_list") || param.type_str.contains("hrtimer") {
            return ExecutionContext::SoftIrq;
        }
        if param.type_str.contains("irqreturn") || param.type_str.contains("irq_handler") {
            return ExecutionContext::HardIrq;
        }
    }

    ExecutionContext::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parameters() {
        let params = parse_parameters("i32 %x, i64 %y, ptr %ptr");
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name, "x");
        assert_eq!(params[0].type_str, "i32");
        assert_eq!(params[1].name, "y");
        assert_eq!(params[2].name, "ptr");
    }

    #[test]
    fn test_split_params() {
        let result = split_params("i32, ptr, i64 %val");
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_simple_function() {
        let ir = r#"
define i32 @test_func(i32 %x) {
entry:
  %cmp = icmp sgt i32 %x, 0
  br i1 %cmp, label %then, label %else
then:
  ret i32 1
else:
  ret i32 0
}
"#;

        let result = parse_ll_text(ir, &KnowledgeBase::default()).unwrap();
        assert!(result.functions.contains_key("test_func"));
    }

    #[test]
    fn test_extract_calls() {
        let ir = r#"
define i32 @caller() {
entry:
  %result = call i32 @callee(i32 42)
  ret i32 %result
}

define i32 @callee(i32 %x) {
  ret i32 %x
}
"#;

        let result = parse_ll_text(ir, &KnowledgeBase::default()).unwrap();
        assert_eq!(result.calls.len(), 1);
        assert_eq!(result.calls[0].caller_function, "caller");
        assert_eq!(result.calls[0].callee, "callee");
    }
}
