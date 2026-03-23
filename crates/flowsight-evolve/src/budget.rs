//! Analysis Budget System
//!
//! Fixed-time-budget analysis inspired by autoresearch's 5-minute training budget.
//! All results under the same budget are directly comparable.
//!
//! ## Design
//!
//! ```text
//! 1. Collect files → sort by priority
//! 2. Start timer
//! 3. Per file: estimate cost → pick analysis level → analyze → accumulate AQS
//! 4. Budget exhausted → output coverage report
//! ```

use crate::aqs::{aggregate_scores, AnalysisQualityScore};
use flowsight_analysis::{AnalysisResult, Analyzer};
use flowsight_knowledge::KnowledgeBase;
use flowsight_parser::get_parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// File priority strategy for budget scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilePriority {
    /// Entry functions first (probe, init, open, ioctl, connect)
    EntryFirst,
    /// Largest files first (more information density)
    LargestFirst,
    /// Smallest files first (maximize file count coverage)
    SmallestFirst,
    /// Alphabetical (deterministic, for reproducible benchmarks)
    Alphabetical,
}

impl Default for FilePriority {
    fn default() -> Self {
        Self::EntryFirst
    }
}

/// Analysis level — progressively cheaper analysis modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisLevel {
    /// Full: CFG + knowledge base + call graph
    Full,
    /// Fast: call graph + knowledge base lookup (skip CFG)
    Fast,
    /// Degraded: function signature + direct call list only
    Degraded,
    /// Skipped: no time left, file recorded but not analyzed
    Skipped,
}

/// Budget configuration
#[derive(Debug, Clone)]
pub struct BudgetConfig {
    /// Wall-clock time limit
    pub time_limit: Duration,
    /// File priority strategy
    pub priority: FilePriority,
}

/// Result of a single file's budgeted analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetedFileResult {
    /// File path
    pub file: String,
    /// Analysis level used
    pub level: AnalysisLevel,
    /// AQS score (None if skipped)
    pub aqs: Option<AnalysisQualityScore>,
    /// Time spent on this file (ms)
    pub elapsed_ms: u64,
    /// File size in bytes
    pub file_size: u64,
    /// Number of functions found
    pub functions: usize,
}

/// Result of a budgeted directory analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetReport {
    /// Directory analyzed
    pub directory: String,
    /// Budget configuration
    pub budget_seconds: f64,
    /// Actual wall-clock time used
    pub elapsed_seconds: f64,
    /// Aggregate AQS
    pub aggregate_aqs: AnalysisQualityScore,
    /// Coverage: files analyzed / total files
    pub files_analyzed: usize,
    pub files_total: usize,
    pub files_skipped: usize,
    pub files_failed: usize,
    /// Coverage percentage
    pub coverage_pct: f64,
    /// Per-file results
    pub per_file: Vec<BudgetedFileResult>,
}

/// Entry-function indicator patterns for priority scoring
const ENTRY_PATTERNS: &[&str] = &[
    "probe", "init", "open", "ioctl", "connect", "bind", "attach",
    "resume", "suspend", "remove", "disconnect", "close", "release",
    "read", "write", "start", "stop", "setup", "configure",
];

/// Score a file for priority scheduling (higher = analyzed first)
fn priority_score(path: &Path, priority: FilePriority) -> i64 {
    match priority {
        FilePriority::EntryFirst => {
            // Quick heuristic: scan filename + first few KB for entry patterns
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let mut score: i64 = 0;
            for pattern in ENTRY_PATTERNS {
                if filename.contains(pattern) {
                    score += 10;
                }
            }

            // Bonus: read first 4KB to check for ops struct patterns
            if let Ok(content) = std::fs::read_to_string(path) {
                let preview = &content[..content.len().min(4096)];
                if preview.contains("struct file_operations")
                    || preview.contains("struct usb_driver")
                    || preview.contains("struct platform_driver")
                    || preview.contains("struct pci_driver")
                    || preview.contains("module_init")
                {
                    score += 20;
                }
                // Larger files get slight priority (more info)
                score += (content.len() / 1000).min(10) as i64;
            }
            score
        }
        FilePriority::LargestFirst => {
            std::fs::metadata(path)
                .map(|m| m.len() as i64)
                .unwrap_or(0)
        }
        FilePriority::SmallestFirst => {
            std::fs::metadata(path)
                .map(|m| -(m.len() as i64))
                .unwrap_or(0)
        }
        FilePriority::Alphabetical => 0, // sort alphabetically later
    }
}

/// Select analysis level based on remaining budget and estimated file cost
fn select_level(file_size: u64, remaining: Duration, elapsed_per_kb: f64) -> AnalysisLevel {
    let size_kb = file_size as f64 / 1024.0;
    let estimated_full = Duration::from_secs_f64(size_kb * elapsed_per_kb);

    let remaining_secs = remaining.as_secs_f64();

    if remaining_secs <= 0.0 {
        AnalysisLevel::Skipped
    } else if estimated_full.as_secs_f64() < remaining_secs * 0.5 {
        // Plenty of time: full analysis
        AnalysisLevel::Full
    } else if estimated_full.as_secs_f64() * 0.3 < remaining_secs {
        // Tight but doable with fast mode
        AnalysisLevel::Fast
    } else if remaining_secs > 0.5 {
        // Very tight: degraded mode
        AnalysisLevel::Degraded
    } else {
        AnalysisLevel::Skipped
    }
}

/// Run budgeted analysis on a directory
pub fn run_budgeted_analysis(
    files: &[PathBuf],
    dir: &Path,
    config: &BudgetConfig,
) -> BudgetReport {
    let total_files = files.len();
    let start = Instant::now();

    // Sort by priority
    let mut sorted_files: Vec<_> = files.iter().cloned().collect();
    match config.priority {
        FilePriority::Alphabetical => {
            sorted_files.sort();
        }
        _ => {
            sorted_files.sort_by(|a, b| {
                let sa = priority_score(a, config.priority);
                let sb = priority_score(b, config.priority);
                sb.cmp(&sa) // descending: highest priority first
            });
        }
    }

    let mut results: Vec<BudgetedFileResult> = Vec::new();
    let mut aqs_scores: Vec<AnalysisQualityScore> = Vec::new();
    let mut files_failed = 0usize;
    // Adaptive cost estimation: start with a guess, refine as we go
    let mut total_kb_processed = 0.0f64;
    let mut total_analysis_time = 0.0f64;
    // Conservative initial estimate: ~1s per KB (CFG analysis is heavy)
    let initial_cost_per_kb = 0.5;

    for file in &sorted_files {
        let elapsed = start.elapsed();
        let remaining = config.time_limit.saturating_sub(elapsed);

        if remaining.is_zero() {
            // Budget exhausted — mark remaining as skipped
            results.push(BudgetedFileResult {
                file: file.to_string_lossy().into_owned(),
                level: AnalysisLevel::Skipped,
                aqs: None,
                elapsed_ms: 0,
                file_size: std::fs::metadata(file).map(|m| m.len()).unwrap_or(0),
                functions: 0,
            });
            continue;
        }

        let file_size = std::fs::metadata(file).map(|m| m.len()).unwrap_or(0);
        let elapsed_per_kb = if total_kb_processed > 0.0 {
            total_analysis_time / total_kb_processed
        } else {
            initial_cost_per_kb
        };

        let level = select_level(file_size, remaining, elapsed_per_kb);

        if level == AnalysisLevel::Skipped {
            results.push(BudgetedFileResult {
                file: file.to_string_lossy().into_owned(),
                level: AnalysisLevel::Skipped,
                aqs: None,
                elapsed_ms: 0,
                file_size,
                functions: 0,
            });
            continue;
        }

        let file_start = Instant::now();
        let result = analyze_with_level(file, level);
        let file_elapsed = file_start.elapsed();

        // Update adaptive cost model
        let size_kb = file_size as f64 / 1024.0;
        if size_kb > 0.0 {
            total_kb_processed += size_kb;
            total_analysis_time += file_elapsed.as_secs_f64();
        }

        match result {
            Ok((aqs, functions)) => {
                aqs_scores.push(aqs.clone());
                results.push(BudgetedFileResult {
                    file: file.to_string_lossy().into_owned(),
                    level,
                    aqs: Some(aqs),
                    elapsed_ms: file_elapsed.as_millis() as u64,
                    file_size,
                    functions,
                });
            }
            Err(_) => {
                files_failed += 1;
                results.push(BudgetedFileResult {
                    file: file.to_string_lossy().into_owned(),
                    level: AnalysisLevel::Degraded,
                    aqs: None,
                    elapsed_ms: file_elapsed.as_millis() as u64,
                    file_size,
                    functions: 0,
                });
            }
        }
    }

    let total_elapsed = start.elapsed();
    let files_analyzed = results.iter().filter(|r| r.aqs.is_some()).count();
    let files_skipped = results.iter().filter(|r| r.level == AnalysisLevel::Skipped).count();

    let aggregate_aqs = aggregate_scores(&aqs_scores);
    let coverage_pct = if total_files > 0 {
        files_analyzed as f64 / total_files as f64 * 100.0
    } else {
        0.0
    };

    BudgetReport {
        directory: dir.to_string_lossy().into_owned(),
        budget_seconds: config.time_limit.as_secs_f64(),
        elapsed_seconds: total_elapsed.as_secs_f64(),
        aggregate_aqs,
        files_analyzed,
        files_total: total_files,
        files_skipped,
        files_failed,
        coverage_pct,
        per_file: results,
    }
}

/// Analyze a single file at a given level
fn analyze_with_level(
    path: &Path,
    level: AnalysisLevel,
) -> anyhow::Result<(AnalysisQualityScore, usize)> {
    let source = std::fs::read_to_string(path)?;
    let parser = get_parser();
    let filename = path.to_string_lossy();

    match level {
        AnalysisLevel::Full | AnalysisLevel::Fast => {
            let mut parse_result = parser
                .parse(&source, &filename)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let mut analyzer = Analyzer::new();
            let analysis = analyzer
                .analyze(&source, &mut parse_result)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let kb = KnowledgeBase::builtin();
            let functions = parse_result.functions.len();
            let aqs = AnalysisQualityScore::compute(&source, &parse_result, &analysis, &kb);
            Ok((aqs, functions))
        }
        AnalysisLevel::Degraded => {
            // Degraded: parse only, skip deep analysis
            let parse_result = parser
                .parse(&source, &filename)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let analysis = AnalysisResult::default();
            let kb = KnowledgeBase::builtin();
            let functions = parse_result.functions.len();
            let aqs = AnalysisQualityScore::compute(&source, &parse_result, &analysis, &kb);
            Ok((aqs, functions))
        }
        AnalysisLevel::Skipped => {
            anyhow::bail!("Cannot analyze at Skipped level")
        }
    }
}

/// Collect .c files from a directory
pub fn collect_c_files(dir: &Path, pattern: &str, recursive: bool) -> Vec<PathBuf> {
    let glob = match globset::GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
    {
        Ok(g) => g.compile_matcher(),
        Err(_) => return Vec::new(),
    };

    let walker = if recursive {
        walkdir::WalkDir::new(dir)
    } else {
        walkdir::WalkDir::new(dir).max_depth(1)
    };

    walker
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && glob.is_match(e.file_name()))
        .map(|e| e.into_path())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_level_plenty_of_time() {
        let level = select_level(10_000, Duration::from_secs(60), 0.01);
        assert_eq!(level, AnalysisLevel::Full);
    }

    #[test]
    fn test_select_level_tight_budget() {
        // 100KB file, 0.01s/KB = 1s estimated, 1.5s remaining
        let level = select_level(100_000, Duration::from_millis(1500), 0.01);
        assert_eq!(level, AnalysisLevel::Fast);
    }

    #[test]
    fn test_select_level_very_tight() {
        // 100KB file, 0.01s/KB = 1s estimated, 0.8s remaining
        // estimated * 0.5 = 0.5s < 0.8s but estimated * 1.0 > 0.8s → not Full
        // estimated * 0.3 = 0.3s < 0.8s → Fast OK
        // So this should be Fast. For Degraded we need even less time:
        // remaining > 0.5s but estimated * 0.3 > remaining → Degraded
        // 200KB file, 0.01s/KB = 2s estimated, 0.6s remaining
        // estimated * 0.3 = 0.6s, remaining = 0.6s → just barely Fast
        // 200KB file, 0.01s/KB = 2s estimated, 0.55s remaining
        // estimated * 0.3 = 0.6s > 0.55s remaining → Degraded
        let level = select_level(200_000, Duration::from_millis(550), 0.01);
        assert_eq!(level, AnalysisLevel::Degraded);
    }

    #[test]
    fn test_select_level_no_time() {
        let level = select_level(1000, Duration::ZERO, 0.01);
        assert_eq!(level, AnalysisLevel::Skipped);
    }

    #[test]
    fn test_select_level_almost_no_time() {
        let level = select_level(50_000, Duration::from_millis(100), 0.01);
        assert_eq!(level, AnalysisLevel::Skipped);
    }

    #[test]
    fn test_priority_score_alphabetical() {
        let score = priority_score(Path::new("/tmp/test.c"), FilePriority::Alphabetical);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_entry_first_probe_file() {
        let tmp = std::env::temp_dir().join("test_probe_driver.c");
        std::fs::write(&tmp, "struct usb_driver drv = { .probe = my_probe };").ok();
        let score = priority_score(&tmp, FilePriority::EntryFirst);
        assert!(score > 0, "probe file should have positive score");
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_budget_empty_files() {
        let config = BudgetConfig {
            time_limit: Duration::from_secs(10),
            priority: FilePriority::Alphabetical,
        };
        let report = run_budgeted_analysis(&[], Path::new("/tmp"), &config);
        assert_eq!(report.files_total, 0);
        assert_eq!(report.files_analyzed, 0);
        assert!((report.coverage_pct - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_real_kernel_file() {
        let kernel_file = Path::new("/Users/sky/linux-kernel/linux/arch/arm/mach-imx/cpu-imx31.c");
        if !kernel_file.exists() {
            return;
        }

        let config = BudgetConfig {
            time_limit: Duration::from_secs(30),
            priority: FilePriority::EntryFirst,
        };
        let files = vec![kernel_file.to_path_buf()];
        let report = run_budgeted_analysis(&files, kernel_file.parent().unwrap(), &config);

        assert_eq!(report.files_analyzed, 1);
        assert_eq!(report.files_total, 1);
        assert!((report.coverage_pct - 100.0).abs() < f64::EPSILON);
        assert!(report.aggregate_aqs.score > 0.0);
    }
}
