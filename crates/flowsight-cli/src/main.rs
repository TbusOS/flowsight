//! FlowSight CLI
//!
//! Command-line interface for code analysis.

use clap::{Parser, Subcommand};
use flowsight_parser::get_parser;
use flowsight_analysis::Analyzer;
use std::path::PathBuf;
use anyhow::Result;

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
        Commands::Analyze { file, output, format } => {
            cmd_analyze(&file, output.as_deref(), &format)?;
        }
        Commands::Flow { file, function } => {
            cmd_flow(&file, &function)?;
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

    println!("   Found {} functions, {} structs", 
        parse_result.functions.len(),
        parse_result.structs.len());

    let source = std::fs::read_to_string(file)?;
    let mut analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source, &mut parse_result)?;

    println!("   Found {} async handlers, {} entry points",
        analysis.async_bindings.len(),
        analysis.entry_points.len());

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

        println!("  {} {}()", 
            match &binding.mechanism {
                flowsight_core::AsyncMechanism::WorkQueue { .. } => "⚙️ ",
                flowsight_core::AsyncMechanism::Timer { .. } => "⏲️ ",
                flowsight_core::AsyncMechanism::Interrupt { .. } => "⚡",
                flowsight_core::AsyncMechanism::Tasklet => "🔄",
                flowsight_core::AsyncMechanism::KThread => "🧵",
                _ => "📍",
            },
            binding.handler);
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

