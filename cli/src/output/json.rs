//! JSON output formatting

use crate::commands::analyze::DirectorySummary;
use anyhow::Result;
use flowsight_analysis::AnalysisResult;
use flowsight_parser::ParseResult;
use std::path::Path;

/// Serialize analysis result as JSON
pub fn format_analysis(
    file: &Path,
    parse_result: &ParseResult,
    analysis: &AnalysisResult,
) -> Result<String> {
    let result = serde_json::json!({
        "file": file.to_string_lossy(),
        "functions": parse_result.functions.len(),
        "structs": parse_result.structs.len(),
        "async_bindings": analysis.async_bindings.len(),
        "entry_points": analysis.entry_points,
        "flow_trees": analysis.flow_trees,
    });
    serde_json::to_string_pretty(&result).map_err(Into::into)
}

/// Serialize a single value as pretty JSON
pub fn to_pretty_json<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}

/// Serialize directory analysis summary as JSON
pub fn format_directory_summary(summary: &DirectorySummary) -> Result<String> {
    let files: Vec<_> = summary
        .per_file
        .iter()
        .map(|f| {
            serde_json::json!({
                "file": f.file,
                "functions": f.functions,
                "structs": f.structs,
                "async_handlers": f.async_handlers,
                "callbacks": f.callbacks,
                "entry_points": f.entry_points,
            })
        })
        .collect();

    let result = serde_json::json!({
        "directory": summary.directory,
        "summary": {
            "files_analyzed": summary.files_analyzed,
            "files_skipped": summary.files_skipped,
            "total_functions": summary.total_functions,
            "total_structs": summary.total_structs,
            "total_async_handlers": summary.total_async_handlers,
            "total_callbacks": summary.total_callbacks,
            "total_entry_points": summary.total_entry_points,
        },
        "files": files,
        "errors": summary.errors,
    });

    serde_json::to_string_pretty(&result).map_err(Into::into)
}
