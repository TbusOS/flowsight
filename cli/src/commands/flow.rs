//! `flowsight flow` and `flowsight trace` commands

use crate::context::AnalysisContext;
use crate::output::{json, text, OutputFormat};
use anyhow::Result;
use flowsight_core::{FlowNode, FlowNodeType};
use std::path::Path;

/// Flow display options
pub struct FlowOptions {
    /// Maximum depth to display (None = unlimited)
    pub max_depth: Option<usize>,
    /// Hide kernel API calls
    pub no_kernel: bool,
    /// Expand async boundaries (TODO: not yet implemented)
    #[allow(dead_code)]
    pub expand_async: bool,
}

impl Default for FlowOptions {
    fn default() -> Self {
        Self {
            max_depth: None,
            no_kernel: false,
            expand_async: false,
        }
    }
}

/// Filter a flow tree by depth and node type, returning a new tree
fn filter_flow_tree(node: &FlowNode, opts: &FlowOptions, current_depth: usize) -> FlowNode {
    let children = if opts.max_depth.is_some_and(|d| current_depth >= d) {
        vec![]
    } else {
        node.children
            .iter()
            .filter(|child| {
                if opts.no_kernel && matches!(child.node_type, FlowNodeType::KernelApi) {
                    return false;
                }
                true
            })
            .map(|child| filter_flow_tree(child, opts, current_depth + 1))
            .collect()
    };

    FlowNode {
        id: node.id.clone(),
        name: node.name.clone(),
        display_name: node.display_name.clone(),
        location: node.location.clone(),
        node_type: node.node_type.clone(),
        children,
        description: node.description.clone(),
        confidence: node.confidence.clone(),
        execution_context: node.execution_context.clone(),
        can_sleep: node.can_sleep,
        source_file: node.source_file.clone(),
        is_kernel_internal: node.is_kernel_internal,
    }
}

/// Show execution flow tree for a function
pub fn run(
    file: &Path,
    function: &str,
    format: &OutputFormat,
    opts: &FlowOptions,
) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;

    // Find the flow tree for the specified function
    for tree in &result.analysis.flow_trees {
        if tree.name == function {
            let filtered = filter_flow_tree(tree, opts, 0);

            match format {
                OutputFormat::Json => {
                    let json_str = json::to_pretty_json(&filtered)?;
                    println!("{}", json_str);
                }
                OutputFormat::Ftrace => {
                    text::print_ftrace_tree(&filtered, 0, &result.parse_result.functions);
                }
                OutputFormat::Markdown => {
                    println!("# Execution Flow: {}()", function);
                    println!();
                    println!("```");
                    text::print_ftrace_tree(&filtered, 0, &result.parse_result.functions);
                    println!("```");
                }
                OutputFormat::Sequence => {
                    anyhow::bail!(
                        "Sequence format is for kb chain/async-chain commands. Use --format text/json/ftrace/markdown for flow."
                    );
                }
                OutputFormat::Text => {
                    text::print_flow_tree(&filtered, 0);
                }
            }
            return Ok(());
        }
    }

    // If not found in flow trees, try to build a simple one from parse result
    if let Some(func) = result.parse_result.functions.get(function) {
        println!("{}()", function);
        if func.calls.is_empty() {
            println!("  (no calls)");
        } else {
            for (i, callee) in func.calls.iter().enumerate() {
                let is_last = i == func.calls.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };
                println!("  {}{}()", prefix, callee);
            }
        }
    } else {
        eprintln!("Function '{}' not found in {}", function, file.display());
        eprintln!();
        // List available functions
        let mut names: Vec<_> = result.parse_result.functions.keys().collect();
        names.sort();
        if !names.is_empty() {
            eprintln!("Available functions:");
            for name in names {
                eprintln!("  {}", name);
            }
        }
    }

    Ok(())
}

/// Show execution flow in ftrace style (legacy trace command)
pub fn run_trace(file: &Path, function: &str, format: &str) -> Result<()> {
    let output_format = match format {
        "ftrace" => OutputFormat::Ftrace,
        "json" => OutputFormat::Json,
        "markdown" => OutputFormat::Markdown,
        other => anyhow::bail!(
            "Unknown trace format: '{}'. Valid options: ftrace, json, markdown",
            other
        ),
    };
    run(file, function, &output_format, &FlowOptions::default())
}
