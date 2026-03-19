//! `flowsight search` - symbol search across source files
//!
//! Two search modes:
//! - **Direct**: Walk directory, parse each file with tree-sitter, find matches
//! - **Indexed**: Query pre-built SQLite index for instant results (--use-index)

use crate::context::AnalysisContext;
use crate::index_db::{IndexDb, SymbolRow};
use crate::output::OutputFormat;
use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use std::path::{Path, PathBuf};

// ============================================================================
// Types
// ============================================================================

/// Options for `flowsight search`
pub struct SearchOptions {
    pub kind_filter: Option<SymbolKind>,
    pub regex: bool,
    pub context_lines: usize,
    pub recursive: bool,
    pub file_pattern: String,
    pub limit: usize,
    pub use_index: bool,
    pub db_path: Option<PathBuf>,
}

/// Symbol type filter for search
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SymbolKind {
    Function,
    Struct,
    Macro,
    Enum,
    Typedef,
    Callback,
}

impl SymbolKind {
    fn as_db_kind(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Struct => "struct",
            Self::Macro => "macro",
            Self::Enum => "enum",
            Self::Typedef => "typedef",
            Self::Callback => "callback",
        }
    }
}

/// A single search result from direct (tree-sitter) scanning
#[derive(Debug, Clone)]
struct DirectMatch {
    name: String,
    kind: String,
    file: PathBuf,
    line: u32,
    signature: Option<String>,
    callback_context: Option<String>,
}

/// Grouped search results for display
struct SearchResults {
    query: String,
    matches_by_kind: Vec<(String, Vec<DirectMatch>)>,
    total_matches: usize,
    total_files: usize,
}

// ============================================================================
// Entry point
// ============================================================================

/// Run the search command
pub fn run(
    query: &str,
    path: Option<&Path>,
    format: &OutputFormat,
    opts: &SearchOptions,
) -> Result<()> {
    if opts.use_index {
        run_indexed_search(query, path, format, opts)
    } else {
        run_direct_search(query, path, format, opts)
    }
}

// ============================================================================
// Direct search (tree-sitter, no index required)
// ============================================================================

/// Scan files with tree-sitter, find matching symbols
fn run_direct_search(
    query: &str,
    path: Option<&Path>,
    format: &OutputFormat,
    opts: &SearchOptions,
) -> Result<()> {
    let search_dir = resolve_search_dir(path)?;
    let files = collect_files(&search_dir, &opts.file_pattern, opts.recursive)?;

    if files.is_empty() {
        eprintln!(
            "No files matching '{}' found in '{}'",
            opts.file_pattern,
            search_dir.display()
        );
        return Ok(());
    }

    let matcher = build_matcher(query, opts.regex)?;

    let all_matches = scan_files_for_symbols(&files, &matcher, &search_dir, opts)?;
    let results = group_results(query, all_matches, &files);

    match format {
        OutputFormat::Json => print_json(&results),
        _ => print_text(&results, opts.context_lines, &files),
    }

    Ok(())
}

/// Resolve the directory to search in
fn resolve_search_dir(path: Option<&Path>) -> Result<PathBuf> {
    match path {
        Some(p) => p
            .canonicalize()
            .with_context(|| format!("Path not found: {}", p.display())),
        None => std::env::current_dir().context("Failed to get current directory"),
    }
}

/// Collect source files matching the pattern
fn collect_files(dir: &Path, pattern: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    let glob = globset::GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .with_context(|| format!("Invalid glob pattern: {}", pattern))?
        .compile_matcher();

    if dir.is_file() {
        return Ok(vec![dir.to_path_buf()]);
    }

    let walker = if recursive {
        walkdir::WalkDir::new(dir)
    } else {
        walkdir::WalkDir::new(dir).max_depth(1)
    };

    let files: Vec<_> = walker
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|name| glob.is_match(name))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    Ok(files)
}

/// Build a name matcher (substring or regex)
fn build_matcher(query: &str, is_regex: bool) -> Result<NameMatcher> {
    if is_regex {
        let re = regex::Regex::new(query)
            .with_context(|| format!("Invalid regex pattern: {}", query))?;
        Ok(NameMatcher::Regex(re))
    } else {
        Ok(NameMatcher::Substring(query.to_lowercase()))
    }
}

enum NameMatcher {
    Substring(String),
    Regex(regex::Regex),
}

impl NameMatcher {
    fn is_match(&self, name: &str) -> bool {
        match self {
            Self::Substring(q) => name.to_lowercase().contains(q.as_str()),
            Self::Regex(re) => re.is_match(name),
        }
    }

    fn is_exact(&self, name: &str) -> bool {
        match self {
            Self::Substring(q) => name.to_lowercase() == *q,
            Self::Regex(_) => false,
        }
    }
}

/// Scan files with tree-sitter and collect matching symbols
fn scan_files_for_symbols(
    files: &[PathBuf],
    matcher: &NameMatcher,
    base_dir: &Path,
    opts: &SearchOptions,
) -> Result<Vec<DirectMatch>> {
    let total = files.len();
    let mut all_matches = Vec::new();
    let mut scanned = 0usize;

    let kind_filter = opts.kind_filter.map(|k| k.as_db_kind().to_string());

    for file_path in files {
        scanned += 1;
        if scanned % 100 == 0 || scanned == total {
            eprint!(
                "\r  {} {}/{} files scanned",
                "[Scanning]".with(Color::DarkCyan),
                scanned,
                total
            );
        }

        let file_matches = scan_single_file(file_path, matcher, base_dir, kind_filter.as_deref());
        match file_matches {
            Ok(matches) => all_matches.extend(matches),
            Err(e) => {
                tracing::debug!("Skipped {}: {}", file_path.display(), e);
            }
        }

        if all_matches.len() >= opts.limit {
            all_matches.truncate(opts.limit);
            break;
        }
    }

    if total > 1 {
        eprintln!();
    }

    // Sort: exact matches first, then alphabetical
    all_matches.sort_by(|a, b| {
        let a_exact = matcher.is_exact(&a.name);
        let b_exact = matcher.is_exact(&b.name);
        b_exact.cmp(&a_exact).then_with(|| a.name.cmp(&b.name))
    });

    Ok(all_matches)
}

/// Parse a single file and find matching symbols
fn scan_single_file(
    file_path: &Path,
    matcher: &NameMatcher,
    base_dir: &Path,
    kind_filter: Option<&str>,
) -> Result<Vec<DirectMatch>> {
    let ctx = AnalysisContext::new();
    let parse_result = ctx.parse_file(file_path)?;

    let rel_path = file_path
        .strip_prefix(base_dir)
        .unwrap_or(file_path)
        .to_path_buf();

    let mut matches = Vec::new();

    // Search functions
    let search_functions = kind_filter
        .map(|k| k == "function" || k == "callback")
        .unwrap_or(true);

    if search_functions {
        for (name, func) in &parse_result.functions {
            if !matcher.is_match(name) {
                continue;
            }

            let kind_str = if func.is_callback {
                "callback"
            } else {
                "function"
            };

            if let Some(filter) = kind_filter {
                if filter != kind_str {
                    continue;
                }
            }

            let line = func.location.as_ref().map(|l| l.line).unwrap_or(0);

            let sig = build_short_signature(name, &func.return_type, &func.params);

            matches.push(DirectMatch {
                name: name.clone(),
                kind: kind_str.to_string(),
                file: rel_path.clone(),
                line,
                signature: Some(sig),
                callback_context: func.callback_context.clone(),
            });
        }
    }

    // Search structs
    let search_structs = kind_filter
        .map(|k| k == "struct")
        .unwrap_or(true);

    if search_structs {
        for (name, st) in &parse_result.structs {
            if !matcher.is_match(name) {
                continue;
            }

            let line = st.location.as_ref().map(|l| l.line).unwrap_or(0);

            matches.push(DirectMatch {
                name: name.clone(),
                kind: "struct".to_string(),
                file: rel_path.clone(),
                line,
                signature: None,
                callback_context: None,
            });
        }
    }

    Ok(matches)
}

/// Build a short function signature
fn build_short_signature(
    name: &str,
    return_type: &str,
    params: &[flowsight_core::Parameter],
) -> String {
    let param_str: String = params
        .iter()
        .map(|p| format!("{} {}", p.type_name, p.name))
        .collect::<Vec<_>>()
        .join(", ");

    format!("{} {}({})", return_type, name, param_str)
}

/// Group matches by kind for display
fn group_results(query: &str, matches: Vec<DirectMatch>, _files: &[PathBuf]) -> SearchResults {
    use std::collections::{BTreeMap, HashSet};

    let mut by_kind: BTreeMap<String, Vec<DirectMatch>> = BTreeMap::new();
    let mut seen_files = HashSet::new();

    for m in &matches {
        seen_files.insert(m.file.to_string_lossy().to_string());
        by_kind
            .entry(m.kind.clone())
            .or_default()
            .push(m.clone());
    }

    let total_matches = matches.len();
    let total_files = seen_files.len();

    let matches_by_kind: Vec<(String, Vec<DirectMatch>)> = by_kind.into_iter().collect();

    SearchResults {
        query: query.to_string(),
        matches_by_kind,
        total_matches,
        total_files,
    }
}

// ============================================================================
// Indexed search (SQLite, requires prior `flowsight index build`)
// ============================================================================

/// Search using pre-built SQLite index
fn run_indexed_search(
    query: &str,
    path: Option<&Path>,
    format: &OutputFormat,
    opts: &SearchOptions,
) -> Result<()> {
    let db_path = find_index_db(path, opts.db_path.as_deref())?;
    let db = IndexDb::open_readonly(&db_path)?;

    let kind_str = opts.kind_filter.map(|k| k.as_db_kind());

    let rows = if opts.regex {
        // For regex, fetch all matching symbols (use LIKE with broad pattern, then filter)
        let broad_results = db.query_symbol_like("%", kind_str, opts.limit * 4)?;
        let re = regex::Regex::new(query)
            .with_context(|| format!("Invalid regex: {}", query))?;
        broad_results
            .into_iter()
            .filter(|r| re.is_match(&r.name))
            .take(opts.limit)
            .collect::<Vec<_>>()
    } else {
        db.query_symbol_like(query, kind_str, opts.limit)?
    };

    if rows.is_empty() {
        eprintln!("No symbols matching '{}' found in index", query);
        eprintln!(
            "Hint: try `flowsight search {} <path> -r` for direct scan",
            query
        );
        return Ok(());
    }

    match format {
        OutputFormat::Json => print_indexed_json(&rows, &db),
        _ => print_indexed_text(query, &rows, &db),
    }

    Ok(())
}

/// Find the index database, searching parent directories
fn find_index_db(dir: Option<&Path>, explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        if p.exists() {
            return Ok(p.to_path_buf());
        }
        anyhow::bail!("Database not found at {}", p.display());
    }

    let start = match dir {
        Some(d) => d
            .canonicalize()
            .with_context(|| format!("Path not found: {}", d.display()))?,
        None => std::env::current_dir()?,
    };

    let mut current = start.as_path();
    loop {
        let candidate = current.join(".flowsight").join("index.db");
        if candidate.exists() {
            return Ok(candidate);
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    anyhow::bail!(
        "No index database found. Run `flowsight index build <dir>` first, \
         or search without --use-index for direct scan."
    )
}

// ============================================================================
// Text output
// ============================================================================

/// Print direct-search results as formatted text
fn print_text(results: &SearchResults, _context_lines: usize, _files: &[PathBuf]) {
    let heading = Color::DarkCyan;
    let dim = Color::DarkGrey;
    let value = Color::Grey;
    let white = Color::White;

    println!();
    println!(
        "{} \"{}\"",
        "FlowSight Search:".with(heading),
        results.query.as_str().with(white),
    );
    let separator_len = 28 + results.query.len();
    println!(
        "{}",
        "=".repeat(separator_len.min(60)).as_str().with(heading)
    );

    if results.total_matches == 0 {
        println!();
        println!("  {}", "No matches found.".with(dim));
        return;
    }

    for (kind, matches) in &results.matches_by_kind {
        let kind_label = pretty_kind_label(kind);
        println!();
        println!(
            "{} ({} {})",
            kind_label.as_str().with(heading),
            matches.len(),
            if matches.len() == 1 { "match" } else { "matches" }
        );
        let rule_len = kind_label.len() + 10;
        println!("{}", "-".repeat(rule_len.min(40)).as_str().with(dim));

        for m in matches {
            let cb_tag = m
                .callback_context
                .as_ref()
                .map(|ctx| format!("  [{}]", ctx))
                .unwrap_or_default();

            println!(
                "  {:<28} {}:{}{}",
                m.name.as_str().with(white),
                m.file.display().to_string().as_str().with(value),
                m.line,
                cb_tag.as_str().with(Color::DarkYellow),
            );

            if let Some(ref sig) = m.signature {
                println!("    {}", sig.as_str().with(dim));
            }
        }
    }

    println!();
    println!(
        "{} {} {} in {} {}",
        "Total:".with(heading),
        results.total_matches,
        if results.total_matches == 1 {
            "match"
        } else {
            "matches"
        },
        results.total_files,
        if results.total_files == 1 {
            "file"
        } else {
            "files"
        },
    );
}

/// Print indexed-search results as formatted text
fn print_indexed_text(query: &str, rows: &[SymbolRow], db: &IndexDb) {
    let heading = Color::DarkCyan;
    let dim = Color::DarkGrey;
    let value = Color::Grey;
    let white = Color::White;

    println!();
    println!(
        "{} \"{}\" {}",
        "FlowSight Search:".with(heading),
        query.with(white),
        "(indexed)".with(dim),
    );
    let separator_len = 38 + query.len();
    println!(
        "{}",
        "=".repeat(separator_len.min(60)).as_str().with(heading)
    );

    // Group by kind
    let grouped = group_symbol_rows(rows);

    let mut total_files = std::collections::HashSet::new();
    for row in rows {
        total_files.insert(row.file_path.as_str());
    }

    for (kind, kind_rows) in &grouped {
        let kind_label = pretty_kind_label(kind);
        println!();
        println!(
            "{} ({} {})",
            kind_label.as_str().with(heading),
            kind_rows.len(),
            if kind_rows.len() == 1 {
                "match"
            } else {
                "matches"
            }
        );
        let rule_len = kind_label.len() + 10;
        println!("{}", "-".repeat(rule_len.min(40)).as_str().with(dim));

        for row in kind_rows {
            let callers_count = db.count_callers(&row.name).unwrap_or(0);
            let subsys = row
                .file_subsystem
                .as_deref()
                .unwrap_or("");

            let callers_tag = if callers_count > 0 {
                format!("  ({} callers)", callers_count)
            } else {
                String::new()
            };

            let subsys_tag = if subsys.is_empty() {
                String::new()
            } else {
                format!("  [{}]", subsys)
            };

            println!(
                "  {:<28} {}:{}{}{}",
                row.name.as_str().with(white),
                row.file_path.as_str().with(value),
                row.line_start,
                subsys_tag.as_str().with(Color::DarkYellow),
                callers_tag.as_str().with(dim),
            );

            if let Some(ref sig) = row.signature {
                println!("    {}", sig.as_str().with(dim));
            }
        }
    }

    println!();
    println!(
        "{} {} {} in {} {}",
        "Total:".with(heading),
        rows.len(),
        if rows.len() == 1 { "match" } else { "matches" },
        total_files.len(),
        if total_files.len() == 1 {
            "file"
        } else {
            "files"
        },
    );
}

/// Group SymbolRows by kind, preserving order
fn group_symbol_rows(rows: &[SymbolRow]) -> Vec<(String, Vec<&SymbolRow>)> {
    use std::collections::BTreeMap;
    let mut by_kind: BTreeMap<String, Vec<&SymbolRow>> = BTreeMap::new();

    for row in rows {
        by_kind.entry(row.kind.clone()).or_default().push(row);
    }

    by_kind.into_iter().collect()
}

/// Make kind labels human-readable
fn pretty_kind_label(kind: &str) -> String {
    match kind {
        "function" => "Functions".to_string(),
        "struct" => "Structs".to_string(),
        "macro" => "Macros".to_string(),
        "enum" => "Enums".to_string(),
        "typedef" => "Typedefs".to_string(),
        "callback" => "Callbacks".to_string(),
        other => {
            let mut s = other.to_string();
            if let Some(first) = s.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            format!("{}s", s)
        }
    }
}

// ============================================================================
// JSON output
// ============================================================================

/// Print direct-search results as JSON
fn print_json(results: &SearchResults) {
    let items: Vec<_> = results
        .matches_by_kind
        .iter()
        .flat_map(|(_, matches)| {
            matches.iter().map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "kind": m.kind,
                    "file": m.file.display().to_string(),
                    "line": m.line,
                    "signature": m.signature,
                    "callback_context": m.callback_context,
                })
            })
        })
        .collect();

    let output = serde_json::json!({
        "query": results.query,
        "total_matches": results.total_matches,
        "total_files": results.total_files,
        "results": items,
    });

    if let Ok(json_str) = serde_json::to_string_pretty(&output) {
        println!("{}", json_str);
    }
}

/// Print indexed-search results as JSON
fn print_indexed_json(rows: &[SymbolRow], db: &IndexDb) {
    let items: Vec<_> = rows
        .iter()
        .map(|r| {
            let callers = db.count_callers(&r.name).unwrap_or(0);
            serde_json::json!({
                "name": r.name,
                "kind": r.kind,
                "file": r.file_path,
                "subsystem": r.file_subsystem,
                "line_start": r.line_start,
                "line_end": r.line_end,
                "is_exported": r.is_exported,
                "signature": r.signature,
                "callers_count": callers,
            })
        })
        .collect();

    let mut total_files = std::collections::HashSet::new();
    for r in rows {
        total_files.insert(r.file_path.as_str());
    }

    let output = serde_json::json!({
        "query": rows.first().map(|_| "").unwrap_or(""),
        "mode": "indexed",
        "total_matches": rows.len(),
        "total_files": total_files.len(),
        "results": items,
    });

    if let Ok(json_str) = serde_json::to_string_pretty(&output) {
        println!("{}", json_str);
    }
}
