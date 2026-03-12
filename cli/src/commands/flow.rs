//! `flowsight flow` and `flowsight trace` commands

use crate::context::AnalysisContext;
use crate::output::{json, text, OutputFormat};
use anyhow::Result;
use std::path::Path;

/// Show execution flow tree for a function
pub fn run(file: &Path, function: &str, format: &OutputFormat) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;

    // Find the flow tree for the specified function
    for tree in &result.analysis.flow_trees {
        if tree.name == function {
            match format {
                OutputFormat::Json => {
                    let json_str = json::to_pretty_json(tree)?;
                    println!("{}", json_str);
                }
                OutputFormat::Ftrace => {
                    text::print_ftrace_tree(tree, 0, &result.parse_result.functions);
                }
                OutputFormat::Markdown => {
                    println!("# Execution Flow: {}()", function);
                    println!();
                    println!("```");
                    text::print_ftrace_tree(tree, 0, &result.parse_result.functions);
                    println!("```");
                }
                OutputFormat::Mermaid => {
                    anyhow::bail!(
                        "Mermaid output format is not yet implemented. Use --format text/json/ftrace/markdown"
                    );
                }
                OutputFormat::Text => {
                    text::print_flow_tree(tree, 0);
                }
            }
            return Ok(());
        }
    }

    // If not found in flow trees, try to build a simple one from parse result
    if let Some(func) = result.parse_result.functions.get(function) {
        println!("{}()", function);
        for callee in &func.calls {
            println!("  └── {}()", callee);
        }
    } else {
        eprintln!("Function '{}' not found", function);
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
    run(file, function, &output_format)
}
