//! CFG command — Control Flow Graph construction and display
//!
//! Commands:
//! - `flowsight cfg <file> <function>` — show CFG for a function
//! - `flowsight errors <file> [function]` — list error paths

use crate::output::OutputFormat;
use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use flowsight_cfg::{
    BlockType, CallKind, CfgBuilder, ControlFlowGraph, ErrorPathDetector, Reachability,
};
use std::path::Path;

// Low-saturation color palette (matching project style)
const C_TITLE: Color = Color::Rgb {
    r: 140,
    g: 185,
    b: 165,
};
const C_OK: Color = Color::Rgb {
    r: 130,
    g: 175,
    b: 140,
};
const C_ERR: Color = Color::Rgb {
    r: 195,
    g: 120,
    b: 120,
};
const C_COND: Color = Color::Rgb {
    r: 170,
    g: 160,
    b: 120,
};
const C_FILE: Color = Color::Rgb {
    r: 155,
    g: 160,
    b: 185,
};
const C_DIM: Color = Color::Rgb {
    r: 110,
    g: 115,
    b: 120,
};
const C_CONTEXT: Color = Color::Rgb {
    r: 140,
    g: 150,
    b: 180,
};
const C_ASYNC: Color = Color::Rgb {
    r: 150,
    g: 140,
    b: 170,
};
const C_DECL: Color = Color::Rgb {
    r: 130,
    g: 130,
    b: 130,
};

/// Run the `cfg` command
pub fn run(file: &Path, function: &str, format: &OutputFormat) -> Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Cannot read file: {}", file.display()))?;

    let builder = CfgBuilder::new();
    let mut cfg = builder
        .build_function_cfg(&source, function)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    cfg.source_file = Some(file.display().to_string());

    // Run error path analysis
    ErrorPathDetector::analyze(&mut cfg);

    match format {
        OutputFormat::Dot => {
            println!("{}", cfg.to_dot());
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        _ => {
            print_cfg_text(&cfg);
        }
    }

    Ok(())
}

/// Run the `errors` command — list error handling paths
pub fn run_errors(file: &Path, function: Option<&str>, format: &OutputFormat) -> Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Cannot read file: {}", file.display()))?;

    let builder = CfgBuilder::new();

    let cfgs: Vec<ControlFlowGraph> = if let Some(func_name) = function {
        let mut cfg = builder
            .build_function_cfg(&source, func_name)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        ErrorPathDetector::analyze(&mut cfg);
        vec![cfg]
    } else {
        let mut all = builder
            .build_all_cfgs(&source)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        for cfg in &mut all {
            ErrorPathDetector::analyze(cfg);
        }
        all
    };

    match format {
        OutputFormat::Json => {
            let errors: Vec<_> = cfgs
                .iter()
                .filter(|c| !c.error_paths.is_empty())
                .map(|c| {
                    serde_json::json!({
                        "function": c.function_name,
                        "error_paths": c.error_paths,
                        "error_calls": c.error_calls().iter()
                            .map(|call| &call.callee)
                            .collect::<Vec<_>>(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&errors)?);
        }
        _ => {
            let filename = file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            println!(
                "{} {}",
                "Error paths in".with(C_TITLE),
                filename.with(C_FILE)
            );
            println!();

            let mut total_errors = 0;

            for cfg in &cfgs {
                if cfg.error_paths.is_empty() {
                    continue;
                }
                total_errors += cfg.error_paths.len();

                println!(
                    "  {} {} error path(s)",
                    format!("{}()", cfg.function_name).with(C_TITLE),
                    cfg.error_paths.len().to_string().with(C_ERR),
                );

                for (i, ep) in cfg.error_paths.iter().enumerate() {
                    let strategy_str = match &ep.strategy {
                        flowsight_cfg::ErrorStrategy::EarlyReturn { error_code } => {
                            format!("return {}", error_code)
                        }
                        flowsight_cfg::ErrorStrategy::GotoCleanup { label } => {
                            format!("goto {}", label)
                        }
                        flowsight_cfg::ErrorStrategy::CleanupAndReturn { label, error_code } => {
                            format!("goto {} -> return {}", label, error_code)
                        }
                    };

                    let prefix = if i == cfg.error_paths.len() - 1 {
                        "    └──"
                    } else {
                        "    ├──"
                    };

                    println!(
                        "{} L{}: {} -> {}",
                        prefix.with(C_DIM),
                        ep.check_line.to_string().with(C_DIM),
                        ep.check_expression.as_str().with(C_ERR),
                        strategy_str.with(C_COND),
                    );

                    if !ep.cleanup_calls.is_empty() {
                        let cleanup_prefix = if i == cfg.error_paths.len() - 1 {
                            "        "
                        } else {
                            "    │   "
                        };
                        println!(
                            "{}cleanup: {}",
                            cleanup_prefix.with(C_DIM),
                            ep.cleanup_calls.join(", ").with(C_FILE),
                        );
                    }
                }
                println!();
            }

            if total_errors == 0 {
                println!("  {}", "No error paths detected.".with(C_DIM));
            } else {
                println!(
                    "{} {} error path(s) across {} function(s)",
                    "Total:".with(C_TITLE),
                    total_errors.to_string().with(C_ERR),
                    cfgs.iter()
                        .filter(|c| !c.error_paths.is_empty())
                        .count()
                        .to_string()
                        .with(C_FILE),
                );
            }
        }
    }

    Ok(())
}

/// Print CFG in human-readable text format
fn print_cfg_text(cfg: &ControlFlowGraph) {
    let stats = cfg.stats();

    // Header
    println!(
        "{} {}",
        "CFG:".with(C_TITLE),
        format!("{}()", cfg.function_name).with(C_FILE),
    );
    if let Some(ref file) = cfg.source_file {
        println!("  {}", file.as_str().with(C_DIM));
    }
    println!(
        "  {}",
        format!(
            "{} blocks, {} edges, {} error paths",
            stats.block_count, stats.edge_count, stats.error_path_count
        )
        .with(C_DIM)
    );
    println!();

    // Calls by reachability
    println!("{}", "Calls:".with(C_TITLE));

    let all_calls = cfg.all_calls();
    if all_calls.is_empty() {
        println!("  {}", "(no calls)".with(C_DIM));
    } else {
        for call in &all_calls {
            let (reach_tag, reach_color) = match &call.reachability {
                Reachability::Always => ("always", C_OK),
                Reachability::Conditional(c) => {
                    // Truncate long conditions
                    let display = if c.len() > 30 { &c[..30] } else { c };
                    // Can't return borrowed, so just print inline
                    print!("  ");
                    print_call_line(call, &format!("if {}", display), C_COND);
                    continue;
                }
                Reachability::ErrorPath => ("error-path", C_ERR),
                Reachability::ConditionalCompilation => ("#ifdef", C_CONTEXT),
            };

            print!("  ");
            print_call_line(call, reach_tag, reach_color);
        }
    }
    println!();

    // Error paths
    if !cfg.error_paths.is_empty() {
        println!("{}", "Error paths:".with(C_TITLE));
        for ep in &cfg.error_paths {
            let strategy_str = match &ep.strategy {
                flowsight_cfg::ErrorStrategy::EarlyReturn { error_code } => {
                    format!("return {}", error_code)
                }
                flowsight_cfg::ErrorStrategy::GotoCleanup { label } => {
                    format!("goto {}", label)
                }
                flowsight_cfg::ErrorStrategy::CleanupAndReturn { label, error_code } => {
                    format!("goto {} -> return {}", label, error_code)
                }
            };

            println!(
                "  L{}: {} -> {}",
                ep.check_line.to_string().with(C_DIM),
                ep.check_expression.as_str().with(C_ERR),
                strategy_str.with(C_COND),
            );

            if !ep.cleanup_calls.is_empty() {
                println!(
                    "    cleanup: {}",
                    ep.cleanup_calls.join(", ").with(C_FILE),
                );
            }
        }
        println!();
    }

    // Block summary
    println!("{}", "Blocks:".with(C_TITLE));
    for block in &cfg.blocks {
        let type_tag = match block.block_type {
            BlockType::Entry => "ENTRY",
            BlockType::Exit => "EXIT",
            BlockType::Normal => "normal",
            BlockType::ErrorHandler => "error-handler",
            BlockType::LoopHeader => "loop-header",
            BlockType::LoopBody => "loop-body",
            BlockType::SwitchDispatch => "switch",
            BlockType::ConditionalCompilation => "#ifdef",
        };

        let type_color = match block.block_type {
            BlockType::Entry => C_OK,
            BlockType::Exit => C_ERR,
            BlockType::ErrorHandler => C_ERR,
            BlockType::LoopHeader | BlockType::LoopBody => C_CONTEXT,
            _ => C_DIM,
        };

        let label_str = block
            .label
            .as_ref()
            .map(|l| format!(" ({})", l))
            .unwrap_or_default();

        println!(
            "  B{} [{}]{} L{}-{}  calls: {}",
            block.id.to_string().with(C_DIM),
            type_tag.with(type_color),
            label_str.with(C_ERR),
            block.line_range.0,
            block.line_range.1,
            block.calls.len(),
        );
    }
}

/// Print a single call line with classification
fn print_call_line(call: &flowsight_cfg::CallSite, reach_tag: &str, reach_color: Color) {
    let kind_tag = match &call.call_kind {
        CallKind::Direct => "",
        CallKind::KernelApi => " [K]",
        CallKind::MacroCall => " [M]",
        CallKind::AsyncRegistration { mechanism, .. } => {
            // Print inline and return
            println!(
                "[{:>12}] {}{} {}",
                reach_tag.with(reach_color),
                call.callee.as_str().with(C_ASYNC),
                format!(" [async:{}]", mechanism).with(C_ASYNC),
                format!("L{}", call.line).with(C_DIM),
            );
            return;
        }
        CallKind::IteratorMacro => " [iter]",
        CallKind::DeclarationMacro => {
            println!(
                "[{:>12}] {}{} {}",
                "declaration".with(C_DECL),
                call.callee.as_str().with(C_DECL),
                " [decl]".with(C_DECL),
                format!("L{}", call.line).with(C_DIM),
            );
            return;
        }
        CallKind::ContextChange { new_context } => {
            println!(
                "[{:>12}] {}{} {}",
                reach_tag.with(reach_color),
                call.callee.as_str().with(C_CONTEXT),
                format!(" [ctx:{}]", new_context).with(C_CONTEXT),
                format!("L{}", call.line).with(C_DIM),
            );
            return;
        }
    };

    println!(
        "[{:>12}] {}{} {}",
        reach_tag.with(reach_color),
        call.callee.as_str().with(C_FILE),
        kind_tag.with(C_DIM),
        format!("L{}", call.line).with(C_DIM),
    );
}
