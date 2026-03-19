//! Graphviz DOT format output
//!
//! Generates DOT graph descriptions from analysis results.
//! Users pipe output to `dot -Tsvg` or `dot -Tpng` for rendering.

use flowsight_analysis::AnalysisResult;
use flowsight_core::{FlowNode, FlowNodeType, FunctionDef};
use flowsight_parser::ParseResult;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

// -- Low-saturation palette (hex) -----------------------------------------

const COLOR_NORMAL: &str = "#f5f5f0";
const COLOR_KERNEL_API: &str = "#d0d0d0";
const COLOR_ASYNC_HANDLER: &str = "#b0c4de";
const COLOR_CALLBACK: &str = "#e8deb5";
const COLOR_ENTRY: &str = "#e8e8e8";
const COLOR_EXTERNAL: &str = "#ddd5d0";
const COLOR_BRANCH: &str = "#d8e8d0";

// -- Public API -----------------------------------------------------------

/// Generate DOT graph from an execution flow tree.
pub fn flow_to_dot(node: &FlowNode, function_name: &str) -> String {
    let mut buf = String::with_capacity(2048);
    let mut edges: Vec<(String, String, Option<String>)> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    write_header(&mut buf, function_name);
    collect_flow_nodes(&mut buf, &mut edges, &mut visited, node);
    write_edges(&mut buf, &edges);
    write_legend(&mut buf);
    write_footer(&mut buf);

    buf
}

/// Generate DOT graph from callers list.
pub fn callers_to_dot(
    function: &str,
    parse_result: &ParseResult,
    async_bindings: &[flowsight_core::AsyncBinding],
) -> String {
    let mut buf = String::with_capacity(1024);

    write_header(&mut buf, &format!("callers of {}", function));
    writeln_buf(
        &mut buf,
        &format!(
            "  \"{}\" [fillcolor=\"{}\", shape=doubleoctagon];",
            escape_dot(function),
            COLOR_ENTRY
        ),
    );

    // Direct callers
    for (name, func) in &parse_result.functions {
        if func.calls.contains(&function.to_string()) {
            write_function_node(&mut buf, name, func);
            writeln_buf(
                &mut buf,
                &format!(
                    "  \"{}\" -> \"{}\" [label=\"direct\"];",
                    escape_dot(name),
                    escape_dot(function)
                ),
            );
        }
    }

    // Async callers
    for binding in async_bindings {
        if binding.handler == function {
            let mechanism_label = format_mechanism_short(&binding.mechanism);
            let via_id = format!("async_{}", escape_dot(&binding.variable));
            writeln_buf(
                &mut buf,
                &format!(
                    "  \"{}\" [fillcolor=\"{}\", label=\"{}\\n[async: {}]\"];",
                    via_id, COLOR_ASYNC_HANDLER, binding.variable, mechanism_label
                ),
            );
            writeln_buf(
                &mut buf,
                &format!(
                    "  \"{}\" -> \"{}\" [style=dashed, label=\"handler\"];",
                    via_id,
                    escape_dot(function)
                ),
            );
        }
    }

    write_legend(&mut buf);
    write_footer(&mut buf);

    buf
}

/// Generate DOT graph from callees list.
pub fn callees_to_dot(function: &str, parse_result: &ParseResult) -> String {
    let mut buf = String::with_capacity(1024);

    write_header(&mut buf, &format!("callees of {}", function));
    writeln_buf(
        &mut buf,
        &format!(
            "  \"{}\" [fillcolor=\"{}\", shape=doubleoctagon];",
            escape_dot(function),
            COLOR_ENTRY
        ),
    );

    if let Some(func) = parse_result.functions.get(function) {
        for callee in &func.calls {
            let is_local = parse_result.functions.contains_key(callee);
            let color = if is_local {
                COLOR_NORMAL
            } else {
                COLOR_EXTERNAL
            };
            let label_suffix = if is_local { "" } else { "\\n[external]" };

            writeln_buf(
                &mut buf,
                &format!(
                    "  \"{}\" [fillcolor=\"{}\", label=\"{}{}\"];",
                    escape_dot(callee),
                    color,
                    escape_dot(callee),
                    label_suffix
                ),
            );
            writeln_buf(
                &mut buf,
                &format!(
                    "  \"{}\" -> \"{}\";",
                    escape_dot(function),
                    escape_dot(callee)
                ),
            );
        }
    }

    write_legend(&mut buf);
    write_footer(&mut buf);

    buf
}

/// Generate full file call graph in DOT format.
pub fn file_call_graph_to_dot(
    parse_result: &ParseResult,
    analysis: &AnalysisResult,
    filename: &str,
    exclude_kernel: bool,
    cluster: bool,
) -> String {
    let mut buf = String::with_capacity(4096);

    write_header(&mut buf, &format!("call graph: {}", filename));

    let entry_set: HashSet<&str> = analysis.entry_points.iter().map(|s| s.as_str()).collect();

    if cluster {
        write_clustered_graph(
            &mut buf,
            parse_result,
            &entry_set,
            exclude_kernel,
        );
    } else {
        write_flat_graph(
            &mut buf,
            parse_result,
            &entry_set,
            exclude_kernel,
        );
    }

    write_legend(&mut buf);
    write_footer(&mut buf);

    buf
}

// -- Internal: graph building ---------------------------------------------

fn write_flat_graph(
    buf: &mut String,
    parse_result: &ParseResult,
    entry_set: &HashSet<&str>,
    exclude_kernel: bool,
) {
    let mut emitted_nodes: HashSet<String> = HashSet::new();

    for (name, func) in &parse_result.functions {
        emit_function_with_edges(
            buf,
            name,
            func,
            parse_result,
            entry_set,
            exclude_kernel,
            &mut emitted_nodes,
        );
    }
}

fn write_clustered_graph(
    buf: &mut String,
    parse_result: &ParseResult,
    entry_set: &HashSet<&str>,
    exclude_kernel: bool,
) {
    // Group functions by call depth (0 = entry points, 1 = called by entries, etc.)
    let depths = compute_call_depths(parse_result, entry_set);
    let max_depth = depths.values().copied().max().unwrap_or(0);

    let mut emitted_nodes: HashSet<String> = HashSet::new();

    for depth in 0..=max_depth {
        let funcs_at_depth: Vec<&str> = depths
            .iter()
            .filter(|(_, &d)| d == depth)
            .map(|(name, _)| name.as_str())
            .collect();

        if funcs_at_depth.is_empty() {
            continue;
        }

        writeln_buf(buf, &format!("  subgraph cluster_{} {{", depth));
        writeln_buf(buf, &format!("    label=\"depth {}\";", depth));
        writeln_buf(buf, "    style=dashed;");
        writeln_buf(buf, "    color=\"#999999\";");

        for name in &funcs_at_depth {
            if let Some(func) = parse_result.functions.get(*name) {
                let shape = if entry_set.contains(name) {
                    "doubleoctagon"
                } else {
                    "box"
                };
                let color = node_color_for_func(func, entry_set.contains(name));
                writeln_buf(
                    buf,
                    &format!(
                        "    \"{}\" [fillcolor=\"{}\", shape={}];",
                        escape_dot(name),
                        color,
                        shape
                    ),
                );
                emitted_nodes.insert(name.to_string());
            }
        }

        writeln_buf(buf, "  }");
    }

    // Edges (outside clusters)
    for (name, func) in &parse_result.functions {
        for callee in &func.calls {
            if exclude_kernel && !parse_result.functions.contains_key(callee) {
                continue;
            }
            if !emitted_nodes.contains(callee) {
                let color = COLOR_EXTERNAL;
                writeln_buf(
                    buf,
                    &format!(
                        "  \"{}\" [fillcolor=\"{}\", label=\"{}\\n[external]\"];",
                        escape_dot(callee),
                        color,
                        escape_dot(callee)
                    ),
                );
                emitted_nodes.insert(callee.clone());
            }
            writeln_buf(
                buf,
                &format!(
                    "  \"{}\" -> \"{}\";",
                    escape_dot(name),
                    escape_dot(callee)
                ),
            );
        }
    }
}

fn emit_function_with_edges(
    buf: &mut String,
    name: &str,
    func: &FunctionDef,
    parse_result: &ParseResult,
    entry_set: &HashSet<&str>,
    exclude_kernel: bool,
    emitted_nodes: &mut HashSet<String>,
) {
    if !emitted_nodes.contains(name) {
        let shape = if entry_set.contains(name) {
            "doubleoctagon"
        } else {
            "box"
        };
        let color = node_color_for_func(func, entry_set.contains(name));
        writeln_buf(
            buf,
            &format!(
                "  \"{}\" [fillcolor=\"{}\", shape={}];",
                escape_dot(name),
                color,
                shape
            ),
        );
        emitted_nodes.insert(name.to_string());
    }

    for callee in &func.calls {
        if exclude_kernel && !parse_result.functions.contains_key(callee) {
            continue;
        }
        if !emitted_nodes.contains(callee) {
            let is_local = parse_result.functions.contains_key(callee);
            let color = if is_local {
                COLOR_KERNEL_API
            } else {
                COLOR_EXTERNAL
            };
            let label_suffix = if is_local { "" } else { "\\n[external]" };
            writeln_buf(
                buf,
                &format!(
                    "  \"{}\" [fillcolor=\"{}\", label=\"{}{}\"];",
                    escape_dot(callee),
                    color,
                    escape_dot(callee),
                    label_suffix
                ),
            );
            emitted_nodes.insert(callee.clone());
        }
        writeln_buf(
            buf,
            &format!(
                "  \"{}\" -> \"{}\";",
                escape_dot(name),
                escape_dot(callee)
            ),
        );
    }
}

fn compute_call_depths(
    parse_result: &ParseResult,
    entry_set: &HashSet<&str>,
) -> HashMap<String, usize> {
    let mut depths: HashMap<String, usize> = HashMap::new();
    let mut queue: Vec<(String, usize)> = entry_set
        .iter()
        .map(|name| (name.to_string(), 0))
        .collect();

    while let Some((name, depth)) = queue.pop() {
        if depths.contains_key(&name) {
            continue;
        }
        depths.insert(name.clone(), depth);

        if let Some(func) = parse_result.functions.get(&name) {
            for callee in &func.calls {
                if parse_result.functions.contains_key(callee) && !depths.contains_key(callee) {
                    queue.push((callee.clone(), depth + 1));
                }
            }
        }
    }

    // Include any functions not reachable from entry points
    for name in parse_result.functions.keys() {
        depths.entry(name.clone()).or_insert(0);
    }

    depths
}

// -- Internal: flow tree traversal ----------------------------------------

fn collect_flow_nodes(
    buf: &mut String,
    edges: &mut Vec<(String, String, Option<String>)>,
    visited: &mut HashSet<String>,
    node: &FlowNode,
) {
    let node_id = stable_node_id(node);

    if visited.contains(&node_id) {
        return;
    }
    visited.insert(node_id.clone());

    let (color, shape, label_suffix) = flow_node_style(node);
    let label = if label_suffix.is_empty() {
        escape_dot(&node.name)
    } else {
        format!("{}\\n{}", escape_dot(&node.name), label_suffix)
    };

    writeln_buf(
        buf,
        &format!(
            "  \"{}\" [fillcolor=\"{}\", shape={}, label=\"{}\"];",
            node_id, color, shape, label
        ),
    );

    for child in &node.children {
        let child_id = stable_node_id(child);
        let edge_label = edge_label_for_node(child);
        edges.push((node_id.clone(), child_id, edge_label));
        collect_flow_nodes(buf, edges, visited, child);
    }
}

fn flow_node_style(node: &FlowNode) -> (&str, &str, &str) {
    match &node.node_type {
        FlowNodeType::EntryPoint => (COLOR_ENTRY, "doubleoctagon", "[entry]"),
        FlowNodeType::KernelApi => (COLOR_KERNEL_API, "box", "[kernel API]"),
        FlowNodeType::AsyncCallback { .. } => (COLOR_ASYNC_HANDLER, "box", "[async handler]"),
        FlowNodeType::External => (COLOR_EXTERNAL, "box", "[external]"),
        FlowNodeType::Branch { condition, .. } => {
            // Use condition as suffix, truncated
            let _ = condition;
            (COLOR_BRANCH, "diamond", "[branch]")
        }
        FlowNodeType::Separator { .. } => (COLOR_NORMAL, "plaintext", ""),
        FlowNodeType::Function => (COLOR_NORMAL, "box", ""),
    }
}

fn edge_label_for_node(child: &FlowNode) -> Option<String> {
    match &child.node_type {
        FlowNodeType::AsyncCallback { mechanism } => {
            Some(format!("async: {}", format_mechanism_short(mechanism)))
        }
        FlowNodeType::Branch {
            branch_type,
            condition,
        } => {
            let bt = format!("{:?}", branch_type).to_lowercase();
            if condition.is_empty() {
                Some(bt)
            } else {
                let short = truncate(condition, 20);
                Some(format!("{}: {}", bt, short))
            }
        }
        _ => None,
    }
}

fn stable_node_id(node: &FlowNode) -> String {
    if node.id.is_empty() {
        node.name.clone()
    } else {
        node.id.clone()
    }
}

// -- Internal: DOT helpers ------------------------------------------------

fn write_header(buf: &mut String, title: &str) {
    writeln_buf(buf, &format!("digraph \"{}\" {{", escape_dot(title)));
    writeln_buf(buf, "  rankdir=TB;");
    writeln_buf(
        buf,
        "  node [shape=box, style=filled, fontname=\"monospace\", fontsize=10];",
    );
    writeln_buf(
        buf,
        "  edge [fontname=\"monospace\", fontsize=8];",
    );
    writeln_buf(buf, "");
}

fn write_edges(buf: &mut String, edges: &[(String, String, Option<String>)]) {
    writeln_buf(buf, "");
    writeln_buf(buf, "  // Edges");
    for (from, to, label) in edges {
        match label {
            Some(lbl) => writeln_buf(
                buf,
                &format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\"];",
                    from,
                    to,
                    escape_dot(lbl)
                ),
            ),
            None => writeln_buf(buf, &format!("  \"{}\" -> \"{}\";", from, to)),
        }
    }
}

fn write_legend(buf: &mut String) {
    writeln_buf(buf, "");
    writeln_buf(buf, "  // Legend");
    writeln_buf(buf, "  subgraph cluster_legend {");
    writeln_buf(buf, "    label=\"Legend\";");
    writeln_buf(buf, "    style=dashed;");
    writeln_buf(buf, "    color=\"#999999\";");
    writeln_buf(buf, "    fontname=\"monospace\";");
    writeln_buf(buf, "    fontsize=9;");
    writeln_buf(
        buf,
        &format!(
            "    leg_entry [fillcolor=\"{}\", shape=doubleoctagon, label=\"entry point\"];",
            COLOR_ENTRY
        ),
    );
    writeln_buf(
        buf,
        &format!(
            "    leg_kernel [fillcolor=\"{}\", label=\"kernel API\"];",
            COLOR_KERNEL_API
        ),
    );
    writeln_buf(
        buf,
        &format!(
            "    leg_async [fillcolor=\"{}\", label=\"async handler\"];",
            COLOR_ASYNC_HANDLER
        ),
    );
    writeln_buf(
        buf,
        &format!(
            "    leg_callback [fillcolor=\"{}\", label=\"callback\"];",
            COLOR_CALLBACK
        ),
    );
    writeln_buf(
        buf,
        &format!(
            "    leg_normal [fillcolor=\"{}\", label=\"function\"];",
            COLOR_NORMAL
        ),
    );
    writeln_buf(buf, "    leg_entry -> leg_kernel -> leg_async -> leg_callback -> leg_normal [style=invis];");
    writeln_buf(buf, "  }");
}

fn write_footer(buf: &mut String) {
    writeln_buf(buf, "}");
}

fn write_function_node(buf: &mut String, name: &str, func: &FunctionDef) {
    let color = if func.is_callback {
        COLOR_CALLBACK
    } else {
        COLOR_NORMAL
    };
    let label_suffix = if func.is_callback {
        func.callback_context
            .as_deref()
            .map(|ctx| format!("\\n[{}]", ctx))
            .unwrap_or_default()
    } else {
        String::new()
    };
    writeln_buf(
        buf,
        &format!(
            "  \"{}\" [fillcolor=\"{}\", label=\"{}{}\"];",
            escape_dot(name),
            color,
            escape_dot(name),
            label_suffix
        ),
    );
}

fn node_color_for_func(func: &FunctionDef, is_entry: bool) -> &'static str {
    if is_entry {
        COLOR_ENTRY
    } else if func.is_callback {
        COLOR_CALLBACK
    } else {
        COLOR_NORMAL
    }
}

fn format_mechanism_short(mechanism: &flowsight_core::AsyncMechanism) -> String {
    match mechanism {
        flowsight_core::AsyncMechanism::WorkQueue { delayed } => {
            if *delayed {
                "delayed_work".to_string()
            } else {
                "work_queue".to_string()
            }
        }
        flowsight_core::AsyncMechanism::Timer { high_resolution } => {
            if *high_resolution {
                "hrtimer".to_string()
            } else {
                "timer".to_string()
            }
        }
        flowsight_core::AsyncMechanism::Interrupt { threaded } => {
            if *threaded {
                "threaded_irq".to_string()
            } else {
                "irq".to_string()
            }
        }
        flowsight_core::AsyncMechanism::Tasklet => "tasklet".to_string(),
        flowsight_core::AsyncMechanism::Softirq => "softirq".to_string(),
        flowsight_core::AsyncMechanism::KThread => "kthread".to_string(),
        flowsight_core::AsyncMechanism::RcuCallback => "rcu".to_string(),
        flowsight_core::AsyncMechanism::Notifier => "notifier".to_string(),
        flowsight_core::AsyncMechanism::Custom(s) => s.clone(),
    }
}

fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

fn writeln_buf(buf: &mut String, line: &str) {
    let _ = writeln!(buf, "{}", line);
}
