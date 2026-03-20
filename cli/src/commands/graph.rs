//! `flowsight callers`, `flowsight callees`, and `flowsight graph full` commands

use crate::context::AnalysisContext;
use crate::index_db::IndexDb;
use crate::output::{dot, OutputFormat};
use anyhow::Result;
use crossterm::style::{Color, Stylize};
use std::path::Path;

const C_TITLE: Color = Color::Rgb { r: 140, g: 185, b: 165 };
const C_FILE: Color = Color::Rgb { r: 155, g: 160, b: 185 };
const C_DIM: Color = Color::Rgb { r: 110, g: 115, b: 120 };
const C_OK: Color = Color::Rgb { r: 130, g: 175, b: 140 };

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

// =========================================================================
// Cross-file commands (using SQLite index)
// =========================================================================

/// Show cross-file callers using SQLite index
pub fn run_cross_callers(
    function: &str,
    db_path: &Path,
    format: &OutputFormat,
    group_by_subsystem: bool,
) -> Result<()> {
    let db = IndexDb::open_readonly(db_path)?;

    match format {
        OutputFormat::Json => {
            if group_by_subsystem {
                let groups = db.query_callers_by_subsystem(function)?;
                let json_val: Vec<_> = groups.iter().map(|(sub, calls)| {
                    serde_json::json!({
                        "subsystem": sub,
                        "count": calls.len(),
                        "callers": calls.iter().map(|c| {
                            serde_json::json!({
                                "caller": c.caller_name,
                                "file": c.caller_file,
                                "line": c.call_line,
                                "indirect": c.is_indirect,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect();
                println!("{}", serde_json::to_string_pretty(&json_val)?);
            } else {
                let callers = db.query_callers(function)?;
                let json_val: Vec<_> = callers.iter().map(|c| {
                    serde_json::json!({
                        "caller": c.caller_name,
                        "file": c.caller_file,
                        "line": c.call_line,
                        "indirect": c.is_indirect,
                    })
                }).collect();
                println!("{}", serde_json::to_string_pretty(&json_val)?);
            }
        }
        _ => {
            if group_by_subsystem {
                let groups = db.query_callers_by_subsystem(function)?;
                let total: usize = groups.iter().map(|(_, v)| v.len()).sum();

                println!(
                    "{} {} ({} callers across {} subsystems)",
                    "Cross-file callers of".with(C_TITLE),
                    format!("{}()", function).with(C_FILE),
                    total.to_string().with(C_OK),
                    groups.len().to_string().with(C_OK),
                );
                println!();

                for (subsystem, callers) in &groups {
                    println!(
                        "  {} ({})",
                        subsystem.as_str().with(C_TITLE),
                        callers.len().to_string().with(C_DIM),
                    );
                    for caller in callers {
                        let file_short = Path::new(&caller.caller_file)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&caller.caller_file);
                        println!(
                            "    {}() {}",
                            caller.caller_name.as_str().with(C_FILE),
                            format!("{}:L{}", file_short, caller.call_line).with(C_DIM),
                        );
                    }
                }
            } else {
                let callers = db.query_callers(function)?;

                println!(
                    "{} {} ({} callers)",
                    "Cross-file callers of".with(C_TITLE),
                    format!("{}()", function).with(C_FILE),
                    callers.len().to_string().with(C_OK),
                );
                println!();

                if callers.is_empty() {
                    println!("  {}", "(No callers found in index)".with(C_DIM));
                } else {
                    for caller in &callers {
                        let file_short = Path::new(&caller.caller_file)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&caller.caller_file);
                        let indirect_tag = if caller.is_indirect { " [indirect]" } else { "" };
                        println!(
                            "  {}(){} {}",
                            caller.caller_name.as_str().with(C_FILE),
                            indirect_tag.with(C_DIM),
                            format!("{}:L{}", file_short, caller.call_line).with(C_DIM),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Show cross-file callees using SQLite index
pub fn run_cross_callees(
    function: &str,
    db_path: &Path,
    format: &OutputFormat,
) -> Result<()> {
    let db = IndexDb::open_readonly(db_path)?;
    let callees = db.query_callees(function)?;

    match format {
        OutputFormat::Json => {
            let json_val: Vec<_> = callees.iter().map(|c| {
                serde_json::json!({
                    "callee": c.callee_name,
                    "file": c.caller_file,
                    "line": c.call_line,
                    "indirect": c.is_indirect,
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&json_val)?);
        }
        _ => {
            println!(
                "{} {} ({} callees)",
                "Cross-file callees of".with(C_TITLE),
                format!("{}()", function).with(C_FILE),
                callees.len().to_string().with(C_OK),
            );
            println!();

            if callees.is_empty() {
                println!("  {}", "(No callees found in index)".with(C_DIM));
            } else {
                // Deduplicate by callee name, count occurrences
                let mut callee_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
                for c in &callees {
                    *callee_counts.entry(&c.callee_name).or_default() += 1;
                }
                let mut sorted: Vec<_> = callee_counts.into_iter().collect();
                sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

                for (callee, count) in &sorted {
                    let count_str = if *count > 1 { format!(" (x{})", count) } else { String::new() };
                    println!(
                        "  {}(){}",
                        callee.with(C_FILE),
                        count_str.with(C_DIM),
                    );
                }
            }
        }
    }

    Ok(())
}
