//! JSON output formatting

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
