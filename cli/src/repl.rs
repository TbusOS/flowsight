//! Interactive REPL mode
//!
//! Provides an interactive shell for FlowSight with:
//! - Persistent context (loaded file, knowledge base)
//! - Command history
//! - Tab completion
//! - Colored output

use crate::commands;
use crate::context::AnalysisContext;
use crate::output::OutputFormat;
use crossterm::style::{Color, Stylize};

// Low-saturation color palette
const C_LOGO: Color = Color::Rgb { r: 130, g: 160, b: 190 };   // steel blue
const C_TITLE: Color = Color::Rgb { r: 140, g: 185, b: 165 };  // sage
const C_OK: Color = Color::Rgb { r: 130, g: 175, b: 140 };     // muted green
const C_FILE: Color = Color::Rgb { r: 155, g: 160, b: 185 };   // lavender grey
const C_PROMPT: Color = Color::Rgb { r: 150, g: 175, b: 155 }; // soft green
const C_ERR: Color = Color::Rgb { r: 195, g: 120, b: 120 };    // dusty red
const C_HEAD: Color = Color::Rgb { r: 190, g: 170, b: 130 };   // sand
const C_DIM: Color = Color::Rgb { r: 110, g: 115, b: 120 };    // warm grey
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{Context, Editor, Helper};
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Print the welcome banner
fn print_banner() {
    let logo = r#"
    ╔═══════════════════════════════════════════════╗
    ║                                               ║
    ║     ███████╗██╗      ██████╗ ██╗    ██╗       ║
    ║     ██╔════╝██║     ██╔═══██╗██║    ██║       ║
    ║     █████╗  ██║     ██║   ██║██║ █╗ ██║       ║
    ║     ██╔══╝  ██║     ██║   ██║██║███╗██║       ║
    ║     ██║     ███████╗╚██████╔╝╚███╔███╔╝       ║
    ║     ╚═╝     ╚══════╝ ╚═════╝  ╚══╝╚══╝       ║
    ║              S I G H T                        ║
    ║                                               ║
    ╚═══════════════════════════════════════════════╝"#;

    println!("{}", logo.with(C_LOGO));
    println!(
        "    {} v{} - Code Execution Flow Analyzer",
        "FlowSight".with(C_TITLE).bold(),
        VERSION
    );
    println!(
        "    {}",
        "Type 'help' for commands, 'quit' to exit".with(C_DIM)
    );
    println!();
}

/// REPL session state
struct Session {
    current_file: Option<PathBuf>,
    format: OutputFormat,
    depth: Option<usize>,
    no_kernel: bool,
}

impl Session {
    fn new() -> Self {
        Self {
            current_file: None,
            format: OutputFormat::Text,
            depth: None,
            no_kernel: false,
        }
    }

    fn file_display(&self) -> String {
        self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("no file")
            .to_string()
    }
}

/// Tab completion helper
struct FlowHelper {
    commands: Vec<String>,
}

impl Helper for FlowHelper {}
impl rustyline::validate::Validator for FlowHelper {}
impl rustyline::highlight::Highlighter for FlowHelper {}
impl rustyline::hint::Hinter for FlowHelper {
    type Hint = String;
}

impl FlowHelper {
    fn new() -> Self {
        Self {
            commands: vec![
                "open".into(),
                "flow".into(),
                "trace".into(),
                "callers".into(),
                "callees".into(),
                "cfg".into(),
                "errors".into(),
                "async".into(),
                "callbacks".into(),
                "analyze".into(),
                "kb".into(),
                "kb stats".into(),
                "kb query".into(),
                "kb chain".into(),
                "kb async-chain".into(),
                "kb match".into(),
                "set".into(),
                "set format".into(),
                "set depth".into(),
                "set no-kernel".into(),
                "ask".into(),
                "explain".into(),
                "status".into(),
                "help".into(),
                "quit".into(),
                "exit".into(),
            ],
        }
    }
}

impl Completer for FlowHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let input = &line[..pos];
        let mut matches = Vec::new();

        for cmd in &self.commands {
            if cmd.starts_with(input) {
                matches.push(Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                });
            }
        }

        Ok((0, matches))
    }
}

/// Run the interactive REPL
pub fn run() -> anyhow::Result<()> {
    print_banner();

    // Load KB once
    let ctx = AnalysisContext::new();
    let kb = ctx.knowledge_base();
    let fw_count = kb.frameworks.len();
    let cb_count: usize = kb.frameworks.values().map(|f| f.callbacks.len()).sum();
    println!(
        "  {} Knowledge base: {} frameworks, {} callbacks loaded",
        ">>".with(C_OK),
        fw_count,
        cb_count
    );
    println!();

    let helper = FlowHelper::new();
    let mut rl = Editor::new()?;
    rl.set_helper(Some(helper));

    // Try to load history
    let history_path = dirs_home().join(".flowsight_history");
    let _ = rl.load_history(&history_path);

    let mut session = Session::new();

    loop {
        let prompt = format!(
            "{} {} ",
            session.file_display().with(C_FILE),
            ">".with(C_PROMPT).bold()
        );

        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);

                match execute_command(line, &mut session) {
                    Ok(should_quit) => {
                        if should_quit {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "Error:".with(C_ERR).bold(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C: use 'quit' to exit");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                println!("{} {}", "Error:".with(C_ERR), e);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_path);
    println!(
        "{}",
        "Goodbye!".with(C_DIM)
    );
    Ok(())
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Execute a REPL command. Returns Ok(true) if should quit.
fn execute_command(input: &str, session: &mut Session) -> anyhow::Result<bool> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(false);
    }

    match parts[0] {
        "quit" | "exit" | "q" => return Ok(true),

        "help" | "h" | "?" => {
            print_help();
        }

        "open" | "o" => {
            if parts.len() < 2 {
                anyhow::bail!("Usage: open <file.c>");
            }
            let path = PathBuf::from(parts[1]);
            if !path.exists() {
                anyhow::bail!("File not found: {}", parts[1]);
            }
            let mut ctx = AnalysisContext::new();
            let result = ctx.analyze_file(&path)?;
            let func_count = result.parse_result.functions.len();
            let entry_count = result.analysis.entry_points.len();
            let async_count = result.analysis.async_bindings.len();

            session.current_file = Some(path);
            println!(
                "  {} {} functions, {} entry points, {} async handlers",
                "Loaded:".with(C_OK),
                func_count,
                entry_count,
                async_count
            );
        }

        "flow" | "f" => {
            let file = require_file(session, parts.get(1))?;
            let func = if session.current_file.is_some() && parts.len() >= 2 {
                parts[1]
            } else if parts.len() >= 3 {
                parts[2]
            } else {
                anyhow::bail!("Usage: flow [file] <function>");
            };

            let opts = commands::flow::FlowOptions {
                max_depth: parse_flag(&parts, "--depth").or(session.depth),
                no_kernel: parts.contains(&"--no-kernel") || session.no_kernel,
                expand_async: parts.contains(&"--expand-async"),
                show_conditions: parts.contains(&"--show-conditions"),
                error_only: parts.contains(&"--error-only"),
                happy_path: parts.contains(&"--happy-path"),
                cross_file: parts.contains(&"--cross-file"),
                index_db: parse_string_flag(&parts, "--index").map(std::path::PathBuf::from),
                cross_file_depth: parse_flag(&parts, "--cross-depth").unwrap_or(3),
            };
            let format = parse_format_flag(&parts).unwrap_or(session.format);
            commands::flow::run(&file, func, &format, &opts)?;
        }

        "trace" | "t" => {
            let file = require_file(session, parts.get(1))?;
            let func = if session.current_file.is_some() && parts.len() >= 2 {
                parts[1]
            } else if parts.len() >= 3 {
                parts[2]
            } else {
                anyhow::bail!("Usage: trace [file] <function>");
            };
            commands::flow::run_trace(&file, func, "ftrace")?;
        }

        "callers" => {
            let file = require_file(session, parts.get(1))?;
            let func = if session.current_file.is_some() && parts.len() >= 2 {
                parts[1]
            } else if parts.len() >= 3 {
                parts[2]
            } else {
                anyhow::bail!("Usage: callers [file] <function>");
            };
            let format = parse_format_flag(&parts).unwrap_or(session.format);
            commands::graph::run_callers(&file, func, &format)?;
        }

        "callees" => {
            let file = require_file(session, parts.get(1))?;
            let func = if session.current_file.is_some() && parts.len() >= 2 {
                parts[1]
            } else if parts.len() >= 3 {
                parts[2]
            } else {
                anyhow::bail!("Usage: callees [file] <function>");
            };
            let format = parse_format_flag(&parts).unwrap_or(session.format);
            commands::graph::run_callees(&file, func, &format)?;
        }

        "cfg" => {
            let file = require_file(session, parts.get(1))?;
            let func = if session.current_file.is_some() && parts.len() >= 2 {
                parts[1]
            } else if parts.len() >= 3 {
                parts[2]
            } else {
                anyhow::bail!("Usage: cfg [file] <function>");
            };
            let format = parse_format_flag(&parts).unwrap_or(session.format);
            commands::cfg::run(&file, func, &format)?;
        }

        "errors" => {
            let file = require_file(session, parts.get(1))?;
            let func = if session.current_file.is_some() && parts.len() >= 2 {
                Some(parts[1])
            } else if parts.len() >= 3 {
                Some(parts[2])
            } else {
                None
            };
            let format = parse_format_flag(&parts).unwrap_or(session.format);
            commands::cfg::run_errors(&file, func, &format)?;
        }

        "async" => {
            let file = require_file(session, parts.get(1))?;
            commands::async_cmd::run_async(&file)?;
        }

        "callbacks" => {
            let file = require_file(session, parts.get(1))?;
            commands::async_cmd::run_callbacks(&file)?;
        }

        "analyze" => {
            let file = require_file(session, parts.get(1))?;
            let format = parse_format_flag(&parts).unwrap_or(session.format);
            let opts = commands::analyze::AnalyzeOptions {
                recursive: parts.contains(&"-r") || parts.contains(&"--recursive"),
                pattern: "*.c".to_string(),
                parallel: None,
                summary: parts.contains(&"--summary"),
            };
            commands::analyze::run(&file, None, &format, &opts)?;
        }

        "kb" => {
            if parts.len() < 2 {
                anyhow::bail!("Usage: kb <stats|query|chain|async-chain|match>");
            }
            let format = parse_format_flag(&parts).unwrap_or(session.format);
            match parts[1] {
                "stats" => commands::kb::run_stats(&format)?,
                "query" => {
                    if parts.len() < 3 {
                        anyhow::bail!("Usage: kb query <term>");
                    }
                    commands::kb::run_query(parts[2], &format)?;
                }
                "chain" => {
                    if parts.len() < 4 {
                        anyhow::bail!("Usage: kb chain <framework> <callback>");
                    }
                    commands::kb::run_chain(parts[2], parts[3], &format)?;
                }
                "async-chain" => {
                    if parts.len() < 3 {
                        anyhow::bail!("Usage: kb async-chain <pattern>");
                    }
                    commands::kb::run_async_chain(parts[2], &format)?;
                }
                "match" => {
                    let file = if parts.len() >= 3 {
                        PathBuf::from(parts[2])
                    } else {
                        require_file(session, None)?
                    };
                    commands::kb::run_match(&file, &format)?;
                }
                other => anyhow::bail!("Unknown kb subcommand: {}", other),
            }
        }

        "set" => {
            if parts.len() < 2 {
                println!("  format:    {:?}", session.format);
                println!("  depth:     {:?}", session.depth);
                println!("  no-kernel: {}", session.no_kernel);
                return Ok(false);
            }
            match parts[1] {
                "format" => {
                    if parts.len() < 3 {
                        anyhow::bail!("Usage: set format <text|json|ftrace|sequence|markdown|dot>");
                    }
                    session.format = match parts[2] {
                        "text" => OutputFormat::Text,
                        "json" => OutputFormat::Json,
                        "ftrace" => OutputFormat::Ftrace,
                        "sequence" => OutputFormat::Sequence,
                        "markdown" => OutputFormat::Markdown,
                        "dot" => OutputFormat::Dot,
                        other => anyhow::bail!("Unknown format: {}", other),
                    };
                    println!("  format = {:?}", session.format);
                }
                "depth" => {
                    if parts.len() < 3 {
                        session.depth = None;
                        println!("  depth = unlimited");
                    } else {
                        let d: usize = parts[2].parse()?;
                        session.depth = Some(d);
                        println!("  depth = {}", d);
                    }
                }
                "no-kernel" => {
                    if parts.len() >= 3 {
                        session.no_kernel = parts[2] == "on" || parts[2] == "true";
                    } else {
                        session.no_kernel = !session.no_kernel;
                    }
                    println!("  no-kernel = {}", session.no_kernel);
                }
                other => anyhow::bail!("Unknown setting: {}", other),
            }
        }

        "status" | "s" => {
            println!(
                "  File:      {}",
                session
                    .current_file
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(none)".into())
            );
            println!("  Format:    {:?}", session.format);
            println!("  Depth:     {:?}", session.depth);
            println!("  No-kernel: {}", session.no_kernel);
        }

        "ask" => {
            if parts.len() < 2 {
                anyhow::bail!("Usage: ask <question>");
            }
            // Join remaining parts as the query (skip flags)
            let query_parts: Vec<&str> = parts[1..]
                .iter()
                .filter(|p| !p.starts_with("--"))
                .copied()
                .collect();
            let query = query_parts.join(" ");
            if query.is_empty() {
                anyhow::bail!("Usage: ask <question>");
            }

            let llm_config = flowsight_llm::config::LlmConfig::with_defaults();
            let opts = commands::ask::AskOptions {
                provider: parse_string_flag(&parts, "--provider").map(|s| s.to_string()),
                model: None,
                file: session.current_file.clone(),
                function: parse_string_flag(&parts, "--function").map(|s| s.to_string()),
                no_stream: parts.contains(&"--no-stream"),
            };
            commands::ask::run(&query, &llm_config, &opts)?;
        }

        "explain" => {
            let file = require_file(session, parts.get(1))?;
            let func = if session.current_file.is_some() && parts.len() >= 2 {
                parts[1]
            } else if parts.len() >= 3 {
                parts[2]
            } else {
                anyhow::bail!("Usage: explain [file] <function>");
            };

            let llm_config = flowsight_llm::config::LlmConfig::with_defaults();
            let provider = parse_string_flag(&parts, "--provider").map(|s| s.to_string());
            let query = format!(
                "Explain the execution flow of the function {}(). \
                 Describe:\n\
                 1. What it does (purpose)\n\
                 2. The normal execution path\n\
                 3. Error handling paths and cleanup\n\
                 4. Async mechanisms and callbacks registered\n\
                 5. Locking and execution context considerations",
                func
            );
            let opts = commands::ask::AskOptions {
                provider,
                model: None,
                file: Some(file),
                function: Some(func.to_string()),
                no_stream: parts.contains(&"--no-stream"),
            };
            commands::ask::run(&query, &llm_config, &opts)?;
        }

        other => {
            anyhow::bail!(
                "Unknown command: '{}'. Type 'help' for available commands.",
                other
            );
        }
    }

    Ok(false)
}

fn require_file(session: &Session, arg: Option<&&str>) -> anyhow::Result<PathBuf> {
    if let Some(path_str) = arg {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Ok(p);
        }
        // If it doesn't exist as a path and we have a current file, treat as function name
        if session.current_file.is_some() {
            // The caller will use current_file
        }
    }

    session
        .current_file
        .clone()
        .ok_or_else(|| anyhow::anyhow!("No file loaded. Use 'open <file.c>' first."))
}

fn parse_flag(parts: &[&str], flag: &str) -> Option<usize> {
    for (i, part) in parts.iter().enumerate() {
        if *part == flag {
            if let Some(val) = parts.get(i + 1) {
                return val.parse().ok();
            }
        }
    }
    None
}

fn parse_string_flag<'a>(parts: &[&'a str], flag: &str) -> Option<&'a str> {
    for (i, part) in parts.iter().enumerate() {
        if *part == flag {
            if let Some(val) = parts.get(i + 1) {
                return Some(val);
            }
        }
    }
    None
}

fn parse_format_flag(parts: &[&str]) -> Option<OutputFormat> {
    for (i, part) in parts.iter().enumerate() {
        if *part == "-F" || *part == "--format" {
            if let Some(val) = parts.get(i + 1) {
                return match *val {
                    "text" => Some(OutputFormat::Text),
                    "json" => Some(OutputFormat::Json),
                    "ftrace" => Some(OutputFormat::Ftrace),
                    "sequence" => Some(OutputFormat::Sequence),
                    "markdown" => Some(OutputFormat::Markdown),
                    "dot" => Some(OutputFormat::Dot),
                    _ => None,
                };
            }
        }
    }
    None
}

fn print_help() {
    println!();
    println!(
        "  {}",
        "File Commands:".with(C_HEAD).bold()
    );
    println!("    open <file>              Load a C source file");
    println!("    analyze [file]           Full analysis of current file");
    println!("    status                   Show current session state");
    println!();
    println!(
        "  {}",
        "Flow Analysis:".with(C_HEAD).bold()
    );
    println!("    flow <func>              Show execution flow tree");
    println!("      --depth N              Limit depth");
    println!("      --no-kernel            Hide kernel API calls");
    println!("    trace <func>             Ftrace-style output");
    println!("    callers <func>           Who calls this function");
    println!("    callees <func>           What this function calls");
    println!("    cfg <func>               Control flow graph (blocks + edges + error paths)");
    println!("    errors [func]            List error handling paths");
    println!("    async                    List async handlers");
    println!("    callbacks                List callback functions");
    println!();
    println!(
        "  {}",
        "Knowledge Base:".with(C_HEAD).bold()
    );
    println!("    kb stats                 KB statistics");
    println!("    kb query <term>          Search KB");
    println!("    kb chain <fw> <cb>       Kernel call chain");
    println!("    kb async-chain <pat>     Async handler chain");
    println!("    kb match [file]          Match file against KB");
    println!();
    println!(
        "  {}",
        "AI / LLM:".with(C_HEAD).bold()
    );
    println!("    ask <question>           Ask LLM about loaded file");
    println!("      --provider <name>      Select provider (openai/claude/ollama)");
    println!("      --function <fn>        Focus on specific function");
    println!("    explain <func>           AI-powered function explanation");
    println!();
    println!(
        "  {}",
        "Settings:".with(C_HEAD).bold()
    );
    println!("    set                      Show all settings");
    println!("    set format <fmt>         text|json|ftrace|sequence|markdown");
    println!("    set depth <N>            Default depth limit");
    println!("    set no-kernel            Toggle kernel API filter");
    println!();
    println!(
        "  {}",
        "Other:".with(C_HEAD).bold()
    );
    println!("    help                     This help");
    println!("    quit                     Exit FlowSight");
    println!();
    println!(
        "  {}",
        "Tip: Use -F sequence with kb commands for sequence diagrams"
            .with(C_DIM)
    );
    println!();
}
