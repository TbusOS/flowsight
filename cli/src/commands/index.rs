//! `flowsight index` command group - cross-file index build, query, stats, update
//!
//! The most important feature for kernel-scale analysis: indexes symbols,
//! calls, and async handlers across thousands of source files into SQLite.

use crate::context::AnalysisContext;
use crate::index_db::{
    build_signature, detect_subsystem, hash_content, is_exported_fn, IndexDb, IndexDbStats,
};
use crate::output::OutputFormat;
use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Default database path relative to the indexed directory
const DEFAULT_DB_DIR: &str = ".flowsight";
const DEFAULT_DB_NAME: &str = "index.db";

// ============================================================================
// Build
// ============================================================================

/// Options for `flowsight index build`
pub struct BuildOptions {
    pub pattern: String,
    pub db_path: Option<PathBuf>,
    pub parallel: Option<usize>,
    pub subsystem: bool,
}

/// Build (or rebuild) the cross-file index for a directory
pub fn run_build(dir: &Path, format: &OutputFormat, opts: &BuildOptions) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("Directory not found: {}", dir.display()))?;

    let db_path = resolve_db_path(&dir, opts.db_path.as_deref());
    let files = collect_source_files(&dir, &opts.pattern)?;

    if files.is_empty() {
        anyhow::bail!(
            "No files matching '{}' found in '{}'",
            opts.pattern,
            dir.display()
        );
    }

    eprint_progress("Scanning", &format!("{} files found", files.len()));

    let db = IndexDb::open(&db_path)?;
    let indexed = index_files_parallel(&db, &files, &dir, opts)?;

    match format {
        OutputFormat::Json => {
            let stats = db.stats()?;
            print_stats_json(&stats)?;
        }
        _ => {
            let stats = db.stats()?;
            print_build_summary(&db_path, &stats, indexed);
        }
    }

    Ok(())
}

/// Parse and index files using rayon parallelism, write results to SQLite
fn index_files_parallel(
    db: &IndexDb,
    files: &[PathBuf],
    base_dir: &Path,
    opts: &BuildOptions,
) -> Result<usize> {
    use rayon::prelude::*;

    if let Some(threads) = opts.parallel {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok();
    }

    let total = files.len();
    let indexed_count = AtomicUsize::new(0);
    let skipped_count = AtomicUsize::new(0);
    let error_count = AtomicUsize::new(0);

    // Phase 1: Parse all files in parallel, collecting results in memory
    let file_results: Vec<_> = files
        .par_iter()
        .map(|file_path| {
            let result = parse_single_file(file_path, base_dir, opts.subsystem);
            let current = indexed_count.fetch_add(1, Ordering::Relaxed) + 1;
            if current % 50 == 0 || current == total {
                eprint_progress(
                    "Parsing",
                    &format!("{}/{} files", current, total),
                );
            }
            (file_path.clone(), result)
        })
        .collect();

    // Phase 2: Write to SQLite (single-threaded, inside transaction)
    db.begin_transaction()?;
    let mut success_count = 0usize;

    for (file_path, result) in &file_results {
        match result {
            Ok(parsed) => {
                if let Err(e) = write_file_to_db(db, parsed) {
                    eprintln!(
                        "  {} write error for {}: {}",
                        "!".with(Color::DarkYellow),
                        file_path.display(),
                        e
                    );
                    error_count.fetch_add(1, Ordering::Relaxed);
                } else {
                    success_count += 1;
                }
            }
            Err(e) => {
                skipped_count.fetch_add(1, Ordering::Relaxed);
                tracing::debug!("Skipped {}: {}", file_path.display(), e);
            }
        }
    }

    db.commit_transaction()?;

    let skipped = skipped_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);
    if skipped > 0 || errors > 0 {
        eprintln!(
            "  {} {} skipped, {} errors",
            "!".with(Color::DarkYellow),
            skipped,
            errors
        );
    }

    Ok(success_count)
}

/// Parsed file data ready to be written to the database
struct ParsedFileData {
    path: String,
    subsystem: Option<String>,
    last_modified: i64,
    hash: String,
    symbols: Vec<ParsedSymbol>,
}

/// A parsed symbol with its calls
struct ParsedSymbol {
    name: String,
    kind: String,
    line_start: u32,
    line_end: u32,
    is_exported: bool,
    signature: Option<String>,
    calls: Vec<(String, u32, bool)>, // (callee_name, call_line, is_indirect)
    async_handler: Option<(String, bool)>, // (mechanism, can_sleep)
}

/// Parse a single file and extract index data (thread-safe, no DB access)
fn parse_single_file(
    file_path: &Path,
    base_dir: &Path,
    detect_subsys: bool,
) -> Result<ParsedFileData> {
    let content = std::fs::read(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let hash = hash_content(&content);
    let _source = String::from_utf8_lossy(&content);

    let mtime = std::fs::metadata(file_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let rel_path = file_path
        .strip_prefix(base_dir)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    let subsystem = if detect_subsys {
        detect_subsystem(&rel_path).or_else(|| detect_subsystem(&file_path.to_string_lossy()))
    } else {
        None
    };

    let mut ctx = AnalysisContext::new();
    let analysis = ctx.analyze_file(file_path)?;

    let mut symbols = Vec::new();

    // Extract functions
    for (name, func) in &analysis.parse_result.functions {
        let (line_start, line_end) = func
            .location
            .as_ref()
            .map(|loc| (loc.line, loc.end_line))
            .unwrap_or((0, 0));

        let kind = if func.is_callback {
            "callback"
        } else {
            "function"
        };

        let sig = build_signature(name, &func.return_type, &func.params);
        let exported = is_exported_fn(&func.attributes, name);

        let calls: Vec<(String, u32, bool)> = func
            .calls
            .iter()
            .map(|callee| (callee.clone(), line_start, false))
            .collect();

        // Check if this function is an async handler
        let async_handler = analysis
            .analysis
            .async_bindings
            .iter()
            .find(|b| b.handler == *name)
            .map(|b| {
                let mechanism = format!("{:?}", b.mechanism);
                let can_sleep = b.context.can_sleep();
                (mechanism, can_sleep)
            });

        symbols.push(ParsedSymbol {
            name: name.clone(),
            kind: kind.to_string(),
            line_start,
            line_end,
            is_exported: exported,
            signature: Some(sig),
            calls,
            async_handler,
        });
    }

    // Extract structs
    for (name, st) in &analysis.parse_result.structs {
        let (line_start, line_end) = st
            .location
            .as_ref()
            .map(|loc| (loc.line, loc.end_line))
            .unwrap_or((0, 0));

        symbols.push(ParsedSymbol {
            name: name.clone(),
            kind: "struct".to_string(),
            line_start,
            line_end,
            is_exported: false,
            signature: None,
            calls: Vec::new(),
            async_handler: None,
        });
    }

    // Add indirect call edges from analysis
    for edge in &analysis.analysis.call_edges {
        if let flowsight_core::CallType::Indirect { .. } = edge.call_type {
            let call_line = edge.location.as_ref().map(|l| l.line).unwrap_or(0);
            if let Some(sym) = symbols.iter_mut().find(|s| s.name == edge.caller) {
                sym.calls.push((edge.callee.clone(), call_line, true));
            }
        }
    }

    Ok(ParsedFileData {
        path: rel_path,
        subsystem,
        last_modified: mtime,
        hash,
        symbols,
    })
}

/// Write a parsed file's data into the SQLite database
fn write_file_to_db(db: &IndexDb, data: &ParsedFileData) -> Result<()> {
    let file_id = db.upsert_file(
        &data.path,
        data.subsystem.as_deref(),
        data.last_modified,
        &data.hash,
    )?;

    db.clear_file_symbols(file_id)?;

    for sym in &data.symbols {
        let sym_id = db.insert_symbol(
            &sym.name,
            &sym.kind,
            file_id,
            sym.line_start,
            sym.line_end,
            sym.is_exported,
            sym.signature.as_deref(),
        )?;

        for (callee, line, indirect) in &sym.calls {
            db.insert_call(sym_id, callee, *line, *indirect)?;
        }

        if let Some((mechanism, can_sleep)) = &sym.async_handler {
            db.insert_async_handler(sym_id, mechanism, *can_sleep)?;
        }
    }

    Ok(())
}

// ============================================================================
// Query
// ============================================================================

/// Options for `flowsight index query`
pub struct QueryOptions {
    pub kind_filter: Option<String>,
    pub subsystem_filter: Option<String>,
    pub db_path: Option<PathBuf>,
}

/// Look up a symbol across the entire indexed codebase
pub fn run_query(
    symbol: &str,
    dir: Option<&Path>,
    format: &OutputFormat,
    opts: &QueryOptions,
) -> Result<()> {
    let db_path = find_db_path(dir, opts.db_path.as_deref())?;
    let db = IndexDb::open_readonly(&db_path)?;

    let results = db.query_symbol(
        symbol,
        opts.kind_filter.as_deref(),
        opts.subsystem_filter.as_deref(),
    )?;

    if results.is_empty() {
        eprintln!("No symbol '{}' found in index", symbol);
        eprintln!("Hint: run `flowsight index build <dir>` first");
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "kind": r.kind,
                        "file": r.file_path,
                        "subsystem": r.file_subsystem,
                        "line_start": r.line_start,
                        "line_end": r.line_end,
                        "is_exported": r.is_exported,
                        "signature": r.signature,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
        _ => {
            print_query_results(symbol, &results, &db)?;
        }
    }

    Ok(())
}

/// Print query results in text format
fn print_query_results(
    symbol: &str,
    results: &[crate::index_db::SymbolRow],
    db: &IndexDb,
) -> Result<()> {
    let label = Color::DarkCyan;
    let dim = Color::DarkGrey;
    let value = Color::Grey;

    println!(
        "{} {} ({} {})",
        "Symbol:".with(label),
        symbol.with(Color::White),
        results.len(),
        if results.len() == 1 {
            "definition"
        } else {
            "definitions"
        }
    );
    println!();

    for row in results {
        let subsys = row
            .file_subsystem
            .as_deref()
            .unwrap_or("(unknown)");
        let exported = if row.is_exported { " [exported]" } else { "" };

        println!(
            "  {} {} {}:{}{}",
            format!("[{}]", row.kind).with(label),
            row.file_path.as_str().with(value),
            "L".with(dim),
            row.line_start.to_string().with(value),
            exported.with(Color::DarkGreen),
        );

        if let Some(ref sig) = row.signature {
            println!("    {}", sig.as_str().with(dim));
        }

        println!(
            "    {} {}",
            "subsystem:".with(dim),
            subsys.with(value)
        );
    }

    // Also show cross-file callers
    let callers = db.query_callers(symbol)?;
    if !callers.is_empty() {
        println!();
        println!(
            "{} ({} call sites)",
            "Cross-file callers:".with(label),
            callers.len()
        );
        for caller in &callers {
            let indirect = if caller.is_indirect {
                " [indirect]"
            } else {
                ""
            };
            println!(
                "  {} {}() at {}:{}{}",
                "->".with(dim),
                caller.caller_name.as_str().with(value),
                caller.caller_file.as_str().with(dim),
                caller.call_line,
                indirect.with(Color::DarkYellow),
            );
        }
    }

    Ok(())
}

// ============================================================================
// Stats
// ============================================================================

/// Show index statistics
pub fn run_stats(dir: Option<&Path>, db_path: Option<&Path>, format: &OutputFormat) -> Result<()> {
    let db_path = find_db_path(dir, db_path)?;
    let db = IndexDb::open_readonly(&db_path)?;
    let stats = db.stats()?;

    match format {
        OutputFormat::Json => print_stats_json(&stats)?,
        _ => print_stats_text(&stats, &db_path),
    }

    Ok(())
}

/// Print stats as JSON
fn print_stats_json(stats: &IndexDbStats) -> Result<()> {
    let json = serde_json::json!({
        "total_files": stats.total_files,
        "total_symbols": stats.total_symbols,
        "total_functions": stats.total_functions,
        "total_structs": stats.total_structs,
        "total_macros": stats.total_macros,
        "total_callbacks": stats.total_callbacks,
        "total_async_handlers": stats.total_async_handlers_count,
        "total_calls": stats.total_calls,
        "subsystems": stats.subsystem_breakdown.iter()
            .map(|(name, count)| serde_json::json!({"name": name, "files": count}))
            .collect::<Vec<_>>(),
        "top_callees": stats.top_callees.iter()
            .map(|(name, count)| serde_json::json!({"name": name, "count": count}))
            .collect::<Vec<_>>(),
        "top_callers": stats.top_callers.iter()
            .map(|(name, count)| serde_json::json!({"name": name, "count": count}))
            .collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// Print stats in human-readable text
fn print_stats_text(stats: &IndexDbStats, db_path: &Path) {
    let label = Color::DarkCyan;
    let value = Color::Grey;
    let dim = Color::DarkGrey;

    println!(
        "{}",
        "FlowSight Index Statistics".with(label)
    );
    println!("{}", "==========================".with(label));
    println!(
        "  {} {}",
        "Database:".with(label),
        db_path.display().to_string().with(dim)
    );
    println!();

    println!(
        "  {} {}",
        "Files indexed:".with(label),
        format_count(stats.total_files).with(value)
    );
    println!(
        "  {} {}",
        "Total symbols:".with(label),
        format_count(stats.total_symbols).with(value)
    );
    println!(
        "    {} {}",
        "Functions:".with(dim),
        format_count(stats.total_functions).with(value)
    );
    println!(
        "    {} {}",
        "Structs:".with(dim),
        format_count(stats.total_structs).with(value)
    );
    println!(
        "    {} {}",
        "Macros:".with(dim),
        format_count(stats.total_macros).with(value)
    );
    println!(
        "    {} {}",
        "Callbacks:".with(dim),
        format_count(stats.total_callbacks).with(value)
    );
    println!(
        "  {} {}",
        "Async handlers:".with(label),
        format_count(stats.total_async_handlers_count).with(value)
    );
    println!(
        "  {} {}",
        "Call edges:".with(label),
        format_count(stats.total_calls).with(value)
    );

    if !stats.subsystem_breakdown.is_empty() {
        println!();
        println!("{}", "Subsystems:".with(label));
        for (name, count) in &stats.subsystem_breakdown {
            println!(
                "  {:<20} {} files",
                name.as_str().with(value),
                count
            );
        }
    }

    if !stats.top_callees.is_empty() {
        println!();
        println!("{}", "Most-called functions:".with(label));
        for (name, count) in &stats.top_callees {
            println!(
                "  {:<40} {} call sites",
                format!("{}()", name).with(value),
                count
            );
        }
    }

    if !stats.top_callers.is_empty() {
        println!();
        println!("{}", "Functions with most calls:".with(label));
        for (name, count) in &stats.top_callers {
            println!(
                "  {:<40} {} callees",
                format!("{}()", name).with(value),
                count
            );
        }
    }
}

// ============================================================================
// Update (incremental)
// ============================================================================

/// Options for `flowsight index update`
pub struct UpdateOptions {
    pub pattern: String,
    pub db_path: Option<PathBuf>,
    pub parallel: Option<usize>,
    pub subsystem: bool,
}

/// Incrementally update the index (only changed files)
pub fn run_update(dir: &Path, format: &OutputFormat, opts: &UpdateOptions) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("Directory not found: {}", dir.display()))?;

    let db_path = resolve_db_path(&dir, opts.db_path.as_deref());

    if !db_path.exists() {
        anyhow::bail!(
            "No index found at {}. Run `flowsight index build` first.",
            db_path.display()
        );
    }

    let db = IndexDb::open(&db_path)?;
    let files = collect_source_files(&dir, &opts.pattern)?;

    // Filter to only files that changed
    let changed_files = filter_changed_files(&db, &files, &dir)?;

    if changed_files.is_empty() {
        eprintln!("Index is up to date. No files changed.");
        return Ok(());
    }

    eprint_progress(
        "Updating",
        &format!("{}/{} files changed", changed_files.len(), files.len()),
    );

    let build_opts = BuildOptions {
        pattern: opts.pattern.clone(),
        db_path: Some(db_path.clone()),
        parallel: opts.parallel,
        subsystem: opts.subsystem,
    };

    let indexed = index_files_parallel(&db, &changed_files, &dir, &build_opts)?;

    match format {
        OutputFormat::Json => {
            let stats = db.stats()?;
            print_stats_json(&stats)?;
        }
        _ => {
            eprintln!(
                "  {} {} files updated",
                "+".with(Color::DarkGreen),
                indexed
            );
        }
    }

    Ok(())
}

/// Filter files to only those whose content hash differs from the database
fn filter_changed_files(
    db: &IndexDb,
    files: &[PathBuf],
    base_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut changed = Vec::new();

    for file in files {
        let content = match std::fs::read(file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let hash = hash_content(&content);
        let rel_path = file
            .strip_prefix(base_dir)
            .unwrap_or(file)
            .to_string_lossy();

        if db.file_needs_reindex(&rel_path, &hash)? {
            changed.push(file.clone());
        }
    }

    Ok(changed)
}

// ============================================================================
// Helpers
// ============================================================================

/// Resolve the database path from options or default location
fn resolve_db_path(dir: &Path, explicit_path: Option<&Path>) -> PathBuf {
    match explicit_path {
        Some(p) => p.to_path_buf(),
        None => dir.join(DEFAULT_DB_DIR).join(DEFAULT_DB_NAME),
    }
}

/// Find an existing database path, searching parent directories
fn find_db_path(dir: Option<&Path>, explicit_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit_path {
        if p.exists() {
            return Ok(p.to_path_buf());
        }
        anyhow::bail!("Database not found at {}", p.display());
    }

    // Search from the given directory (or cwd) upwards
    let start = match dir {
        Some(d) => d.canonicalize()?,
        None => std::env::current_dir()?,
    };

    let mut current = start.as_path();
    loop {
        let candidate = current.join(DEFAULT_DB_DIR).join(DEFAULT_DB_NAME);
        if candidate.exists() {
            return Ok(candidate);
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    anyhow::bail!(
        "No index database found. Run `flowsight index build <dir>` first."
    )
}

/// Walk directory and collect files matching the glob pattern
fn collect_source_files(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let glob = globset::GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .with_context(|| format!("Invalid glob pattern: {}", pattern))?
        .compile_matcher();

    let files: Vec<_> = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| glob.is_match(name))
                .unwrap_or(false)
        })
        .map(|entry| entry.into_path())
        .collect();

    Ok(files)
}

/// Print a progress message to stderr
fn eprint_progress(phase: &str, message: &str) {
    eprintln!(
        "  {} {}",
        format!("[{}]", phase).with(Color::DarkCyan),
        message.with(Color::Grey)
    );
}

/// Print build summary
fn print_build_summary(db_path: &Path, stats: &IndexDbStats, indexed: usize) {
    let label = Color::DarkCyan;
    let value = Color::Grey;

    println!();
    println!(
        "{}",
        "FlowSight Index Built".with(label)
    );
    println!("{}", "=====================".with(label));
    println!(
        "  {} {}",
        "Database:".with(label),
        db_path.display().to_string().with(value)
    );
    println!(
        "  {} {}",
        "Files indexed:".with(label),
        format_count(indexed).with(value)
    );
    println!(
        "  {} {}",
        "Functions:".with(label),
        format_count(stats.total_functions).with(value)
    );
    println!(
        "  {} {}",
        "Structs:".with(label),
        format_count(stats.total_structs).with(value)
    );
    println!(
        "  {} {}",
        "Callbacks:".with(label),
        format_count(stats.total_callbacks).with(value)
    );
    println!(
        "  {} {}",
        "Call edges:".with(label),
        format_count(stats.total_calls).with(value)
    );
    println!(
        "  {} {}",
        "Async handlers:".with(label),
        format_count(stats.total_async_handlers_count).with(value)
    );

    if !stats.subsystem_breakdown.is_empty() {
        println!();
        println!("{}", "Subsystems detected:".with(label));
        for (name, count) in &stats.subsystem_breakdown {
            println!(
                "  {:<20} {} files",
                name.as_str().with(value),
                count
            );
        }
    }
}

/// Format a number with comma separators
fn format_count(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}
