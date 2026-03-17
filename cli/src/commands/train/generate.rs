//! Training data generation from kernel source analysis
//!
//! Walks a directory of C files, analyzes each with the FlowSight engine,
//! and produces JSONL training examples across multiple categories.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::context::{AnalysisContext, FileAnalysis};

use super::formats::{self, SftExample};
use super::{TrainCategory, TrainFormat};

/// Configuration for training data generation
pub struct GenerateConfig {
    pub dir: PathBuf,
    pub format: TrainFormat,
    pub output: Option<PathBuf>,
    pub categories: Vec<TrainCategory>,
    pub max_depth: usize,
    pub min_complexity: usize,
}

/// Run the training data generation pipeline
pub fn run(config: &GenerateConfig) -> Result<()> {
    let c_files = collect_c_files(&config.dir)?;

    if c_files.is_empty() {
        anyhow::bail!("No .c files found in {}", config.dir.display());
    }

    eprintln!(
        "Generating training data from {} C files...",
        c_files.len()
    );

    let mut ctx = AnalysisContext::new();
    let categories: HashSet<TrainCategory> = config.categories.iter().copied().collect();
    let mut writer = open_writer(config.output.as_deref())?;
    let mut total_examples = 0usize;

    for file_path in &c_files {
        match ctx.analyze_file(file_path) {
            Ok(analysis) => {
                let count = generate_from_file(
                    &analysis,
                    &categories,
                    &config.format,
                    config.max_depth,
                    config.min_complexity,
                    &mut writer,
                )?;
                total_examples += count;
                if count > 0 {
                    eprintln!("  {} -> {} examples", file_path.display(), count);
                }
            }
            Err(e) => {
                eprintln!("  SKIP {} ({})", file_path.display(), e);
            }
        }
    }

    writer.flush()?;
    eprintln!();
    eprintln!("Total: {} training examples generated", total_examples);

    Ok(())
}

/// Collect all .c files from a directory recursively
fn collect_c_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "c") {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

/// Open a JSONL writer (file or stdout)
fn open_writer(output: Option<&Path>) -> Result<Box<dyn Write>> {
    match output {
        Some(path) => {
            let file = std::fs::File::create(path)
                .with_context(|| format!("Cannot create {}", path.display()))?;
            Ok(Box::new(std::io::BufWriter::new(file)))
        }
        None => Ok(Box::new(std::io::BufWriter::new(std::io::stdout()))),
    }
}

/// Generate training examples from a single analyzed file
fn generate_from_file(
    analysis: &FileAnalysis,
    categories: &HashSet<TrainCategory>,
    format: &TrainFormat,
    max_depth: usize,
    min_complexity: usize,
    writer: &mut dyn Write,
) -> Result<usize> {
    let file_str = analysis.file.to_string_lossy().to_string();
    let mut count = 0usize;

    if categories.contains(&TrainCategory::Flow) {
        count += generate_flow_examples(analysis, &file_str, format, max_depth, min_complexity, writer)?;
    }
    if categories.contains(&TrainCategory::Async) {
        count += generate_async_examples(analysis, &file_str, format, writer)?;
    }
    if categories.contains(&TrainCategory::Callbacks) {
        count += generate_callback_examples(analysis, &file_str, format, writer)?;
    }
    if categories.contains(&TrainCategory::Chains) {
        count += generate_chain_examples(analysis, &file_str, format, writer)?;
    }
    if categories.contains(&TrainCategory::Patterns) {
        count += generate_pattern_examples(analysis, &file_str, format, writer)?;
    }

    Ok(count)
}

/// Write a single SFT example, converting to the target format as needed
fn write_example(
    sft: &SftExample,
    format: &TrainFormat,
    writer: &mut dyn Write,
) -> Result<()> {
    let line = match format {
        TrainFormat::Sft => formats::to_jsonl_line(sft)?,
        TrainFormat::Dpo => {
            let dpo = formats::sft_to_dpo(sft);
            formats::to_jsonl_line(&dpo)?
        }
        TrainFormat::Chatml => {
            let chatml = formats::sft_to_chatml(sft);
            formats::to_jsonl_line(&chatml)?
        }
    };
    writeln!(writer, "{}", line)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Category: flow - execution flow Q&A pairs
// ---------------------------------------------------------------------------

/// Generate flow analysis examples for functions meeting complexity threshold
fn generate_flow_examples(
    analysis: &FileAnalysis,
    file_str: &str,
    format: &TrainFormat,
    max_depth: usize,
    min_complexity: usize,
    writer: &mut dyn Write,
) -> Result<usize> {
    let mut count = 0usize;

    for tree in &analysis.analysis.flow_trees {
        let func = analysis.parse_result.functions.get(&tree.name);
        let call_count = func.map(|f| f.calls.len()).unwrap_or(0);

        if call_count < min_complexity {
            continue;
        }

        let snippet = extract_function_snippet(&analysis.source, &tree.name, func);
        let flow_text = render_flow_tree(tree, 0, max_depth);
        let total_len = snippet.len() + flow_text.len();

        let sft = SftExample {
            instruction: format!(
                "Analyze the execution flow of function {} in the Linux kernel",
                tree.name
            ),
            input: snippet,
            output: flow_text,
            meta: formats::build_meta(file_str, Some(&tree.name), &TrainCategory::Flow, total_len),
        };

        write_example(&sft, format, writer)?;
        count += 1;
    }

    Ok(count)
}

/// Render a FlowNode tree into a human-readable text description
fn render_flow_tree(
    node: &flowsight_core::FlowNode,
    depth: usize,
    max_depth: usize,
) -> String {
    if depth > max_depth {
        return String::new();
    }

    let indent = "  ".repeat(depth);
    let mut lines = Vec::new();

    let type_tag = match &node.node_type {
        flowsight_core::FlowNodeType::AsyncCallback { .. } => " [async]",
        flowsight_core::FlowNodeType::KernelApi => " [kernel API]",
        flowsight_core::FlowNodeType::External => " [external]",
        flowsight_core::FlowNodeType::EntryPoint => " [entry]",
        _ => "",
    };

    let step = depth + 1;
    lines.push(format!("{}{}. {}(){}", indent, step, node.name, type_tag));

    if let Some(ref desc) = node.description {
        lines.push(format!("{}   - {}", indent, desc));
    }

    for child in &node.children {
        let child_text = render_flow_tree(child, depth + 1, max_depth);
        if !child_text.is_empty() {
            lines.push(child_text);
        }
    }

    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Category: async - async handler analysis
// ---------------------------------------------------------------------------

/// Generate async mechanism analysis examples
fn generate_async_examples(
    analysis: &FileAnalysis,
    file_str: &str,
    format: &TrainFormat,
    writer: &mut dyn Write,
) -> Result<usize> {
    let mut count = 0usize;

    for binding in &analysis.analysis.async_bindings {
        let mechanism_str = format!("{:?}", binding.mechanism);
        let context_str = if binding.context.can_sleep() {
            "process context (can sleep)"
        } else {
            "atomic context (cannot sleep)"
        };

        let snippet = extract_function_snippet(
            &analysis.source,
            &binding.handler,
            analysis.parse_result.functions.get(&binding.handler),
        );

        let output = format!(
            "The function {}() is an asynchronous handler registered via {}.\n\n\
             Mechanism: {}\n\
             Variable: {}\n\
             Execution context: {}\n\n\
             This handler is invoked asynchronously by the kernel's {} subsystem. \
             It runs in {} and {}.",
            binding.handler,
            binding.variable,
            mechanism_str,
            binding.variable,
            context_str,
            mechanism_str,
            context_str,
            if binding.context.can_sleep() {
                "may perform blocking operations like memory allocation with GFP_KERNEL"
            } else {
                "must not perform any blocking operations (no sleeping, no mutex_lock)"
            },
        );

        let total_len = snippet.len() + output.len();
        let sft = SftExample {
            instruction: format!(
                "Explain the async handler {}() and its execution context in the Linux kernel",
                binding.handler
            ),
            input: snippet,
            output,
            meta: formats::build_meta(
                file_str,
                Some(&binding.handler),
                &TrainCategory::Async,
                total_len,
            ),
        };

        write_example(&sft, format, writer)?;
        count += 1;
    }

    Ok(count)
}

// ---------------------------------------------------------------------------
// Category: callbacks - callback mechanism Q&A
// ---------------------------------------------------------------------------

/// Generate callback mechanism examples
fn generate_callback_examples(
    analysis: &FileAnalysis,
    file_str: &str,
    format: &TrainFormat,
    writer: &mut dyn Write,
) -> Result<usize> {
    let mut count = 0usize;

    for (name, func) in &analysis.parse_result.functions {
        if !func.is_callback {
            continue;
        }

        let context = func.callback_context.as_deref().unwrap_or("unknown");
        let snippet = extract_function_snippet(&analysis.source, name, Some(func));
        let params_desc = describe_params(&func.params);

        let output = format!(
            "The function {}() is a callback registered in the context: {}.\n\n\
             Parameters:\n{}\n\n\
             Return type: {}\n\n\
             This callback is invoked by the kernel framework when the corresponding \
             event occurs. Functions it calls: {}.",
            name,
            context,
            params_desc,
            func.return_type,
            if func.calls.is_empty() {
                "(none)".to_string()
            } else {
                func.calls.join(", ")
            },
        );

        let total_len = snippet.len() + output.len();
        let sft = SftExample {
            instruction: format!(
                "What does the callback function {} do in the Linux kernel?",
                name
            ),
            input: snippet,
            output,
            meta: formats::build_meta(
                file_str,
                Some(name),
                &TrainCategory::Callbacks,
                total_len,
            ),
        };

        write_example(&sft, format, writer)?;
        count += 1;
    }

    Ok(count)
}

/// Describe function parameters as a bulleted list
fn describe_params(params: &[flowsight_core::Parameter]) -> String {
    if params.is_empty() {
        return "  (none)".to_string();
    }
    params
        .iter()
        .map(|p| format!("  - {} ({})", p.name, p.type_name))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Category: chains - call chain tracing Q&A
// ---------------------------------------------------------------------------

/// Generate call chain tracing examples for entry points
fn generate_chain_examples(
    analysis: &FileAnalysis,
    file_str: &str,
    format: &TrainFormat,
    writer: &mut dyn Write,
) -> Result<usize> {
    let mut count = 0usize;

    for entry in &analysis.analysis.entry_points {
        let func = analysis.parse_result.functions.get(entry);
        if func.is_none() {
            continue;
        }
        let func = func.unwrap();

        if func.calls.is_empty() {
            continue;
        }

        let snippet = extract_function_snippet(&analysis.source, entry, Some(func));
        let chain_text = build_call_chain_text(entry, func, &analysis.parse_result.functions);

        let total_len = snippet.len() + chain_text.len();
        let sft = SftExample {
            instruction: format!(
                "Trace the call chain starting from {} in the Linux kernel source",
                entry
            ),
            input: snippet,
            output: chain_text,
            meta: formats::build_meta(file_str, Some(entry), &TrainCategory::Chains, total_len),
        };

        write_example(&sft, format, writer)?;
        count += 1;
    }

    Ok(count)
}

/// Build a text description of the call chain from an entry point
fn build_call_chain_text(
    entry: &str,
    func: &flowsight_core::FunctionDef,
    all_functions: &std::collections::HashMap<String, flowsight_core::FunctionDef>,
) -> String {
    let mut lines = vec![format!("Call chain from {}():", entry)];
    let mut visited = HashSet::new();
    visited.insert(entry.to_string());

    append_chain_lines(&mut lines, func, all_functions, &mut visited, 1, 4);

    lines.join("\n")
}

/// Recursively append call chain lines with depth limit
fn append_chain_lines(
    lines: &mut Vec<String>,
    func: &flowsight_core::FunctionDef,
    all_functions: &std::collections::HashMap<String, flowsight_core::FunctionDef>,
    visited: &mut HashSet<String>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth {
        return;
    }

    let indent = "  ".repeat(depth);
    for callee_name in &func.calls {
        let is_local = all_functions.contains_key(callee_name);
        let tag = if is_local { "" } else { " [external]" };
        lines.push(format!("{}-> {}(){}", indent, callee_name, tag));

        if is_local && !visited.contains(callee_name) {
            visited.insert(callee_name.clone());
            if let Some(callee_func) = all_functions.get(callee_name) {
                append_chain_lines(lines, callee_func, all_functions, visited, depth + 1, max_depth);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Category: patterns - design pattern recognition Q&A
// ---------------------------------------------------------------------------

/// Generate design pattern recognition examples
fn generate_pattern_examples(
    analysis: &FileAnalysis,
    file_str: &str,
    format: &TrainFormat,
    writer: &mut dyn Write,
) -> Result<usize> {
    let mut count = 0usize;

    let patterns = detect_patterns(analysis);
    if patterns.is_empty() {
        return Ok(0);
    }

    let file_name = analysis
        .file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let output = format_detected_patterns(&patterns, file_name);
    let snippet = extract_file_summary(&analysis.source, 60);
    let total_len = snippet.len() + output.len();

    let sft = SftExample {
        instruction: format!(
            "Identify the Linux kernel design patterns used in {}",
            file_name
        ),
        input: snippet,
        output,
        meta: formats::build_meta(file_str, None, &TrainCategory::Patterns, total_len),
    };

    write_example(&sft, format, writer)?;
    count += 1;

    Ok(count)
}

/// A detected design pattern in the source
struct DetectedPattern {
    name: String,
    evidence: String,
}

/// Detect common kernel design patterns in an analyzed file
fn detect_patterns(analysis: &FileAnalysis) -> Vec<DetectedPattern> {
    let mut patterns = Vec::new();

    // Check for ops table pattern
    let has_ops_table = analysis
        .parse_result
        .structs
        .values()
        .any(|s| s.fields.iter().any(|f| f.is_function_ptr));

    if has_ops_table {
        patterns.push(DetectedPattern {
            name: "Operations Table (vtable)".to_string(),
            evidence: "Struct with function pointer fields acts as a virtual dispatch table"
                .to_string(),
        });
    }

    // Check for init/exit pattern
    let has_init = analysis.source.contains("module_init");
    let has_exit = analysis.source.contains("module_exit");
    if has_init && has_exit {
        patterns.push(DetectedPattern {
            name: "Module Init/Exit Lifecycle".to_string(),
            evidence: "module_init/module_exit macros define driver lifecycle".to_string(),
        });
    }

    // Check for async deferral pattern
    if !analysis.analysis.async_bindings.is_empty() {
        patterns.push(DetectedPattern {
            name: "Async Deferral (top-half / bottom-half)".to_string(),
            evidence: format!(
                "{} async binding(s) found (work queues, timers, etc.)",
                analysis.analysis.async_bindings.len()
            ),
        });
    }

    // Check for error-goto cleanup pattern
    if analysis.source.contains("goto err") || analysis.source.contains("goto fail") {
        patterns.push(DetectedPattern {
            name: "Error-Goto Cleanup".to_string(),
            evidence: "goto-based error handling for resource cleanup on failure paths".to_string(),
        });
    }

    // Check for reference counting pattern
    if analysis.source.contains("kref") || analysis.source.contains("refcount") {
        patterns.push(DetectedPattern {
            name: "Reference Counting".to_string(),
            evidence: "kref or refcount_t used for object lifetime management".to_string(),
        });
    }

    // Check for container_of pattern
    if analysis.source.contains("container_of") {
        patterns.push(DetectedPattern {
            name: "Container-Of (embedded struct)".to_string(),
            evidence: "container_of macro used to recover parent struct from embedded member"
                .to_string(),
        });
    }

    patterns
}

/// Format detected patterns into a readable training output
fn format_detected_patterns(patterns: &[DetectedPattern], file_name: &str) -> String {
    let mut lines = vec![format!(
        "Design patterns identified in {}:",
        file_name
    )];
    lines.push(String::new());

    for (i, pat) in patterns.iter().enumerate() {
        lines.push(format!("{}. **{}**", i + 1, pat.name));
        lines.push(format!("   {}", pat.evidence));
        lines.push(String::new());
    }

    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Extract the source code of a specific function (first N lines)
fn extract_function_snippet(
    source: &str,
    func_name: &str,
    func_def: Option<&flowsight_core::FunctionDef>,
) -> String {
    let start_line = func_def
        .and_then(|f| f.location.as_ref())
        .map(|loc| loc.line as usize)
        .unwrap_or(0);

    if start_line == 0 {
        // Fallback: search for function name in source
        return find_function_by_name(source, func_name);
    }

    let lines: Vec<&str> = source.lines().collect();
    let start = start_line.saturating_sub(1); // 0-indexed
    let max_lines = 40;
    let end = (start + max_lines).min(lines.len());

    // Find the closing brace within the window
    let mut brace_depth = 0i32;
    for (i, line) in lines[start..end].iter().enumerate() {
        for ch in line.chars() {
            if ch == '{' {
                brace_depth += 1;
            } else if ch == '}' {
                brace_depth -= 1;
                if brace_depth == 0 {
                    let actual_end = start + i + 1;
                    return lines[start..actual_end].join("\n");
                }
            }
        }
    }

    // If we couldn't find the end, return what we have with truncation marker
    let result = lines[start..end].join("\n");
    if end < lines.len() {
        format!("{}\n// ... (truncated)", result)
    } else {
        result
    }
}

/// Fallback function search by scanning source text
fn find_function_by_name(source: &str, func_name: &str) -> String {
    let pattern = format!("{func_name}(");
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.contains(&pattern) && !line.trim_start().starts_with("//") {
            let start = i;
            let end = (i + 30).min(lines.len());
            return lines[start..end].join("\n");
        }
    }

    String::new()
}

/// Extract first N lines as a file summary snippet
fn extract_file_summary(source: &str, max_lines: usize) -> String {
    source
        .lines()
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n")
}
