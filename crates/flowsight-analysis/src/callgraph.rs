//! Call graph construction
//!
//! 核心功能：将用户代码的执行流与内核调用链关联
//!
//! 当检测到入口点函数（如 probe, work handler）时，
//! 自动注入完整的内核调用链，让用户看到真正的执行流程。

use flowsight_core::{AsyncBinding, AsyncMechanism, CallEdge, CallType, FlowNode, FlowNodeType, CallConfidence, ConfidenceLevel};
use flowsight_knowledge::{KnowledgeBase, CallChain};
use flowsight_parser::ParseResult;
use std::collections::HashSet;

/// Build call edges from parse result
pub fn build_call_edges(
    parse_result: &ParseResult,
    async_bindings: &[AsyncBinding],
) -> Vec<CallEdge> {
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
            confidence: None,
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
                confidence: Some(CallConfidence {
                    level: ConfidenceLevel::Unknown,
                    reason: "External function - definition not found".to_string(),
                }),
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
            if let Some(child) =
                build_flow_tree(callee, parse_result, async_bindings, visited, depth + 1)
            {
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
                confidence: Some(CallConfidence {
                    level: ConfidenceLevel::Certain,
                    reason: "Direct call to kernel API".to_string(),
                }),
            });
        }
    }

    // Add async handlers triggered by this function
    for binding in async_bindings {
        for trigger_loc in &binding.trigger_locations {
            if let Some(func_loc) = &func.location {
                if trigger_loc.line >= func_loc.line && trigger_loc.line <= func_loc.end_line {
                    // This function triggers an async handler
                    if let Some(async_child) = build_flow_tree(
                        &binding.handler,
                        parse_result,
                        async_bindings,
                        visited,
                        depth + 1,
                    ) {
                        children.push(async_child);
                    }
                }
            }
        }
    }

    visited.remove(entry);

    // Set confidence based on node type
    let confidence = match &node_type {
        FlowNodeType::EntryPoint => Some(CallConfidence {
            level: ConfidenceLevel::Certain,
            reason: "Callback entry point from ops table".to_string(),
        }),
        FlowNodeType::AsyncCallback { .. } => Some(CallConfidence {
            level: ConfidenceLevel::Certain,
            reason: "Async callback with known binding".to_string(),
        }),
        FlowNodeType::Function => None, // Direct call - certain by default
        FlowNodeType::KernelApi => Some(CallConfidence {
            level: ConfidenceLevel::Certain,
            reason: "Kernel API call".to_string(),
        }),
        FlowNodeType::External => Some(CallConfidence {
            level: ConfidenceLevel::Unknown,
            reason: "External function - definition not available".to_string(),
        }),
    };

    Some(FlowNode {
        id: entry.to_string(),
        name: entry.to_string(),
        display_name,
        location: func.location.clone(),
        node_type,
        children,
        description: func.callback_context.clone(),
        confidence,
    })
}

/// ⭐ 构建带完整内核调用链的执行流树
/// 
/// 当检测到入口点函数时，自动在前面注入内核调用链，
/// 让用户看到从触发源到用户代码的完整执行路径。
pub fn build_full_flow_tree(
    entry: &str,
    parse_result: &ParseResult,
    async_bindings: &[AsyncBinding],
    kb: &KnowledgeBase,
) -> Option<FlowNode> {
    let mut visited = HashSet::new();
    
    // 首先构建用户代码的流树
    let user_tree = build_flow_tree(entry, parse_result, async_bindings, &mut visited, 0)?;
    
    // 检查是否有关联的内核调用链
    let func = parse_result.functions.get(entry)?;
    
    if let Some(ctx) = &func.callback_context {
        // 尝试查找对应的内核调用链
        if let Some(ref call_chain) = find_call_chain_for_context(ctx, kb) {
            // 注入内核调用链，将用户函数作为最后一个节点
            return Some(inject_kernel_chain(call_chain, user_tree));
        }
    }
    
    // 检查是否是异步 handler
    for binding in async_bindings {
        if binding.handler == entry {
            if let Some(ref call_chain) = get_async_handler_chain(&binding.mechanism, kb) {
                return Some(inject_kernel_chain(call_chain, user_tree));
            }
        }
    }
    
    Some(user_tree)
}

/// 根据 callback context 查找对应的内核调用链
fn find_call_chain_for_context(ctx: &str, kb: &KnowledgeBase) -> Option<CallChain> {
    // 解析 context，如 "usb_driver.probe" -> (usb_driver, probe)
    let parts: Vec<&str> = ctx.split('.').collect();
    if parts.len() == 2 {
        if let Some(chain) = kb.get_callback_call_chain(parts[0], parts[1]) {
            return Some(chain.clone());
        }
    }
    
    // 常见模式匹配
    match ctx {
        "probe" | "usb_probe" | "usb_driver.probe" => {
            kb.get_callback_call_chain("usb_driver", "probe").cloned()
        }
        "disconnect" | "usb_disconnect" | "usb_driver.disconnect" => {
            kb.get_callback_call_chain("usb_driver", "disconnect").cloned()
        }
        "open" | "file_operations.open" => {
            kb.get_callback_call_chain("file_operations", "open").cloned()
        }
        _ => None
    }
}

/// 获取异步机制的 handler 调用链
fn get_async_handler_chain(mechanism: &AsyncMechanism, kb: &KnowledgeBase) -> Option<CallChain> {
    match mechanism {
        AsyncMechanism::WorkQueue { .. } => {
            kb.get_async_handler_chain("work_struct").cloned()
        }
        AsyncMechanism::Timer { .. } => {
            kb.get_async_handler_chain("timer_list").cloned()
        }
        _ => None
    }
}

/// 将内核调用链注入到用户代码树前面
fn inject_kernel_chain(call_chain: &CallChain, user_tree: FlowNode) -> FlowNode {
    if call_chain.nodes.is_empty() {
        return user_tree;
    }

    // 从触发源开始构建树
    let trigger_node = FlowNode {
        id: "trigger-source".to_string(),
        name: call_chain.trigger_source.clone(),
        display_name: format!("🎯 {}", call_chain.trigger_source),
        location: None,
        node_type: FlowNodeType::External,
        children: vec![build_kernel_chain_tree(&call_chain.nodes, user_tree, 0)],
        description: Some(call_chain.name.clone()),
        confidence: Some(CallConfidence {
            level: ConfidenceLevel::Certain,
            reason: "Kernel call chain trigger".to_string(),
        }),
    };

    trigger_node
}

/// 递归构建内核调用链树
fn build_kernel_chain_tree(nodes: &[flowsight_knowledge::CallChainNode], user_tree: FlowNode, idx: usize) -> FlowNode {
    if idx >= nodes.len() {
        // 所有内核节点都处理完了，返回用户树
        return user_tree;
    }

    let node = &nodes[idx];

    // 如果是用户入口点，替换为实际的用户树
    if node.is_user_entry {
        // 用户树替换这个占位符
        return FlowNode {
            id: format!("kernel-{}", idx),
            name: node.function.clone(),
            display_name: format!("🔗 {} → {}", node.function, user_tree.name),
            location: None,
            node_type: FlowNodeType::KernelApi,
            children: vec![user_tree],
            description: node.description.clone(),
            confidence: Some(CallConfidence {
                level: ConfidenceLevel::Certain,
                reason: "Kernel to user callback".to_string(),
            }),
        };
    }

    // 继续构建链
    let child = build_kernel_chain_tree(nodes, user_tree, idx + 1);

    let icon = match node.context {
        flowsight_knowledge::ExecutionContext::HardIrq => "⚡",
        flowsight_knowledge::ExecutionContext::SoftIrq => "🔄",
        flowsight_knowledge::ExecutionContext::Process => "📦",
        flowsight_knowledge::ExecutionContext::User => "👤",
        flowsight_knowledge::ExecutionContext::Unknown => "❓",
    };

    FlowNode {
        id: format!("kernel-{}", idx),
        name: node.function.clone(),
        display_name: format!("{} {}()", icon, node.function),
        location: None,
        node_type: FlowNodeType::KernelApi,
        children: vec![child],
        description: Some(format!(
            "{} | {}",
            node.file.as_deref().unwrap_or("kernel"),
            node.description.as_deref().unwrap_or("")
        )),
        confidence: Some(CallConfidence {
            level: ConfidenceLevel::Certain,
            reason: "Kernel internal call".to_string(),
        }),
    }
}
