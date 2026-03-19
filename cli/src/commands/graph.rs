//! `flowsight callers`, `flowsight callees`, and `flowsight graph full` commands

use crate::context::AnalysisContext;
use crate::output::{dot, OutputFormat};
use anyhow::Result;
use std::path::Path;

/// Options for the `graph full` command.
pub struct GraphFullOptions {
    /// Hide external/kernel API calls
    pub exclude_kernel: bool,
    /// Group nodes by call depth in cluster subgraphs
    pub cluster: bool,
}

/// Show who calls a function
pub fn run_callers(file: &Path, function: &str, format: &OutputFormat) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;

    if matches!(format, OutputFormat::Dot) {
        let dot_output = dot::callers_to_dot(
            function,
            &result.parse_result,
            &result.analysis.async_bindings,
        );
        print!("{}", dot_output);
        return Ok(());
    }

    print_callers_text(function, &result)
}

/// Show what a function calls
pub fn run_callees(file: &Path, function: &str, format: &OutputFormat) -> Result<()> {
    let ctx = AnalysisContext::new();
    let parse_result = ctx.parse_file(file)?;

    if matches!(format, OutputFormat::Dot) {
        let dot_output = dot::callees_to_dot(function, &parse_result);
        print!("{}", dot_output);
        return Ok(());
    }

    print_callees_text(function, &parse_result)
}

/// Full file call graph
pub fn run_full(
    file: &Path,
    format: &OutputFormat,
    opts: &GraphFullOptions,
) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;
    let filename = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    match format {
        OutputFormat::Dot => {
            let dot_output = dot::file_call_graph_to_dot(
                &result.parse_result,
                &result.analysis,
                filename,
                opts.exclude_kernel,
                opts.cluster,
            );
            print!("{}", dot_output);
        }
        _ => {
            println!("Call graph for {}:", filename);
            println!();
            for (name, func) in &result.parse_result.functions {
                if func.calls.is_empty() {
                    continue;
                }
                println!("  {}():", name);
                for callee in &func.calls {
                    let suffix = if result.parse_result.functions.contains_key(callee) {
                        ""
                    } else {
                        " [External]"
                    };
                    println!("    -> {}(){}", callee, suffix);
                }
            }
        }
    }

    Ok(())
}

// -- Text output helpers (unchanged logic from original) ------------------

fn print_callers_text(
    function: &str,
    result: &crate::context::FileAnalysis,
) -> Result<()> {
    println!("Callers of {}():", function);
    println!();

    let mut found = false;

    for (name, func) in &result.parse_result.functions {
        if func.calls.contains(&function.to_string()) {
            found = true;
            let loc = func
                .location
                .as_ref()
                .map(|l| {
                    let filename = Path::new(&l.file)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&l.file);
                    format!("{}:{}", filename, l.line)
                })
                .unwrap_or_default();
            println!("  -> {}() [Direct]", name);
            if !loc.is_empty() {
                println!("     at {}", loc);
            }
        }
    }

    for binding in &result.analysis.async_bindings {
        if binding.handler == function {
            found = true;
            let mechanism = format!("{:?}", binding.mechanism);
            println!("  -> [Async: {}]", mechanism);
            if !binding.variable.is_empty() {
                println!("     via {}", binding.variable);
            }
        }
    }

    if !found {
        println!("  (No callers found - may be an entry point)");
    }

    Ok(())
}

fn print_callees_text(
    function: &str,
    parse_result: &flowsight_parser::ParseResult,
) -> Result<()> {
    println!("{}() calls:", function);
    println!();

    if let Some(func) = parse_result.functions.get(function) {
        if func.calls.is_empty() {
            println!("  (No function calls found)");
        } else {
            for (i, callee) in func.calls.iter().enumerate() {
                let is_last = i == func.calls.len() - 1;
                let prefix = if is_last { "  \u{2514}\u{2500}\u{2500} " } else { "  \u{251c}\u{2500}\u{2500} " };

                let suffix = if parse_result.functions.contains_key(callee) {
                    ""
                } else {
                    " [External]"
                };

                println!("{}{}(){}", prefix, callee, suffix);
            }
        }
    } else {
        eprintln!("Function '{}' not found", function);
    }

    Ok(())
}
