//! Tauri Commands

use flowsight_parser::get_parser;
use flowsight_analysis::Analyzer;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub file: String,
    pub functions_count: usize,
    pub structs_count: usize,
    pub async_handlers_count: usize,
    pub entry_points: Vec<String>,
    pub flow_trees: Vec<flowsight_core::FlowNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub return_type: String,
    pub line: u32,
    pub is_callback: bool,
    pub callback_context: Option<String>,
    pub calls: Vec<String>,
}

/// Analyze a source file
#[tauri::command]
pub async fn analyze_file(path: String) -> Result<AnalysisResult, String> {
    let path = PathBuf::from(&path);
    
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path)
        .map_err(|e| e.to_string())?;

    let source = std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())?;
    
    let mut analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    Ok(AnalysisResult {
        file: path.to_string_lossy().to_string(),
        functions_count: parse_result.functions.len(),
        structs_count: parse_result.structs.len(),
        async_handlers_count: analysis.async_bindings.len(),
        entry_points: analysis.entry_points,
        flow_trees: analysis.flow_trees,
    })
}

/// Get list of functions in a file
#[tauri::command]
pub async fn get_functions(path: String) -> Result<Vec<FunctionInfo>, String> {
    let path = PathBuf::from(&path);
    
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path)
        .map_err(|e| e.to_string())?;

    let source = std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())?;
    
    let mut analyzer = Analyzer::new();
    let _ = analyzer.analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    let functions: Vec<FunctionInfo> = parse_result.functions
        .into_iter()
        .map(|(name, func)| FunctionInfo {
            name,
            return_type: func.return_type,
            line: func.location.map(|l| l.line).unwrap_or(0),
            is_callback: func.is_callback,
            callback_context: func.callback_context,
            calls: func.calls,
        })
        .collect();

    Ok(functions)
}

/// Read file content
#[tauri::command]
pub async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())
}

