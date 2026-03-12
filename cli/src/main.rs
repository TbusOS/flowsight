//! FlowSight CLI
//!
//! Code execution flow analysis tool and kernel expert model training pipeline.

mod commands;
mod context;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::flow::FlowOptions;
use output::OutputFormat;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "flowsight")]
#[command(version, about = "Code execution flow analysis tool")]
#[command(long_about = "Analyze code execution flows, resolve function pointers, \
    trace async mechanisms, and generate training data for kernel expert models.")]
struct Cli {
    /// Output format
    #[arg(short = 'F', long, global = true, default_value = "text")]
    format: OutputFormat,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

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
    },

    /// Show execution flow for a function
    Flow {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name
        #[arg(value_name = "FUNCTION")]
        function: String,

        /// Maximum depth to display
        #[arg(short, long)]
        depth: Option<usize>,

        /// Hide kernel API calls from output
        #[arg(long)]
        no_kernel: bool,

        /// Expand async boundaries (show deferred execution)
        #[arg(long)]
        expand_async: bool,
    },

    /// Show execution flow in ftrace style
    Trace {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name
        #[arg(value_name = "FUNCTION")]
        function: String,

        /// Trace output format (ftrace, markdown, json)
        #[arg(long, default_value = "ftrace")]
        trace_format: String,
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

    /// List all async handlers in a file
    Async {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// List all callback functions in a file
    Callbacks {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Knowledge base query and inspection
    #[command(subcommand)]
    Kb(KbCommands),
}

#[derive(Subcommand)]
enum KbCommands {
    /// Show knowledge base statistics
    Stats,

    /// Search knowledge base for a term
    Query {
        /// Search term
        #[arg(value_name = "TERM")]
        term: String,
    },

    /// Show kernel call chain for a framework callback
    Chain {
        /// Framework name (e.g., usb_driver)
        #[arg(value_name = "FRAMEWORK")]
        framework: String,

        /// Callback name (e.g., probe)
        #[arg(value_name = "CALLBACK")]
        callback: String,
    },

    /// Show async handler call chain
    AsyncChain {
        /// Async pattern name (e.g., work_struct)
        #[arg(value_name = "PATTERN")]
        pattern: String,
    },

    /// Match a source file against knowledge base patterns
    Match {
        /// Source file to match
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { file, output } => {
            commands::analyze::run(&file, output.as_deref(), &cli.format)?;
        }
        Commands::Flow {
            file,
            function,
            depth,
            no_kernel,
            expand_async,
        } => {
            let opts = FlowOptions {
                max_depth: depth,
                no_kernel,
                expand_async,
            };
            commands::flow::run(&file, &function, &cli.format, &opts)?;
        }
        Commands::Trace {
            file,
            function,
            trace_format,
        } => {
            commands::flow::run_trace(&file, &function, &trace_format)?;
        }
        Commands::Callers { file, function } => {
            commands::graph::run_callers(&file, &function)?;
        }
        Commands::Callees { file, function } => {
            commands::graph::run_callees(&file, &function)?;
        }
        Commands::Async { file } => {
            commands::async_cmd::run_async(&file)?;
        }
        Commands::Callbacks { file } => {
            commands::async_cmd::run_callbacks(&file)?;
        }
        Commands::Kb(kb_cmd) => match kb_cmd {
            KbCommands::Stats => {
                commands::kb::run_stats(&cli.format)?;
            }
            KbCommands::Query { term } => {
                commands::kb::run_query(&term, &cli.format)?;
            }
            KbCommands::Chain {
                framework,
                callback,
            } => {
                commands::kb::run_chain(&framework, &callback, &cli.format)?;
            }
            KbCommands::AsyncChain { pattern } => {
                commands::kb::run_async_chain(&pattern, &cli.format)?;
            }
            KbCommands::Match { file } => {
                commands::kb::run_match(&file, &cli.format)?;
            }
        },
    }

    Ok(())
}
