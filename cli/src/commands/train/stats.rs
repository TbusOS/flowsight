//! `flowsight train stats` - display statistics about generated JSONL data

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Statistics collected from a JSONL training data file
struct TrainStats {
    total_examples: usize,
    category_counts: HashMap<String, usize>,
    total_tokens: usize,
    min_tokens: usize,
    max_tokens: usize,
    format_detected: String,
    source_files: HashMap<String, usize>,
}

/// Run stats analysis on a JSONL file
pub fn run(file: &Path) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("Cannot read {}", file.display()))?;

    let stats = compute_stats(&content)?;
    print_stats(&stats, file);

    Ok(())
}

/// Parse all lines and compute aggregate statistics
fn compute_stats(content: &str) -> Result<TrainStats> {
    let mut stats = TrainStats {
        total_examples: 0,
        category_counts: HashMap::new(),
        total_tokens: 0,
        min_tokens: usize::MAX,
        max_tokens: 0,
        format_detected: "unknown".to_string(),
        source_files: HashMap::new(),
    };

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let value: serde_json::Value = serde_json::from_str(trimmed).with_context(|| {
            format!("Invalid JSON on line {}", line_num + 1)
        })?;

        stats.total_examples += 1;

        // Detect format from first line
        if stats.total_examples == 1 {
            stats.format_detected = detect_format(&value);
        }

        // Extract metadata
        if let Some(meta) = value.get("meta") {
            extract_meta_stats(&mut stats, meta);
        }

        // Estimate tokens from content size
        let content_len = trimmed.len();
        let estimated_tokens = content_len / 4;
        stats.total_tokens += estimated_tokens;
        stats.min_tokens = stats.min_tokens.min(estimated_tokens);
        stats.max_tokens = stats.max_tokens.max(estimated_tokens);
    }

    if stats.total_examples == 0 {
        stats.min_tokens = 0;
    }

    Ok(stats)
}

/// Detect which training format a JSON object represents
fn detect_format(value: &serde_json::Value) -> String {
    if value.get("messages").is_some() {
        "chatml".to_string()
    } else if value.get("chosen").is_some() && value.get("rejected").is_some() {
        "dpo".to_string()
    } else if value.get("instruction").is_some() {
        "sft".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Extract category and source file counts from the meta field
fn extract_meta_stats(stats: &mut TrainStats, meta: &serde_json::Value) {
    if let Some(category) = meta.get("category").and_then(|v| v.as_str()) {
        *stats.category_counts.entry(category.to_string()).or_insert(0) += 1;
    }

    if let Some(source) = meta.get("source_file").and_then(|v| v.as_str()) {
        // Use just the filename for cleaner display
        let display_name = Path::new(source)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(source);
        *stats.source_files.entry(display_name.to_string()).or_insert(0) += 1;
    }
}

/// Print formatted statistics to stdout
fn print_stats(stats: &TrainStats, file: &Path) {
    println!("Training Data Statistics: {}", file.display());
    println!();
    println!("  Format:     {}", stats.format_detected);
    println!("  Examples:   {}", stats.total_examples);
    println!("  Est tokens: {} total", stats.total_tokens);
    if stats.total_examples > 0 {
        println!(
            "  Per example: {}-{} tokens (avg {})",
            stats.min_tokens,
            stats.max_tokens,
            stats.total_tokens / stats.total_examples
        );
    }

    println!();
    println!("Category distribution:");
    let mut sorted_cats: Vec<_> = stats.category_counts.iter().collect();
    sorted_cats.sort_by(|a, b| b.1.cmp(a.1));
    for (category, count) in &sorted_cats {
        let pct = if stats.total_examples > 0 {
            (**count as f64 / stats.total_examples as f64) * 100.0
        } else {
            0.0
        };
        println!("  {:<15} {:>5} ({:>5.1}%)", category, count, pct);
    }

    println!();
    println!("Source files: {} unique", stats.source_files.len());
    let mut sorted_files: Vec<_> = stats.source_files.iter().collect();
    sorted_files.sort_by(|a, b| b.1.cmp(a.1));
    let display_count = sorted_files.len().min(10);
    for (file_name, count) in sorted_files.iter().take(display_count) {
        println!("  {:<40} {:>4} examples", file_name, count);
    }
    if sorted_files.len() > display_count {
        println!("  ... and {} more files", sorted_files.len() - display_count);
    }

    // Quality metrics
    println!();
    println!("Quality metrics:");
    let categories_present = stats.category_counts.len();
    let diversity_score = if categories_present >= 5 {
        "excellent"
    } else if categories_present >= 3 {
        "good"
    } else if categories_present >= 1 {
        "limited"
    } else {
        "none"
    };
    println!("  Category diversity: {} ({})", categories_present, diversity_score);
    if stats.total_examples > 0 {
        let avg_tokens = stats.total_tokens / stats.total_examples;
        let detail_score = if avg_tokens >= 500 {
            "high detail"
        } else if avg_tokens >= 200 {
            "moderate detail"
        } else {
            "low detail"
        };
        println!("  Avg detail level:   {} tokens ({})", avg_tokens, detail_score);
    }
}
