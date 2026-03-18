//! FlowSight CLI
//!
//! Code execution flow analysis tool and kernel expert model training pipeline.

mod commands;
mod context;
mod index_db;
mod output;
mod repl;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
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
    /// Analyze source file(s) or directory
    Analyze {
        /// Source file or directory to analyze
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Recursively analyze all matching files in directory
        #[arg(short, long)]
        recursive: bool,

        /// File pattern for directory scan (default: "*.c")
        #[arg(short, long, default_value = "*.c")]
        pattern: String,

        /// Number of parallel workers (default: number of CPUs)
        #[arg(short = 'j', long)]
        parallel: Option<usize>,

        /// Show only summary statistics
        #[arg(long)]
        summary: bool,
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

    /// Cross-file index: build, query, stats, update
    #[command(subcommand)]
    Index(IndexCommands),

    /// Detect Linux kernel coding patterns (locking, error handling, lifecycle, etc.)
    Patterns {
        /// Source file or directory to scan
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Recursively scan directory
        #[arg(short, long)]
        recursive: bool,

        /// Filter by pattern category
        #[arg(short, long, value_name = "CATEGORY")]
        category: Option<commands::patterns::PatternCategory>,

        /// File pattern for directory scan (default: "*.c")
        #[arg(short = 'g', long = "glob", default_value = "*.c")]
        pattern_glob: String,

        /// Show only summary counts
        #[arg(long)]
        summary: bool,
    },

    /// Generate training data for kernel expert LLM fine-tuning
    #[command(subcommand)]
    Train(commands::train::TrainCommands),

    /// Generate shell completions
    #[command(hide = true)]
    Completions {
        /// Shell type (bash, zsh, fish, powershell, elvish)
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Interactive REPL mode
    #[command(alias = "i")]
    Interactive,
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

#[derive(Subcommand)]
enum IndexCommands {
    /// Build cross-file index from a directory
    Build {
        /// Directory to index
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        /// Recursive scan (default: true)
        #[arg(short, long, default_value_t = true)]
        recursive: bool,

        /// File filter pattern
        #[arg(short, long, default_value = "*.c")]
        pattern: String,

        /// SQLite database path (default: <dir>/.flowsight/index.db)
        #[arg(long)]
        db: Option<PathBuf>,

        /// Number of parallel workers
        #[arg(short = 'j', long)]
        parallel: Option<usize>,

        /// Auto-detect kernel subsystem boundaries
        #[arg(long)]
        subsystem: bool,
    },

    /// Look up a symbol across all indexed files
    Query {
        /// Symbol name to search for
        #[arg(value_name = "SYMBOL")]
        symbol: String,

        /// Filter by symbol type (function, struct, macro, callback)
        #[arg(short = 't', long = "type")]
        kind: Option<String>,

        /// Filter by kernel subsystem
        #[arg(short, long)]
        subsystem: Option<String>,

        /// SQLite database path
        #[arg(long)]
        db: Option<PathBuf>,
    },

    /// Show index statistics
    Stats {
        /// SQLite database path
        #[arg(long)]
        db: Option<PathBuf>,
    },

    /// Incremental update (only changed files)
    Update {
        /// Directory to update
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        /// File filter pattern
        #[arg(short, long, default_value = "*.c")]
        pattern: String,

        /// SQLite database path
        #[arg(long)]
        db: Option<PathBuf>,

        /// Number of parallel workers
        #[arg(short = 'j', long)]
        parallel: Option<usize>,

        /// Auto-detect kernel subsystem boundaries
        #[arg(long)]
        subsystem: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // No arguments -> launch REPL
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        return repl::run();
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            path,
            output,
            recursive,
            pattern,
            parallel,
            summary,
        } => {
            let opts = commands::analyze::AnalyzeOptions {
                recursive,
                pattern,
                parallel,
                summary,
            };
            commands::analyze::run(&path, output.as_deref(), &cli.format, &opts)?;
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
        Commands::Index(idx_cmd) => match idx_cmd {
            IndexCommands::Build {
                dir,
                recursive: _,
                pattern,
                db,
                parallel,
                subsystem,
            } => {
                let opts = commands::index::BuildOptions {
                    pattern,
                    db_path: db,
                    parallel,
                    subsystem,
                };
                commands::index::run_build(&dir, &cli.format, &opts)?;
            }
            IndexCommands::Query {
                symbol,
                kind,
                subsystem,
                db,
            } => {
                let opts = commands::index::QueryOptions {
                    kind_filter: kind,
                    subsystem_filter: subsystem,
                    db_path: db,
                };
                commands::index::run_query(&symbol, None, &cli.format, &opts)?;
            }
            IndexCommands::Stats { db } => {
                commands::index::run_stats(None, db.as_deref(), &cli.format)?;
            }
            IndexCommands::Update {
                dir,
                pattern,
                db,
                parallel,
                subsystem,
            } => {
                let opts = commands::index::UpdateOptions {
                    pattern,
                    db_path: db,
                    parallel,
                    subsystem,
                };
                commands::index::run_update(&dir, &cli.format, &opts)?;
            }
        },
        Commands::Patterns {
            path,
            recursive,
            category,
            pattern_glob,
            summary,
        } => {
            let opts = commands::patterns::PatternsOptions {
                recursive,
                pattern_glob,
                category,
                summary,
            };
            commands::patterns::run(&path, &cli.format, &opts)?;
        }
        Commands::Train(train_cmd) => {
            commands::train::run(&train_cmd)?;
        }
        Commands::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "flowsight",
                &mut std::io::stdout(),
            );
        }
        Commands::Interactive => {
            repl::run()?;
        }
    }

    Ok(())
}
