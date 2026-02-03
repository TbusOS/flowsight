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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
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
    let content = std::fs::read_to_string(path).map_err(LlvmError::IoError)?;
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
    // Look for ; ModuleID = "name" or 'name' first
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("; ModuleID") {
            // Try double quotes first, then single quotes
            if let Some(name) = trimmed.split('"').nth(1) {
                return name.to_string();
            }
            if let Some(name) = trimmed.split('\'').nth(1) {
                return name.to_string();
            }
        }
    }

    // Fall back to source_filename
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("source_filename") {
            if let Some(name) = trimmed.split('"').nth(1) {
                // Extract basename from path and remove extension
                let basename = name.rsplit('/').next().unwrap_or(name);
                return basename.to_string();
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

    // Match named types: %struct_name = type { ... }
    // Handles: %struct_name = type { field1, field2 }
    let type_re = regex::Regex::new(r"%([\w.]+)\s*=\s*type\s*\{([^}]+)\}").unwrap();

    for cap in type_re.captures_iter(content) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let fields_str = cap.get(2).unwrap().as_str();

        let fields: Vec<IrType> = fields_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(IrType::parse_from_str)
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
            current_func.clear();
        }

        if in_function {
            current_func.push_str(line);
            current_func.push('\n');

            brace_count += line.matches('{').count();
            brace_count = brace_count.saturating_sub(line.matches('}').count());

            // End of function: when brace count is 0 and we have a closing brace
            if brace_count == 0 && trimmed.ends_with('}') {
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
    let mut return_type = header_cap.name("ret")?.as_str().trim().to_string();

    // Clean up return type - remove function attributes and keywords
    // Common attributes: noinline, inline, always_inline, uwtable, ssp, sspreg,
    // noreturn, nounwind, nonnull, readnone, readonly, argmemonly, protected, dso_local, etc.
    let cleanup_re = regex::Regex::new(r"\b(noinline|inline|always_inline|uwtable|ssp|sspreg|noreturn|nounwind|nonnull|readnone|readonly|argmemonly|memtag_safety|shadowcallstack|protected|visibility|dso_local|#\d+)\b").unwrap();
    return_type = cleanup_re.replace_all(&return_type, "").trim().to_string();

    // Also remove any extra whitespace
    return_type = return_type.split_whitespace().collect::<Vec<_>>().join(" ");

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
    // Note: [\w.]+ matches word characters AND dots (for labels like 'for.body')
    let block_re = regex::Regex::new(r"(?m)^([\w.]+):\s*(?:;.*)?$").unwrap();

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
        let end = labels
            .get(i + 1)
            .map(|(_, p)| *p)
            .unwrap_or(func_body.len());

        let block_content = &func_body[start..end];
        let instructions = extract_instructions(block_content);

        // Extract terminator before moving instructions
        let terminator = instructions
            .last()
            .filter(|i| is_terminator(&i.opcode))
            .cloned();

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

    // Match LLVM instructions - includes instructions without destination (like ret, br)
    // Pattern matches:
    //   %result = add i32 %a, %b  (instruction with destination)
    //   ret i32 1                   (terminator without destination)
    //   br label %dest              (branch without destination)
    let instr_re = regex::Regex::new(
        r"(?m)^\s+(?:%)?(\w+)\s*=\s*([\w.]+)([^;]*?)(;.*)?$|^\s+([\w.]+)([^;]*?)(;.*)?$",
    )
    .unwrap();

    for cap in instr_re.captures_iter(block_content) {
        // Check if we matched the first pattern (with destination)
        if let Some(dest_match) = cap.get(1) {
            let dest = Some(dest_match.as_str().to_string());
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
        } else if let Some(opcode_match) = cap.get(5) {
            // Matched the second pattern (no destination)
            let opcode = opcode_match.as_str().to_string();
            let operands = cap
                .get(6)
                .unwrap()
                .as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            instructions.push(IrInstruction {
                opcode,
                dest: None,
                type_str: String::new(),
                operands,
                location: None,
            });
        }
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
    let br_re = regex::Regex::new(r"br\s+(?:i1\s+\w+\s*,?\s*)?label\s+%").unwrap();
    let switch_re = regex::Regex::new(r"switch\s+.*label\s+%").unwrap();

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
    // Pre-compile block regex outside the loop
    let block_re = regex::Regex::new(r"(?m)^([\w.]+):").unwrap();

    for cap in br_re.captures_iter(func_body) {
        // Find which block this branch is in
        let match_pos = cap.get(0).unwrap().start();
        let before = &func_body[..match_pos];

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

    // Match call and invoke instructions - handles both:
    //   %result = call i32 @func(args)    (with destination)
    //   %result = tail call i32 @func(args)  (with tail prefix)
    //   %result = cold call i32 @func(args) (with cold prefix)
    //   call void @func(args)              (without destination)
    // Simpler pattern that matches return type more reliably
    let call_re_with_dest = regex::Regex::new(
        r"(?m)^\s+%(\w+)\s*=\s*(?:tail\s+|cold\s+)?call\s+.*?@([\w\.]+)\s*\(([^)]*)\)",
    )
    .unwrap();

    // Also match tail/call without destination
    let call_re_tail_no_dest =
        regex::Regex::new(r"(?m)^\s*(?:tail\s+)?call\s+.*?@([\w\.]+)\s*\(([^)]*)\)").unwrap();

    let call_re_no_dest = regex::Regex::new(
        r"(?m)^\s+(call|invoke)\s+(?:cold\s+)?(?:<[^>]*>\s*)?([^@]*?)@([\w\.]+)\s*\(([^)]*)\)",
    )
    .unwrap();

    // Handle indirect calls (function pointer calls): call void %8(i32 noundef %9)
    // Pattern: call [return_type] %register(args)
    // Captures: 1 = register, 2 = args
    let call_re_indirect_no_dest =
        regex::Regex::new(r"(?m)^\s*(?:tail\s+)?call\s+\S+\s+%(\w+)\s*\(([^)]*)\)").unwrap();

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
        // Check for basic blocks - lines starting with a label like "7:"
        // The label may be followed by comments like "7: ; preds = %2"
        if let Some(cap) = block_re.captures(line) {
            let block_name = cap.get(1).unwrap().as_str().to_string();
            // Only consider blocks that are at the start of a line (after optional whitespace)
            let line_stripped = line.trim_start();
            if line_stripped.starts_with(&format!("{}:", block_name)) {
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

    // Process calls with destination
    for cap in call_re_with_dest.captures_iter(content) {
        let callee = cap.get(2).unwrap().as_str().to_string();
        let args_str = cap.get(3).unwrap().as_str();

        // Skip intrinsics and standard library
        if callee.starts_with("llvm.") || callee.starts_with("@llvm") {
            continue;
        }

        // Determine caller block
        let call_pos = cap.get(0).unwrap().start();
        let caller_block = find_caller_block(content, &block_re, call_pos);

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

    // Process calls without destination
    for cap in call_re_no_dest.captures_iter(content) {
        let callee = cap.get(3).unwrap().as_str().to_string();
        let args_str = cap.get(4).unwrap().as_str();

        // Skip intrinsics and standard library
        if callee.starts_with("llvm.") || callee.starts_with("@llvm") {
            continue;
        }

        // Determine caller block
        let call_pos = cap.get(0).unwrap().start();
        let caller_block = find_caller_block(content, &block_re, call_pos);

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

    // Process calls with tail/call prefix (no destination)
    for cap in call_re_tail_no_dest.captures_iter(content) {
        let callee = cap.get(1).unwrap().as_str().to_string();
        let args_str = cap.get(2).unwrap().as_str();

        // Skip intrinsics and standard library
        if callee.starts_with("llvm.") || callee.starts_with("@llvm") {
            continue;
        }

        // Determine caller block
        let call_pos = cap.get(0).unwrap().start();
        let caller_block = find_caller_block(content, &block_re, call_pos);

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

        // Avoid duplicates
        if !calls
            .iter()
            .any(|c| c.caller_function == caller_function && c.callee == callee)
        {
            calls.push(IrCall {
                caller_block,
                caller_function,
                callee,
                is_indirect,
                arguments,
            });
        }
    }

    // Process indirect calls (function pointer calls)
    // These are calls without destination: call void %8(i32)
    for cap in call_re_indirect_no_dest.captures_iter(content) {
        let callee = cap.get(1).unwrap().as_str().to_string(); // Function pointer register
        let args_str = cap.get(2).unwrap().as_str();

        // Determine caller block
        let call_pos = cap.get(0).unwrap().start();
        let caller_block = find_caller_block(content, &block_re, call_pos);

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

        let indirect_callee = format!("[function_ptr:{}]", callee);

        // Avoid duplicates
        if !calls
            .iter()
            .any(|c| c.caller_function == caller_function && c.callee == indirect_callee)
        {
            calls.push(IrCall {
                caller_block,
                caller_function,
                callee: indirect_callee,
                is_indirect: true,
                arguments,
            });
        }
    }

    calls
}

/// Find the caller block given a call position
fn find_caller_block(content: &str, block_re: &regex::Regex, call_pos: usize) -> String {
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

    block_positions
        .iter()
        .max_by_key(|(_, p)| *p)
        .map(|(n, _)| n.clone())
        .unwrap_or_else(|| String::from("unknown"))
}

/// Detect if a function is a callback
fn detect_callback(
    name: &str,
    _params: &[IrParameter],
    kb: &KnowledgeBase,
) -> (bool, Option<String>) {
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

    #[test]
    fn test_parse_complex_function() {
        let ir = r#"
; ModuleID = 'test_module'
source_filename = "test.c"

define i32 @my_driver_isr(i32 %irq, ptr %dev_id) {
entry:
  %data = load i32, ptr %dev_id
  %cmp = icmp eq i32 %data, 0
  br i1 %cmp, label %handle_normal, label %handle_error

handle_normal:
  call void @schedule_work(ptr %dev_id)
  ret i32 IRQ_HANDLED

handle_error:
  ret i32 IRQ_NONE
}

declare void @schedule_work(ptr)

define void @work_handler(ptr %work) {
entry:
  ret void
}
"#;

        let result = parse_ll_text(ir, &KnowledgeBase::default()).unwrap();
        assert_eq!(result.module_name, "test_module");
        assert!(result.functions.contains_key("my_driver_isr"));
        assert!(result.functions.contains_key("work_handler"));
        // Should extract the schedule_work call
        assert!(result.calls.iter().any(|c| c.callee == "schedule_work"));
    }

    #[test]
    fn test_parse_branch_conditions() {
        let ir = r#"
define i32 @check_value(i32 %x) {
entry:
  %cmp = icmp sgt i32 %x, 0
  br i1 %cmp, label %positive, label %non_positive

positive:
  ret i32 1

non_positive:
  ret i32 0
}
"#;

        let result = parse_ll_text(ir, &KnowledgeBase::default()).unwrap();
        let func = result.functions.get("check_value").unwrap();

        // Should have 2 basic blocks: entry and positive (non_positive might be merged)
        assert!(!func.blocks.is_empty());

        // Find entry block
        let entry = func.blocks.iter().find(|b| b.name == "entry").unwrap();
        // Should have a branch terminator
        assert!(entry.terminator.is_some());
    }

    #[test]
    fn test_parse_struct_types() {
        let ir = r#"
%struct.work_struct = type { atomic_t, ptr }

define void @init_work(ptr %work) {
entry:
  ret void
}
"#;

        let result = parse_ll_text(ir, &KnowledgeBase::default()).unwrap();
        assert!(result.types.contains_key("struct.work_struct"));
    }

    #[test]
    fn test_parse_void_function() {
        let ir = r#"
define void @my_callback(ptr %arg1, i32 %arg2) {
entry:
  ret void
}
"#;

        let result = parse_ll_text(ir, &KnowledgeBase::default()).unwrap();
        let func = result.functions.get("my_callback").unwrap();
        assert_eq!(func.return_type, "void");
        assert_eq!(func.parameters.len(), 2);
    }

    #[test]
    fn test_parse_kernel_style_ir() {
        // Test parsing a file that mimics Linux kernel LLVM IR
        let ir = r#"; Test LLVM IR file mimicking Linux kernel structure
source_filename = "drivers/net/dummy.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-linux-gnu"

; Type definitions
%struct.net_device = type { i32, i8*, %struct.net_device_ops, %struct.ethtool_ops, i64 }
%struct.net_device_ops = type { i64 (i8*)*, i64 (i8*)* }

; Function: dummy_xmit - netdev ndo_start_xmit handler
define i32 @dummy_xmit(i8* nocapture readonly %skb, i8* nocapture %dev) local_unnamed_addr #0 {
entry:
  %call = tail call i32 @netif_rx(i8* %skb)
  ret i32 %call
}

; Function: dummy_get_stats64
define void @dummy_get_stats64(i8* nocapture %dev, i8* nocapture %storage) local_unnamed_addr #0 {
entry:
  ret void
}

; Function: dummy_loop_test with control flow
define i32 @dummy_loop_test(i32 %n) {
entry:
  %cmp = icmp sgt i32 %n, 0
  br i1 %cmp, label %for.body, label %for.end

for.body:
  %i.0 = phi i32 [ %add, %for.body ], [ 0, %entry ]
  %add = add nuw nsw i32 %i.0, 1
  %cmp2 = icmp slt i32 %add, %n
  br i1 %cmp2, label %for.body, label %for.end

for.end:
  ret i32 0
}

; Function: dummy_switch_test
define i32 @dummy_switch_test(i32 %x) {
entry:
  switch i32 %x, label %default [
    i32 0, label %case0
    i32 1, label %case1
  ]

case0:
  ret i32 1

case1:
  ret i32 2

default:
  ret i32 0
}

declare i32 @netif_rx(i8*)
"#;

        let result = parse_ll_text(ir, &KnowledgeBase::default()).unwrap();

        // Should have module name from source_filename
        assert_eq!(result.module_name, "dummy.c");

        // Should parse kernel driver functions
        assert!(result.functions.contains_key("dummy_xmit"));
        assert!(result.functions.contains_key("dummy_get_stats64"));
        assert!(result.functions.contains_key("dummy_loop_test"));
        assert!(result.functions.contains_key("dummy_switch_test"));

        // Should extract netif_rx call
        assert!(result.calls.iter().any(|c| c.callee == "netif_rx"));

        // Should parse struct type
        assert!(result.types.contains_key("struct.net_device"));

        // Loop test should have multiple basic blocks
        let loop_func = result.functions.get("dummy_loop_test").unwrap();
        assert!(
            loop_func.blocks.len() >= 2,
            "Loop should have entry and body blocks"
        );

        // Switch test should have multiple basic blocks
        let switch_func = result.functions.get("dummy_switch_test").unwrap();
        assert!(
            switch_func.blocks.len() >= 2,
            "Switch should have multiple case blocks"
        );
    }
}
