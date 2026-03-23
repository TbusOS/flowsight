//! `flowsight experiment` — experiment tracking for the keep/discard loop
//!
//! Manages experiments: create baseline, record rounds, view history,
//! find best result. Inspired by autoresearch's autonomous loop.

use anyhow::Result;
use crossterm::style::{Color, Stylize};
use flowsight_evolve::experiment::{ExperimentStatus, ExperimentTracker};
use std::path::Path;

use crate::output::OutputFormat;

const C_TITLE: Color = Color::Rgb {
    r: 140,
    g: 185,
    b: 165,
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
const C_FILE: Color = Color::Rgb {
    r: 155,
    g: 160,
    b: 185,
};

/// List all experiments
pub fn run_list(format: &OutputFormat) -> Result<()> {
    let tracker = ExperimentTracker::new(Path::new("."));
    let experiments = tracker.list();

    if experiments.is_empty() {
        match format {
            OutputFormat::Json => println!("[]"),
            _ => println!("{}", "No experiments found.".with(C_DIM)),
        }
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let mut items = Vec::new();
            for name in &experiments {
                if let Ok(meta) = tracker.load_meta(name) {
                    items.push(serde_json::json!({
                        "name": meta.name,
                        "branch": meta.branch,
                        "baseline_aqs": meta.baseline_aqs,
                        "rounds": meta.rounds,
                        "target_dir": meta.target_dir,
                    }));
                }
            }
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        _ => {
            println!();
            println!("  {}", "Experiments".with(C_TITLE));
            println!("  {}", "─".repeat(50).with(C_DIM));
            for name in &experiments {
                if let Ok(meta) = tracker.load_meta(name) {
                    println!(
                        "  {} {} {} {}",
                        meta.name.with(C_FILE),
                        format!("baseline={:.2}", meta.baseline_aqs).with(C_SCORE),
                        format!("rounds={}", meta.rounds).with(C_DIM),
                        format!("branch={}", meta.branch).with(C_DIM),
                    );
                }
            }
            println!();
        }
    }

    Ok(())
}

/// Start a new experiment
pub fn run_start(
    name: &str,
    target_dir: &str,
    budget: f64,
    format: &OutputFormat,
) -> Result<()> {
    let tracker = ExperimentTracker::new(Path::new("."));

    // Measure baseline AQS
    let files = flowsight_evolve::budget::collect_c_files(Path::new(target_dir), "*.c", true);
    if files.is_empty() {
        anyhow::bail!("No .c files found in {}", target_dir);
    }

    let config = flowsight_evolve::budget::BudgetConfig {
        time_limit: std::time::Duration::from_secs_f64(budget),
        priority: flowsight_evolve::budget::FilePriority::EntryFirst,
    };

    eprintln!(
        "{}",
        format!("Measuring baseline AQS ({} files, {:.0}s budget)...", files.len(), budget)
            .with(C_DIM)
    );

    let report =
        flowsight_evolve::budget::run_budgeted_analysis(&files, Path::new(target_dir), &config);
    let baseline = report.aggregate_aqs.score;

    let meta = tracker
        .create(name, baseline, target_dir, "*.c", budget)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&meta)?);
        }
        _ => {
            println!();
            println!(
                "  {} {}",
                "Experiment created:".with(C_TITLE),
                name.with(C_FILE),
            );
            println!(
                "  {} {:.4}",
                "Baseline AQS:".with(C_DIM),
                baseline.to_string().with(C_SCORE),
            );
            println!(
                "  {} {}",
                "Branch:".with(C_DIM),
                meta.branch.with(C_FILE),
            );
            println!(
                "  {} {}",
                "Target:".with(C_DIM),
                target_dir.with(C_FILE),
            );
            println!();
        }
    }

    Ok(())
}

/// Show experiment log
pub fn run_log(name: &str, format: &OutputFormat) -> Result<()> {
    let tracker = ExperimentTracker::new(Path::new("."));
    let meta = tracker
        .load_meta(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let entries = tracker
        .read_log(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        _ => {
            println!();
            println!(
                "  {} {} {}",
                "Experiment Log:".with(C_TITLE),
                name.with(C_FILE),
                format!("(baseline={:.4})", meta.baseline_aqs).with(C_DIM),
            );
            println!("  {}", "─".repeat(70).with(C_DIM));
            println!(
                "  {:<6} {:<10} {:<8} {:<8} {:<8} {:<8} {}",
                "round".with(C_DIM),
                "commit".with(C_DIM),
                "before".with(C_DIM),
                "after".with(C_DIM),
                "delta".with(C_DIM),
                "status".with(C_DIM),
                "description".with(C_DIM),
            );

            for entry in &entries {
                let status_color = match entry.status {
                    ExperimentStatus::Keep => C_OK,
                    ExperimentStatus::Discard => C_ERR,
                };
                let delta_color = if entry.delta > 0.0 { C_OK } else { C_ERR };
                let commit_str = entry
                    .commit
                    .as_deref()
                    .map(|c| &c[..c.len().min(7)])
                    .unwrap_or("(none)");

                println!(
                    "  {:<6} {:<10} {:<8} {:<8} {} {:<8} {}",
                    entry.round.to_string().with(C_DIM),
                    commit_str.with(C_FILE),
                    format!("{:.4}", entry.aqs_before).with(C_DIM),
                    format!("{:.4}", entry.aqs_after).with(C_SCORE),
                    format!("{:+.4}", entry.delta).with(delta_color),
                    entry.status.to_string().with(status_color),
                    entry.description.as_str().with(C_DIM),
                );
            }

            if entries.is_empty() {
                println!("  {}", "(no rounds recorded yet)".with(C_DIM));
            }
            println!();
        }
    }

    Ok(())
}

/// Show best experiment result
pub fn run_best(name: &str, format: &OutputFormat) -> Result<()> {
    let tracker = ExperimentTracker::new(Path::new("."));
    let meta = tracker
        .load_meta(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let best = tracker
        .best_round(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&best)?);
        }
        _ => {
            println!();
            match best {
                Some(entry) => {
                    let improvement = entry.aqs_after - meta.baseline_aqs;
                    println!(
                        "  {} round {} {}",
                        "Best result:".with(C_TITLE),
                        entry.round.to_string().with(C_SCORE),
                        format!("(+{:.4} from baseline)", improvement).with(C_OK),
                    );
                    println!(
                        "  {} {:.4} → {:.4}",
                        "AQS:".with(C_DIM),
                        meta.baseline_aqs.to_string().with(C_DIM),
                        entry.aqs_after.to_string().with(C_SCORE),
                    );
                    if let Some(ref commit) = entry.commit {
                        println!(
                            "  {} {}",
                            "Commit:".with(C_DIM),
                            commit.as_str().with(C_FILE),
                        );
                    }
                    println!(
                        "  {} {}",
                        "Description:".with(C_DIM),
                        entry.description.as_str().with(C_DIM),
                    );
                }
                None => {
                    println!(
                        "  {}",
                        "No successful rounds found.".with(C_WARN),
                    );
                }
            }
            println!();
        }
    }

    Ok(())
}
