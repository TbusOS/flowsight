//! `flowsight diff` command - compare execution flows between two file versions
//!
//! Tree-diff algorithm compares flow trees structurally, categorizing changes as
//! ADDED, REMOVED, CHANGED, or MOVED. Supports function-level and call-level diffs.

use crate::context::AnalysisContext;
use crate::output::OutputFormat;
use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use flowsight_core::FlowNode;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ──────────────────────────────────────────────────────────────────────────────
// Options
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for the diff command
pub struct DiffOptions {
    /// Maximum depth for flow comparison (None = unlimited)
    pub max_depth: Option<usize>,
    /// Ignore call ordering differences
    pub ignore_order: bool,
    /// Include unchanged paths in output
    pub show_common: bool,
    /// Only show function-level summary (skip call-level details)
    pub summary_only: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            max_depth: None,
            ignore_order: false,
            show_common: false,
            summary_only: false,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Diff data model (immutable)
// ──────────────────────────────────────────────────────────────────────────────

/// Category of a single call-level change
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ChangeKind {
    /// Call exists only in file B
    Added,
    /// Call exists only in file A
    Removed,
    /// Call present in both but name differs (e.g., API upgrade)
    Changed { old_name: String, new_name: String },
    /// Call present in both but position differs
    Moved { old_index: usize, new_index: usize },
    /// Call is identical in both versions
    Unchanged,
}

/// A single call-level diff entry
#[derive(Debug, Clone, Serialize)]
pub struct CallDiff {
    /// Function name (uses the "newer" name when Changed)
    pub name: String,
    /// Change category
    pub kind: ChangeKind,
    /// Depth in the flow tree
    pub depth: usize,
    /// Children diffs (recursive)
    pub children: Vec<CallDiff>,
}

/// Function-level change category
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum FunctionChange {
    /// Function only in file B
    New,
    /// Function only in file A
    Removed,
    /// Function exists in both but flow changed
    Modified,
    /// Function exists in both with identical flow
    Unchanged,
}

/// Summary of a single function's status
#[derive(Debug, Clone, Serialize)]
pub struct FunctionDiffEntry {
    pub name: String,
    pub change: FunctionChange,
    /// Number of calls in file A (0 if New)
    pub calls_a: usize,
    /// Number of calls in file B (0 if Removed)
    pub calls_b: usize,
}

/// Complete diff result for a pair of files
#[derive(Debug, Clone, Serialize)]
pub struct FlowDiffResult {
    /// Path of file A
    pub file_a: String,
    /// Path of file B
    pub file_b: String,
    /// Per-function diff (only functions that have call-level diffs)
    pub function_diffs: Vec<FunctionFlowDiff>,
    /// High-level function-change list
    pub function_summary: Vec<FunctionDiffEntry>,
    /// Aggregate counts
    pub stats: DiffStats,
}

/// Call-level diff for one function
#[derive(Debug, Clone, Serialize)]
pub struct FunctionFlowDiff {
    pub function_name: String,
    pub call_diffs: Vec<CallDiff>,
}

/// Aggregate statistics
#[derive(Debug, Clone, Default, Serialize)]
pub struct DiffStats {
    pub added_calls: usize,
    pub removed_calls: usize,
    pub changed_calls: usize,
    pub moved_calls: usize,
    pub unchanged_calls: usize,
    pub new_functions: usize,
    pub removed_functions: usize,
    pub modified_functions: usize,
    pub unchanged_functions: usize,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public entry points
// ──────────────────────────────────────────────────────────────────────────────

/// Compare execution flows of a specific function between two files
pub fn run(
    file_a: &Path,
    file_b: &Path,
    function: Option<&str>,
    format: &OutputFormat,
    opts: &DiffOptions,
) -> Result<()> {
    let mut ctx = AnalysisContext::new();

    let result_a = ctx
        .analyze_file(file_a)
        .with_context(|| format!("Failed to analyze file A: {}", file_a.display()))?;
    let result_b = ctx
        .analyze_file(file_b)
        .with_context(|| format!("Failed to analyze file B: {}", file_b.display()))?;

    // Build lookup tables: function name -> (flow_tree_opt, call_list)
    let flows_a = build_flow_lookup(&result_a.analysis.flow_trees, &result_a.parse_result.functions);
    let flows_b = build_flow_lookup(&result_b.analysis.flow_trees, &result_b.parse_result.functions);

    let diff_result = match function {
        Some(fname) => diff_single_function(
            file_a,
            file_b,
            fname,
            &flows_a,
            &flows_b,
            opts,
        )?,
        None => diff_all_functions(file_a, file_b, &flows_a, &flows_b, opts)?,
    };

    render(&diff_result, format, opts)
}

// ──────────────────────────────────────────────────────────────────────────────
// Flow lookup helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Lightweight representation of a function's call tree for diffing
#[derive(Debug, Clone)]
struct FuncFlowInfo {
    /// Ordered list of direct callees
    calls: Vec<String>,
    /// Full flow tree (reserved for future recursive sub-tree diffing)
    #[allow(dead_code)]
    flow_tree: Option<FlowNode>,
}

/// Build name -> FuncFlowInfo from analysis results
fn build_flow_lookup(
    flow_trees: &[FlowNode],
    functions: &HashMap<String, flowsight_core::FunctionDef>,
) -> HashMap<String, FuncFlowInfo> {
    let mut map = HashMap::new();

    // First, populate from parse-level functions (guaranteed to have all functions)
    for (name, fdef) in functions {
        map.insert(
            name.clone(),
            FuncFlowInfo {
                calls: fdef.calls.clone(),
                flow_tree: None,
            },
        );
    }

    // Overlay with richer flow trees where available
    for tree in flow_trees {
        let calls: Vec<String> = tree.children.iter().map(|c| c.name.clone()).collect();
        map.insert(
            tree.name.clone(),
            FuncFlowInfo {
                calls,
                flow_tree: Some(tree.clone()),
            },
        );
    }

    map
}

// ──────────────────────────────────────────────────────────────────────────────
// Single-function diff
// ──────────────────────────────────────────────────────────────────────────────

fn diff_single_function(
    file_a: &Path,
    file_b: &Path,
    function: &str,
    flows_a: &HashMap<String, FuncFlowInfo>,
    flows_b: &HashMap<String, FuncFlowInfo>,
    opts: &DiffOptions,
) -> Result<FlowDiffResult> {
    let info_a = flows_a.get(function);
    let info_b = flows_b.get(function);

    if info_a.is_none() && info_b.is_none() {
        anyhow::bail!(
            "Function '{}' not found in either file.\n  File A: {}\n  File B: {}",
            function,
            file_a.display(),
            file_b.display()
        );
    }

    let empty = FuncFlowInfo {
        calls: vec![],
        flow_tree: None,
    };

    let fa = info_a.unwrap_or(&empty);
    let fb = info_b.unwrap_or(&empty);

    let call_diffs = diff_call_lists(&fa.calls, &fb.calls, opts, 1);

    let mut stats = DiffStats::default();
    accumulate_stats(&call_diffs, &mut stats);

    let change = match (info_a.is_some(), info_b.is_some()) {
        (false, true) => {
            stats.new_functions = 1;
            FunctionChange::New
        }
        (true, false) => {
            stats.removed_functions = 1;
            FunctionChange::Removed
        }
        _ if stats.added_calls == 0
            && stats.removed_calls == 0
            && stats.changed_calls == 0
            && stats.moved_calls == 0 =>
        {
            stats.unchanged_functions = 1;
            FunctionChange::Unchanged
        }
        _ => {
            stats.modified_functions = 1;
            FunctionChange::Modified
        }
    };

    let function_summary = vec![FunctionDiffEntry {
        name: function.to_string(),
        change,
        calls_a: fa.calls.len(),
        calls_b: fb.calls.len(),
    }];

    let function_diffs = vec![FunctionFlowDiff {
        function_name: function.to_string(),
        call_diffs,
    }];

    Ok(FlowDiffResult {
        file_a: file_a.display().to_string(),
        file_b: file_b.display().to_string(),
        function_diffs,
        function_summary,
        stats,
    })
}

// ──────────────────────────────────────────────────────────────────────────────
// All-functions diff
// ──────────────────────────────────────────────────────────────────────────────

fn diff_all_functions(
    file_a: &Path,
    file_b: &Path,
    flows_a: &HashMap<String, FuncFlowInfo>,
    flows_b: &HashMap<String, FuncFlowInfo>,
    opts: &DiffOptions,
) -> Result<FlowDiffResult> {
    let names_a: HashSet<&String> = flows_a.keys().collect();
    let names_b: HashSet<&String> = flows_b.keys().collect();

    let mut function_summary = Vec::new();
    let mut function_diffs = Vec::new();
    let mut stats = DiffStats::default();

    let empty = FuncFlowInfo {
        calls: vec![],
        flow_tree: None,
    };

    // Functions only in A (removed)
    let mut removed: Vec<&String> = names_a.difference(&names_b).copied().collect();
    removed.sort();
    for name in &removed {
        let fa = flows_a.get(*name).unwrap_or(&empty);
        stats.removed_functions += 1;
        function_summary.push(FunctionDiffEntry {
            name: (*name).clone(),
            change: FunctionChange::Removed,
            calls_a: fa.calls.len(),
            calls_b: 0,
        });
    }

    // Functions only in B (new)
    let mut added: Vec<&String> = names_b.difference(&names_a).copied().collect();
    added.sort();
    for name in &added {
        let fb = flows_b.get(*name).unwrap_or(&empty);
        stats.new_functions += 1;
        function_summary.push(FunctionDiffEntry {
            name: (*name).clone(),
            change: FunctionChange::New,
            calls_a: 0,
            calls_b: fb.calls.len(),
        });
    }

    // Functions in both
    let mut common: Vec<&String> = names_a.intersection(&names_b).copied().collect();
    common.sort();
    for name in &common {
        let fa = flows_a.get(*name).unwrap_or(&empty);
        let fb = flows_b.get(*name).unwrap_or(&empty);

        let call_diffs = diff_call_lists(&fa.calls, &fb.calls, opts, 1);
        let mut local_stats = DiffStats::default();
        accumulate_stats(&call_diffs, &mut local_stats);

        let has_changes = local_stats.added_calls > 0
            || local_stats.removed_calls > 0
            || local_stats.changed_calls > 0
            || local_stats.moved_calls > 0;

        if has_changes {
            stats.modified_functions += 1;
            stats.added_calls += local_stats.added_calls;
            stats.removed_calls += local_stats.removed_calls;
            stats.changed_calls += local_stats.changed_calls;
            stats.moved_calls += local_stats.moved_calls;
            stats.unchanged_calls += local_stats.unchanged_calls;

            function_summary.push(FunctionDiffEntry {
                name: (*name).clone(),
                change: FunctionChange::Modified,
                calls_a: fa.calls.len(),
                calls_b: fb.calls.len(),
            });

            function_diffs.push(FunctionFlowDiff {
                function_name: (*name).clone(),
                call_diffs,
            });
        } else {
            stats.unchanged_functions += 1;
            stats.unchanged_calls += local_stats.unchanged_calls;

            if opts.show_common {
                function_summary.push(FunctionDiffEntry {
                    name: (*name).clone(),
                    change: FunctionChange::Unchanged,
                    calls_a: fa.calls.len(),
                    calls_b: fb.calls.len(),
                });
            }
        }
    }

    Ok(FlowDiffResult {
        file_a: file_a.display().to_string(),
        file_b: file_b.display().to_string(),
        function_diffs,
        function_summary,
        stats,
    })
}

// ──────────────────────────────────────────────────────────────────────────────
// Tree-diff algorithm (LCS-based)
// ──────────────────────────────────────────────────────────────────────────────

/// Compare two ordered call lists and produce a list of CallDiff entries.
///
/// Uses longest-common-subsequence (LCS) to align matching calls, then
/// classifies unmatched entries as Added/Removed. When `ignore_order` is set,
/// calls present in both sides (regardless of position) are treated as
/// Unchanged or Moved.
fn diff_call_lists(
    calls_a: &[String],
    calls_b: &[String],
    opts: &DiffOptions,
    depth: usize,
) -> Vec<CallDiff> {
    if let Some(max) = opts.max_depth {
        if depth > max {
            return vec![];
        }
    }

    if opts.ignore_order {
        return diff_call_lists_unordered(calls_a, calls_b, depth);
    }

    // LCS table
    let n = calls_a.len();
    let m = calls_b.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            if calls_a[i - 1] == calls_b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Back-trace to build diff
    let mut i = n;
    let mut j = m;

    // We build the diff list in reverse, then flip it
    let mut reversed = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && calls_a[i - 1] == calls_b[j - 1] {
            // Match
            reversed.push(CallDiff {
                name: calls_a[i - 1].clone(),
                kind: ChangeKind::Unchanged,
                depth,
                children: vec![],
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            // Added in B
            reversed.push(CallDiff {
                name: calls_b[j - 1].clone(),
                kind: ChangeKind::Added,
                depth,
                children: vec![],
            });
            j -= 1;
        } else {
            // Removed from A
            reversed.push(CallDiff {
                name: calls_a[i - 1].clone(),
                kind: ChangeKind::Removed,
                depth,
                children: vec![],
            });
            i -= 1;
        }
    }

    reversed.reverse();

    // Post-process: detect CHANGED pairs (adjacent Removed+Added with similar names)
    detect_changed_pairs(reversed)
}

/// Unordered diff: treats the call lists as sets, detecting moved items
fn diff_call_lists_unordered(calls_a: &[String], calls_b: &[String], depth: usize) -> Vec<CallDiff> {
    let set_a: HashSet<&String> = calls_a.iter().collect();
    let set_b: HashSet<&String> = calls_b.iter().collect();

    let mut result = Vec::new();

    // Present in both
    for name in calls_b.iter() {
        if set_a.contains(name) {
            let idx_a = calls_a.iter().position(|c| c == name);
            let idx_b = calls_b.iter().position(|c| c == name);
            let kind = match (idx_a, idx_b) {
                (Some(ia), Some(ib)) if ia != ib => ChangeKind::Moved {
                    old_index: ia,
                    new_index: ib,
                },
                _ => ChangeKind::Unchanged,
            };
            result.push(CallDiff {
                name: name.clone(),
                kind,
                depth,
                children: vec![],
            });
        } else {
            result.push(CallDiff {
                name: name.clone(),
                kind: ChangeKind::Added,
                depth,
                children: vec![],
            });
        }
    }

    // Removed (in A but not B)
    for name in calls_a.iter() {
        if !set_b.contains(name) {
            result.push(CallDiff {
                name: name.clone(),
                kind: ChangeKind::Removed,
                depth,
                children: vec![],
            });
        }
    }

    result
}

/// Detect adjacent Removed+Added pairs that look like renames (fuzzy match)
fn detect_changed_pairs(diffs: Vec<CallDiff>) -> Vec<CallDiff> {
    let mut result: Vec<CallDiff> = Vec::with_capacity(diffs.len());
    let mut skip_next = false;

    for (idx, diff) in diffs.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if diff.kind == ChangeKind::Removed {
            // Look ahead for an Added entry that could be a rename
            if let Some(next) = diffs.get(idx + 1) {
                if next.kind == ChangeKind::Added && is_likely_rename(&diff.name, &next.name) {
                    result.push(CallDiff {
                        name: next.name.clone(),
                        kind: ChangeKind::Changed {
                            old_name: diff.name.clone(),
                            new_name: next.name.clone(),
                        },
                        depth: diff.depth,
                        children: vec![],
                    });
                    skip_next = true;
                    continue;
                }
            }
        }

        result.push(diff.clone());
    }

    result
}

/// Heuristic: two function names are likely a rename if they share a common
/// prefix/suffix of sufficient length, or one contains the other.
fn is_likely_rename(old: &str, new: &str) -> bool {
    if old == new {
        return false;
    }

    // One contains the other
    if old.contains(new) || new.contains(old) {
        return true;
    }

    // Shared prefix >= 4 chars
    let common_prefix = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .count();
    if common_prefix >= 4 {
        return true;
    }

    // Shared suffix >= 4 chars
    let common_suffix = old
        .chars()
        .rev()
        .zip(new.chars().rev())
        .take_while(|(a, b)| a == b)
        .count();
    if common_suffix >= 4 {
        return true;
    }

    false
}

// ──────────────────────────────────────────────────────────────────────────────
// Statistics accumulation
// ──────────────────────────────────────────────────────────────────────────────

fn accumulate_stats(diffs: &[CallDiff], stats: &mut DiffStats) {
    for d in diffs {
        match &d.kind {
            ChangeKind::Added => stats.added_calls += 1,
            ChangeKind::Removed => stats.removed_calls += 1,
            ChangeKind::Changed { .. } => stats.changed_calls += 1,
            ChangeKind::Moved { .. } => stats.moved_calls += 1,
            ChangeKind::Unchanged => stats.unchanged_calls += 1,
        }
        accumulate_stats(&d.children, stats);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Rendering
// ──────────────────────────────────────────────────────────────────────────────

fn render(diff: &FlowDiffResult, format: &OutputFormat, opts: &DiffOptions) -> Result<()> {
    match format {
        OutputFormat::Json => render_json(diff),
        _ => render_text(diff, opts),
    }
}

fn render_json(diff: &FlowDiffResult) -> Result<()> {
    let json_str =
        serde_json::to_string_pretty(diff).context("Failed to serialize diff to JSON")?;
    println!("{}", json_str);
    Ok(())
}

fn render_text(diff: &FlowDiffResult, opts: &DiffOptions) -> Result<()> {
    // Low-saturation palette (per project convention)
    let header_color = Color::DarkCyan;
    let added_color = Color::DarkGreen;
    let removed_color = Color::DarkRed;
    let changed_color = Color::DarkYellow;
    let dim_color = Color::DarkGrey;
    let label_color = Color::Grey;

    let file_a_short = short_path(&diff.file_a);
    let file_b_short = short_path(&diff.file_b);

    // ── Per-function flow diffs ──────────────────────────────────────────
    if !opts.summary_only {
        for fdiff in &diff.function_diffs {
            let title = format!("FlowSight Diff: {}", fdiff.function_name);
            let bar = "\u{2550}".repeat(title.len());

            println!();
            println!("{}", title.with(header_color));
            println!("{}", bar.with(header_color));
            println!(
                "  {} {}",
                "File A:".with(label_color),
                file_a_short.as_str().with(dim_color)
            );
            println!(
                "  {} {}",
                "File B:".with(label_color),
                file_b_short.as_str().with(dim_color)
            );
            println!();

            let section = "Execution Flow Changes:";
            let section_bar = "\u{2500}".repeat(section.len());
            println!("{}", section.with(header_color));
            println!("{}", section_bar.with(header_color));

            println!(
                "  {}()",
                fdiff.function_name.as_str().with(label_color)
            );

            for cd in &fdiff.call_diffs {
                print_call_diff(cd, 2, opts, added_color, removed_color, changed_color, dim_color);
            }

            println!();
        }
    }

    // ── Summary ──────────────────────────────────────────────────────────
    let s = &diff.stats;

    println!("{}", "Summary:".with(header_color));
    let summary_bar = "\u{2500}".repeat("Summary:".len());
    println!("{}", summary_bar.with(header_color));

    if s.added_calls > 0 {
        println!(
            "  {} {}",
            format!("Added calls:   {}", s.added_calls).with(added_color),
            list_names_by_kind(&diff.function_diffs, |k| matches!(k, ChangeKind::Added))
                .with(dim_color)
        );
    }
    if s.removed_calls > 0 {
        println!(
            "  {} {}",
            format!("Removed calls: {}", s.removed_calls).with(removed_color),
            list_names_by_kind(&diff.function_diffs, |k| matches!(k, ChangeKind::Removed))
                .with(dim_color)
        );
    }
    if s.changed_calls > 0 {
        println!(
            "  {} {}",
            format!("Changed calls: {}", s.changed_calls).with(changed_color),
            list_names_by_kind(&diff.function_diffs, |k| matches!(k, ChangeKind::Changed { .. }))
                .with(dim_color)
        );
    }
    if s.moved_calls > 0 {
        println!(
            "  {}",
            format!("Moved calls:   {}", s.moved_calls).with(changed_color)
        );
    }
    println!(
        "  {}",
        format!("Unchanged:     {}", s.unchanged_calls).with(dim_color)
    );

    // Function-level summary
    println!();
    println!("{}", "Function-level changes:".with(header_color));
    let flc_bar = "\u{2500}".repeat("Function-level changes:".len());
    println!("{}", flc_bar.with(header_color));

    let new_fn: Vec<&str> = diff
        .function_summary
        .iter()
        .filter(|e| e.change == FunctionChange::New)
        .map(|e| e.name.as_str())
        .collect();
    let removed_fn: Vec<&str> = diff
        .function_summary
        .iter()
        .filter(|e| e.change == FunctionChange::Removed)
        .map(|e| e.name.as_str())
        .collect();
    let modified_fn: Vec<&str> = diff
        .function_summary
        .iter()
        .filter(|e| e.change == FunctionChange::Modified)
        .map(|e| e.name.as_str())
        .collect();

    print_fn_list("New functions:     ", &new_fn, added_color);
    print_fn_list("Removed functions: ", &removed_fn, removed_color);
    print_fn_list("Modified functions:", &modified_fn, changed_color);

    if s.unchanged_functions > 0 {
        println!(
            "  {} {}",
            "Unchanged functions:".with(dim_color),
            s.unchanged_functions.to_string().as_str().with(dim_color)
        );
    }

    Ok(())
}

/// Print a single CallDiff line with proper indentation and coloring
fn print_call_diff(
    cd: &CallDiff,
    indent: usize,
    opts: &DiffOptions,
    added_color: Color,
    removed_color: Color,
    changed_color: Color,
    dim_color: Color,
) {
    let pad = "  ".repeat(indent);

    match &cd.kind {
        ChangeKind::Unchanged => {
            if opts.show_common {
                println!(
                    "{}  {}{}",
                    pad,
                    cd.name.as_str().with(dim_color),
                    "()".with(dim_color),
                );
            }
        }
        ChangeKind::Added => {
            println!(
                "{}{}",
                pad,
                format!("+   {}()    (ADDED)", cd.name).with(added_color)
            );
        }
        ChangeKind::Removed => {
            println!(
                "{}{}",
                pad,
                format!("-   {}()    (REMOVED)", cd.name).with(removed_color)
            );
        }
        ChangeKind::Changed { old_name, new_name } => {
            println!(
                "{}{}",
                pad,
                format!("~   {} -> {}()    (CHANGED)", old_name, new_name).with(changed_color)
            );
        }
        ChangeKind::Moved {
            old_index,
            new_index,
        } => {
            println!(
                "{}{}",
                pad,
                format!(
                    "~   {}()    (MOVED: #{} -> #{})",
                    cd.name, old_index, new_index
                )
                .with(changed_color)
            );
        }
    }

    for child in &cd.children {
        print_call_diff(child, indent + 1, opts, added_color, removed_color, changed_color, dim_color);
    }
}

/// Collect names matching a predicate from all function diffs
fn list_names_by_kind<F>(diffs: &[FunctionFlowDiff], pred: F) -> String
where
    F: Fn(&ChangeKind) -> bool,
{
    let names: Vec<&str> = diffs
        .iter()
        .flat_map(|fd| fd.call_diffs.iter())
        .filter(|cd| pred(&cd.kind))
        .map(|cd| cd.name.as_str())
        .collect();

    if names.is_empty() {
        String::new()
    } else {
        format!("({})", names.join(", "))
    }
}

/// Print a labelled list of function names
fn print_fn_list(label: &str, names: &[&str], color: Color) {
    if names.is_empty() {
        println!("  {} {}", label.with(color), "(none)".with(Color::DarkGrey));
    } else {
        println!(
            "  {} {}",
            label.with(color),
            names.join(", ").with(color)
        );
    }
}

/// Shorten a path for display (keep last 3 components)
fn short_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 3 {
        path.to_string()
    } else {
        format!(".../{}", parts[parts.len() - 3..].join("/"))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_call_lists() {
        let a = vec!["foo".to_string(), "bar".to_string()];
        let b = vec!["foo".to_string(), "bar".to_string()];
        let opts = DiffOptions::default();

        let diffs = diff_call_lists(&a, &b, &opts, 0);
        assert!(diffs.iter().all(|d| d.kind == ChangeKind::Unchanged));
        assert_eq!(diffs.len(), 2);
    }

    #[test]
    fn test_added_calls() {
        let a = vec!["foo".to_string()];
        let b = vec!["foo".to_string(), "bar".to_string()];
        let opts = DiffOptions::default();

        let diffs = diff_call_lists(&a, &b, &opts, 0);
        let added: Vec<_> = diffs.iter().filter(|d| d.kind == ChangeKind::Added).collect();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].name, "bar");
    }

    #[test]
    fn test_removed_calls() {
        let a = vec!["foo".to_string(), "bar".to_string()];
        let b = vec!["foo".to_string()];
        let opts = DiffOptions::default();

        let diffs = diff_call_lists(&a, &b, &opts, 0);
        let removed: Vec<_> = diffs
            .iter()
            .filter(|d| d.kind == ChangeKind::Removed)
            .collect();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].name, "bar");
    }

    #[test]
    fn test_changed_detection() {
        let a = vec!["request_irq".to_string()];
        let b = vec!["request_threaded_irq".to_string()];
        let opts = DiffOptions::default();

        let diffs = diff_call_lists(&a, &b, &opts, 0);
        let changed: Vec<_> = diffs
            .iter()
            .filter(|d| matches!(&d.kind, ChangeKind::Changed { .. }))
            .collect();
        assert_eq!(changed.len(), 1);
    }

    #[test]
    fn test_is_likely_rename() {
        assert!(is_likely_rename("request_irq", "request_threaded_irq"));
        assert!(is_likely_rename("device_create_file", "device_create_file_ns"));
        assert!(!is_likely_rename("foo", "bar"));
        assert!(!is_likely_rename("abc", "xyz"));
    }

    #[test]
    fn test_unordered_diff_detects_moved() {
        let a = vec!["foo".to_string(), "bar".to_string()];
        let b = vec!["bar".to_string(), "foo".to_string()];
        let opts = DiffOptions {
            ignore_order: true,
            ..Default::default()
        };

        let diffs = diff_call_lists(&a, &b, &opts, 0);
        let moved: Vec<_> = diffs
            .iter()
            .filter(|d| matches!(&d.kind, ChangeKind::Moved { .. }))
            .collect();
        assert_eq!(moved.len(), 2);
    }

    #[test]
    fn test_depth_limit() {
        let a = vec!["foo".to_string()];
        let b = vec!["bar".to_string()];
        let opts = DiffOptions {
            max_depth: Some(0),
            ..Default::default()
        };

        let diffs = diff_call_lists(&a, &b, &opts, 1);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_empty_lists() {
        let a: Vec<String> = vec![];
        let b: Vec<String> = vec![];
        let opts = DiffOptions::default();

        let diffs = diff_call_lists(&a, &b, &opts, 0);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_accumulate_stats() {
        let diffs = vec![
            CallDiff {
                name: "a".into(),
                kind: ChangeKind::Added,
                depth: 0,
                children: vec![],
            },
            CallDiff {
                name: "b".into(),
                kind: ChangeKind::Removed,
                depth: 0,
                children: vec![],
            },
            CallDiff {
                name: "c".into(),
                kind: ChangeKind::Unchanged,
                depth: 0,
                children: vec![],
            },
        ];
        let mut stats = DiffStats::default();
        accumulate_stats(&diffs, &mut stats);
        assert_eq!(stats.added_calls, 1);
        assert_eq!(stats.removed_calls, 1);
        assert_eq!(stats.unchanged_calls, 1);
    }

    #[test]
    fn test_short_path() {
        assert_eq!(
            short_path("/a/b/c/d/e.c"),
            ".../c/d/e.c"
        );
        assert_eq!(short_path("a/b.c"), "a/b.c");
    }
}
