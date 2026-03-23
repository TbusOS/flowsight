//! `flowsight quality` — Analysis Quality Score (AQS) measurement
//!
//! Computes a single scalar metric (0.0–1.0) measuring analysis depth
//! and accuracy across five dimensions. Foundation for the autonomous
//! evolution loop (v0.8.0 autoresearch pattern).

use anyhow::Result;
use crossterm::style::{Color, Stylize};
use flowsight_evolve::aqs::{aggregate_scores, AnalysisQualityScore};
use std::path::Path;

use crate::context::AnalysisContext;
use crate::output::OutputFormat;

const C_TITLE: Color = Color::Rgb {
    r: 140,
    g: 185,
    b: 165,
};
const C_FILE: Color = Color::Rgb {
    r: 155,
    g: 160,
    b: 185,
};
const C_OK: Color = Color::Rgb {
    r: 130,
    g: 175,
    b: 140,
};
const C_WARN: Color = Color::Rgb {
    r: 170,
    g: 160,
    b: 120,
};
const C_ERR: Color = Color::Rgb {
    r: 195,
    g: 120,
    b: 120,
};
const C_DIM: Color = Color::Rgb {
    r: 110,
    g: 115,
    b: 120,
};
const C_SCORE: Color = Color::Rgb {
    r: 190,
    g: 170,
    b: 130,
};

/// Run quality analysis on a file or directory
pub fn run(
    path: &Path,
    format: &OutputFormat,
    recursive: bool,
    pattern: &str,
) -> Result<()> {
    if path.is_dir() {
        run_directory(path, format, recursive, pattern)
    } else {
        run_file(path, format)
    }
}

/// Compute AQS for a single file
fn run_file(path: &Path, format: &OutputFormat) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let file_analysis = ctx.analyze_file(path)?;
    let kb = ctx.knowledge_base();

    let aqs = AnalysisQualityScore::compute(
        &file_analysis.source,
        &file_analysis.parse_result,
        &file_analysis.analysis,
        kb,
    );

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&aqs)?);
        }
        _ => {
            print_aqs_report(path, &aqs);
        }
    }

    Ok(())
}

/// Compute AQS for a directory
fn run_directory(
    dir: &Path,
    format: &OutputFormat,
    recursive: bool,
    pattern: &str,
) -> Result<()> {
    let glob = globset::GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()?
        .compile_matcher();

    let walker = if recursive {
        walkdir::WalkDir::new(dir)
    } else {
        walkdir::WalkDir::new(dir).max_depth(1)
    };

    let files: Vec<_> = walker
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && glob.is_match(e.file_name()))
        .map(|e| e.into_path())
        .collect();

    if files.is_empty() {
        println!(
            "{}",
            "No matching files found.".with(C_WARN)
        );
        return Ok(());
    }

    let mut ctx = AnalysisContext::new();
    let mut scores = Vec::new();
    let mut failures = 0usize;

    for file in &files {
        match ctx.analyze_file(file) {
            Ok(file_analysis) => {
                let kb = ctx.knowledge_base();
                let aqs = AnalysisQualityScore::compute(
                    &file_analysis.source,
                    &file_analysis.parse_result,
                    &file_analysis.analysis,
                    kb,
                );
                scores.push((file.clone(), aqs));
            }
            Err(_) => {
                failures += 1;
            }
        }
    }

    let file_scores: Vec<_> = scores.iter().map(|(_, aqs)| aqs.clone()).collect();
    let aggregate = aggregate_scores(&file_scores);

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "directory": dir.to_string_lossy(),
                "files_analyzed": scores.len(),
                "files_failed": failures,
                "aggregate": aggregate,
                "per_file": scores.iter().map(|(path, aqs)| {
                    serde_json::json!({
                        "file": path.to_string_lossy(),
                        "score": aqs.score,
                    })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            print_directory_report(dir, &scores, &aggregate, failures);
        }
    }

    Ok(())
}

fn score_color(score: f64) -> Color {
    if score >= 0.8 {
        C_OK
    } else if score >= 0.5 {
        C_WARN
    } else {
        C_ERR
    }
}

fn score_bar(score: f64, width: usize) -> String {
    let filled = (score * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn print_aqs_report(path: &Path, aqs: &AnalysisQualityScore) {
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    println!();
    println!(
        "  {} {}",
        "Analysis Quality Score".with(C_TITLE),
        filename.with(C_FILE)
    );
    println!("  {}", "─".repeat(50).with(C_DIM));
    println!();

    // Overall score
    let color = score_color(aqs.score);
    println!(
        "  {}  {} {}",
        score_bar(aqs.score, 20).with(color),
        format!("{:.2}", aqs.score).with(C_SCORE),
        format!("({}%)", (aqs.score * 100.0).round() as u32).with(C_DIM),
    );
    println!();

    // Dimension breakdown
    println!("  {}", "Dimensions".with(C_TITLE));
    print_dimension(
        "Direct call resolution",
        aqs.dimensions.direct_call_resolution,
        0.30,
        &aqs.stats.direct_calls_resolved.to_string(),
        &aqs.stats.total_calls.to_string(),
    );
    print_dimension(
        "Indirect call resolution",
        aqs.dimensions.indirect_call_resolution,
        0.20,
        &aqs.stats.indirect_calls_resolved.to_string(),
        &aqs.stats.indirect_calls_total.to_string(),
    );
    print_dimension(
        "Knowledge base coverage",
        aqs.dimensions.kb_coverage,
        0.20,
        &aqs.stats.kernel_api_covered.to_string(),
        &aqs.stats.kernel_api_calls.to_string(),
    );
    print_dimension(
        "Error path coverage",
        aqs.dimensions.error_path_coverage,
        0.15,
        &aqs.stats.error_paths_detected.to_string(),
        &aqs.stats.functions_with_error_potential.to_string(),
    );
    print_dimension(
        "Cross-file resolution",
        aqs.dimensions.cross_file_resolution,
        0.15,
        &aqs.stats.external_symbols_resolved.to_string(),
        &aqs.stats.external_symbols_total.to_string(),
    );

    println!();
    println!(
        "  {} {} functions, {} call edges",
        "Stats:".with(C_DIM),
        aqs.stats.functions_analyzed.to_string().with(C_TITLE),
        aqs.stats.total_calls.to_string().with(C_TITLE),
    );
    println!();
}

fn print_dimension(name: &str, score: f64, weight: f64, resolved: &str, total: &str) {
    let color = score_color(score);
    let weight_pct = (weight * 100.0) as u32;
    println!(
        "  {} {:<26} {} {} {}",
        score_bar(score, 10).with(color),
        name.with(C_DIM),
        format!("{:.0}%", score * 100.0).with(color),
        format!("({}/{})", resolved, total).with(C_DIM),
        format!("w={:>2}%", weight_pct).with(C_DIM),
    );
}

fn print_directory_report(
    dir: &Path,
    scores: &[(std::path::PathBuf, AnalysisQualityScore)],
    aggregate: &AnalysisQualityScore,
    failures: usize,
) {
    println!();
    println!(
        "  {} {}",
        "Directory AQS Report".with(C_TITLE),
        dir.display().to_string().with(C_FILE)
    );
    println!("  {}", "─".repeat(60).with(C_DIM));
    println!();

    // Aggregate score
    let color = score_color(aggregate.score);
    println!(
        "  {}  {} {}",
        score_bar(aggregate.score, 20).with(color),
        format!("{:.2}", aggregate.score).with(C_SCORE),
        format!(
            "({} files, {} failed)",
            scores.len(),
            failures
        )
        .with(C_DIM),
    );
    println!();

    // Dimension breakdown for aggregate
    println!("  {}", "Aggregate Dimensions".with(C_TITLE));
    print_dimension(
        "Direct call resolution",
        aggregate.dimensions.direct_call_resolution,
        0.30,
        &aggregate.stats.direct_calls_resolved.to_string(),
        &aggregate.stats.total_calls.to_string(),
    );
    print_dimension(
        "Indirect call resolution",
        aggregate.dimensions.indirect_call_resolution,
        0.20,
        &aggregate.stats.indirect_calls_resolved.to_string(),
        &aggregate.stats.indirect_calls_total.to_string(),
    );
    print_dimension(
        "Knowledge base coverage",
        aggregate.dimensions.kb_coverage,
        0.20,
        &aggregate.stats.kernel_api_covered.to_string(),
        &aggregate.stats.kernel_api_calls.to_string(),
    );
    print_dimension(
        "Error path coverage",
        aggregate.dimensions.error_path_coverage,
        0.15,
        &aggregate.stats.error_paths_detected.to_string(),
        &aggregate.stats.functions_with_error_potential.to_string(),
    );
    print_dimension(
        "Cross-file resolution",
        aggregate.dimensions.cross_file_resolution,
        0.15,
        &aggregate.stats.external_symbols_resolved.to_string(),
        &aggregate.stats.external_symbols_total.to_string(),
    );

    // Per-file scores (sorted by score ascending — worst first)
    println!();
    println!("  {}", "Per-File Scores".with(C_TITLE));

    let mut sorted: Vec<_> = scores.iter().collect();
    sorted.sort_by(|a, b| a.1.score.partial_cmp(&b.1.score).unwrap());

    for (path, aqs) in &sorted {
        let fname = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let color = score_color(aqs.score);
        println!(
            "  {} {} {}",
            score_bar(aqs.score, 8).with(color),
            format!("{:.2}", aqs.score).with(color),
            fname.with(C_FILE),
        );
    }

    println!();
    println!(
        "  {} {} functions, {} call edges across {} files",
        "Total:".with(C_DIM),
        aggregate.stats.functions_analyzed.to_string().with(C_TITLE),
        aggregate.stats.total_calls.to_string().with(C_TITLE),
        scores.len().to_string().with(C_TITLE),
    );
    println!();
}
