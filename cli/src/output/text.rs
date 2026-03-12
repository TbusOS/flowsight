//! Human-readable text output formatting

use flowsight_core::{AsyncMechanism, FlowNode, FlowNodeType, FunctionDef};
use std::collections::HashMap;

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
