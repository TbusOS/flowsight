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

    /// Show execution flow for a function (with CFG-aware reachability)
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

        /// Show branch conditions on each call
        #[arg(long)]
        show_conditions: bool,

        /// Show only error handling paths
        #[arg(long)]
        error_only: bool,

        /// Show only normal execution path (hide error paths)
        #[arg(long)]
        happy_path: bool,

        /// Expand external calls using cross-file index
        #[arg(long)]
        cross_file: bool,

        /// SQLite index database for cross-file expansion
        #[arg(long, value_name = "DB")]
        index: Option<PathBuf>,

        /// Max cross-file expansion depth (default: 3)
        #[arg(long, default_value = "3")]
        cross_depth: usize,
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

    /// Show who calls a function (use --index for cross-file)
    Callers {
        /// Source file (or function name when using --index)
        #[arg(value_name = "FILE_OR_FUNCTION")]
        file: PathBuf,

        /// Function name (omit when using --index with function as first arg)
        #[arg(value_name = "FUNCTION")]
        function: Option<String>,

        /// Use SQLite index for cross-file analysis
        #[arg(long)]
        index: Option<PathBuf>,

        /// Group results by kernel subsystem (only with --index)
        #[arg(long)]
        group_by_subsystem: bool,
    },

    /// Show what a function calls (use --index for cross-file)
    Callees {
        /// Source file (or function name when using --index)
        #[arg(value_name = "FILE_OR_FUNCTION")]
        file: PathBuf,

        /// Function name (omit when using --index with function as first arg)
        #[arg(value_name = "FUNCTION")]
        function: Option<String>,

        /// Use SQLite index for cross-file analysis
        #[arg(long)]
        index: Option<PathBuf>,
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

    /// Find call chain path between two functions (requires --index)
    Path {
        /// Starting function
        #[arg(long, value_name = "FUNCTION")]
        from: String,

        /// Target function
        #[arg(long, value_name = "FUNCTION")]
        to: String,

        /// SQLite index database path
        #[arg(long, value_name = "DB")]
        index: PathBuf,

        /// Maximum search depth (default: 10)
        #[arg(long, default_value = "10")]
        max_depth: usize,

        /// Find all paths (up to 20), not just shortest
        #[arg(long)]
        all: bool,
    },

    /// Show subsystem dependency graph (requires --index)
    SubsystemDeps {
        /// SQLite index database path
        #[arg(long, value_name = "DB")]
        index: PathBuf,

        /// Only show edges involving these subsystems (comma-separated)
        #[arg(long, value_name = "LIST")]
        focus: Option<String>,

        /// Minimum call count to show an edge (default: 1)
        #[arg(long, default_value = "1")]
        min_calls: usize,
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

    /// Ask a natural language question about code (uses LLM)
    Ask {
        /// Your question
        #[arg(value_name = "QUERY")]
        query: String,

        /// LLM provider (openai, claude, ollama, etc.)
        #[arg(long)]
        provider: Option<String>,

        /// Source file for analysis context
        #[arg(long, short)]
        file: Option<PathBuf>,

        /// Function name for focused context
        #[arg(long)]
        function: Option<String>,

        /// Disable streaming (wait for full response)
        #[arg(long)]
        no_stream: bool,
    },

    /// AI-powered function explanation (uses LLM + CFG analysis context)
    Explain {
        /// Source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function name
        #[arg(value_name = "FUNCTION")]
        function: String,

        /// LLM provider
        #[arg(long)]
        provider: Option<String>,

        /// Disable streaming
        #[arg(long)]
        no_stream: bool,
    },

    /// List configured LLM providers
    LlmProviders,

    /// Test LLM provider connectivity
    LlmTest {
        /// Provider name to test (default: default provider)
        #[arg(value_name = "PROVIDER")]
        provider: Option<String>,
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
            show_conditions,
            error_only,
            happy_path,
            cross_file,
            index,
            cross_depth,
        } => {
            let analysis_cfg = cfg.analysis.as_ref();
            let opts = FlowOptions {
                max_depth: depth.or_else(|| analysis_cfg.and_then(|a| a.max_depth)),
                no_kernel: no_kernel
                    || analysis_cfg.and_then(|a| a.no_kernel).unwrap_or(false),
                expand_async: expand_async
                    || analysis_cfg.and_then(|a| a.expand_async).unwrap_or(false),
                show_conditions,
                error_only,
                happy_path,
                cross_file,
                index_db: index,
                cross_file_depth: cross_depth,
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
        Commands::Path { from, to, index, max_depth, all } => {
            let opts = commands::path::PathOptions {
                max_depth,
                all_paths: all,
            };
            commands::path::run(&from, &to, &index, &format, &opts)?;
        }
        Commands::SubsystemDeps { index, focus, min_calls } => {
            let opts = commands::subsystem::SubsystemDepsOptions {
                focus: focus.map(|f| f.split(',').map(|s| s.trim().to_string()).collect()),
                min_calls,
            };
            commands::subsystem::run(&index, &format, &opts)?;
        }
        Commands::Cfg { file, function } => {
            commands::cfg::run(&file, &function, &format)?;
        }
        Commands::Errors { file, function } => {
            commands::cfg::run_errors(&file, function.as_deref(), &format)?;
        }
        Commands::Callers { file, function, index, group_by_subsystem } => {
            if let Some(db_path) = index {
                // Cross-file mode: first arg is function name
                let func_name = file.to_string_lossy();
                commands::graph::run_cross_callers(&func_name, &db_path, &format, group_by_subsystem)?;
            } else {
                let func = function.as_deref().unwrap_or_else(|| {
                    eprintln!("Usage: flowsight callers <file> <function>");
                    std::process::exit(1);
                });
                commands::graph::run_callers(&file, func, &format)?;
            }
        }
        Commands::Callees { file, function, index } => {
            if let Some(db_path) = index {
                // Cross-file mode: first arg is function name
                let func_name = file.to_string_lossy();
                commands::graph::run_cross_callees(&func_name, &db_path, &format)?;
            } else {
                let func = function.as_deref().unwrap_or_else(|| {
                    eprintln!("Usage: flowsight callees <file> <function>");
                    std::process::exit(1);
                });
                commands::graph::run_callees(&file, func, &format)?;
            }
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
        Commands::Ask {
            query,
            provider,
            file,
            function,
            no_stream,
        } => {
            let llm_cfg = cfg.llm.clone().unwrap_or_else(flowsight_llm::config::LlmConfig::with_defaults);
            let opts = commands::ask::AskOptions {
                provider,
                model: None,
                file,
                function,
                no_stream,
            };
            commands::ask::run(&query, &llm_cfg, &opts)?;
        }
        Commands::Explain {
            file,
            function,
            provider,
            no_stream,
        } => {
            let llm_cfg = cfg.llm.clone().unwrap_or_else(flowsight_llm::config::LlmConfig::with_defaults);
            let query = format!(
                "Explain the execution flow of {}(). Describe:\n\
                 1. Purpose and what it does\n\
                 2. Normal execution path\n\
                 3. Error handling paths and cleanup\n\
                 4. Async mechanisms and callbacks\n\
                 5. Locking and execution context",
                function
            );
            let opts = commands::ask::AskOptions {
                provider,
                model: None,
                file: Some(file),
                function: Some(function),
                no_stream,
            };
            commands::ask::run(&query, &llm_cfg, &opts)?;
        }
        Commands::LlmProviders => {
            let llm_cfg = cfg.llm.clone().unwrap_or_else(flowsight_llm::config::LlmConfig::with_defaults);
            commands::ask::run_list_providers(&llm_cfg)?;
        }
        Commands::LlmTest { provider } => {
            let llm_cfg = cfg.llm.clone().unwrap_or_else(flowsight_llm::config::LlmConfig::with_defaults);
            commands::ask::run_test_provider(&llm_cfg, provider.as_deref())?;
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
