//! `flowsight analyze` command - analyze source files and directories

use crate::context::AnalysisContext;
use crate::output::{json, text, OutputFormat};
use anyhow::{Context, Result};
use flowsight_core::CallType;
use std::path::Path;
use std::sync::Mutex;

/// Options for the analyze command
pub struct AnalyzeOptions {
    pub recursive: bool,
    pub pattern: String,
    pub parallel: Option<usize>,
    pub summary: bool,
}

/// Statistics collected from a single file analysis
#[derive(Clone, Default)]
pub struct FileStats {
    pub file: String,
    pub functions: usize,
    pub structs: usize,
    pub async_handlers: usize,
    pub callbacks: usize,
    pub entry_points: usize,
}

/// Aggregated statistics across all analyzed files
#[derive(Clone, Default)]
pub struct DirectorySummary {
    pub directory: String,
    pub files_analyzed: usize,
    pub files_skipped: usize,
    pub total_functions: usize,
    pub total_structs: usize,
    pub total_async_handlers: usize,
    pub total_callbacks: usize,
    pub total_entry_points: usize,
    pub per_file: Vec<FileStats>,
    pub errors: Vec<String>,
}

/// Entry point: dispatch to single-file or directory analysis
pub fn run(
    path: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
    opts: &AnalyzeOptions,
) -> Result<()> {
    if path.is_dir() {
        if !opts.recursive {
            anyhow::bail!(
                "'{}' is a directory. Use --recursive (-r) to analyze directories.",
                path.display()
            );
        }
        run_directory(path, output, format, opts)
    } else {
        run_single_file(path, output, format)
    }
}

/// Analyze a single source file and print results
fn run_single_file(path: &Path, output: Option<&Path>, format: &OutputFormat) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(path)?;

    match format {
        OutputFormat::Json => {
            let json_str =
                json::format_analysis(&result.file, &result.parse_result, &result.analysis)?;
            write_output(&json_str, output)?;
        }
        _ => {
            text::print_file_analysis(
                &result.file,
                result.parse_result.functions.len(),
                result.parse_result.structs.len(),
                result.analysis.async_bindings.len(),
                result.analysis.entry_points.len(),
            );
        }
    }

    Ok(())
}

/// Analyze all matching files in a directory
fn run_directory(
    dir: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
    opts: &AnalyzeOptions,
) -> Result<()> {
    let files = collect_matching_files(dir, &opts.pattern)?;

    if files.is_empty() {
        anyhow::bail!(
            "No files matching '{}' found in '{}'",
            opts.pattern,
            dir.display()
        );
    }

    let summary = analyze_files_parallel(&files, dir, opts.parallel)?;

    match format {
        OutputFormat::Json => {
            let json_str = json::format_directory_summary(&summary)?;
            write_output(&json_str, output)?;
        }
        _ => {
            if opts.summary {
                text::print_directory_summary(&summary);
            } else {
                text::print_directory_full(&summary);
            }
        }
    }

    Ok(())
}

/// Walk directory and collect files matching the glob pattern
fn collect_matching_files(dir: &Path, pattern: &str) -> Result<Vec<std::path::PathBuf>> {
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

/// Analyze files using rayon for parallelism, return aggregated summary
fn analyze_files_parallel(
    files: &[std::path::PathBuf],
    dir: &Path,
    parallel: Option<usize>,
) -> Result<DirectorySummary> {
    use rayon::prelude::*;

    if let Some(threads) = parallel {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok(); // Ignore if already initialized
    }

    let summary = Mutex::new(DirectorySummary {
        directory: dir.to_string_lossy().into_owned(),
        ..Default::default()
    });

    files.par_iter().for_each(|file_path| {
        let result = analyze_one_file(file_path);
        let mut guard = summary.lock().expect("summary mutex poisoned");
        match result {
            Ok(stats) => {
                guard.files_analyzed += 1;
                guard.total_functions += stats.functions;
                guard.total_structs += stats.structs;
                guard.total_async_handlers += stats.async_handlers;
                guard.total_callbacks += stats.callbacks;
                guard.total_entry_points += stats.entry_points;
                guard.per_file.push(stats);
            }
            Err(err) => {
                guard.files_skipped += 1;
                guard.errors.push(format!("{}: {}", file_path.display(), err));
            }
        }
    });

    let mut result = summary.into_inner().expect("summary mutex poisoned");
    result.per_file.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(result)
}

/// Analyze a single file, returning statistics (called from parallel context)
fn analyze_one_file(path: &Path) -> Result<FileStats> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(path)?;

    let callbacks = result
        .analysis
        .call_edges
        .iter()
        .filter(|e| matches!(e.call_type, CallType::Indirect { .. }))
        .count();

    Ok(FileStats {
        file: path.to_string_lossy().into_owned(),
        functions: result.parse_result.functions.len(),
        structs: result.parse_result.structs.len(),
        async_handlers: result.analysis.async_bindings.len(),
        callbacks,
        entry_points: result.analysis.entry_points.len(),
    })
}

/// Write output string to file or stdout
fn write_output(content: &str, output: Option<&Path>) -> Result<()> {
    if let Some(out_path) = output {
        std::fs::write(out_path, content)
            .with_context(|| format!("Failed to write {}", out_path.display()))?;
        eprintln!("Output written to: {}", out_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}
