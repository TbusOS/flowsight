//! `flowsight patterns` command - kernel pattern detection
//!
//! Detects common Linux kernel coding patterns using tree-sitter AST analysis:
//! - Locking patterns (spinlock, mutex, rwlock, irqsave)
//! - Error handling (goto cleanup, IS_ERR, return -ERRNO)
//! - Lifecycle patterns (module_init/exit, probe/remove, alloc/free)
//! - Async patterns (INIT_WORK, request_irq, completion)
//! - Memory patterns (devm_*, GFP flags, DMA, barriers)
//! - RCU patterns (rcu_read_lock/unlock, rcu_dereference)

use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Category filter for pattern detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum PatternCategory {
    Locking,
    Error,
    Lifecycle,
    Async,
    Memory,
    Rcu,
}

impl std::fmt::Display for PatternCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Locking => write!(f, "locking"),
            Self::Error => write!(f, "error"),
            Self::Lifecycle => write!(f, "lifecycle"),
            Self::Async => write!(f, "async"),
            Self::Memory => write!(f, "memory"),
            Self::Rcu => write!(f, "rcu"),
        }
    }
}

/// Options for the patterns command
pub struct PatternsOptions {
    pub recursive: bool,
    pub pattern_glob: String,
    pub category: Option<PatternCategory>,
    pub summary: bool,
}

// ---------------------------------------------------------------------------
// Pattern instance types (serializable for JSON output)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct PatternReport {
    pub file: String,
    pub locking: LockingReport,
    pub error_handling: ErrorReport,
    pub lifecycle: LifecycleReport,
    pub async_patterns: AsyncReport,
    pub memory: MemoryReport,
    pub rcu: RcuReport,
    pub total_patterns: usize,
    pub total_issues: usize,
    pub quality_pct: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct LockingReport {
    pub spin_lock_pairs: Vec<LockPair>,
    pub mutex_pairs: Vec<LockPair>,
    pub rwlock_pairs: Vec<LockPair>,
    pub irqsave_pairs: Vec<LockPair>,
    pub issues: Vec<PatternIssue>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LockPair {
    pub lock_fn: String,
    pub unlock_fn: String,
    pub function: String,
    pub lock_line: u32,
    pub unlock_line: Option<u32>,
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorReport {
    pub goto_cleanup: Vec<GotoInstance>,
    pub is_err_checks: Vec<CallInstance>,
    pub errno_returns: Vec<ErrnoReturn>,
    pub unchecked_calls: Vec<UncheckedCall>,
    pub issues: Vec<PatternIssue>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GotoInstance {
    pub label: String,
    pub function: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallInstance {
    pub call_name: String,
    pub function: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrnoReturn {
    pub errno: String,
    pub function: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UncheckedCall {
    pub call_name: String,
    pub function: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LifecycleReport {
    pub module_init_exit: Option<InitExitPair>,
    pub probe_remove: Vec<ProbeRemovePair>,
    pub register_unregister: Vec<RegisterPair>,
    pub alloc_free: AllocFreeSummary,
    pub issues: Vec<PatternIssue>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitExitPair {
    pub init_fn: String,
    pub exit_fn: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeRemovePair {
    pub probe_fn: String,
    pub remove_fn: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterPair {
    pub register_call: String,
    pub unregister_call: Option<String>,
    pub function: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllocFreeSummary {
    pub kzalloc_count: usize,
    pub kfree_count: usize,
    pub devm_count: usize,
    pub potential_leaks: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AsyncReport {
    pub work_items: Vec<AsyncInstance>,
    pub irq_handlers: Vec<AsyncInstance>,
    pub timers: Vec<AsyncInstance>,
    pub tasklets: Vec<AsyncInstance>,
    pub completions: Vec<AsyncInstance>,
    pub issues: Vec<PatternIssue>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AsyncInstance {
    pub init_call: String,
    pub handler: String,
    pub function: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryReport {
    pub devm_allocs: Vec<CallInstance>,
    pub kzalloc_calls: Vec<CallInstance>,
    pub kmalloc_calls: Vec<CallInstance>,
    pub dma_allocs: Vec<CallInstance>,
    pub gfp_kernel: usize,
    pub gfp_atomic: usize,
    pub barriers: Vec<CallInstance>,
    pub issues: Vec<PatternIssue>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RcuReport {
    pub read_sections: Vec<LockPair>,
    pub dereference_calls: Vec<CallInstance>,
    pub assign_pointer_calls: Vec<CallInstance>,
    pub synchronize_calls: Vec<CallInstance>,
    pub call_rcu_calls: Vec<CallInstance>,
    pub srcu_calls: Vec<CallInstance>,
    pub issues: Vec<PatternIssue>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub line: Option<u32>,
    pub function: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum IssueSeverity {
    Warning,
    Info,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run the patterns command on a single file or directory
pub fn run(
    path: &Path,
    format: &crate::output::OutputFormat,
    opts: &PatternsOptions,
) -> Result<()> {
    if path.is_dir() {
        if !opts.recursive {
            anyhow::bail!(
                "'{}' is a directory. Use --recursive (-r) to scan directories.",
                path.display()
            );
        }
        run_directory(path, format, opts)
    } else {
        run_single_file(path, format, opts)
    }
}

fn run_single_file(
    path: &Path,
    format: &crate::output::OutputFormat,
    opts: &PatternsOptions,
) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let report = detect_patterns(path, &source, opts.category)?;

    match format {
        crate::output::OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{json}");
        }
        _ => {
            if opts.summary {
                print_summary(&report);
            } else {
                print_full_report(&report, opts.category);
            }
        }
    }
    Ok(())
}

fn run_directory(
    dir: &Path,
    format: &crate::output::OutputFormat,
    opts: &PatternsOptions,
) -> Result<()> {
    let glob = globset::GlobBuilder::new(&opts.pattern_glob)
        .literal_separator(true)
        .build()
        .with_context(|| format!("Invalid glob pattern: {}", opts.pattern_glob))?
        .compile_matcher();

    let files: Vec<_> = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| glob.is_match(n))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    if files.is_empty() {
        anyhow::bail!(
            "No files matching '{}' found in '{}'",
            opts.pattern_glob,
            dir.display()
        );
    }

    let mut reports: Vec<PatternReport> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for file_path in &files {
        match std::fs::read_to_string(file_path) {
            Ok(source) => match detect_patterns(file_path, &source, opts.category) {
                Ok(report) => reports.push(report),
                Err(e) => errors.push(format!("{}: {}", file_path.display(), e)),
            },
            Err(e) => errors.push(format!("{}: {}", file_path.display(), e)),
        }
    }

    match format {
        crate::output::OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&reports)?;
            println!("{json}");
        }
        _ => {
            for report in &reports {
                if opts.summary {
                    print_summary(report);
                } else {
                    print_full_report(report, opts.category);
                }
                println!();
            }
            if !errors.is_empty() {
                let warn_color = Color::DarkYellow;
                println!("{}", "Warnings:".with(warn_color));
                for err in &errors {
                    println!("  {}", err.as_str().with(warn_color));
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Core detection using tree-sitter AST
// ---------------------------------------------------------------------------

/// Detect all kernel patterns in a source file using tree-sitter AST
pub fn detect_patterns(
    path: &Path,
    source: &str,
    category: Option<PatternCategory>,
) -> Result<PatternReport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_c::language())
        .map_err(|e| anyhow::anyhow!("Failed to load C grammar: {}", e))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("tree-sitter failed to parse {}", path.display()))?;

    let root = tree.root_node();

    // Collect per-function bodies for analysis
    let func_bodies = collect_function_bodies(root, source);

    // Collect all call expressions with line numbers across the whole file
    let all_calls = collect_all_calls(root, source);

    // Also gather goto statements
    let all_gotos = collect_goto_statements(root, source);

    // Gather return statements with errno
    let errno_returns = collect_errno_returns(root, source);

    let should_detect = |cat: PatternCategory| -> bool {
        category.map_or(true, |c| c == cat)
    };

    let locking = if should_detect(PatternCategory::Locking) {
        detect_locking(&func_bodies, &all_calls)
    } else {
        LockingReport {
            spin_lock_pairs: vec![],
            mutex_pairs: vec![],
            rwlock_pairs: vec![],
            irqsave_pairs: vec![],
            issues: vec![],
            total: 0,
        }
    };

    let error_handling = if should_detect(PatternCategory::Error) {
        detect_error_handling(&func_bodies, &all_calls, &all_gotos, &errno_returns, source)
    } else {
        ErrorReport {
            goto_cleanup: vec![],
            is_err_checks: vec![],
            errno_returns: vec![],
            unchecked_calls: vec![],
            issues: vec![],
            total: 0,
        }
    };

    let lifecycle = if should_detect(PatternCategory::Lifecycle) {
        detect_lifecycle(source, &func_bodies, &all_calls)
    } else {
        LifecycleReport {
            module_init_exit: None,
            probe_remove: vec![],
            register_unregister: vec![],
            alloc_free: AllocFreeSummary {
                kzalloc_count: 0,
                kfree_count: 0,
                devm_count: 0,
                potential_leaks: 0,
            },
            issues: vec![],
            total: 0,
        }
    };

    let async_patterns = if should_detect(PatternCategory::Async) {
        detect_async(&all_calls)
    } else {
        AsyncReport {
            work_items: vec![],
            irq_handlers: vec![],
            timers: vec![],
            tasklets: vec![],
            completions: vec![],
            issues: vec![],
            total: 0,
        }
    };

    let memory = if should_detect(PatternCategory::Memory) {
        detect_memory(&all_calls, source)
    } else {
        MemoryReport {
            devm_allocs: vec![],
            kzalloc_calls: vec![],
            kmalloc_calls: vec![],
            dma_allocs: vec![],
            gfp_kernel: 0,
            gfp_atomic: 0,
            barriers: vec![],
            issues: vec![],
            total: 0,
        }
    };

    let rcu = if should_detect(PatternCategory::Rcu) {
        detect_rcu(&func_bodies, &all_calls)
    } else {
        RcuReport {
            read_sections: vec![],
            dereference_calls: vec![],
            assign_pointer_calls: vec![],
            synchronize_calls: vec![],
            call_rcu_calls: vec![],
            srcu_calls: vec![],
            issues: vec![],
            total: 0,
        }
    };

    let total_patterns = locking.total
        + error_handling.total
        + lifecycle.total
        + async_patterns.total
        + memory.total
        + rcu.total;

    let total_issues = locking.issues.len()
        + error_handling.issues.len()
        + lifecycle.issues.len()
        + async_patterns.issues.len()
        + memory.issues.len()
        + rcu.issues.len();

    let quality_pct = if total_patterns == 0 {
        100
    } else {
        let issue_ratio = total_issues as f64 / total_patterns as f64;
        ((1.0 - issue_ratio.min(1.0)) * 100.0) as u8
    };

    Ok(PatternReport {
        file: path.to_string_lossy().into_owned(),
        locking,
        error_handling,
        lifecycle,
        async_patterns,
        memory,
        rcu,
        total_patterns,
        total_issues,
        quality_pct,
    })
}

// ---------------------------------------------------------------------------
// AST helpers
// ---------------------------------------------------------------------------

/// Information about a function body for per-function analysis
struct FunctionBody {
    name: String,
    /// All call expressions within this function: (call_name, line)
    calls: Vec<(String, u32)>,
    /// Start and end lines
    start_line: u32,
    end_line: u32,
}

/// Walk AST and collect function bodies with their internal calls
fn collect_function_bodies(root: tree_sitter::Node, source: &str) -> Vec<FunctionBody> {
    let mut bodies = Vec::new();
    collect_function_bodies_recursive(root, source, &mut bodies);
    bodies
}

fn collect_function_bodies_recursive(
    node: tree_sitter::Node,
    source: &str,
    bodies: &mut Vec<FunctionBody>,
) {
    if node.kind() == "function_definition" {
        if let Some(fb) = extract_function_body(node, source) {
            bodies.push(fb);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_function_bodies_recursive(child, source, bodies);
    }
}

fn extract_function_body(node: tree_sitter::Node, source: &str) -> Option<FunctionBody> {
    let name = extract_func_name_from_def(node, source)?;
    let start_line = node.start_position().row as u32 + 1;
    let end_line = node.end_position().row as u32 + 1;

    let mut calls = Vec::new();
    // Find compound_statement
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "compound_statement" {
            collect_calls_recursive(child, source, &mut calls);
        }
    }

    Some(FunctionBody {
        name,
        calls,
        start_line,
        end_line,
    })
}

fn extract_func_name_from_def(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "function_declarator" | "pointer_declarator" => {
                if let Some(n) = extract_identifier_deep(child, source) {
                    return Some(n);
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_identifier_deep(node: tree_sitter::Node, source: &str) -> Option<String> {
    if node.kind() == "identifier" {
        return Some(node_text(node, source));
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => return Some(node_text(child, source)),
            "function_declarator" | "pointer_declarator" => {
                if let Some(n) = extract_identifier_deep(child, source) {
                    return Some(n);
                }
            }
            _ => {}
        }
    }
    None
}

/// Collect all call expressions with their line numbers from a subtree
fn collect_calls_recursive(
    node: tree_sitter::Node,
    source: &str,
    calls: &mut Vec<(String, u32)>,
) {
    if node.kind() == "call_expression" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name = node_text(child, source);
                let line = node.start_position().row as u32 + 1;
                calls.push((name, line));
                break;
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_calls_recursive(child, source, calls);
    }
}

/// A call across the entire file: (function_name_containing, call_name, line)
struct FileCall {
    enclosing_fn: String,
    call_name: String,
    line: u32,
}

fn collect_all_calls(root: tree_sitter::Node, source: &str) -> Vec<FileCall> {
    let bodies = collect_function_bodies(root, source);
    let mut result = Vec::new();
    for body in &bodies {
        for (call_name, line) in &body.calls {
            result.push(FileCall {
                enclosing_fn: body.name.clone(),
                call_name: call_name.clone(),
                line: *line,
            });
        }
    }
    result
}

/// Collect goto statements: (label, enclosing_function, line)
struct GotoStmt {
    label: String,
    function: String,
    line: u32,
}

fn collect_goto_statements(root: tree_sitter::Node, source: &str) -> Vec<GotoStmt> {
    let bodies = collect_function_bodies(root, source);
    let mut gotos = Vec::new();
    for body in &bodies {
        collect_gotos_in_subtree_by_lines(root, source, &body.name, body.start_line, body.end_line, &mut gotos);
    }
    gotos
}

fn collect_gotos_in_subtree_by_lines(
    node: tree_sitter::Node,
    source: &str,
    fn_name: &str,
    start_line: u32,
    end_line: u32,
    gotos: &mut Vec<GotoStmt>,
) {
    if node.kind() == "goto_statement" {
        let line = node.start_position().row as u32 + 1;
        if line >= start_line && line <= end_line {
            // Extract the label identifier
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" || child.kind() == "statement_identifier" {
                    gotos.push(GotoStmt {
                        label: node_text(child, source),
                        function: fn_name.to_string(),
                        line,
                    });
                    break;
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_gotos_in_subtree_by_lines(child, source, fn_name, start_line, end_line, gotos);
    }
}

/// Collect return statements with -ERRNO values
fn collect_errno_returns(root: tree_sitter::Node, source: &str) -> Vec<(String, String, u32)> {
    let re = regex::Regex::new(r"return\s+(-E[A-Z]+)").ok();
    let bodies = collect_function_bodies(root, source);
    let mut results = Vec::new();

    if let Some(re) = re {
        for body in &bodies {
            // Scan source lines within this function
            for (i, line_text) in source.lines().enumerate() {
                let line_num = i as u32 + 1;
                if line_num >= body.start_line && line_num <= body.end_line {
                    if let Some(cap) = re.captures(line_text) {
                        if let Some(m) = cap.get(1) {
                            results.push((
                                body.name.clone(),
                                m.as_str().to_string(),
                                line_num,
                            ));
                        }
                    }
                }
            }
        }
    }
    results
}

fn node_text(node: tree_sitter::Node, source: &str) -> String {
    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
}

// ---------------------------------------------------------------------------
// Locking detection
// ---------------------------------------------------------------------------

fn detect_locking(func_bodies: &[FunctionBody], all_calls: &[FileCall]) -> LockingReport {
    let lock_pairs_def: &[(&str, &str)] = &[
        ("spin_lock", "spin_unlock"),
        ("spin_lock_bh", "spin_unlock_bh"),
    ];
    let mutex_pairs_def: &[(&str, &str)] = &[
        ("mutex_lock", "mutex_unlock"),
        ("mutex_lock_interruptible", "mutex_unlock"),
        ("mutex_lock_killable", "mutex_unlock"),
    ];
    let rwlock_pairs_def: &[(&str, &str)] = &[
        ("read_lock", "read_unlock"),
        ("write_lock", "write_unlock"),
        ("read_lock_bh", "read_unlock_bh"),
        ("write_lock_bh", "write_unlock_bh"),
    ];
    let irqsave_pairs_def: &[(&str, &str)] = &[
        ("spin_lock_irqsave", "spin_unlock_irqrestore"),
        ("spin_lock_irq", "spin_unlock_irq"),
        ("read_lock_irqsave", "read_unlock_irqrestore"),
        ("write_lock_irqsave", "write_unlock_irqrestore"),
    ];

    let spin_lock_pairs = find_lock_pairs(func_bodies, lock_pairs_def);
    let mutex_pairs = find_lock_pairs(func_bodies, mutex_pairs_def);
    let rwlock_pairs = find_lock_pairs(func_bodies, rwlock_pairs_def);
    let irqsave_pairs = find_lock_pairs(func_bodies, irqsave_pairs_def);

    let mut issues = Vec::new();

    // Check for unmatched locks
    for pair in spin_lock_pairs
        .iter()
        .chain(mutex_pairs.iter())
        .chain(rwlock_pairs.iter())
        .chain(irqsave_pairs.iter())
    {
        if !pair.matched {
            issues.push(PatternIssue {
                severity: IssueSeverity::Warning,
                message: format!(
                    "{} without matching {} on error path",
                    pair.lock_fn, pair.unlock_fn
                ),
                line: Some(pair.lock_line),
                function: Some(pair.function.clone()),
            });
        }
    }

    // Detect nested locks (mutex inside spinlock)
    detect_nested_lock_issues(func_bodies, all_calls, &mut issues);

    let total = spin_lock_pairs.len()
        + mutex_pairs.len()
        + rwlock_pairs.len()
        + irqsave_pairs.len();

    LockingReport {
        spin_lock_pairs,
        mutex_pairs,
        rwlock_pairs,
        irqsave_pairs,
        issues,
        total,
    }
}

fn find_lock_pairs(
    func_bodies: &[FunctionBody],
    pair_defs: &[(&str, &str)],
) -> Vec<LockPair> {
    let mut pairs = Vec::new();

    for body in func_bodies {
        for &(lock_name, unlock_name) in pair_defs {
            let locks: Vec<_> = body
                .calls
                .iter()
                .filter(|(name, _)| name == lock_name)
                .collect();
            let unlocks: Vec<_> = body
                .calls
                .iter()
                .filter(|(name, _)| name == unlock_name)
                .collect();

            for (i, &(ref _ln, lock_line)) in locks.iter().enumerate() {
                let unlock_line = unlocks.get(i).map(|(_, line)| *line);
                pairs.push(LockPair {
                    lock_fn: lock_name.to_string(),
                    unlock_fn: unlock_name.to_string(),
                    function: body.name.clone(),
                    lock_line: *lock_line,
                    unlock_line,
                    matched: unlock_line.is_some(),
                });
            }
        }
    }
    pairs
}

fn detect_nested_lock_issues(
    func_bodies: &[FunctionBody],
    _all_calls: &[FileCall],
    issues: &mut Vec<PatternIssue>,
) {
    let spin_names: &[&str] = &[
        "spin_lock",
        "spin_lock_bh",
        "spin_lock_irqsave",
        "spin_lock_irq",
    ];
    let mutex_names: &[&str] = &[
        "mutex_lock",
        "mutex_lock_interruptible",
        "mutex_lock_killable",
    ];

    for body in func_bodies {
        let mut in_spinlock = false;
        for (call_name, line) in &body.calls {
            if spin_names.contains(&call_name.as_str()) {
                in_spinlock = true;
            }
            if call_name == "spin_unlock"
                || call_name == "spin_unlock_bh"
                || call_name == "spin_unlock_irqrestore"
                || call_name == "spin_unlock_irq"
            {
                in_spinlock = false;
            }
            if in_spinlock && mutex_names.contains(&call_name.as_str()) {
                issues.push(PatternIssue {
                    severity: IssueSeverity::Warning,
                    message: format!(
                        "Nested locks: {} inside spinlock (review needed)",
                        call_name
                    ),
                    line: Some(*line),
                    function: Some(body.name.clone()),
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Error handling detection
// ---------------------------------------------------------------------------

fn detect_error_handling(
    func_bodies: &[FunctionBody],
    all_calls: &[FileCall],
    all_gotos: &[GotoStmt],
    errno_returns: &[(String, String, u32)],
    source: &str,
) -> ErrorReport {
    // goto cleanup / goto err_*
    let goto_cleanup: Vec<GotoInstance> = all_gotos
        .iter()
        .filter(|g| {
            let l = g.label.as_str();
            l.starts_with("err")
                || l.starts_with("fail")
                || l.starts_with("out")
                || l.starts_with("cleanup")
                || l.starts_with("exit")
                || l.starts_with("bail")
                || l.starts_with("unlock")
                || l.starts_with("free")
        })
        .map(|g| GotoInstance {
            label: g.label.clone(),
            function: g.function.clone(),
            line: g.line,
        })
        .collect();

    // IS_ERR / PTR_ERR checks
    let is_err_checks: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| c.call_name == "IS_ERR" || c.call_name == "PTR_ERR" || c.call_name == "IS_ERR_OR_NULL")
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    // -ERRNO returns
    let errnos: Vec<ErrnoReturn> = errno_returns
        .iter()
        .map(|(func, errno, line)| ErrnoReturn {
            errno: errno.clone(),
            function: func.clone(),
            line: *line,
        })
        .collect();

    // Unchecked return values of common kernel functions that return pointers/errors
    let check_fns: &[&str] = &[
        "platform_get_resource",
        "devm_ioremap_resource",
        "devm_ioremap",
        "clk_get",
        "devm_clk_get",
        "devm_regulator_get",
        "request_mem_region",
        "ioremap",
        "kmalloc",
        "kzalloc",
        "devm_kzalloc",
    ];
    let unchecked_calls = find_unchecked_calls(func_bodies, check_fns, source);

    let mut issues = Vec::new();
    for uc in &unchecked_calls {
        issues.push(PatternIssue {
            severity: IssueSeverity::Warning,
            message: format!("{}() return value not checked", uc.call_name),
            line: Some(uc.line),
            function: Some(uc.function.clone()),
        });
    }

    let total = goto_cleanup.len() + is_err_checks.len() + errnos.len();

    ErrorReport {
        goto_cleanup,
        is_err_checks,
        errno_returns: errnos,
        unchecked_calls,
        issues,
        total,
    }
}

/// Detect calls to functions that return error-indicating values where the return
/// value is not assigned to a variable (heuristic: the call is a standalone
/// expression_statement rather than part of an assignment or condition).
fn find_unchecked_calls(
    func_bodies: &[FunctionBody],
    check_fns: &[&str],
    source: &str,
) -> Vec<UncheckedCall> {
    let mut results = Vec::new();

    // Heuristic: for each call to a checkable function, look at the source line.
    // If the line does NOT contain '=' or 'if' or 'IS_ERR' before the call,
    // it's likely unchecked.
    for body in func_bodies {
        for (call_name, line) in &body.calls {
            if !check_fns.contains(&call_name.as_str()) {
                continue;
            }
            // Get the source line
            if let Some(line_text) = source.lines().nth((*line as usize).saturating_sub(1)) {
                let trimmed = line_text.trim();
                let has_assignment = trimmed.contains('=') && !trimmed.contains("==");
                let has_if = trimmed.starts_with("if");
                let has_err_check = trimmed.contains("IS_ERR");
                if !has_assignment && !has_if && !has_err_check {
                    results.push(UncheckedCall {
                        call_name: call_name.clone(),
                        function: body.name.clone(),
                        line: *line,
                    });
                }
            }
        }
    }
    results
}

// ---------------------------------------------------------------------------
// Lifecycle detection
// ---------------------------------------------------------------------------

fn detect_lifecycle(
    source: &str,
    func_bodies: &[FunctionBody],
    all_calls: &[FileCall],
) -> LifecycleReport {
    let mut issues = Vec::new();

    // module_init / module_exit
    let init_re = regex::Regex::new(r"module_init\s*\(\s*(\w+)\s*\)").ok();
    let exit_re = regex::Regex::new(r"module_exit\s*\(\s*(\w+)\s*\)").ok();

    let init_fn = init_re
        .as_ref()
        .and_then(|re| re.captures(source))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());
    let exit_fn = exit_re
        .as_ref()
        .and_then(|re| re.captures(source))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let module_init_exit = init_fn.map(|init| {
        if exit_fn.is_none() {
            issues.push(PatternIssue {
                severity: IssueSeverity::Info,
                message: "module_init without module_exit (built-in module?)".to_string(),
                line: None,
                function: None,
            });
        }
        InitExitPair {
            init_fn: init,
            exit_fn: exit_fn.clone(),
        }
    });

    // probe / remove pairs (from ops tables or function names)
    let probe_remove = detect_probe_remove(func_bodies);

    // register / unregister calls
    let register_unregister = detect_register_unregister(all_calls);

    // alloc / free
    let alloc_free = detect_alloc_free(all_calls, &mut issues);

    let total = module_init_exit.as_ref().map_or(0, |_| 1)
        + probe_remove.len()
        + register_unregister.len()
        + alloc_free.kzalloc_count
        + alloc_free.devm_count;

    LifecycleReport {
        module_init_exit,
        probe_remove,
        register_unregister,
        alloc_free,
        issues,
        total,
    }
}

fn detect_probe_remove(func_bodies: &[FunctionBody]) -> Vec<ProbeRemovePair> {
    let mut pairs = Vec::new();
    let probe_fns: Vec<_> = func_bodies
        .iter()
        .filter(|b| {
            b.name.contains("probe")
                || b.name.contains("_init")
                || b.name.contains("_attach")
        })
        .collect();
    let remove_fns: Vec<_> = func_bodies
        .iter()
        .filter(|b| {
            b.name.contains("remove")
                || b.name.contains("_exit")
                || b.name.contains("_detach")
                || b.name.contains("disconnect")
        })
        .collect();

    for probe in &probe_fns {
        // Try to find matching remove by prefix
        let prefix = probe
            .name
            .split("probe")
            .next()
            .or_else(|| probe.name.split("_init").next())
            .or_else(|| probe.name.split("_attach").next())
            .unwrap_or("");

        let matching_remove = remove_fns
            .iter()
            .find(|r| !prefix.is_empty() && r.name.starts_with(prefix))
            .map(|r| r.name.clone());

        pairs.push(ProbeRemovePair {
            probe_fn: probe.name.clone(),
            remove_fn: matching_remove,
        });
    }
    pairs
}

fn detect_register_unregister(all_calls: &[FileCall]) -> Vec<RegisterPair> {
    let register_re = regex::Regex::new(r"^(\w+_)?register(_\w+)?$").ok();
    let mut pairs = Vec::new();
    let mut seen_registers: Vec<String> = Vec::new();

    for call in all_calls {
        if let Some(ref re) = register_re {
            if re.is_match(&call.call_name)
                && !call.call_name.starts_with("un")
                && !call.call_name.starts_with("de")
                && call.call_name.contains("register")
            {
                if seen_registers.contains(&call.call_name) {
                    continue;
                }
                seen_registers.push(call.call_name.clone());

                // Look for matching unregister
                let unregister_name = if call.call_name.starts_with("register") {
                    format!("un{}", call.call_name)
                } else {
                    call.call_name.replace("register", "unregister")
                };

                let has_unregister = all_calls
                    .iter()
                    .any(|c| c.call_name == unregister_name);

                pairs.push(RegisterPair {
                    register_call: call.call_name.clone(),
                    unregister_call: if has_unregister {
                        Some(unregister_name)
                    } else {
                        None
                    },
                    function: call.enclosing_fn.clone(),
                    line: call.line,
                });
            }
        }
    }
    pairs
}

fn detect_alloc_free(
    all_calls: &[FileCall],
    issues: &mut Vec<PatternIssue>,
) -> AllocFreeSummary {
    let kzalloc_count = all_calls
        .iter()
        .filter(|c| c.call_name == "kzalloc" || c.call_name == "kmalloc")
        .count();
    let kfree_count = all_calls
        .iter()
        .filter(|c| c.call_name == "kfree")
        .count();
    let devm_count = all_calls
        .iter()
        .filter(|c| c.call_name.starts_with("devm_"))
        .count();

    // Non-devm allocs without matching frees
    let non_devm_allocs = all_calls
        .iter()
        .filter(|c| c.call_name == "kzalloc" || c.call_name == "kmalloc")
        .count();

    let potential_leaks = if non_devm_allocs > kfree_count {
        let leak_count = non_devm_allocs - kfree_count;
        if leak_count > 0 {
            issues.push(PatternIssue {
                severity: IssueSeverity::Warning,
                message: format!(
                    "{} potential memory leak(s): {} alloc(s) vs {} kfree(s)",
                    leak_count, non_devm_allocs, kfree_count
                ),
                line: None,
                function: None,
            });
        }
        leak_count
    } else {
        0
    };

    AllocFreeSummary {
        kzalloc_count,
        kfree_count,
        devm_count,
        potential_leaks,
    }
}

// ---------------------------------------------------------------------------
// Async detection
// ---------------------------------------------------------------------------

fn detect_async(all_calls: &[FileCall]) -> AsyncReport {
    let work_items: Vec<AsyncInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "INIT_WORK"
                || c.call_name == "INIT_DELAYED_WORK"
                || c.call_name == "DECLARE_WORK"
        })
        .map(|c| AsyncInstance {
            init_call: c.call_name.clone(),
            handler: String::new(), // Would need arg parsing for exact handler
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let irq_handlers: Vec<AsyncInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "request_irq"
                || c.call_name == "request_threaded_irq"
                || c.call_name == "devm_request_irq"
                || c.call_name == "devm_request_threaded_irq"
        })
        .map(|c| AsyncInstance {
            init_call: c.call_name.clone(),
            handler: String::new(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let timers: Vec<AsyncInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "setup_timer"
                || c.call_name == "timer_setup"
                || c.call_name == "mod_timer"
                || c.call_name == "add_timer"
                || c.call_name == "hrtimer_init"
        })
        .map(|c| AsyncInstance {
            init_call: c.call_name.clone(),
            handler: String::new(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let tasklets: Vec<AsyncInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "tasklet_init"
                || c.call_name == "tasklet_schedule"
                || c.call_name == "tasklet_hi_schedule"
        })
        .map(|c| AsyncInstance {
            init_call: c.call_name.clone(),
            handler: String::new(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let completions: Vec<AsyncInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "init_completion"
                || c.call_name == "reinit_completion"
                || c.call_name == "wait_for_completion"
                || c.call_name == "wait_for_completion_timeout"
                || c.call_name == "complete"
                || c.call_name == "complete_all"
        })
        .map(|c| AsyncInstance {
            init_call: c.call_name.clone(),
            handler: String::new(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let total = work_items.len()
        + irq_handlers.len()
        + timers.len()
        + tasklets.len()
        + completions.len();

    AsyncReport {
        work_items,
        irq_handlers,
        timers,
        tasklets,
        completions,
        issues: vec![],
        total,
    }
}

// ---------------------------------------------------------------------------
// Memory detection
// ---------------------------------------------------------------------------

fn detect_memory(all_calls: &[FileCall], source: &str) -> MemoryReport {
    let devm_allocs: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| c.call_name.starts_with("devm_"))
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let kzalloc_calls: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| c.call_name == "kzalloc" || c.call_name == "kcalloc")
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let kmalloc_calls: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| c.call_name == "kmalloc" || c.call_name == "krealloc")
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let dma_allocs: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "dma_alloc_coherent"
                || c.call_name == "dma_alloc_attrs"
                || c.call_name == "dma_pool_alloc"
        })
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    // Count GFP flags by scanning source text
    let gfp_kernel = source.matches("GFP_KERNEL").count();
    let gfp_atomic = source.matches("GFP_ATOMIC").count()
        + source.matches("GFP_NOWAIT").count();

    let barrier_names: &[&str] = &[
        "wmb", "rmb", "mb", "smp_wmb", "smp_rmb", "smp_mb",
        "smp_store_release", "smp_load_acquire",
        "dma_wmb", "dma_rmb",
    ];
    let barriers: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| barrier_names.contains(&c.call_name.as_str()))
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let mut issues = Vec::new();
    // Check GFP_KERNEL in potential atomic context (heuristic: inside spin_lock)
    // This is a rough heuristic; a proper check would require control flow analysis
    if gfp_kernel > 0 && gfp_atomic > 0 {
        issues.push(PatternIssue {
            severity: IssueSeverity::Info,
            message: format!(
                "Mixed GFP flags: {} GFP_KERNEL + {} GFP_ATOMIC (verify context correctness)",
                gfp_kernel, gfp_atomic
            ),
            line: None,
            function: None,
        });
    }

    let total = devm_allocs.len()
        + kzalloc_calls.len()
        + kmalloc_calls.len()
        + dma_allocs.len()
        + barriers.len();

    MemoryReport {
        devm_allocs,
        kzalloc_calls,
        kmalloc_calls,
        dma_allocs,
        gfp_kernel,
        gfp_atomic,
        barriers,
        issues,
        total,
    }
}

// ---------------------------------------------------------------------------
// RCU detection
// ---------------------------------------------------------------------------

fn detect_rcu(func_bodies: &[FunctionBody], all_calls: &[FileCall]) -> RcuReport {
    // rcu_read_lock / rcu_read_unlock pairs
    let rcu_pair_defs: &[(&str, &str)] = &[
        ("rcu_read_lock", "rcu_read_unlock"),
        ("rcu_read_lock_bh", "rcu_read_unlock_bh"),
        ("rcu_read_lock_sched", "rcu_read_unlock_sched"),
    ];
    let read_sections = find_lock_pairs(func_bodies, rcu_pair_defs);

    let dereference_calls: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "rcu_dereference"
                || c.call_name == "rcu_dereference_bh"
                || c.call_name == "rcu_dereference_sched"
                || c.call_name == "rcu_dereference_protected"
                || c.call_name == "rcu_dereference_check"
        })
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let assign_pointer_calls: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "rcu_assign_pointer"
                || c.call_name == "RCU_INIT_POINTER"
        })
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let synchronize_calls: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name == "synchronize_rcu"
                || c.call_name == "synchronize_rcu_expedited"
        })
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let call_rcu_calls: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| c.call_name == "call_rcu" || c.call_name == "call_rcu_bh")
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let srcu_calls: Vec<CallInstance> = all_calls
        .iter()
        .filter(|c| {
            c.call_name.starts_with("srcu_")
                || c.call_name == "init_srcu_struct"
                || c.call_name == "cleanup_srcu_struct"
                || c.call_name == "synchronize_srcu"
        })
        .map(|c| CallInstance {
            call_name: c.call_name.clone(),
            function: c.enclosing_fn.clone(),
            line: c.line,
        })
        .collect();

    let mut issues = Vec::new();
    // Check for rcu_dereference outside rcu_read_lock section (rough heuristic)
    for deref in &dereference_calls {
        let in_section = read_sections
            .iter()
            .any(|s| s.function == deref.function && s.matched);
        if !in_section && !read_sections.iter().any(|s| s.function == deref.function) {
            issues.push(PatternIssue {
                severity: IssueSeverity::Warning,
                message: format!(
                    "{} outside rcu_read_lock section",
                    deref.call_name
                ),
                line: Some(deref.line),
                function: Some(deref.function.clone()),
            });
        }
    }

    let total = read_sections.len()
        + dereference_calls.len()
        + assign_pointer_calls.len()
        + synchronize_calls.len()
        + call_rcu_calls.len()
        + srcu_calls.len();

    RcuReport {
        read_sections,
        dereference_calls,
        assign_pointer_calls,
        synchronize_calls,
        call_rcu_calls,
        srcu_calls,
        issues,
        total,
    }
}

// ---------------------------------------------------------------------------
// Text output
// ---------------------------------------------------------------------------

fn print_full_report(report: &PatternReport, category: Option<PatternCategory>) {
    let label = Color::DarkCyan;
    let dim = Color::DarkGrey;
    let ok_color = Color::DarkGreen;
    let warn_color = Color::DarkYellow;
    let value = Color::Grey;

    // Header
    let title = format!("FlowSight Pattern Analysis: {}", report.file);
    println!("{}", title.as_str().with(label));
    println!(
        "{}",
        "\u{2550}".repeat(title.len().min(72)).as_str().with(label)
    );
    println!();

    let should_show = |cat: PatternCategory| -> bool {
        category.map_or(true, |c| c == cat)
    };

    // Locking
    if should_show(PatternCategory::Locking) && report.locking.total > 0 {
        println!(
            "{} ({} instances)",
            "Locking".with(label),
            report.locking.total.to_string().as_str().with(value)
        );
        println!(
            "{}",
            "\u{2500}".repeat(24).as_str().with(dim)
        );
        print_lock_group("spin_lock/unlock", &report.locking.spin_lock_pairs, ok_color, value);
        print_lock_group("mutex_lock/unlock", &report.locking.mutex_pairs, ok_color, value);
        print_lock_group("rwlock", &report.locking.rwlock_pairs, ok_color, value);
        print_lock_group("irqsave/restore", &report.locking.irqsave_pairs, ok_color, value);
        print_issues(&report.locking.issues, warn_color);
        println!();
    }

    // Error handling
    if should_show(PatternCategory::Error) && report.error_handling.total > 0 {
        println!(
            "{} ({} instances)",
            "Error Handling".with(label),
            report.error_handling.total.to_string().as_str().with(value)
        );
        println!(
            "{}",
            "\u{2500}".repeat(30).as_str().with(dim)
        );
        if !report.error_handling.goto_cleanup.is_empty() {
            println!(
                "  {:<24} {} uses (standard pattern {})",
                "goto cleanup:".with(value),
                report.error_handling.goto_cleanup.len().to_string().as_str().with(value),
                "\u{2713}".with(ok_color)
            );
        }
        if !report.error_handling.is_err_checks.is_empty() {
            println!(
                "  {:<24} {} uses",
                "IS_ERR checks:".with(value),
                report.error_handling.is_err_checks.len().to_string().as_str().with(value)
            );
        }
        if !report.error_handling.errno_returns.is_empty() {
            // Group by errno
            let mut errno_counts: HashMap<String, usize> = HashMap::new();
            for er in &report.error_handling.errno_returns {
                *errno_counts.entry(er.errno.clone()).or_insert(0) += 1;
            }
            let mut sorted_errnos: Vec<_> = errno_counts.into_iter().collect();
            sorted_errnos.sort_by(|a, b| b.1.cmp(&a.1));
            let summary: Vec<String> = sorted_errnos
                .iter()
                .take(5)
                .map(|(errno, count)| format!("{}({})", errno, count))
                .collect();
            println!(
                "  {:<24} {} uses [{}]",
                "Return -ERRNO:".with(value),
                report.error_handling.errno_returns.len().to_string().as_str().with(value),
                summary.join(", ").as_str().with(dim)
            );
        }
        print_issues(&report.error_handling.issues, warn_color);
        println!();
    }

    // Lifecycle
    if should_show(PatternCategory::Lifecycle) && report.lifecycle.total > 0 {
        println!(
            "{} ({} pairs)",
            "Lifecycle".with(label),
            report.lifecycle.total.to_string().as_str().with(value)
        );
        println!(
            "{}",
            "\u{2500}".repeat(22).as_str().with(dim)
        );
        if let Some(ref ie) = report.lifecycle.module_init_exit {
            let exit_str = ie
                .exit_fn
                .as_deref()
                .unwrap_or("(none)");
            let check = if ie.exit_fn.is_some() {
                format!("{}", "\u{2713}".with(ok_color))
            } else {
                format!("{}", "\u{26a0}".with(warn_color))
            };
            println!(
                "  module_init -> {:<20} {} module_exit -> {:<20} {}",
                ie.init_fn.as_str().with(value),
                "\u{2194}".with(dim),
                exit_str.with(value),
                check
            );
        }
        for pr in &report.lifecycle.probe_remove {
            let remove_str = pr
                .remove_fn
                .as_deref()
                .unwrap_or("(none)");
            let check = if pr.remove_fn.is_some() {
                format!("{}", "\u{2713}".with(ok_color))
            } else {
                format!("{}", "\u{26a0}".with(warn_color))
            };
            println!(
                "  probe -> {:<24} {} remove -> {:<24} {}",
                pr.probe_fn.as_str().with(value),
                "\u{2194}".with(dim),
                remove_str.with(value),
                check
            );
        }

        let af = &report.lifecycle.alloc_free;
        if af.kzalloc_count > 0 || af.devm_count > 0 {
            if af.kzalloc_count > 0 {
                let check = if af.potential_leaks == 0 {
                    format!("matched {}", "\u{2713}".with(ok_color))
                } else {
                    format!(
                        "{} {} potential leak(s)",
                        "\u{26a0}".with(warn_color),
                        af.potential_leaks
                    )
                };
                println!(
                    "  alloc: {} kzalloc/kmalloc       {} free: {} kfree  {}",
                    af.kzalloc_count.to_string().as_str().with(value),
                    "\u{2194}".with(dim),
                    af.kfree_count.to_string().as_str().with(value),
                    check
                );
            }
            if af.devm_count > 0 {
                println!(
                    "  devm_* managed: {} allocations  (auto-cleanup {})",
                    af.devm_count.to_string().as_str().with(value),
                    "\u{2713}".with(ok_color)
                );
            }
        }

        for rp in &report.lifecycle.register_unregister {
            let unreg_str = rp
                .unregister_call
                .as_deref()
                .unwrap_or("(none)");
            let check = if rp.unregister_call.is_some() {
                format!("{}", "\u{2713}".with(ok_color))
            } else {
                format!("{}", "\u{26a0}".with(warn_color))
            };
            println!(
                "  {} {} {}  {}",
                rp.register_call.as_str().with(value),
                "\u{2194}".with(dim),
                unreg_str.with(value),
                check
            );
        }

        print_issues(&report.lifecycle.issues, warn_color);
        println!();
    }

    // Async
    if should_show(PatternCategory::Async) && report.async_patterns.total > 0 {
        println!(
            "{} ({} handlers)",
            "Async".with(label),
            report.async_patterns.total.to_string().as_str().with(value)
        );
        println!(
            "{}",
            "\u{2500}".repeat(20).as_str().with(dim)
        );
        for item in &report.async_patterns.work_items {
            println!(
                "  {}: in {}() (line {})",
                item.init_call.as_str().with(value),
                item.function.as_str().with(value),
                item.line
            );
        }
        for item in &report.async_patterns.irq_handlers {
            println!(
                "  {}: in {}() (line {})",
                item.init_call.as_str().with(value),
                item.function.as_str().with(value),
                item.line
            );
        }
        for item in &report.async_patterns.timers {
            println!(
                "  {}: in {}() (line {})",
                item.init_call.as_str().with(value),
                item.function.as_str().with(value),
                item.line
            );
        }
        for item in &report.async_patterns.tasklets {
            println!(
                "  {}: in {}() (line {})",
                item.init_call.as_str().with(value),
                item.function.as_str().with(value),
                item.line
            );
        }
        for item in &report.async_patterns.completions {
            println!(
                "  {}: in {}() (line {})",
                item.init_call.as_str().with(value),
                item.function.as_str().with(value),
                item.line
            );
        }
        print_issues(&report.async_patterns.issues, warn_color);
        println!();
    }

    // Memory
    if should_show(PatternCategory::Memory) && report.memory.total > 0 {
        println!(
            "{} ({} allocations)",
            "Memory".with(label),
            report.memory.total.to_string().as_str().with(value)
        );
        println!(
            "{}",
            "\u{2500}".repeat(24).as_str().with(dim)
        );
        if !report.memory.devm_allocs.is_empty() {
            println!(
                "  {:<24} {} (managed, safe {})",
                "devm_* allocations:".with(value),
                report.memory.devm_allocs.len().to_string().as_str().with(value),
                "\u{2713}".with(ok_color)
            );
        }
        if !report.memory.kzalloc_calls.is_empty() {
            println!(
                "  {:<24} {}",
                "kzalloc/kcalloc:".with(value),
                report.memory.kzalloc_calls.len().to_string().as_str().with(value)
            );
        }
        if !report.memory.kmalloc_calls.is_empty() {
            println!(
                "  {:<24} {}",
                "kmalloc/krealloc:".with(value),
                report.memory.kmalloc_calls.len().to_string().as_str().with(value)
            );
        }
        if !report.memory.dma_allocs.is_empty() {
            println!(
                "  {:<24} {}",
                "DMA allocations:".with(value),
                report.memory.dma_allocs.len().to_string().as_str().with(value)
            );
        }
        if report.memory.gfp_kernel > 0 {
            println!(
                "  {:<24} {} uses (process context)",
                "GFP_KERNEL:".with(value),
                report.memory.gfp_kernel.to_string().as_str().with(value)
            );
        }
        if report.memory.gfp_atomic > 0 {
            println!(
                "  {:<24} {} uses (interrupt context)",
                "GFP_ATOMIC:".with(value),
                report.memory.gfp_atomic.to_string().as_str().with(value)
            );
        }
        if !report.memory.barriers.is_empty() {
            println!(
                "  {:<24} {}",
                "Memory barriers:".with(value),
                report.memory.barriers.len().to_string().as_str().with(value)
            );
        }
        print_issues(&report.memory.issues, warn_color);
        println!();
    }

    // RCU
    if should_show(PatternCategory::Rcu) && report.rcu.total > 0 {
        println!(
            "{} ({} instances)",
            "RCU".with(label),
            report.rcu.total.to_string().as_str().with(value)
        );
        println!(
            "{}",
            "\u{2500}".repeat(18).as_str().with(dim)
        );
        if !report.rcu.read_sections.is_empty() {
            let matched = report.rcu.read_sections.iter().filter(|s| s.matched).count();
            let total = report.rcu.read_sections.len();
            let check = if matched == total {
                format!("all matched {}", "\u{2713}".with(ok_color))
            } else {
                format!(
                    "{}/{} matched {}",
                    matched,
                    total,
                    "\u{26a0}".with(warn_color)
                )
            };
            println!(
                "  {:<24} {} pairs ({})",
                "rcu_read_lock/unlock:".with(value),
                total.to_string().as_str().with(value),
                check
            );
        }
        if !report.rcu.dereference_calls.is_empty() {
            println!(
                "  {:<24} {}",
                "rcu_dereference:".with(value),
                report.rcu.dereference_calls.len().to_string().as_str().with(value)
            );
        }
        if !report.rcu.assign_pointer_calls.is_empty() {
            println!(
                "  {:<24} {}",
                "rcu_assign_pointer:".with(value),
                report.rcu.assign_pointer_calls.len().to_string().as_str().with(value)
            );
        }
        if !report.rcu.synchronize_calls.is_empty() {
            println!(
                "  {:<24} {}",
                "synchronize_rcu:".with(value),
                report.rcu.synchronize_calls.len().to_string().as_str().with(value)
            );
        }
        if !report.rcu.call_rcu_calls.is_empty() {
            println!(
                "  {:<24} {}",
                "call_rcu:".with(value),
                report.rcu.call_rcu_calls.len().to_string().as_str().with(value)
            );
        }
        if !report.rcu.srcu_calls.is_empty() {
            println!(
                "  {:<24} {}",
                "SRCU:".with(value),
                report.rcu.srcu_calls.len().to_string().as_str().with(value)
            );
        }
        print_issues(&report.rcu.issues, warn_color);
        println!();
    }

    // Summary
    print_summary(report);
}

fn print_lock_group(
    label_text: &str,
    pairs: &[LockPair],
    ok_color: Color,
    value_color: Color,
) {
    if pairs.is_empty() {
        return;
    }
    let matched = pairs.iter().filter(|p| p.matched).count();
    let total = pairs.len();
    let check = if matched == total {
        format!("all matched {}", "\u{2713}".with(ok_color))
    } else {
        format!("{}/{} matched", matched, total)
    };
    println!(
        "  {:<28} {} pairs ({})",
        format!("{}:", label_text).as_str().with(value_color),
        total.to_string().as_str().with(value_color),
        check
    );
}

fn print_issues(issues: &[PatternIssue], warn_color: Color) {
    for issue in issues {
        let prefix = match issue.severity {
            IssueSeverity::Warning => "\u{26a0}",
            IssueSeverity::Info => "\u{2139}",
        };
        let loc = match (&issue.line, &issue.function) {
            (Some(line), Some(func)) => format!(" at line {} in {}()", line, func),
            (Some(line), None) => format!(" at line {}", line),
            (None, Some(func)) => format!(" in {}()", func),
            (None, None) => String::new(),
        };
        println!(
            "  {} {}{}",
            prefix.with(warn_color),
            issue.message.as_str().with(warn_color),
            loc.as_str().with(warn_color)
        );
    }
}

fn print_summary(report: &PatternReport) {
    let label = Color::DarkCyan;
    let value = Color::Grey;
    let ok_color = Color::DarkGreen;
    let warn_color = Color::DarkYellow;

    println!("{}", "Summary".with(label));
    println!("{}", "\u{2500}".repeat(10).as_str().with(Color::DarkGrey));
    println!(
        "  Total patterns: {}",
        report.total_patterns.to_string().as_str().with(value)
    );

    if report.total_issues > 0 {
        // Break down issues
        let mut parts = Vec::new();
        let lock_issues = report.locking.issues.len();
        let mem_issues = report.lifecycle.issues.len() + report.memory.issues.len();
        let err_issues = report.error_handling.issues.len();
        let rcu_issues = report.rcu.issues.len();
        if lock_issues > 0 {
            parts.push(format!("{} lock issue(s)", lock_issues));
        }
        if mem_issues > 0 {
            parts.push(format!("{} memory issue(s)", mem_issues));
        }
        if err_issues > 0 {
            parts.push(format!("{} error-handling issue(s)", err_issues));
        }
        if rcu_issues > 0 {
            parts.push(format!("{} RCU issue(s)", rcu_issues));
        }
        println!(
            "  Issues found: {} ({})",
            report.total_issues.to_string().as_str().with(warn_color),
            parts.join(", ").as_str().with(warn_color)
        );
    } else {
        println!(
            "  Issues found: {}",
            "0".with(ok_color)
        );
    }

    // Quality bar
    let filled = (report.quality_pct as usize) / 10;
    let empty = 10 - filled;
    let bar = format!(
        "{}{}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty)
    );
    let quality_label = match report.quality_pct {
        90..=100 => "Excellent",
        75..=89 => "Good",
        50..=74 => "Fair",
        _ => "Needs Review",
    };
    let bar_color = match report.quality_pct {
        90..=100 => ok_color,
        75..=89 => Color::DarkCyan,
        50..=74 => warn_color,
        _ => Color::DarkRed,
    };
    println!(
        "  Code quality: {} {}% ({})",
        bar.as_str().with(bar_color),
        report.quality_pct,
        quality_label.with(bar_color)
    );
}
