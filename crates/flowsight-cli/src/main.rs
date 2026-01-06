//! FlowSight CLI
//!
//! Command-line interface for code analysis.

use anyhow::Result;
use clap::{Parser, Subcommand};
use flowsight_analysis::Analyzer;
use flowsight_parser::get_parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "flowsight")]
#[command(author, version, about = "Code flow analysis tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a source file
    Analyze {
        /// Source file to analyze
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show execution flow for a function
    Flow {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name
        #[arg(value_name = "FUNCTION")]
        function: String,
    },
    
    /// Show execution flow in ftrace style
    Trace {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name
        #[arg(value_name = "FUNCTION")]
        function: String,
        
        /// Output format (ftrace, markdown, json)
        #[arg(short, long, default_value = "ftrace")]
        format: String,
    },
    
    /// Show who calls a function
    Callers {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name
        #[arg(value_name = "FUNCTION")]
        function: String,
    },
    
    /// Show what a function calls
    Callees {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name
        #[arg(value_name = "FUNCTION")]
        function: String,
    },

    /// List all async handlers
    Async {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// List all callbacks
    Callbacks {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            file,
            output,
            format,
        } => {
            cmd_analyze(&file, output.as_deref(), &format)?;
        }
        Commands::Flow { file, function } => {
            cmd_flow(&file, &function)?;
        }
        Commands::Trace { file, function, format } => {
            cmd_trace(&file, &function, &format)?;
        }
        Commands::Callers { file, function } => {
            cmd_callers(&file, &function)?;
        }
        Commands::Callees { file, function } => {
            cmd_callees(&file, &function)?;
        }
        Commands::Async { file } => {
            cmd_async(&file)?;
        }
        Commands::Callbacks { file } => {
            cmd_callbacks(&file)?;
        }
    }

    Ok(())
}

fn cmd_analyze(file: &PathBuf, output: Option<&std::path::Path>, format: &str) -> Result<()> {
    println!("📂 Analyzing: {}", file.display());

    let parser = get_parser();
    let mut parse_result = parser.parse_file(file)?;

    println!(
        "   Found {} functions, {} structs",
        parse_result.functions.len(),
        parse_result.structs.len()
    );

    let source = std::fs::read_to_string(file)?;
    let mut analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source, &mut parse_result)?;

    println!(
        "   Found {} async handlers, {} entry points",
        analysis.async_bindings.len(),
        analysis.entry_points.len()
    );

    if format == "json" {
        let result = serde_json::json!({
            "file": file.to_string_lossy(),
            "functions": parse_result.functions.len(),
            "structs": parse_result.structs.len(),
            "async_bindings": analysis.async_bindings.len(),
            "entry_points": analysis.entry_points,
            "flow_trees": analysis.flow_trees,
        });

        let json = serde_json::to_string_pretty(&result)?;

        if let Some(out_path) = output {
            std::fs::write(out_path, &json)?;
            println!("   Output written to: {}", out_path.display());
        } else {
            println!("{}", json);
        }
    } else {
        println!("\n📊 Summary:");
        println!("   Functions: {}", parse_result.functions.len());
        println!("   Structs: {}", parse_result.structs.len());
        println!("   Async handlers: {}", analysis.async_bindings.len());
        println!("   Entry points: {:?}", analysis.entry_points);
    }

    Ok(())
}

fn cmd_flow(file: &PathBuf, function: &str) -> Result<()> {
    let parser = get_parser();
    let mut parse_result = parser.parse_file(file)?;

    let source = std::fs::read_to_string(file)?;
    let mut analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source, &mut parse_result)?;

    // Find the flow tree for the specified function
    for tree in &analysis.flow_trees {
        if tree.name == function {
            print_flow_tree(tree, 0);
            return Ok(());
        }
    }

    // If not found in flow trees, try to build one
    if let Some(func) = parse_result.functions.get(function) {
        println!("{}()", function);
        for callee in &func.calls {
            println!("  └── {}()", callee);
        }
    } else {
        println!("Function '{}' not found", function);
    }

    Ok(())
}

fn print_flow_tree(node: &flowsight_core::FlowNode, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!("{}{}", prefix, node.display_name);

    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == node.children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        print!("{}{}", prefix, connector);
        print_flow_tree(child, indent + 1);
    }
}

fn cmd_async(file: &PathBuf) -> Result<()> {
    let parser = get_parser();
    let mut parse_result = parser.parse_file(file)?;

    let source = std::fs::read_to_string(file)?;
    let mut analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source, &mut parse_result)?;

    println!("🔄 Async Handlers in {}:", file.display());
    println!();

    for binding in &analysis.async_bindings {
        let mechanism = format!("{:?}", binding.mechanism);
        let context = if binding.context.can_sleep() {
            "can sleep"
        } else {
            "cannot sleep"
        };

        println!(
            "  {} {}()",
            match &binding.mechanism {
                flowsight_core::AsyncMechanism::WorkQueue { .. } => "⚙️ ",
                flowsight_core::AsyncMechanism::Timer { .. } => "⏲️ ",
                flowsight_core::AsyncMechanism::Interrupt { .. } => "⚡",
                flowsight_core::AsyncMechanism::Tasklet => "🔄",
                flowsight_core::AsyncMechanism::KThread => "🧵",
                _ => "📍",
            },
            binding.handler
        );
        println!("     Type: {}", mechanism);
        println!("     Context: {}", context);
        if !binding.variable.is_empty() {
            println!("     Variable: {}", binding.variable);
        }
        println!();
    }

    Ok(())
}

fn cmd_callbacks(file: &PathBuf) -> Result<()> {
    let parser = get_parser();
    let mut parse_result = parser.parse_file(file)?;

    let source = std::fs::read_to_string(file)?;
    let mut analyzer = Analyzer::new();
    let _ = analyzer.analyze(&source, &mut parse_result)?;

    println!("🔌 Callbacks in {}:", file.display());
    println!();

    for (name, func) in &parse_result.functions {
        if func.is_callback {
            let context = func.callback_context.as_deref().unwrap_or("unknown");
            println!("  {}()", name);
            println!("     Context: {}", context);
            println!();
        }
    }

    Ok(())
}

/// Print execution flow in ftrace style
fn cmd_trace(file: &PathBuf, function: &str, format: &str) -> Result<()> {
    let parser = get_parser();
    let mut parse_result = parser.parse_file(file)?;

    let source = std::fs::read_to_string(file)?;
    let mut analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source, &mut parse_result)?;

    // Find the flow tree for the specified function
    let tree = analysis.flow_trees.iter().find(|t| t.name == function);
    
    match format {
        "ftrace" => {
            if let Some(tree) = tree {
                print_ftrace_tree(tree, 0, &parse_result.functions);
            } else {
                println!("Function '{}' not found in entry points", function);
            }
        }
        "markdown" => {
            println!("# Execution Flow: {}()", function);
            println!();
            println!("```");
            if let Some(tree) = tree {
                print_ftrace_tree(tree, 0, &parse_result.functions);
            }
            println!("```");
        }
        "json" => {
            if let Some(tree) = tree {
                let json = serde_json::to_string_pretty(tree)?;
                println!("{}", json);
            }
        }
        _ => {
            println!("Unknown format: {}", format);
        }
    }

    Ok(())
}

fn print_ftrace_tree(node: &flowsight_core::FlowNode, depth: usize, functions: &std::collections::HashMap<String, flowsight_core::FunctionDef>) {
    let indent = "  ".repeat(depth);
    let cpu = " 0)";
    
    // Get line number info
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
    
    // Get async info
    let async_tag = match &node.node_type {
        flowsight_core::FlowNodeType::AsyncCallback { mechanism } => {
            match mechanism {
                flowsight_core::AsyncMechanism::WorkQueue { .. } => " [WQ]",
                flowsight_core::AsyncMechanism::Timer { .. } => " [TM]",
                flowsight_core::AsyncMechanism::Interrupt { .. } => " [IRQ]",
                flowsight_core::AsyncMechanism::Tasklet => " [TL]",
                flowsight_core::AsyncMechanism::KThread => " [KT]",
                _ => " [A]",
            }
        }
        flowsight_core::FlowNodeType::KernelApi => " [K]",
        flowsight_core::FlowNodeType::External => " [E]",
        _ => "",
    };
    
    if node.children.is_empty() {
        println!("{}{} {} |{}{}();{}", cpu, line_info, indent, indent, node.name, async_tag);
    } else {
        println!("{}{} {} |{}{}() {{{}", cpu, line_info, indent, indent, node.name, async_tag);
        for child in &node.children {
            print_ftrace_tree(child, depth + 1, functions);
        }
        println!("{}{} {} |{}}}", cpu, line_info, indent, indent);
    }
}

/// Show who calls a function
fn cmd_callers(file: &PathBuf, function: &str) -> Result<()> {
    let parser = get_parser();
    let mut parse_result = parser.parse_file(file)?;

    let source = std::fs::read_to_string(file)?;
    let mut analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source, &mut parse_result)?;

    println!("📥 Callers of {}():", function);
    println!();

    let mut found = false;
    
    // Direct callers
    for (name, func) in &parse_result.functions {
        if func.calls.contains(&function.to_string()) {
            found = true;
            let loc = func.location.as_ref()
                .map(|l| format!("{}:{}", l.file.split('/').last().unwrap_or(&l.file), l.line))
                .unwrap_or_default();
            println!("  → {}() [Direct]", name);
            if !loc.is_empty() {
                println!("     at {}", loc);
            }
        }
    }
    
    // Async callers (via bindings)
    for binding in &analysis.async_bindings {
        if binding.handler == function {
            found = true;
            let mechanism = format!("{:?}", binding.mechanism);
            println!("  → [Async: {}]", mechanism);
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

/// Show what a function calls
fn cmd_callees(file: &PathBuf, function: &str) -> Result<()> {
    let parser = get_parser();
    let parse_result = parser.parse_file(file)?;

    println!("📤 {}() calls:", function);
    println!();

    if let Some(func) = parse_result.functions.get(function) {
        if func.calls.is_empty() {
            println!("  (No function calls found)");
        } else {
            for (i, callee) in func.calls.iter().enumerate() {
                let is_last = i == func.calls.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };
                
                // Check if callee is known
                let suffix = if parse_result.functions.contains_key(callee) {
                    ""
                } else {
                    " [External]"
                };
                
                println!("  {}{}(){}", prefix, callee, suffix);
            }
        }
    } else {
        println!("  Function '{}' not found", function);
    }

    Ok(())
}
