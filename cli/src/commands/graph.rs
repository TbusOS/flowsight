//! `flowsight callers` and `flowsight callees` commands

use crate::context::AnalysisContext;
use anyhow::Result;
use std::path::Path;

/// Show who calls a function
pub fn run_callers(file: &Path, function: &str) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;

    println!("Callers of {}():", function);
    println!();

    let mut found = false;

    // Direct callers
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

    // Async callers (via bindings)
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

/// Show what a function calls
pub fn run_callees(file: &Path, function: &str) -> Result<()> {
    let ctx = AnalysisContext::new();
    let parse_result = ctx.parse_file(file)?;

    println!("{}() calls:", function);
    println!();

    if let Some(func) = parse_result.functions.get(function) {
        if func.calls.is_empty() {
            println!("  (No function calls found)");
        } else {
            for (i, callee) in func.calls.iter().enumerate() {
                let is_last = i == func.calls.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };

                let suffix = if parse_result.functions.contains_key(callee) {
                    ""
                } else {
                    " [External]"
                };

                println!("  {}{}(){}", prefix, callee, suffix);
            }
        }
    } else {
        eprintln!("Function '{}' not found", function);
    }

    Ok(())
}
