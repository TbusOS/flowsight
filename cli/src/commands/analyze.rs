//! `flowsight analyze` command - analyze source files

use crate::context::AnalysisContext;
use crate::output::{json, OutputFormat};
use anyhow::Result;
use std::path::Path;

/// Analyze a source file and print results
pub fn run(file: &Path, output: Option<&Path>, format: &OutputFormat) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;

    match format {
        OutputFormat::Json => {
            let json_str =
                json::format_analysis(&result.file, &result.parse_result, &result.analysis)?;
            if let Some(out_path) = output {
                std::fs::write(out_path, &json_str)?;
                eprintln!("Output written to: {}", out_path.display());
            } else {
                println!("{}", json_str);
            }
        }
        _ => {
            println!("Analyzing: {}", file.display());
            println!(
                "  Found {} functions, {} structs",
                result.parse_result.functions.len(),
                result.parse_result.structs.len()
            );
            println!(
                "  Found {} async handlers, {} entry points",
                result.analysis.async_bindings.len(),
                result.analysis.entry_points.len()
            );
            println!();
            println!("Summary:");
            println!("  Functions: {}", result.parse_result.functions.len());
            println!("  Structs: {}", result.parse_result.structs.len());
            println!(
                "  Async handlers: {}",
                result.analysis.async_bindings.len()
            );
            println!("  Entry points: {:?}", result.analysis.entry_points);
        }
    }

    Ok(())
}
