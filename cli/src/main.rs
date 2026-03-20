//! FlowSight CLI
//!
//! Code execution flow analysis tool and kernel expert model training pipeline.

mod commands;
mod config;
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

    /// Show control flow graph for a function (CFG with error paths + macro semantics)
    Cfg {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name
        #[arg(value_name = "FUNCTION")]
        function: String,
    },

    /// List error handling paths in a file or function
    Errors {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name (optional — all functions if omitted)
        #[arg(value_name = "FUNCTION")]
        function: Option<String>,
    },

    /// Full file call graph (use -F dot for Graphviz DOT output)
    Graph {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Hide external/kernel API calls from the graph
        #[arg(long)]
        exclude_kernel: bool,

        /// Group nodes by call depth in cluster subgraphs
        #[arg(long)]
        cluster: bool,
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

    /// Compare execution flows between two file versions
    Diff {
        /// First source file (base version)
        #[arg(value_name = "FILE_A")]
        file_a: PathBuf,

        /// Second source file (new version)
        #[arg(value_name = "FILE_B")]
        file_b: PathBuf,

        /// Compare only this function (default: all functions)
        #[arg(value_name = "FUNCTION")]
        function: Option<String>,

        /// Maximum depth for flow comparison
        #[arg(short, long)]
        depth: Option<usize>,

        /// Ignore call ordering changes
        #[arg(long)]
        ignore_order: bool,

        /// Also show unchanged paths in output
        #[arg(long)]
        show_common: bool,

        /// Only show which functions changed (skip call-level details)
        #[arg(long)]
        summary: bool,
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

    /// Search for symbols across source files
    Search {
        /// Symbol name or pattern to search for
        #[arg(value_name = "QUERY")]
        query: String,

        /// Directory or file to search in (default: current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Filter by symbol type
        #[arg(short = 't', long = "type", value_enum)]
        kind: Option<commands::search::SymbolKind>,

        /// Treat query as regex pattern
        #[arg(short = 'e', long = "regex")]
        regex: bool,

        /// Show N lines of source context around each match
        #[arg(short = 'c', long = "context", default_value = "0")]
        context_lines: usize,

        /// Recursively scan directory
        #[arg(short, long)]
        recursive: bool,

        /// File filter pattern (default: "*.c")
        #[arg(long, default_value = "*.c")]
        pattern: String,

        /// Maximum number of results (default: 50)
        #[arg(short, long, default_value = "50")]
        limit: usize,

        /// Use pre-built SQLite index for faster search
        #[arg(long)]
        use_index: bool,

        /// SQLite database path (for --use-index)
        #[arg(long)]
        db: Option<PathBuf>,
    },

    /// Generate self-contained HTML analysis report
    Report {
        /// Source file or directory to analyze
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output file (default: flowsight-report.html)
        #[arg(short, long, default_value = "flowsight-report.html")]
        output: PathBuf,

        /// Recursively analyze all matching files in directory
        #[arg(short, long)]
        recursive: bool,

        /// File pattern for directory scan (default: "*.c")
        #[arg(short, long, default_value = "*.c")]
        pattern: String,

        /// Report title
        #[arg(long)]
        title: Option<String>,

        /// Include source code snippets in per-file details
        #[arg(long)]
        include_source: bool,
    },

    /// Kernel execution flow scenarios
    #[command(subcommand)]
    Scenario(ScenarioCommands),

    /// Generate training data for kernel expert LLM fine-tuning
    #[command(subcommand)]
    Train(commands::train::TrainCommands),

    /// Project configuration management
    #[command(subcommand)]
    Config(ConfigCommands),

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
enum ConfigCommands {
    /// Show active configuration (merged from file + defaults)
    Show,

    /// Create a template .flowsight.toml in the current directory
    Init,

    /// Print the path of the active config file
    Path,
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

#[derive(Subcommand)]
enum ScenarioCommands {
    /// List all available kernel scenarios
    List,

    /// Show scenario details
    Show {
        /// Scenario name or partial match
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Run a scenario, optionally binding to a real source file
    Run {
        /// Scenario name or partial match
        #[arg(value_name = "NAME")]
        name: String,

        /// Bind a driver source file (e.g., --bind driver=path/to/file.c)
        #[arg(long = "bind", value_name = "KEY=VALUE")]
        bindings: Vec<String>,

        /// Maximum call expansion depth
        #[arg(short, long)]
        depth: Option<usize>,
    },

    /// Create a custom scenario template file
    Create {
        /// Template name
        #[arg(value_name = "NAME")]
        name: String,
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

    // Load project config (optional, never fails the CLI)
    let cfg = config::load_from_cwd().unwrap_or_default();

    // Merge: CLI flags override config file values.
    // Clap sets `format` to default "text" even when user didn't pass -F,
    // so we only override from config when the user did not explicitly set the flag.
    let format = if std::env::args().any(|a| a == "-F" || a.starts_with("--format")) {
        cli.format
    } else {
        cfg.global
            .as_ref()
            .and_then(|g| g.format.as_deref())
            .and_then(|f| f.parse::<OutputFormat>().ok())
            .unwrap_or(cli.format)
    };

    let verbose = if cli.verbose {
        true
    } else {
        cfg.global
            .as_ref()
            .and_then(|g| g.verbose)
            .unwrap_or(false)
    };

    let _ = verbose; // available for future use

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
            commands::analyze::run(&path, output.as_deref(), &format, &opts)?;
        }
        Commands::Flow {
            file,
            function,
            depth,
            no_kernel,
            expand_async,
        } => {
            let analysis_cfg = cfg.analysis.as_ref();
            let opts = FlowOptions {
                max_depth: depth.or_else(|| analysis_cfg.and_then(|a| a.max_depth)),
                no_kernel: no_kernel
                    || analysis_cfg.and_then(|a| a.no_kernel).unwrap_or(false),
                expand_async: expand_async
                    || analysis_cfg.and_then(|a| a.expand_async).unwrap_or(false),
            };
            commands::flow::run(&file, &function, &format, &opts)?;
        }
        Commands::Trace {
            file,
            function,
            trace_format,
        } => {
            commands::flow::run_trace(&file, &function, &trace_format)?;
        }
        Commands::Cfg { file, function } => {
            commands::cfg::run(&file, &function, &format)?;
        }
        Commands::Errors { file, function } => {
            commands::cfg::run_errors(&file, function.as_deref(), &format)?;
        }
        Commands::Callers { file, function } => {
            commands::graph::run_callers(&file, &function, &format)?;
        }
        Commands::Callees { file, function } => {
            commands::graph::run_callees(&file, &function, &format)?;
        }
        Commands::Graph {
            file,
            exclude_kernel,
            cluster,
        } => {
            let opts = commands::graph::GraphFullOptions {
                exclude_kernel,
                cluster,
            };
            commands::graph::run_full(&file, &format, &opts)?;
        }
        Commands::Async { file } => {
            commands::async_cmd::run_async(&file)?;
        }
        Commands::Callbacks { file } => {
            commands::async_cmd::run_callbacks(&file)?;
        }
        Commands::Diff {
            file_a,
            file_b,
            function,
            depth,
            ignore_order,
            show_common,
            summary,
        } => {
            let opts = commands::diff::DiffOptions {
                max_depth: depth,
                ignore_order,
                show_common,
                summary_only: summary,
            };
            commands::diff::run(
                &file_a,
                &file_b,
                function.as_deref(),
                &format,
                &opts,
            )?;
        }
        Commands::Kb(kb_cmd) => match kb_cmd {
            KbCommands::Stats => {
                commands::kb::run_stats(&format)?;
            }
            KbCommands::Query { term } => {
                commands::kb::run_query(&term, &format)?;
            }
            KbCommands::Chain {
                framework,
                callback,
            } => {
                commands::kb::run_chain(&framework, &callback, &format)?;
            }
            KbCommands::AsyncChain { pattern } => {
                commands::kb::run_async_chain(&pattern, &format)?;
            }
            KbCommands::Match { file } => {
                commands::kb::run_match(&file, &format)?;
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
                commands::index::run_build(&dir, &format, &opts)?;
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
                commands::index::run_query(&symbol, None, &format, &opts)?;
            }
            IndexCommands::Stats { db } => {
                commands::index::run_stats(None, db.as_deref(), &format)?;
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
                commands::index::run_update(&dir, &format, &opts)?;
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
            commands::patterns::run(&path, &format, &opts)?;
        }
        Commands::Search {
            query,
            path,
            kind,
            regex,
            context_lines,
            recursive,
            pattern,
            limit,
            use_index,
            db,
        } => {
            let opts = commands::search::SearchOptions {
                kind_filter: kind,
                regex,
                context_lines,
                recursive,
                file_pattern: pattern,
                limit,
                use_index,
                db_path: db,
            };
            commands::search::run(&query, path.as_deref(), &format, &opts)?;
        }
        Commands::Report {
            path,
            output,
            recursive,
            pattern,
            title,
            include_source,
        } => {
            let opts = commands::report::ReportOptions {
                output,
                recursive,
                pattern,
                title,
                include_source,
            };
            commands::report::run(&path, &opts)?;
        }
        Commands::Scenario(scenario_cmd) => match scenario_cmd {
            ScenarioCommands::List => {
                commands::scenario::run_list(&format)?;
            }
            ScenarioCommands::Show { name } => {
                commands::scenario::run_show(&name, &format)?;
            }
            ScenarioCommands::Run {
                name,
                bindings,
                depth,
            } => {
                let opts = parse_scenario_bindings(&bindings, depth);
                commands::scenario::run_scenario(&name, &format, &opts)?;
            }
            ScenarioCommands::Create { name } => {
                commands::scenario::run_create(&name)?;
            }
        },
        Commands::Train(train_cmd) => {
            commands::train::run(&train_cmd)?;
        }
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Show => {
                commands::config::run_show(&cfg)?;
            }
            ConfigCommands::Init => {
                commands::config::run_init()?;
            }
            ConfigCommands::Path => {
                commands::config::run_path(&cfg)?;
            }
        },
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

/// Parse --bind KEY=VALUE pairs into ScenarioRunOptions
fn parse_scenario_bindings(
    bindings: &[String],
    depth: Option<usize>,
) -> commands::scenario::ScenarioRunOptions {
    let mut bind_driver = None;
    let mut bind_function = None;

    for binding in bindings {
        if let Some((key, value)) = binding.split_once('=') {
            match key.trim() {
                "driver" => bind_driver = Some(value.trim().to_string()),
                "function" => bind_function = Some(value.trim().to_string()),
                _ => {
                    eprintln!("Unknown bind key: '{}' (valid: driver, function)", key);
                }
            }
        }
    }

    commands::scenario::ScenarioRunOptions {
        bind_driver,
        bind_function,
        max_depth: depth,
    }
}
