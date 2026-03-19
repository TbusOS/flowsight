//! `flowsight report` command - self-contained HTML report generation
//!
//! Generates a professional, interactive HTML report from analysis results.
//! The report is fully self-contained with no external dependencies -
//! all CSS and JavaScript are embedded inline.

use crate::commands::patterns;
use crate::context::AnalysisContext;
use anyhow::{Context, Result};
use flowsight_core::{AsyncMechanism, CallType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Options for the report command
pub struct ReportOptions {
    pub output: PathBuf,
    pub recursive: bool,
    pub pattern: String,
    pub title: Option<String>,
    pub include_source: bool,
}

/// Per-file data collected for the report
#[allow(dead_code)]
struct FileReportData {
    file: String,
    functions: Vec<FunctionEntry>,
    async_handlers: Vec<AsyncEntry>,
    call_edges: Vec<CallEdgeEntry>,
    pattern_report: Option<patterns::PatternReport>,
    error: Option<String>,
}

/// A function entry for the report table
#[derive(Clone)]
#[allow(dead_code)]
struct FunctionEntry {
    name: String,
    file: String,
    line: u32,
    return_type: String,
    is_callback: bool,
    is_async: bool,
    call_count: usize,
    called_by_count: usize,
}

/// An async handler entry
struct AsyncEntry {
    handler: String,
    mechanism: String,
    variable: String,
    file: String,
    line: u32,
}

/// A call edge entry
#[allow(dead_code)]
struct CallEdgeEntry {
    caller: String,
    callee: String,
    call_type: String,
    file: String,
}

/// Aggregated report data
struct ReportData {
    title: String,
    generated_at: String,
    base_path: String,
    total_files: usize,
    total_functions: usize,
    total_async_handlers: usize,
    total_callbacks: usize,
    total_call_edges: usize,
    files: Vec<FileReportData>,
    errors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run the report command
pub fn run(path: &Path, opts: &ReportOptions) -> Result<()> {
    let data = if path.is_dir() {
        if !opts.recursive {
            anyhow::bail!(
                "'{}' is a directory. Use --recursive (-r) to scan directories.",
                path.display()
            );
        }
        collect_directory_data(path, opts)?
    } else {
        collect_single_file_data(path, opts)?
    };

    let html = render_html(&data, opts.include_source)?;

    std::fs::write(&opts.output, &html)
        .with_context(|| format!("Failed to write report to {}", opts.output.display()))?;

    eprintln!(
        "Report written to: {} ({} files, {} functions)",
        opts.output.display(),
        data.total_files,
        data.total_functions
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Data collection
// ---------------------------------------------------------------------------

/// Collect data from a single file
fn collect_single_file_data(path: &Path, opts: &ReportOptions) -> Result<ReportData> {
    let file_data = analyze_one_file(path)?;
    let title = opts
        .title
        .clone()
        .unwrap_or_else(|| format!("FlowSight: {}", path.display()));

    let total_functions = file_data.functions.len();
    let total_async = file_data.async_handlers.len();
    let total_callbacks = file_data
        .functions
        .iter()
        .filter(|f| f.is_callback)
        .count();
    let total_edges = file_data.call_edges.len();

    Ok(ReportData {
        title,
        generated_at: format_utc_now(),
        base_path: path.to_string_lossy().into_owned(),
        total_files: 1,
        total_functions,
        total_async_handlers: total_async,
        total_callbacks,
        total_call_edges: total_edges,
        files: vec![file_data],
        errors: Vec::new(),
    })
}

/// Collect data from a directory of files
fn collect_directory_data(dir: &Path, opts: &ReportOptions) -> Result<ReportData> {
    let files = collect_matching_files(dir, &opts.pattern)?;

    if files.is_empty() {
        anyhow::bail!(
            "No files matching '{}' found in '{}'",
            opts.pattern,
            dir.display()
        );
    }

    let title = opts
        .title
        .clone()
        .unwrap_or_else(|| format!("FlowSight: {}", dir.display()));

    let results: Mutex<Vec<FileReportData>> = Mutex::new(Vec::new());
    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());

    use rayon::prelude::*;
    files.par_iter().for_each(|file_path| {
        match analyze_one_file(file_path) {
            Ok(data) => {
                if let Ok(mut guard) = results.lock() {
                    guard.push(data);
                }
            }
            Err(e) => {
                if let Ok(mut guard) = errors.lock() {
                    guard.push(format!("{}: {}", file_path.display(), e));
                }
            }
        }
    });

    let mut file_results = results
        .into_inner()
        .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?;
    file_results.sort_by(|a, b| a.file.cmp(&b.file));

    let collected_errors = errors
        .into_inner()
        .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?;

    let total_functions: usize = file_results.iter().map(|f| f.functions.len()).sum();
    let total_async: usize = file_results.iter().map(|f| f.async_handlers.len()).sum();
    let total_callbacks: usize = file_results
        .iter()
        .flat_map(|f| f.functions.iter())
        .filter(|func| func.is_callback)
        .count();
    let total_edges: usize = file_results.iter().map(|f| f.call_edges.len()).sum();

    Ok(ReportData {
        title,
        generated_at: format_utc_now(),
        base_path: dir.to_string_lossy().into_owned(),
        total_files: file_results.len(),
        total_functions,
        total_async_handlers: total_async,
        total_callbacks,
        total_call_edges: total_edges,
        files: file_results,
        errors: collected_errors,
    })
}

/// Analyze a single file and extract report data
fn analyze_one_file(path: &Path) -> Result<FileReportData> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(path)?;
    let file_str = path.to_string_lossy().into_owned();

    // Build function entries
    let async_handler_names: Vec<String> = result
        .analysis
        .async_bindings
        .iter()
        .map(|b| b.handler.clone())
        .collect();

    let functions: Vec<FunctionEntry> = result
        .parse_result
        .functions
        .values()
        .map(|f| {
            let line = f.location.as_ref().map(|l| l.line).unwrap_or(0);
            FunctionEntry {
                name: f.name.clone(),
                file: file_str.clone(),
                line,
                return_type: f.return_type.clone(),
                is_callback: f.is_callback,
                is_async: async_handler_names.contains(&f.name),
                call_count: f.calls.len(),
                called_by_count: f.called_by.len(),
            }
        })
        .collect();

    // Build async entries
    let async_handlers: Vec<AsyncEntry> = result
        .analysis
        .async_bindings
        .iter()
        .map(|b| {
            let line = b.bind_location.as_ref().map(|l| l.line).unwrap_or(0);
            AsyncEntry {
                handler: b.handler.clone(),
                mechanism: format_mechanism(&b.mechanism),
                variable: b.variable.clone(),
                file: file_str.clone(),
                line,
            }
        })
        .collect();

    // Build call edge entries
    let call_edges: Vec<CallEdgeEntry> = result
        .analysis
        .call_edges
        .iter()
        .map(|e| CallEdgeEntry {
            caller: e.caller.clone(),
            callee: e.callee.clone(),
            call_type: format_call_type(&e.call_type),
            file: file_str.clone(),
        })
        .collect();

    // Try pattern detection (best-effort, don't fail the whole file)
    let source = std::fs::read_to_string(path).unwrap_or_default();
    let pattern_report = patterns::detect_patterns(path, &source, None).ok();

    Ok(FileReportData {
        file: file_str,
        functions,
        async_handlers,
        call_edges,
        pattern_report,
        error: None,
    })
}

/// Walk directory and collect files matching the glob pattern
fn collect_matching_files(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
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

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn format_mechanism(mechanism: &AsyncMechanism) -> String {
    match mechanism {
        AsyncMechanism::WorkQueue { delayed } => {
            if *delayed {
                "WorkQueue (delayed)".to_string()
            } else {
                "WorkQueue".to_string()
            }
        }
        AsyncMechanism::Timer { high_resolution } => {
            if *high_resolution {
                "Timer (hrtimer)".to_string()
            } else {
                "Timer".to_string()
            }
        }
        AsyncMechanism::Interrupt { threaded } => {
            if *threaded {
                "IRQ (threaded)".to_string()
            } else {
                "IRQ".to_string()
            }
        }
        AsyncMechanism::Tasklet => "Tasklet".to_string(),
        AsyncMechanism::Softirq => "Softirq".to_string(),
        AsyncMechanism::KThread => "KThread".to_string(),
        AsyncMechanism::RcuCallback => "RCU Callback".to_string(),
        AsyncMechanism::Notifier => "Notifier".to_string(),
        AsyncMechanism::Custom(name) => format!("Custom({})", name),
    }
}

fn format_call_type(call_type: &CallType) -> String {
    match call_type {
        CallType::Direct => "direct".to_string(),
        CallType::Indirect { .. } => "indirect".to_string(),
        CallType::Async { mechanism } => format!("async({})", format_mechanism(mechanism)),
    }
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Format current UTC time without chrono dependency
fn format_utc_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Simple UTC timestamp (no chrono needed)
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Approximate date from days since epoch (1970-01-01)
    let mut y = 1970i64;
    let mut remaining = days_since_epoch as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days: [i64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut m = 0usize;
    for (i, &d) in month_days.iter().enumerate() {
        if remaining < d {
            m = i;
            break;
        }
        remaining -= d;
    }
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        y,
        m + 1,
        remaining + 1,
        hours,
        minutes,
        seconds
    )
}

// ---------------------------------------------------------------------------
// HTML rendering
// ---------------------------------------------------------------------------

/// Render the complete HTML report
fn render_html(data: &ReportData, _include_source: bool) -> Result<String> {
    let mut html = String::with_capacity(64 * 1024);

    write_html_head(&mut html, &data.title);
    write_html_body_open(&mut html);
    write_header_section(&mut html, data);
    write_stat_cards(&mut html, data);
    write_file_index(&mut html, data);
    write_function_table(&mut html, data);
    write_call_graph_summary(&mut html, data);
    write_pattern_summary(&mut html, data);
    write_async_summary(&mut html, data);
    write_per_file_details(&mut html, data);
    write_errors_section(&mut html, data);
    write_javascript(&mut html);
    write_html_close(&mut html);

    Ok(html)
}

fn write_html_head(html: &mut String, title: &str) {
    html.push_str(&format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{}</title>
<style>
:root {{
  --bg: #1a1a2e;
  --surface: #16213e;
  --surface-hover: #1a2745;
  --text: #d4d4d8;
  --text-dim: #71717a;
  --accent: #0f3460;
  --highlight: #c9515e;
  --success: #4ecca3;
  --warning: #d4a853;
  --info: #5b8fb9;
  --border: #27273f;
  --badge-async-bg: #1a3a5c;
  --badge-async-fg: #6cb4ee;
  --badge-cb-bg: #3a3a1c;
  --badge-cb-fg: #d4c46c;
  --badge-direct-bg: #1c3a2c;
  --badge-direct-fg: #6cd4a3;
  --badge-indirect-bg: #3a2a1c;
  --badge-indirect-fg: #d4a06c;
}}

* {{ box-sizing: border-box; margin: 0; padding: 0; }}

body {{
  background: var(--bg);
  color: var(--text);
  font-family: 'SF Mono', 'Cascadia Code', 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 14px;
  line-height: 1.6;
  padding: 24px;
}}

.container {{ max-width: 1280px; margin: 0 auto; }}

h1 {{ color: var(--highlight); font-size: 1.6em; margin-bottom: 4px; }}
h2 {{ color: var(--highlight); font-size: 1.2em; margin: 32px 0 12px; padding-bottom: 6px; border-bottom: 1px solid var(--border); }}
h3 {{ color: var(--info); font-size: 1em; margin: 16px 0 8px; }}

a {{ color: var(--info); text-decoration: none; }}
a:hover {{ text-decoration: underline; }}

.header {{ margin-bottom: 24px; }}
.header .subtitle {{ color: var(--text-dim); font-size: 0.9em; }}

.stat-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
  margin: 16px 0 24px;
}}

.stat-card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 16px;
  text-align: center;
}}

.stat-value {{
  font-size: 2em;
  font-weight: bold;
  color: var(--success);
  line-height: 1.2;
}}

.stat-label {{
  color: var(--text-dim);
  font-size: 0.85em;
  margin-top: 4px;
}}

table {{
  width: 100%;
  border-collapse: collapse;
  margin: 12px 0;
  font-size: 0.9em;
}}

th {{
  background: var(--accent);
  color: var(--text);
  padding: 8px 12px;
  text-align: left;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  position: sticky;
  top: 0;
}}

th:hover {{ background: #12406e; }}
th .sort-indicator {{ margin-left: 4px; opacity: 0.5; }}
th.sorted .sort-indicator {{ opacity: 1; }}

td {{
  padding: 6px 12px;
  border-bottom: 1px solid var(--border);
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}}

tr:hover {{ background: var(--surface-hover); }}

.badge {{
  display: inline-block;
  padding: 1px 8px;
  border-radius: 4px;
  font-size: 0.8em;
  font-weight: 500;
  vertical-align: middle;
}}

.badge-async {{ background: var(--badge-async-bg); color: var(--badge-async-fg); }}
.badge-callback {{ background: var(--badge-cb-bg); color: var(--badge-cb-fg); }}
.badge-direct {{ background: var(--badge-direct-bg); color: var(--badge-direct-fg); }}
.badge-indirect {{ background: var(--badge-indirect-bg); color: var(--badge-indirect-fg); }}
.badge-warning {{ background: #3a2a1c; color: var(--warning); }}
.badge-info {{ background: #1c2a3a; color: var(--info); }}

details {{
  margin: 8px 0;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}}

summary {{
  cursor: pointer;
  padding: 10px 14px;
  background: var(--surface);
  font-weight: 500;
  list-style: none;
  display: flex;
  align-items: center;
  gap: 8px;
}}

summary::-webkit-details-marker {{ display: none; }}
summary::before {{ content: '\25B6'; font-size: 0.7em; transition: transform 0.2s; }}
details[open] > summary::before {{ transform: rotate(90deg); }}

summary:hover {{ background: var(--surface-hover); }}

details .detail-content {{ padding: 12px 14px; }}

.search-box {{
  width: 100%;
  padding: 8px 12px;
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  margin-bottom: 12px;
  font-family: inherit;
  font-size: 0.9em;
  outline: none;
}}

.search-box:focus {{ border-color: var(--info); }}
.search-box::placeholder {{ color: var(--text-dim); }}

.nav-list {{ list-style: none; }}
.nav-list li {{
  padding: 4px 0;
  display: flex;
  justify-content: space-between;
}}
.nav-list .count {{ color: var(--text-dim); font-size: 0.9em; }}

.pattern-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 10px;
  margin: 10px 0;
}}

.pattern-card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 12px;
}}

.pattern-card .cat-name {{ color: var(--info); font-weight: 500; margin-bottom: 4px; }}
.pattern-card .cat-count {{ font-size: 1.4em; font-weight: bold; color: var(--success); }}
.pattern-card .cat-issues {{ color: var(--warning); font-size: 0.85em; margin-top: 4px; }}

.error-list {{
  background: #2e1a1a;
  border: 1px solid #5a2020;
  border-radius: 6px;
  padding: 12px;
  margin: 8px 0;
}}

.error-item {{ padding: 4px 0; color: #d48080; font-size: 0.85em; }}

.hidden {{ display: none; }}

.top-callers {{
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}}

@media (max-width: 768px) {{
  .top-callers {{ grid-template-columns: 1fr; }}
  .stat-grid {{ grid-template-columns: repeat(2, 1fr); }}
  .pattern-grid {{ grid-template-columns: 1fr; }}
}}
</style>
</head>
"#,
        html_escape(title)
    ));
}

fn write_html_body_open(html: &mut String) {
    html.push_str("<body>\n<div class=\"container\">\n");
}

fn write_header_section(html: &mut String, data: &ReportData) {
    html.push_str(&format!(
        r#"<div class="header">
<h1>{}</h1>
<div class="subtitle">Generated: {} | Path: {}</div>
</div>
"#,
        html_escape(&data.title),
        html_escape(&data.generated_at),
        html_escape(&data.base_path),
    ));
}

fn write_stat_cards(html: &mut String, data: &ReportData) {
    html.push_str(r#"<div class="stat-grid">"#);

    let cards = [
        (format_count(data.total_files), "Files Analyzed"),
        (format_count(data.total_functions), "Functions"),
        (format_count(data.total_callbacks), "Callbacks"),
        (format_count(data.total_async_handlers), "Async Handlers"),
        (format_count(data.total_call_edges), "Call Edges"),
        (format_count(data.errors.len()), "Errors"),
    ];

    for (value, label) in &cards {
        html.push_str(&format!(
            r#"<div class="stat-card"><div class="stat-value">{}</div><div class="stat-label">{}</div></div>"#,
            html_escape(value),
            label,
        ));
    }

    html.push_str("</div>\n");
}

fn write_file_index(html: &mut String, data: &ReportData) {
    html.push_str(r#"<h2 id="file-index">File Index</h2>"#);
    html.push_str(
        r#"<input type="text" class="search-box" id="file-search" placeholder="Filter files..." onkeyup="filterFiles()">"#,
    );
    html.push_str(r#"<ul class="nav-list" id="file-list">"#);

    for (i, file) in data.files.iter().enumerate() {
        let short_name = shorten_path(&file.file, &data.base_path);
        html.push_str(&format!(
            r##"<li class="file-item" data-name="{}"><a href="#file-{}">{}</a> <span class="count">{} fn</span></li>"##,
            html_escape(&short_name.to_lowercase()),
            i,
            html_escape(&short_name),
            file.functions.len(),
        ));
    }

    html.push_str("</ul>\n");
}

fn write_function_table(html: &mut String, data: &ReportData) {
    html.push_str(r#"<h2 id="functions">Function Table</h2>"#);
    html.push_str(
        r#"<input type="text" class="search-box" id="func-search" placeholder="Filter functions..." onkeyup="filterFunctions()">"#,
    );
    html.push_str(r#"<div style="overflow-x:auto;"><table id="func-table">"#);
    html.push_str(r#"<thead><tr>"#);

    let headers = [
        ("Name", "name"),
        ("File", "file"),
        ("Line", "line"),
        ("Return", "return"),
        ("Type", "type"),
        ("Calls", "calls"),
        ("CalledBy", "calledby"),
    ];

    for (label, col) in &headers {
        html.push_str(&format!(
            r#"<th onclick="sortTable('func-table', '{col}')" data-col="{col}">{label}<span class="sort-indicator">&#x25B4;</span></th>"#,
        ));
    }

    html.push_str("</tr></thead>\n<tbody>\n");

    for file_data in &data.files {
        let short_file = shorten_path(&file_data.file, &data.base_path);
        for func in &file_data.functions {
            let type_badge = if func.is_async {
                r#"<span class="badge badge-async">async</span>"#
            } else if func.is_callback {
                r#"<span class="badge badge-callback">callback</span>"#
            } else {
                ""
            };

            html.push_str(&format!(
                r#"<tr data-name="{}" data-file="{}" data-line="{}" data-return="{}" data-type="{}" data-calls="{}" data-calledby="{}">
<td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>
</tr>
"#,
                html_escape(&func.name.to_lowercase()),
                html_escape(&short_file.to_lowercase()),
                func.line,
                html_escape(&func.return_type),
                if func.is_async {
                    "async"
                } else if func.is_callback {
                    "callback"
                } else {
                    "normal"
                },
                func.call_count,
                func.called_by_count,
                html_escape(&func.name),
                html_escape(&short_file),
                func.line,
                html_escape(&func.return_type),
                type_badge,
                func.call_count,
                func.called_by_count,
            ));
        }
    }

    html.push_str("</tbody></table></div>\n");
}

fn write_call_graph_summary(html: &mut String, data: &ReportData) {
    html.push_str(r#"<h2 id="call-graph">Call Graph Summary</h2>"#);

    // Collect caller/callee counts
    let mut caller_counts: HashMap<String, usize> = HashMap::new();
    let mut callee_counts: HashMap<String, usize> = HashMap::new();

    for file_data in &data.files {
        for edge in &file_data.call_edges {
            *caller_counts.entry(edge.caller.clone()).or_insert(0) += 1;
            *callee_counts.entry(edge.callee.clone()).or_insert(0) += 1;
        }
    }

    let mut top_callers: Vec<_> = caller_counts.into_iter().collect();
    top_callers.sort_by(|a, b| b.1.cmp(&a.1));
    top_callers.truncate(15);

    let mut top_callees: Vec<_> = callee_counts.into_iter().collect();
    top_callees.sort_by(|a, b| b.1.cmp(&a.1));
    top_callees.truncate(15);

    html.push_str(r#"<div class="top-callers">"#);

    // Top callers column
    html.push_str("<div>");
    html.push_str(r#"<h3>Top Callers (most outgoing calls)</h3>"#);
    html.push_str(r#"<table><thead><tr><th>Function</th><th>Calls</th></tr></thead><tbody>"#);
    for (name, count) in &top_callers {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>\n",
            html_escape(name),
            count,
        ));
    }
    html.push_str("</tbody></table></div>\n");

    // Top callees column
    html.push_str("<div>");
    html.push_str(r#"<h3>Top Callees (most called functions)</h3>"#);
    html.push_str(r#"<table><thead><tr><th>Function</th><th>Called</th></tr></thead><tbody>"#);
    for (name, count) in &top_callees {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>\n",
            html_escape(name),
            count,
        ));
    }
    html.push_str("</tbody></table></div>\n");

    html.push_str("</div>\n");
}

fn write_pattern_summary(html: &mut String, data: &ReportData) {
    html.push_str(r#"<h2 id="patterns">Pattern Summary</h2>"#);

    let mut total_locking: usize = 0;
    let mut total_error: usize = 0;
    let mut total_lifecycle: usize = 0;
    let mut total_async: usize = 0;
    let mut total_memory: usize = 0;
    let mut total_rcu: usize = 0;
    let mut total_issues: usize = 0;

    for file_data in &data.files {
        if let Some(ref pr) = file_data.pattern_report {
            total_locking += pr.locking.total;
            total_error += pr.error_handling.total;
            total_lifecycle += pr.lifecycle.total;
            total_async += pr.async_patterns.total;
            total_memory += pr.memory.total;
            total_rcu += pr.rcu.total;
            total_issues += pr.total_issues;
        }
    }

    let categories = [
        ("Locking", total_locking, "spinlock, mutex, rwlock, irqsave"),
        (
            "Error Handling",
            total_error,
            "goto cleanup, IS_ERR, errno",
        ),
        (
            "Lifecycle",
            total_lifecycle,
            "init/exit, probe/remove, alloc/free",
        ),
        (
            "Async",
            total_async,
            "work queues, IRQ, timers, completions",
        ),
        ("Memory", total_memory, "devm, GFP, DMA, barriers"),
        ("RCU", total_rcu, "read locks, dereference, synchronize"),
    ];

    html.push_str(r#"<div class="pattern-grid">"#);

    for (name, count, desc) in &categories {
        html.push_str(&format!(
            r#"<div class="pattern-card">
<div class="cat-name">{name}</div>
<div class="cat-count">{count}</div>
<div style="color:var(--text-dim);font-size:0.8em">{desc}</div>
</div>"#,
        ));
    }

    html.push_str("</div>\n");

    if total_issues > 0 {
        html.push_str(&format!(
            r#"<div style="margin-top:8px;color:var(--warning);font-size:0.9em">{} potential issues detected across all files</div>"#,
            total_issues,
        ));
    }
}

fn write_async_summary(html: &mut String, data: &ReportData) {
    html.push_str(r#"<h2 id="async-handlers">Async Handler Summary</h2>"#);

    // Group by mechanism type
    let mut by_mechanism: HashMap<String, Vec<&AsyncEntry>> = HashMap::new();

    for file_data in &data.files {
        for entry in &file_data.async_handlers {
            by_mechanism
                .entry(entry.mechanism.clone())
                .or_default()
                .push(entry);
        }
    }

    if by_mechanism.is_empty() {
        html.push_str(
            r#"<div style="color:var(--text-dim);padding:8px">No async handlers detected.</div>"#,
        );
        return;
    }

    let mut mechanisms: Vec<_> = by_mechanism.into_iter().collect();
    mechanisms.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (mechanism, entries) in &mechanisms {
        html.push_str(&format!(
            r#"<details><summary>{} <span class="badge badge-async">{} handlers</span></summary><div class="detail-content">"#,
            html_escape(mechanism),
            entries.len(),
        ));

        html.push_str(
            r#"<table><thead><tr><th>Handler</th><th>Variable</th><th>File</th><th>Line</th></tr></thead><tbody>"#,
        );

        for entry in entries {
            let short_file = shorten_path(&entry.file, &data.base_path);
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                html_escape(&entry.handler),
                html_escape(&entry.variable),
                html_escape(&short_file),
                entry.line,
            ));
        }

        html.push_str("</tbody></table></div></details>\n");
    }
}

fn write_per_file_details(html: &mut String, data: &ReportData) {
    html.push_str(r#"<h2 id="per-file">Per-File Details</h2>"#);

    for (i, file_data) in data.files.iter().enumerate() {
        let short_name = shorten_path(&file_data.file, &data.base_path);
        let fn_count = file_data.functions.len();
        let async_count = file_data.async_handlers.len();
        let cb_count = file_data
            .functions
            .iter()
            .filter(|f| f.is_callback)
            .count();

        html.push_str(&format!(
            r#"<details id="file-{}"><summary>{} <span class="badge badge-info">{} fn</span>"#,
            i,
            html_escape(&short_name),
            fn_count,
        ));

        if async_count > 0 {
            html.push_str(&format!(
                r#" <span class="badge badge-async">{} async</span>"#,
                async_count,
            ));
        }
        if cb_count > 0 {
            html.push_str(&format!(
                r#" <span class="badge badge-callback">{} cb</span>"#,
                cb_count,
            ));
        }

        html.push_str(r#"</summary><div class="detail-content">"#);

        // Functions table
        if !file_data.functions.is_empty() {
            html.push_str(r#"<h3>Functions</h3>"#);
            html.push_str(
                r#"<table><thead><tr><th>Name</th><th>Line</th><th>Return</th><th>Type</th><th>Calls</th><th>CalledBy</th></tr></thead><tbody>"#,
            );

            let mut sorted_funcs = file_data.functions.clone();
            sorted_funcs.sort_by_key(|f| f.line);

            for func in &sorted_funcs {
                let type_str = if func.is_async {
                    r#"<span class="badge badge-async">async</span>"#
                } else if func.is_callback {
                    r#"<span class="badge badge-callback">callback</span>"#
                } else {
                    ""
                };

                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    html_escape(&func.name),
                    func.line,
                    html_escape(&func.return_type),
                    type_str,
                    func.call_count,
                    func.called_by_count,
                ));
            }

            html.push_str("</tbody></table>\n");
        }

        // Async handlers
        if !file_data.async_handlers.is_empty() {
            html.push_str(r#"<h3>Async Handlers</h3>"#);
            html.push_str(
                r#"<table><thead><tr><th>Handler</th><th>Mechanism</th><th>Variable</th><th>Line</th></tr></thead><tbody>"#,
            );

            for entry in &file_data.async_handlers {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    html_escape(&entry.handler),
                    html_escape(&entry.mechanism),
                    html_escape(&entry.variable),
                    entry.line,
                ));
            }

            html.push_str("</tbody></table>\n");
        }

        // Pattern summary for this file
        if let Some(ref pr) = file_data.pattern_report {
            if pr.total_patterns > 0 {
                html.push_str(r#"<h3>Patterns</h3>"#);
                html.push_str(r#"<div class="pattern-grid">"#);

                let file_cats = [
                    ("Locking", pr.locking.total, pr.locking.issues.len()),
                    ("Error", pr.error_handling.total, pr.error_handling.issues.len()),
                    ("Lifecycle", pr.lifecycle.total, pr.lifecycle.issues.len()),
                    ("Async", pr.async_patterns.total, pr.async_patterns.issues.len()),
                    ("Memory", pr.memory.total, pr.memory.issues.len()),
                    ("RCU", pr.rcu.total, pr.rcu.issues.len()),
                ];

                for (name, count, issues) in &file_cats {
                    if *count > 0 {
                        html.push_str(&format!(
                            r#"<div class="pattern-card"><div class="cat-name">{name}</div><div class="cat-count">{count}</div>"#,
                        ));
                        if *issues > 0 {
                            html.push_str(&format!(
                                r#"<div class="cat-issues">{issues} issues</div>"#,
                            ));
                        }
                        html.push_str("</div>");
                    }
                }

                html.push_str("</div>\n");

                // Show quality score
                html.push_str(&format!(
                    r#"<div style="margin-top:6px;font-size:0.85em">Quality score: <span style="color:var(--success);font-weight:bold">{}%</span></div>"#,
                    pr.quality_pct,
                ));
            }
        }

        html.push_str("</div></details>\n");
    }
}

fn write_errors_section(html: &mut String, data: &ReportData) {
    if data.errors.is_empty() {
        return;
    }

    html.push_str(r#"<h2 id="errors">Errors</h2>"#);
    html.push_str(r#"<div class="error-list">"#);

    for err in &data.errors {
        html.push_str(&format!(
            r#"<div class="error-item">{}</div>"#,
            html_escape(err),
        ));
    }

    html.push_str("</div>\n");
}

fn write_javascript(html: &mut String) {
    html.push_str(
        r##"<script>
/* File index filtering */
function filterFiles() {
  var input = document.getElementById('file-search');
  var filter = input.value.toLowerCase();
  var items = document.querySelectorAll('.file-item');
  items.forEach(function(item) {
    var name = item.getAttribute('data-name') || '';
    item.style.display = name.indexOf(filter) !== -1 ? '' : 'none';
  });
}

/* Function table filtering */
function filterFunctions() {
  var input = document.getElementById('func-search');
  var filter = input.value.toLowerCase();
  var rows = document.querySelectorAll('#func-table tbody tr');
  rows.forEach(function(row) {
    var name = row.getAttribute('data-name') || '';
    var file = row.getAttribute('data-file') || '';
    var ret = row.getAttribute('data-return') || '';
    var tp = row.getAttribute('data-type') || '';
    var text = name + ' ' + file + ' ' + ret + ' ' + tp;
    row.style.display = text.indexOf(filter) !== -1 ? '' : 'none';
  });
}

/* Table sorting */
var sortState = {};

function sortTable(tableId, col) {
  var table = document.getElementById(tableId);
  if (!table) return;
  var tbody = table.querySelector('tbody');
  var rows = Array.from(tbody.querySelectorAll('tr'));
  var ascending = !(sortState[tableId + '-' + col] === true);
  sortState[tableId + '-' + col] = ascending;

  /* Reset sort indicators */
  table.querySelectorAll('th').forEach(function(th) {
    th.classList.remove('sorted');
    var ind = th.querySelector('.sort-indicator');
    if (ind) ind.innerHTML = '\u25B4';
  });

  /* Mark active header */
  var activeHeader = table.querySelector('th[data-col="' + col + '"]');
  if (activeHeader) {
    activeHeader.classList.add('sorted');
    var ind = activeHeader.querySelector('.sort-indicator');
    if (ind) ind.innerHTML = ascending ? '\u25B4' : '\u25BE';
  }

  rows.sort(function(a, b) {
    var va = a.getAttribute('data-' + col) || '';
    var vb = b.getAttribute('data-' + col) || '';
    /* Try numeric sort first */
    var na = parseFloat(va);
    var nb = parseFloat(vb);
    if (!isNaN(na) && !isNaN(nb)) {
      return ascending ? na - nb : nb - na;
    }
    /* Fall back to string sort */
    return ascending ? va.localeCompare(vb) : vb.localeCompare(va);
  });

  rows.forEach(function(row) { tbody.appendChild(row); });
}

/* Smooth scroll for anchor links */
document.querySelectorAll('a[href^="#"]').forEach(function(a) {
  a.addEventListener('click', function(e) {
    var target = document.querySelector(this.getAttribute('href'));
    if (target) {
      e.preventDefault();
      target.scrollIntoView({ behavior: 'smooth', block: 'start' });
      /* Open details if target is inside one */
      var details = target.closest('details');
      if (details) details.open = true;
    }
  });
});
</script>
"##,
    );
}

fn write_html_close(html: &mut String) {
    html.push_str("</div>\n</body>\n</html>\n");
}

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------

/// Shorten a file path relative to a base path
fn shorten_path(file: &str, base: &str) -> String {
    if let Some(stripped) = file.strip_prefix(base) {
        stripped.trim_start_matches('/').to_string()
    } else {
        file.to_string()
    }
}

/// Format a number with comma separators (e.g., 1234 -> "1,234")
fn format_count(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}
