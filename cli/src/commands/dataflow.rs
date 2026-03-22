//! `flowsight dataflow` — data flow analysis (def-use chains, reaching definitions)

use crate::output::OutputFormat;
use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use flowsight_cpg::CpgBuilder;
use std::path::Path;

const C_TITLE: Color = Color::Rgb { r: 140, g: 185, b: 165 };
const C_FILE: Color = Color::Rgb { r: 155, g: 160, b: 185 };
const C_DIM: Color = Color::Rgb { r: 110, g: 115, b: 120 };
const C_OK: Color = Color::Rgb { r: 130, g: 175, b: 140 };
const C_ERR: Color = Color::Rgb { r: 195, g: 120, b: 120 };
const C_COND: Color = Color::Rgb { r: 170, g: 160, b: 120 };
const C_DEF: Color = Color::Rgb { r: 140, g: 150, b: 180 };

/// Run the `dataflow` command
pub fn run(
    file: &Path,
    function: &str,
    format: &OutputFormat,
    variable: Option<&str>,
) -> Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Cannot read file: {}", file.display()))?;

    let mut builder = CpgBuilder::new();
    let cpg = builder
        .build_function_cpg(&source, function)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&cpg)?);
        }
        _ => {
            if let Some(var) = variable {
                print_variable_flow(&cpg, var);
            } else {
                print_full_dataflow(&cpg);
            }
        }
    }

    Ok(())
}

/// Print data flow for a specific variable
fn print_variable_flow(cpg: &flowsight_cpg::CodePropertyGraph, var_name: &str) {
    let defs = cpg.defs_of(var_name);
    let uses = cpg.uses_of(var_name);

    println!(
        "{} {} in {}()",
        "Data flow of".with(C_TITLE),
        format!("'{}'", var_name).with(C_FILE),
        cpg.function_name.as_str().with(C_FILE),
    );
    println!();

    if defs.is_empty() && uses.is_empty() {
        println!("  {}", format!("Variable '{}' not found", var_name).with(C_DIM));
        return;
    }

    // Definitions
    println!("{}", "  Definitions:".with(C_TITLE));
    if defs.is_empty() {
        println!("    {}", "(none)".with(C_DIM));
    }
    for def in &defs {
        let kind_str = match &def.kind {
            flowsight_cpg::DefKind::Declaration => "decl",
            flowsight_cpg::DefKind::Assignment => "assign",
            flowsight_cpg::DefKind::CompoundAssignment(op) => op,
            flowsight_cpg::DefKind::Parameter { index } => {
                print!("    ");
                println!(
                    "L{}: {} (parameter #{})",
                    def.line.to_string().with(C_DIM),
                    var_name.with(C_DEF),
                    index,
                );
                continue;
            }
            flowsight_cpg::DefKind::CallReturn { callee } => {
                print!("    ");
                println!(
                    "L{}: {} = {}()",
                    def.line.to_string().with(C_DIM),
                    var_name.with(C_DEF),
                    callee.as_str().with(C_FILE),
                );
                continue;
            }
            flowsight_cpg::DefKind::FieldAssignment { base, field } => {
                print!("    ");
                println!(
                    "L{}: {}->{}",
                    def.line.to_string().with(C_DIM),
                    base.as_str().with(C_DEF),
                    field.as_str().with(C_FILE),
                );
                continue;
            }
        };
        print!("    ");
        let rhs = def.rhs_text.as_deref().unwrap_or("?");
        let rhs_short = if rhs.len() > 40 { &rhs[..40] } else { rhs };
        println!(
            "L{}: {} = {} [{}]",
            def.line.to_string().with(C_DIM),
            var_name.with(C_DEF),
            rhs_short.with(C_FILE),
            kind_str.with(C_COND),
        );
    }
    println!();

    // Uses
    println!("{}", "  Uses:".with(C_TITLE));
    if uses.is_empty() {
        println!("    {}", "(none)".with(C_DIM));
    }
    for u in &uses {
        let kind_str = match &u.kind {
            flowsight_cpg::UseKind::Argument { callee, arg_index } => {
                format!("arg #{} of {}()", arg_index, callee)
            }
            flowsight_cpg::UseKind::Condition => "condition".to_string(),
            flowsight_cpg::UseKind::RhsExpression => "rhs".to_string(),
            flowsight_cpg::UseKind::ReturnValue => "return".to_string(),
            flowsight_cpg::UseKind::Dereference => "deref".to_string(),
            flowsight_cpg::UseKind::FieldAccess { field } => format!("field .{}", field),
            flowsight_cpg::UseKind::GotoCondition => "goto-cond".to_string(),
            flowsight_cpg::UseKind::Expression => "expr".to_string(),
        };
        let color = match &u.kind {
            flowsight_cpg::UseKind::ReturnValue => C_ERR,
            flowsight_cpg::UseKind::Condition => C_COND,
            _ => C_OK,
        };
        println!(
            "    L{}: {} [{}]",
            u.line.to_string().with(C_DIM),
            var_name.with(C_FILE),
            kind_str.with(color),
        );
    }
    println!();

    // Def-use chains
    let chains: Vec<_> = cpg.def_use_chains.iter().filter(|c| c.var_name == var_name).collect();
    if !chains.is_empty() {
        println!("{}", "  Def-use chains:".with(C_TITLE));
        for chain in &chains {
            println!(
                "    def L{} → used at L{}",
                chain.def_line.to_string().with(C_DEF),
                chain.use_lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ").with(C_OK),
            );
        }
    }
}

/// Print full data flow summary
fn print_full_dataflow(cpg: &flowsight_cpg::CodePropertyGraph) {
    let filename = cpg.source_file.as_deref().unwrap_or("");

    println!(
        "{} {}() {}",
        "Data flow:".with(C_TITLE),
        cpg.function_name.as_str().with(C_FILE),
        if filename.is_empty() { String::new() } else { format!("({})", filename) }.with(C_DIM),
    );
    println!(
        "  {}",
        format!("{}", cpg.stats).with(C_DIM),
    );
    println!();

    // Parameters
    if !cpg.parameters.is_empty() {
        println!("{}", "  Parameters:".with(C_TITLE));
        for p in &cpg.parameters {
            let uses = cpg.uses_of(&p.name);
            println!(
                "    {} {} ({} uses)",
                p.type_name.as_str().with(C_DIM),
                p.name.as_str().with(C_DEF),
                uses.len().to_string().with(C_OK),
            );
        }
        println!();
    }

    // Variables with their flow
    let vars = cpg.all_variables();
    let non_param_vars: Vec<_> = vars
        .iter()
        .filter(|v| !cpg.parameters.iter().any(|p| &p.name == *v))
        .collect();

    if !non_param_vars.is_empty() {
        println!("{}", "  Local variables:".with(C_TITLE));
        for var in &non_param_vars {
            let defs = cpg.defs_of(var);
            let uses = cpg.uses_of(var);
            let def_summary: Vec<String> = defs.iter().map(|d| {
                match &d.kind {
                    flowsight_cpg::DefKind::CallReturn { callee } => format!("L{}={}()", d.line, callee),
                    flowsight_cpg::DefKind::FieldAssignment { base, field } => format!("L{} {}->{}",d.line, base, field),
                    _ => format!("L{}", d.line),
                }
            }).collect();

            println!(
                "    {} ({} defs: {}, {} uses)",
                var.as_str().with(C_DEF),
                defs.len(),
                def_summary.join(", ").with(C_DIM),
                uses.len().to_string().with(C_OK),
            );
        }
        println!();
    }

    // Returns
    if !cpg.returns.is_empty() {
        println!("{}", "  Returns:".with(C_TITLE));
        for r in &cpg.returns {
            let tag = if r.is_error { "[error]" } else { "[ok]" };
            let color = if r.is_error { C_ERR } else { C_OK };
            println!(
                "    L{}: {} {}",
                r.line.to_string().with(C_DIM),
                tag.with(color),
                r.expression.as_deref().unwrap_or("void").with(C_FILE),
            );
        }
    }
}
