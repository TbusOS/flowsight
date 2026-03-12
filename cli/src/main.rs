//! FlowSight CLI
//!
//! Code execution flow analysis tool and kernel expert model training pipeline.

mod commands;
mod context;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
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
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { file, output } => {
            commands::analyze::run(&file, output.as_deref(), &cli.format)?;
        }
        Commands::Flow { file, function } => {
            commands::flow::run(&file, &function, &cli.format)?;
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
    }

    Ok(())
}
