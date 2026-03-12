//! `flowsight async` and `flowsight callbacks` commands

use crate::context::AnalysisContext;
use crate::output::text;
use anyhow::Result;
use std::path::Path;

/// List all async handlers in a file
pub fn run_async(file: &Path) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;

    println!("Async Handlers in {}:", file.display());
    println!();

    for binding in &result.analysis.async_bindings {
        let mechanism = format!("{:?}", binding.mechanism);
        let context = if binding.context.can_sleep() {
            "can sleep"
        } else {
            "cannot sleep"
        };

        println!(
            "  {} {}()",
            text::mechanism_icon(&binding.mechanism),
            binding.handler
        );
        println!("     Type: {}", mechanism);
        println!("     Context: {}", context);
        if !binding.variable.is_empty() {
            println!("     Variable: {}", binding.variable);
        }
        println!();
    }

    Ok(())
}

/// List all callbacks in a file
pub fn run_callbacks(file: &Path) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;

    println!("Callbacks in {}:", file.display());
    println!();

    for (name, func) in &result.parse_result.functions {
        if func.is_callback {
            let context = func.callback_context.as_deref().unwrap_or("unknown");
            println!("  {}()", name);
            println!("     Context: {}", context);
            println!();
        }
    }

    Ok(())
}
