//! Call graph construction

use flowsight_core::{CallEdge, CallType, AsyncBinding, FlowNode, FlowNodeType, AsyncMechanism};
use flowsight_parser::ParseResult;
use std::collections::HashSet;

/// Build call edges from parse result
pub fn build_call_edges(parse_result: &ParseResult, async_bindings: &[AsyncBinding]) -> Vec<CallEdge> {
    let mut edges = Vec::new();

    // Direct calls
    for (caller_name, caller) in &parse_result.functions {
        for callee_name in &caller.calls {
            edges.push(CallEdge {
                caller: caller_name.clone(),
                callee: callee_name.clone(),
                location: caller.location.clone(),
                call_type: CallType::Direct,
            });
        }
    }

    // Async calls
    for binding in async_bindings {
        // Find functions that contain trigger patterns
        for (func_name, func) in &parse_result.functions {
            // Check if this function triggers the async handler
            for trigger_loc in &binding.trigger_locations {
                if let Some(func_loc) = &func.location {
                    // Simple check: trigger is within function range
                    if trigger_loc.line >= func_loc.line && trigger_loc.line <= func_loc.end_line {
                        edges.push(CallEdge {
                            caller: func_name.clone(),
                            callee: binding.handler.clone(),
                            location: Some(trigger_loc.clone()),
                            call_type: CallType::Async {
                                mechanism: binding.mechanism.clone(),
                            },
                        });
                    }
                }
            }
        }
    }

    edges
}

/// Build execution flow tree for an entry point
pub fn build_flow_tree(
    entry: &str,
    parse_result: &ParseResult,
    async_bindings: &[AsyncBinding],
    visited: &mut HashSet<String>,
    depth: usize,
) -> Option<FlowNode> {
    const MAX_DEPTH: usize = 20;

    if depth > MAX_DEPTH {
        return None;
    }

    // Allow revisiting at different depths, but not in current call stack
    if visited.contains(entry) {
        // Return a reference node instead of None
        return Some(FlowNode {
            id: format!("{}-ref-{}", entry, depth),
            name: entry.to_string(),
            display_name: format!("↩️ {}() [递归]", entry),
            location: None,
            node_type: FlowNodeType::Function,
            children: vec![],
            description: Some("递归调用，点击入口点查看完整树".to_string()),
        });
    }

    visited.insert(entry.to_string());

    // If function not found in parse result, create a basic node
    let func = match parse_result.functions.get(entry) {
        Some(f) => f,
        None => {
            visited.remove(entry);
            return Some(FlowNode {
                id: entry.to_string(),
                name: entry.to_string(),
                display_name: format!("📦 {}()", entry),
                location: None,
                node_type: FlowNodeType::External,
                children: vec![],
                description: Some("External function".to_string()),
            });
        }
    };

    let node_type = if func.is_callback {
        if let Some(ctx) = &func.callback_context {
            if ctx.starts_with("async_") {
                // Find the async mechanism
                let mechanism = async_bindings
                    .iter()
                    .find(|b| b.handler == entry)
                    .map(|b| b.mechanism.clone())
                    .unwrap_or(AsyncMechanism::Custom("unknown".into()));
                FlowNodeType::AsyncCallback { mechanism }
            } else {
                FlowNodeType::EntryPoint
            }
        } else {
            FlowNodeType::EntryPoint
        }
    } else {
        FlowNodeType::Function
    };

    let display_name = match &node_type {
        FlowNodeType::EntryPoint => {
            let ctx = func.callback_context.as_deref().unwrap_or("callback");
            format!("🔌 [{}] {}()", ctx, entry)
        }
        FlowNodeType::AsyncCallback { mechanism } => {
            let icon = match mechanism {
                AsyncMechanism::WorkQueue { .. } => "⚙️",
                AsyncMechanism::Timer { .. } => "⏲️",
                AsyncMechanism::Interrupt { .. } => "⚡",
                AsyncMechanism::Tasklet => "🔄",
                AsyncMechanism::KThread => "🧵",
                _ => "📍",
            };
            format!("{} {}()", icon, entry)
        }
        _ => format!("{}()", entry),
    };

    // Build children
    let mut children = Vec::new();
    for callee in &func.calls {
        if parse_result.functions.contains_key(callee) {
            // Recurse for internal functions
            if let Some(child) = build_flow_tree(callee, parse_result, async_bindings, visited, depth + 1) {
                children.push(child);
            }
        } else {
            // External/kernel API
            children.push(FlowNode {
                id: format!("{}-{}", entry, callee),
                name: callee.clone(),
                display_name: format!("{}()", callee),
                location: None,
                node_type: FlowNodeType::KernelApi,
                children: vec![],
                description: None,
            });
        }
    }

    // Add async handlers triggered by this function
    for binding in async_bindings {
        for trigger_loc in &binding.trigger_locations {
            if let Some(func_loc) = &func.location {
                if trigger_loc.line >= func_loc.line && trigger_loc.line <= func_loc.end_line {
                    // This function triggers an async handler
                    if let Some(async_child) = build_flow_tree(&binding.handler, parse_result, async_bindings, visited, depth + 1) {
                        children.push(async_child);
                    }
                }
            }
        }
    }

    visited.remove(entry);

    Some(FlowNode {
        id: entry.to_string(),
        name: entry.to_string(),
        display_name,
        location: func.location.clone(),
        node_type,
        children,
        description: func.callback_context.clone(),
    })
}

