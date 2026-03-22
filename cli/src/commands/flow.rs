//! `flowsight flow` and `flowsight trace` commands
//!
//! Enhanced with CFG-aware reachability annotations:
//! - [always] / [conditional] / [error-path] tags on each call
//! - --show-conditions: show branch conditions
//! - --error-only: show only error handling paths
//! - --happy-path: show only normal execution path

use crate::context::AnalysisContext;
use crate::index_db::IndexDb;
use crate::output::{dot, json, text, OutputFormat};
use anyhow::Result;
use crossterm::style::{Color, Stylize};
use flowsight_cfg::{CfgBuilder, CallKind, ErrorPathDetector, Reachability};
use flowsight_core::{FlowNode, FlowNodeType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Low-saturation color palette
const C_OK: Color = Color::Rgb { r: 130, g: 175, b: 140 };
const C_ERR: Color = Color::Rgb { r: 195, g: 120, b: 120 };
const C_COND: Color = Color::Rgb { r: 170, g: 160, b: 120 };
const C_DIM: Color = Color::Rgb { r: 110, g: 115, b: 120 };
const C_CTX: Color = Color::Rgb { r: 140, g: 150, b: 180 };

/// Flow display options
pub struct FlowOptions {
    /// Maximum depth to display (None = unlimited)
    pub max_depth: Option<usize>,
    /// Hide kernel API calls
    pub no_kernel: bool,
    /// Expand async boundaries (TODO: not yet implemented)
    #[allow(dead_code)]
    pub expand_async: bool,
    /// Show branch conditions on each call
    pub show_conditions: bool,
    /// Show only error handling paths
    pub error_only: bool,
    /// Show only normal execution path (hide error paths)
    pub happy_path: bool,
    /// Cross-file expansion: resolve external calls via index
    pub cross_file: bool,
    /// SQLite index database path (required for cross-file)
    pub index_db: Option<PathBuf>,
    /// Maximum cross-file expansion depth
    pub cross_file_depth: usize,
}

impl Default for FlowOptions {
    fn default() -> Self {
        Self {
            max_depth: None,
            no_kernel: false,
            expand_async: false,
            show_conditions: false,
            error_only: false,
            happy_path: false,
            cross_file: false,
            index_db: None,
            cross_file_depth: 3,
        }
    }
}

/// Filter a flow tree by depth and node type, returning a new tree
fn filter_flow_tree(node: &FlowNode, opts: &FlowOptions, current_depth: usize) -> FlowNode {
    let children = if opts.max_depth.is_some_and(|d| current_depth >= d) {
        vec![]
    } else {
        node.children
            .iter()
            .filter(|child| {
                if opts.no_kernel && matches!(child.node_type, FlowNodeType::KernelApi) {
                    return false;
                }
                // Filter out helper macros that are noise in flow output
                if is_flow_noise(&child.name) {
                    return false;
                }
                true
            })
            .map(|child| filter_flow_tree(child, opts, current_depth + 1))
            .collect()
    };

    FlowNode {
        id: node.id.clone(),
        name: node.name.clone(),
        display_name: node.display_name.clone(),
        location: node.location.clone(),
        node_type: node.node_type.clone(),
        children,
        description: node.description.clone(),
        confidence: node.confidence.clone(),
        execution_context: node.execution_context.clone(),
        can_sleep: node.can_sleep,
        source_file: node.source_file.clone(),
        is_kernel_internal: node.is_kernel_internal,
    }
}

/// Helper macros / type casts that are noise in flow output
fn is_flow_noise(name: &str) -> bool {
    matches!(
        name,
        "IS_ERR" | "IS_ERR_OR_NULL" | "PTR_ERR" | "ERR_PTR" | "ERR_CAST"
            | "likely" | "unlikely" | "__builtin_expect"
            | "BUG" | "BUG_ON" | "WARN" | "WARN_ON" | "WARN_ONCE"
            | "BUILD_BUG_ON" | "BUILD_BUG_ON_ZERO"
            | "container_of" | "offsetof" | "sizeof"
            | "min" | "max" | "clamp"
            | "ARRAY_SIZE" | "FIELD_SIZEOF"
            | "ACCESS_ONCE" | "READ_ONCE" | "WRITE_ONCE"
            | "smp_store_release" | "smp_load_acquire"
            | "to_usb_device" | "to_usb_interface"
            | "to_platform_device" | "to_pci_dev"
            | "to_i2c_client" | "to_spi_device"
    )
}

/// Build a reachability map for function calls using CFG
fn build_reachability_map(source: &str, function: &str) -> HashMap<String, Vec<(Reachability, CallKind)>> {
    let builder = CfgBuilder::new();
    let mut map: HashMap<String, Vec<(Reachability, CallKind)>> = HashMap::new();

    if let Ok(mut cfg) = builder.build_function_cfg(source, function) {
        ErrorPathDetector::analyze(&mut cfg);
        for call in cfg.all_calls() {
            map.entry(call.callee.clone())
                .or_default()
                .push((call.reachability.clone(), call.call_kind.clone()));
        }
    }

    map
}

/// Print enhanced flow tree with CFG reachability annotations
fn print_enhanced_flow_tree(
    node: &FlowNode,
    indent: usize,
    reachability_map: &HashMap<String, Vec<(Reachability, CallKind)>>,
    opts: &FlowOptions,
    is_last: bool,
) {
    let prefix = if indent == 0 {
        String::new()
    } else {
        let connector = if is_last { "└── " } else { "├── " };
        format!("{}{}", "  ".repeat(indent - 1), connector)
    };

    // Look up reachability for this call
    let reach_info = reachability_map.get(&node.name);

    // Determine the primary reachability
    let (reach_tag, reach_color) = if let Some(entries) = reach_info {
        // Use the first entry's reachability
        match &entries[0].0 {
            Reachability::Always => ("always", C_OK),
            Reachability::Conditional(c) => {
                let tag = if opts.show_conditions {
                    format!("if {}", if c.len() > 25 { &c[..25] } else { c })
                } else {
                    "conditional".to_string()
                };
                // We need to handle this specially since tag is owned
                print!("{}", prefix);
                print_reach_node(&node.name, &node.display_name, &tag, C_COND, reach_info);
                print_children_enhanced(node, indent, reachability_map, opts);
                return;
            }
            Reachability::ErrorPath => ("error-path", C_ERR),
            Reachability::ConditionalCompilation => ("#ifdef", C_CTX),
        }
    } else {
        // Not in CFG map — could be entry point or kernel chain node
        if node.is_kernel_internal {
            ("kernel", C_DIM)
        } else if indent == 0 {
            ("entry", C_OK)
        } else {
            ("", C_DIM)
        }
    };

    // Apply error-only / happy-path filters
    if opts.error_only {
        if reach_tag != "error-path" && indent > 0 {
            // Still recurse to find error-path children
            let has_error_children = has_reachability_in_subtree(node, reachability_map, &Reachability::ErrorPath);
            if !has_error_children {
                return;
            }
        }
    }
    if opts.happy_path && reach_tag == "error-path" {
        return;
    }

    // Print the node
    print!("{}", prefix);
    print_reach_node(&node.name, &node.display_name, reach_tag, reach_color, reach_info);

    print_children_enhanced(node, indent, reachability_map, opts);
}

/// Print children of an enhanced flow node
fn print_children_enhanced(
    node: &FlowNode,
    indent: usize,
    reachability_map: &HashMap<String, Vec<(Reachability, CallKind)>>,
    opts: &FlowOptions,
) {
    if opts.max_depth.is_some_and(|d| indent >= d) {
        return;
    }

    let visible_children: Vec<_> = node.children.iter()
        .filter(|child| {
            if opts.no_kernel && matches!(child.node_type, FlowNodeType::KernelApi) {
                return false;
            }
            true
        })
        .collect();

    for (i, child) in visible_children.iter().enumerate() {
        let is_last = i == visible_children.len() - 1;
        print_enhanced_flow_tree(child, indent + 1, reachability_map, opts, is_last);
    }
}

/// Print a single node line with reachability tag
fn print_reach_node(
    _name: &str,
    display_name: &str,
    reach_tag: &str,
    reach_color: Color,
    reach_info: Option<&Vec<(Reachability, CallKind)>>,
) {
    // Determine call kind tag
    let kind_tag = if let Some(entries) = reach_info {
        match &entries[0].1 {
            CallKind::AsyncRegistration { mechanism, .. } => format!(" [async:{}]", mechanism),
            CallKind::ContextChange { new_context } => format!(" [ctx:{}]", new_context),
            CallKind::DeclarationMacro => " [decl]".to_string(),
            CallKind::IteratorMacro => " [iter]".to_string(),
            _ => String::new(),
        }
    } else {
        String::new()
    };

    if reach_tag.is_empty() {
        println!("{}{}", display_name, kind_tag.with(C_DIM));
    } else {
        let tag_display = format!("[{}]", reach_tag);
        println!(
            "{} {}{}",
            tag_display.with(reach_color),
            display_name,
            kind_tag.with(C_DIM),
        );
    }
}

/// Check if any call in a subtree has a specific reachability
fn has_reachability_in_subtree(
    node: &FlowNode,
    map: &HashMap<String, Vec<(Reachability, CallKind)>>,
    target: &Reachability,
) -> bool {
    if let Some(entries) = map.get(&node.name) {
        if entries.iter().any(|(r, _)| r == target) {
            return true;
        }
    }
    node.children.iter().any(|c| has_reachability_in_subtree(c, map, target))
}

/// Expand external function calls by looking them up in the index
fn expand_cross_file(
    node: &FlowNode,
    db: &IndexDb,
    ctx: &mut AnalysisContext,
    expanded: &mut HashMap<String, Option<FlowNode>>,
    depth: usize,
    max_depth: usize,
) -> FlowNode {
    if depth >= max_depth {
        return node.clone();
    }

    let children: Vec<FlowNode> = node.children.iter().map(|child| {
        // If this is an External node, try to resolve it
        if matches!(child.node_type, FlowNodeType::External | FlowNodeType::KernelApi)
            && child.children.is_empty()
        {
            // Check cache
            if let Some(cached) = expanded.get(&child.name) {
                return cached.clone().unwrap_or_else(|| child.clone());
            }

            // Try to find in index
            if let Ok(symbols) = db.query_symbol(&child.name, Some("function"), None) {
                if let Some(sym) = symbols.first() {
                    // Parse the file containing this function
                    let file_path = Path::new(&sym.file_path);
                    if file_path.exists() {
                        if let Ok(result) = ctx.analyze_file(file_path) {
                            // Find the flow tree for this function
                            for tree in &result.analysis.flow_trees {
                                if tree.name == child.name {
                                    let mut expanded_tree = tree.clone();
                                    // Mark as cross-file
                                    expanded_tree.source_file = Some(sym.file_path.clone());
                                    expanded_tree.display_name = format!(
                                        "{}() [{}]",
                                        child.name,
                                        Path::new(&sym.file_path)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("?")
                                    );

                                    // Recursively expand this tree too
                                    let result = expand_cross_file(
                                        &expanded_tree, db, ctx, expanded, depth + 1, max_depth,
                                    );
                                    expanded.insert(child.name.clone(), Some(result.clone()));
                                    return result;
                                }
                            }
                        }
                    }
                }
            }

            // Not found — cache as None
            expanded.insert(child.name.clone(), None);
            child.clone()
        } else {
            // Recurse into existing children
            expand_cross_file(child, db, ctx, expanded, depth, max_depth)
        }
    }).collect();

    FlowNode {
        children,
        ..node.clone()
    }
}

/// Show execution flow tree for a function
pub fn run(
    file: &Path,
    function: &str,
    format: &OutputFormat,
    opts: &FlowOptions,
) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;

    // Build CFG reachability map for enhanced display
    let source = std::fs::read_to_string(file).unwrap_or_default();
    let reachability_map = build_reachability_map(&source, function);
    let use_enhanced = !reachability_map.is_empty()
        || opts.show_conditions
        || opts.error_only
        || opts.happy_path;

    // Find the flow tree for the specified function
    for tree in &result.analysis.flow_trees {
        if tree.name == function {
            let mut filtered = filter_flow_tree(tree, opts, 0);

            // Cross-file expansion
            if opts.cross_file {
                if let Some(db_path) = &opts.index_db {
                    if let Ok(db) = IndexDb::open_readonly(db_path) {
                        let mut expanded_cache = HashMap::new();
                        filtered = expand_cross_file(
                            &filtered,
                            &db,
                            &mut ctx,
                            &mut expanded_cache,
                            0,
                            opts.cross_file_depth,
                        );
                    }
                }
            }

            match format {
                OutputFormat::Json => {
                    let json_str = json::to_pretty_json(&filtered)?;
                    println!("{}", json_str);
                }
                OutputFormat::Ftrace => {
                    text::print_ftrace_tree(&filtered, 0, &result.parse_result.functions);
                }
                OutputFormat::Markdown => {
                    println!("# Execution Flow: {}()", function);
                    println!();
                    println!("```");
                    text::print_ftrace_tree(&filtered, 0, &result.parse_result.functions);
                    println!("```");
                }
                OutputFormat::Dot => {
                    let dot_output = dot::flow_to_dot(&filtered, function);
                    print!("{}", dot_output);
                }
                OutputFormat::Sequence => {
                    anyhow::bail!(
                        "Sequence format is for kb chain/async-chain commands. Use --format text/json/ftrace/markdown/dot for flow."
                    );
                }
                OutputFormat::Text => {
                    if use_enhanced {
                        print_enhanced_flow_tree(&filtered, 0, &reachability_map, opts, true);
                    } else {
                        text::print_flow_tree(&filtered, 0);
                    }
                }
            }
            return Ok(());
        }
    }

    // If not found in flow trees, try to build a simple one from parse result
    if let Some(func) = result.parse_result.functions.get(function) {
        println!("{}()", function);
        if func.calls.is_empty() {
            println!("  (no calls)");
        } else {
            for (i, callee) in func.calls.iter().enumerate() {
                let is_last = i == func.calls.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };
                println!("  {}{}()", prefix, callee);
            }
        }
    } else {
        eprintln!("Function '{}' not found in {}", function, file.display());
        eprintln!();
        // List available functions
        let mut names: Vec<_> = result.parse_result.functions.keys().collect();
        names.sort();
        if !names.is_empty() {
            eprintln!("Available functions:");
            for name in names {
                eprintln!("  {}", name);
            }
        }
    }

    Ok(())
}

/// Show execution flow in ftrace style (legacy trace command)
pub fn run_trace(file: &Path, function: &str, format: &str) -> Result<()> {
    let output_format = match format {
        "ftrace" => OutputFormat::Ftrace,
        "json" => OutputFormat::Json,
        "markdown" => OutputFormat::Markdown,
        other => anyhow::bail!(
            "Unknown trace format: '{}'. Valid options: ftrace, json, markdown",
            other
        ),
    };
    run(file, function, &output_format, &FlowOptions::default())
}
