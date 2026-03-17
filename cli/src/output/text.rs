//! Human-readable text output formatting

use crate::commands::analyze::DirectorySummary;
use crossterm::style::{Color, Stylize};
use flowsight_core::{AsyncMechanism, FlowNode, FlowNodeType, FunctionDef};
use std::collections::HashMap;
use std::path::Path;

/// Print a flow tree in indented tree format
pub fn print_flow_tree(node: &FlowNode, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!("{}{}", prefix, node.display_name);

    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == node.children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        print!("{}{}", prefix, connector);
        print_flow_tree(child, indent + 1);
    }
}

/// Get the icon for an async mechanism type
pub fn mechanism_icon(mechanism: &AsyncMechanism) -> &'static str {
    match mechanism {
        AsyncMechanism::WorkQueue { .. } => "⚙️ ",
        AsyncMechanism::Timer { .. } => "⏲️ ",
        AsyncMechanism::Interrupt { .. } => "⚡",
        AsyncMechanism::Tasklet => "🔄",
        AsyncMechanism::KThread => "🧵",
        _ => "📍",
    }
}

/// Print ftrace-style flow tree
pub fn print_ftrace_tree(
    node: &FlowNode,
    depth: usize,
    functions: &HashMap<String, FunctionDef>,
) {
    let indent = "  ".repeat(depth);
    let cpu = " 0)";

    let line_info = if let Some(loc) = &node.location {
        format!("L{:<4}", loc.line)
    } else if let Some(func) = functions.get(&node.name) {
        if let Some(loc) = &func.location {
            format!("L{:<4}", loc.line)
        } else {
            "     ".to_string()
        }
    } else {
        "     ".to_string()
    };

    let async_tag = match &node.node_type {
        FlowNodeType::AsyncCallback { mechanism } => match mechanism {
            AsyncMechanism::WorkQueue { .. } => " [WQ]",
            AsyncMechanism::Timer { .. } => " [TM]",
            AsyncMechanism::Interrupt { .. } => " [IRQ]",
            AsyncMechanism::Tasklet => " [TL]",
            AsyncMechanism::KThread => " [KT]",
            _ => " [A]",
        },
        FlowNodeType::KernelApi => " [K]",
        FlowNodeType::External => " [E]",
        _ => "",
    };

    if node.children.is_empty() {
        println!(
            "{}{} {} |{}{}();{}",
            cpu, line_info, indent, indent, node.name, async_tag
        );
    } else {
        println!(
            "{}{} {} |{}{}() {{{}",
            cpu, line_info, indent, indent, node.name, async_tag
        );
        for child in &node.children {
            print_ftrace_tree(child, depth + 1, functions);
        }
        println!("{}{} {} |{}}}", cpu, line_info, indent, indent);
    }
}

/// Format a number with comma separators (e.g. 1234 -> "1,234")
fn format_count(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

/// Print single-file analysis result (text format)
pub fn print_file_analysis(
    file: &Path,
    functions: usize,
    structs: usize,
    async_handlers: usize,
    entry_points: usize,
) {
    let label_color = Color::DarkCyan;
    let value_color = Color::Grey;

    println!(
        "{} {}",
        "Analyzing:".with(label_color),
        file.display().to_string().with(value_color)
    );
    println!(
        "  Functions:      {}",
        format_count(functions).with(value_color)
    );
    println!(
        "  Structs:        {}",
        format_count(structs).with(value_color)
    );
    println!(
        "  Async handlers: {}",
        format_count(async_handlers).with(value_color)
    );
    println!(
        "  Entry points:   {}",
        format_count(entry_points).with(value_color)
    );
}

/// Print directory analysis with per-file details followed by summary
pub fn print_directory_full(summary: &DirectorySummary) {
    let label_color = Color::DarkCyan;
    let dim_color = Color::DarkGrey;

    for stats in &summary.per_file {
        println!(
            "{} fn:{:<4} async:{:<3} cb:{:<3} ep:{:<3}  {}",
            "  ".with(dim_color),
            stats.functions,
            stats.async_handlers,
            stats.callbacks,
            stats.entry_points,
            stats.file.as_str().with(dim_color),
        );
    }

    if !summary.errors.is_empty() {
        println!();
        println!("{}", "Warnings:".with(Color::DarkYellow));
        for err in &summary.errors {
            println!("  {}", err.as_str().with(Color::DarkYellow));
        }
    }

    println!();
    print_summary_block(summary, &label_color);
}

/// Print only the summary block (used by --summary flag)
pub fn print_directory_summary(summary: &DirectorySummary) {
    let label_color = Color::DarkCyan;
    print_summary_block(summary, &label_color);
}

/// Render the summary statistics box
fn print_summary_block(summary: &DirectorySummary, label_color: &Color) {
    let value_color = Color::Grey;
    let title = "FlowSight Analysis Summary";
    let line = "\u{2550}".repeat(title.len());

    println!("{}", title.with(*label_color));
    println!("{}", line.with(*label_color));
    println!(
        "  {} {}",
        "Directory:".with(*label_color),
        summary.directory.as_str().with(value_color)
    );
    println!(
        "  {} {}",
        "Files analyzed:".with(*label_color),
        format_count(summary.files_analyzed).with(value_color)
    );
    println!(
        "  {} {}",
        "Total functions:".with(*label_color),
        format_count(summary.total_functions).with(value_color)
    );
    println!(
        "  {} {}",
        "Async handlers:".with(*label_color),
        format_count(summary.total_async_handlers).with(value_color)
    );
    println!(
        "  {} {}",
        "Callbacks:".with(*label_color),
        format_count(summary.total_callbacks).with(value_color)
    );
    println!(
        "  {} {}",
        "Entry points:".with(*label_color),
        format_count(summary.total_entry_points).with(value_color)
    );

    if summary.files_skipped > 0 {
        println!(
            "  {} {}",
            "Errors:".with(Color::DarkYellow),
            format!("{} files skipped", summary.files_skipped).with(Color::DarkYellow)
        );
    }
}
