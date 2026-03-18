//! Tauri Commands

use flowsight_ai;
use flowsight_analysis::flow_builder::{
    BuildOptions, FlowBuilder, FunctionInfo as BuilderFunctionInfo,
};
use flowsight_analysis::Analyzer;
use flowsight_core::ExecutionFlow;
use flowsight_index::SymbolIndex;
use flowsight_parser::get_parser;
use flowsight_parser::parallel::ParallelParser;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tauri::Emitter;
use walkdir::WalkDir;

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
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let mut analyzer = Analyzer::new();
    let analysis = analyzer
        .analyze(&source, &mut parse_result)
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
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let mut analyzer = Analyzer::new();
    let _ = analyzer
        .analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    let functions: Vec<FunctionInfo> = parse_result
        .functions
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
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

/// Write file content
#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Write binary file from base64 encoded content
#[tauri::command]
pub async fn write_file_base64(path: String, base64: String) -> Result<(), String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(())
}

/// Global index state - 使用 RwLock 优化并发读取性能
static INDEX: Lazy<RwLock<SymbolIndex>> = Lazy::new(|| RwLock::new(SymbolIndex::new()));

/// Project information
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub files_count: usize,
    pub functions_count: usize,
    pub structs_count: usize,
    pub indexed: bool,
}

/// Search result (enhanced version)
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub kind: String, // "function", "struct", "macro", "variable", "typedef"
    pub file_path: String,
    pub line: u32,
    pub preview: String,  // Code snippet preview
    pub match_score: u32, // 0-100 match score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_callback: Option<bool>,
}

/// Search options for filtering
#[derive(Debug, Deserialize, Default)]
pub struct SearchOptions {
    /// Project path (optional, uses current indexed project if not specified)
    pub project_path: Option<String>,
    /// Include functions in search
    #[serde(default = "default_true")]
    pub include_functions: bool,
    /// Include structs in search
    #[serde(default = "default_true")]
    pub include_structs: bool,
    /// Include macros in search
    #[serde(default)]
    pub include_macros: bool,
    /// Maximum number of results
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

fn default_true() -> bool {
    true
}

fn default_max_results() -> usize {
    50
}

/// Open a project directory - returns immediately, indexing happens in background
#[tauri::command]
pub async fn open_project(
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<ProjectInfo, String> {
    let project_path = PathBuf::from(&path);

    if !project_path.is_dir() {
        return Err("Path is not a directory".into());
    }

    // Clear previous index
    {
        let mut index = INDEX.write().map_err(|e| e.to_string())?;
        *index = SymbolIndex::new();
    }

    // Spawn background indexing task with larger stack (8MB)
    let path_clone = path.clone();
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .name("indexer".into())
        .spawn(move || {
            index_project_background(project_path, app_handle);
        })
        .ok();

    // Return immediately with placeholder info
    Ok(ProjectInfo {
        path: path_clone,
        files_count: 0,
        functions_count: 0,
        structs_count: 0,
        indexed: false, // Will be updated via events
    })
}

/// Background indexing function
fn index_project_background(project_path: PathBuf, app_handle: tauri::AppHandle) {
    let _ = app_handle.emit(
        "index-progress",
        serde_json::json!({
            "phase": "scanning",
            "current": 0,
            "total": 0,
            "message": "Scanning files..."
        }),
    );

    // Scan files
    let mut c_files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry
            .path()
            .extension()
            .map(|ext| ext == "c" || ext == "h")
            .unwrap_or(false)
        {
            c_files.push(entry.path().to_path_buf());
            if c_files.len() % 2000 == 0 {
                let _ = app_handle.emit(
                    "index-progress",
                    serde_json::json!({
                        "phase": "scanning",
                        "current": c_files.len(),
                        "total": 0,
                        "message": format!("Found {} files...", c_files.len())
                    }),
                );
            }
        }
    }

    let total = c_files.len();
    let _ = app_handle.emit(
        "index-progress",
        serde_json::json!({
            "phase": "parsing",
            "current": 0,
            "total": total,
            "message": format!("Parsing {} files...", total)
        }),
    );

    // Parse in parallel
    let parallel_parser = ParallelParser::new();
    let results = parallel_parser.parse_files(&c_files);

    let _ = app_handle.emit(
        "index-progress",
        serde_json::json!({
            "phase": "indexing",
            "current": 0,
            "total": total,
            "message": "Building index..."
        }),
    );

    // Build index
    if let Ok(mut index) = INDEX.write() {
        for (i, (file, result)) in results.iter().enumerate() {
            if let Ok(parse_result) = result {
                for (_, func) in &parse_result.functions {
                    index.add_function(func.clone(), file);
                }
                for (_, st) in &parse_result.structs {
                    index.add_struct(st.clone());
                }
            }
            if i % 2000 == 0 && i > 0 {
                let _ = app_handle.emit(
                    "index-progress",
                    serde_json::json!({
                        "phase": "indexing",
                        "current": i,
                        "total": total,
                        "message": format!("Indexed {}/{}", i, total)
                    }),
                );
            }
        }

        let stats = index.stats();
        let _ = app_handle.emit(
            "index-progress",
            serde_json::json!({
                "phase": "done",
                "current": total,
                "total": total,
                "files": total,
                "functions": stats.total_functions,
                "structs": stats.total_structs,
                "message": format!("Done! {} files, {} functions", total, stats.total_functions)
            }),
        );
    }
}

/// Search for symbols in the index
///
/// Enhanced search with options:
/// - `query`: Search query string
/// - `options`: Optional search options (filters, max results)
///
/// Returns results sorted by match score (highest first)
#[tauri::command]
pub async fn search_symbols(
    query: String,
    options: Option<SearchOptions>,
) -> Result<Vec<SearchResult>, String> {
    let index = INDEX.read().map_err(|e| e.to_string())?;
    let query_lower = query.to_lowercase();
    let opts = options.unwrap_or_default();

    let max_results = if opts.max_results == 0 {
        50
    } else {
        opts.max_results
    };

    let mut results = Vec::new();

    // Search functions
    if opts.include_functions {
        for (name, func) in &index.functions {
            if let Some(score) = calculate_match_score(name, &query, &query_lower) {
                let file_path = func
                    .location
                    .as_ref()
                    .map(|l| l.file.clone())
                    .unwrap_or_default();
                let line = func.location.as_ref().map(|l| l.line).unwrap_or(0);

                // Generate preview: function signature
                let preview = generate_function_preview(func);

                results.push(SearchResult {
                    name: name.clone(),
                    kind: "function".into(),
                    file_path,
                    line,
                    preview,
                    match_score: score,
                    is_callback: Some(func.is_callback),
                });
            }
        }
    }

    // Search structs
    if opts.include_structs {
        for (name, st) in &index.structs {
            if let Some(score) = calculate_match_score(name, &query, &query_lower) {
                let file_path = st
                    .location
                    .as_ref()
                    .map(|l| l.file.clone())
                    .unwrap_or_default();
                let line = st.location.as_ref().map(|l| l.line).unwrap_or(0);

                // Generate preview: struct with field count
                let preview = generate_struct_preview(st);

                results.push(SearchResult {
                    name: name.clone(),
                    kind: "struct".into(),
                    file_path,
                    line,
                    preview,
                    match_score: score,
                    is_callback: None,
                });
            }
        }
    }

    // Sort by match score (highest first), then by name
    results.sort_by(|a, b| {
        b.match_score
            .cmp(&a.match_score)
            .then_with(|| a.name.len().cmp(&b.name.len()))
            .then_with(|| a.name.cmp(&b.name))
    });

    // Limit results
    results.truncate(max_results);

    Ok(results)
}

/// Calculate match score (0-100) for a symbol name against query
/// Returns None if no match
fn calculate_match_score(name: &str, query: &str, query_lower: &str) -> Option<u32> {
    let name_lower = name.to_lowercase();

    // Exact match = 100
    if name == query {
        return Some(100);
    }

    // Case-insensitive exact match = 95
    if name_lower == *query_lower {
        return Some(95);
    }

    // Starts with query = 90
    if name_lower.starts_with(query_lower) {
        return Some(90);
    }

    // Ends with query = 80
    if name_lower.ends_with(query_lower) {
        return Some(80);
    }

    // Contains query = 70 - penalty for distance from start
    if let Some(pos) = name_lower.find(query_lower) {
        let distance_penalty = (pos as u32).min(20);
        return Some(70 - distance_penalty);
    }

    // Fuzzy match: all query chars appear in order
    if fuzzy_match(&name_lower, query_lower) {
        return Some(40);
    }

    None
}

/// Check if all characters of query appear in name in order (fuzzy match)
fn fuzzy_match(name: &str, query: &str) -> bool {
    let mut name_chars = name.chars().peekable();

    for qc in query.chars() {
        loop {
            match name_chars.next() {
                Some(nc) if nc == qc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }

    true
}

/// Generate preview string for a function
fn generate_function_preview(func: &flowsight_core::FunctionDef) -> String {
    let params: Vec<String> = func
        .params
        .iter()
        .map(|p| {
            if p.name.is_empty() {
                p.type_name.clone()
            } else {
                format!("{} {}", p.type_name, p.name)
            }
        })
        .collect();

    let params_str = if params.is_empty() {
        "void".to_string()
    } else if params.len() > 3 {
        format!("{}, ...", params[..2].join(", "))
    } else {
        params.join(", ")
    };

    format!("{} {}({})", func.return_type, func.name, params_str)
}

/// Generate preview string for a struct
fn generate_struct_preview(st: &flowsight_core::StructDef) -> String {
    let field_count = st.fields.len();

    if field_count == 0 {
        format!("struct {} {{ }}", st.name)
    } else if field_count <= 2 {
        let fields: Vec<String> = st
            .fields
            .iter()
            .map(|f| format!("{} {}", f.type_name, f.name))
            .collect();
        format!("struct {} {{ {} }}", st.name, fields.join("; "))
    } else {
        format!("struct {} {{ ... {} fields }}", st.name, field_count)
    }
}

/// Get index statistics
#[tauri::command]
pub async fn get_index_stats() -> Result<IndexStats, String> {
    let index = INDEX.read().map_err(|e| e.to_string())?;
    let stats = index.stats();

    Ok(IndexStats {
        functions: stats.total_functions,
        structs: stats.total_structs,
        files: stats.total_files,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexStats {
    pub functions: usize,
    pub structs: usize,
    pub files: usize,
}

/// Function detail with location info
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDetail {
    pub name: String,
    pub return_type: String,
    pub file: Option<String>,
    pub line: u32,
    pub end_line: u32,
    pub is_callback: bool,
    pub callback_context: Option<String>,
    pub calls: Vec<String>,
    pub called_by: Vec<String>,
    pub params: Vec<ParamInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub type_name: String,
}

/// Local variable information
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalVarInfo {
    pub name: String,
    pub type_name: String,
}

/// Extended function detail from file parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDetailExt {
    pub name: String,
    pub return_type: String,
    pub params: Vec<ParamInfo>,
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub is_callback: bool,
    pub callback_context: Option<String>,
    pub calls: Vec<String>,
    pub called_by: Vec<String>,
    pub local_variables: Vec<LocalVarInfo>,
    pub complexity: u32,
    pub doc_comment: Option<String>,
}

/// Get function detail from index
#[tauri::command]
pub async fn get_function_detail(name: String) -> Result<Option<FunctionDetail>, String> {
    let index = INDEX.read().map_err(|e| e.to_string())?;

    if let Some(func) = index.get_function(&name) {
        Ok(Some(FunctionDetail {
            name: func.name.clone(),
            return_type: func.return_type.clone(),
            file: func.location.as_ref().map(|l| l.file.clone()),
            line: func.location.as_ref().map(|l| l.line).unwrap_or(0),
            end_line: func.location.as_ref().map(|l| l.line + 10).unwrap_or(0), // Approximate
            is_callback: func.is_callback,
            callback_context: func.callback_context.clone(),
            calls: func.calls.clone(),
            called_by: func.called_by.clone(),
            params: func
                .params
                .iter()
                .map(|p| ParamInfo {
                    name: p.name.clone(),
                    type_name: p.type_name.clone(),
                })
                .collect(),
        }))
    } else {
        Ok(None)
    }
}

/// Get all functions with their locations (for code navigation)
#[tauri::command]
pub async fn get_function_locations(path: String) -> Result<Vec<FunctionLocation>, String> {
    let path = PathBuf::from(&path);

    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let mut analyzer = Analyzer::new();
    let _ = analyzer
        .analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    let locations: Vec<FunctionLocation> = parse_result
        .functions
        .into_iter()
        .map(|(name, func)| FunctionLocation {
            name,
            line: func.location.as_ref().map(|l| l.line).unwrap_or(0),
            column: func.location.as_ref().map(|l| l.column).unwrap_or(0),
            is_callback: func.is_callback,
        })
        .collect();

    Ok(locations)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionLocation {
    pub name: String,
    pub line: u32,
    pub column: u32,
    pub is_callback: bool,
}

/// File node for file tree
#[derive(Debug, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
    pub extension: Option<String>,
}

/// List directory contents for file tree
#[tauri::command]
pub async fn list_directory(path: String, recursive: bool) -> Result<Vec<FileNode>, String> {
    let dir_path = PathBuf::from(&path);

    if !dir_path.is_dir() {
        return Err("Path is not a directory".into());
    }

    fn read_dir_entries(
        path: &Path,
        recursive: bool,
        depth: usize,
    ) -> Result<Vec<FileNode>, String> {
        if depth > 10 {
            return Ok(vec![]); // Limit depth
        }

        let mut entries: Vec<FileNode> = std::fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .filter(|e| {
                // Skip hidden files and common non-source directories
                let name = e.file_name().to_string_lossy().to_string();
                !name.starts_with('.')
                    && ![
                        "node_modules",
                        "target",
                        "build",
                        "dist",
                        "__pycache__",
                        ".git",
                    ]
                    .contains(&name.as_str())
            })
            .map(|e| {
                let path = e.path();
                let name = e.file_name().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                let extension = if !is_dir {
                    path.extension()
                        .map(|ext| ext.to_string_lossy().to_string())
                } else {
                    None
                };

                let children = if is_dir && recursive {
                    read_dir_entries(&path, recursive, depth + 1).ok()
                } else {
                    None
                };

                FileNode {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir,
                    children,
                    extension,
                }
            })
            .collect();

        // Sort: directories first, then by name
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(entries)
    }

    read_dir_entries(&dir_path, recursive, 0)
}

/// Expand a single directory (lazy loading)
#[tauri::command]
pub async fn expand_directory(path: String) -> Result<Vec<FileNode>, String> {
    list_directory(path, false).await
}

/// Export flow analysis text to file
#[tauri::command]
pub async fn export_flow_text(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

/// Create a new file
#[tauri::command]
pub async fn create_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    // Check if file already exists
    if path.exists() {
        return Err("File already exists".to_string());
    }

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    // Create empty file
    std::fs::File::create(&path).map_err(|e| format!("Failed to create file: {}", e))?;

    Ok(())
}

/// Create a new directory
#[tauri::command]
pub async fn create_directory(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if path.exists() {
        return Err("Directory already exists".to_string());
    }

    std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))
}

/// Rename a file or directory
#[tauri::command]
pub async fn rename_file(old_path: String, new_path: String) -> Result<(), String> {
    let old_path = PathBuf::from(&old_path);
    let new_path = PathBuf::from(&new_path);

    if !old_path.exists() {
        return Err("Source path does not exist".to_string());
    }

    if new_path.exists() {
        return Err("Target path already exists".to_string());
    }

    std::fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename: {}", e))
}

/// Delete a file or directory
#[tauri::command]
pub async fn delete_file_or_dir(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if path.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete directory: {}", e))
    } else {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))
    }
}

/// Caller information
#[derive(Debug, Serialize)]
pub struct CallerInfo {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub call_type: String,
    pub async_mechanism: Option<String>,
}

/// Get callers of a function
#[tauri::command]
pub async fn get_function_callers(
    function_name: String,
    _project_path: Option<String>,
) -> Result<std::collections::HashMap<String, Vec<CallerInfo>>, String> {
    let index = INDEX.read().map_err(|e| e.to_string())?;

    let mut callers = Vec::new();

    // Search through all functions to find callers
    for (name, func) in &index.functions {
        // Check if this function calls the target
        if func.calls.contains(&function_name) {
            let call_type = if func.is_callback { "async" } else { "direct" };

            callers.push(CallerInfo {
                name: name.clone(),
                file: func
                    .location
                    .as_ref()
                    .map(|l| l.file.clone())
                    .unwrap_or_default(),
                line: func.location.as_ref().map(|l| l.line).unwrap_or(0),
                call_type: call_type.to_string(),
                async_mechanism: func.callback_context.clone(),
            });
        }
    }

    // Also check async bindings for indirect callers
    // This would require tracking async bindings in the index

    let mut result = std::collections::HashMap::new();
    result.insert("callers".to_string(), callers);
    Ok(result)
}

/// Scenario request for symbolic execution
#[derive(Debug, Deserialize)]
pub struct ScenarioRequest {
    pub name: String,
    pub entry_function: String,
    pub bindings: Vec<ScenarioBinding>,
    pub options: Option<ScenarioOptionsReq>,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioBinding {
    pub path: String,
    pub value: String,
    #[serde(rename = "type")]
    pub value_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioOptionsReq {
    pub follow_async: Option<bool>,
    pub show_kernel_api: Option<bool>,
    pub max_depth: Option<usize>,
}

/// Scenario execution result
#[derive(Debug, Serialize)]
pub struct ScenarioResult {
    pub success: bool,
    pub path: Vec<ScenarioState>,
    pub annotated_flow_tree: Option<flowsight_core::FlowNode>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScenarioState {
    pub function: String,
    pub line: u32,
    pub variables: std::collections::HashMap<String, String>,
}

/// Execute scenario-based symbolic analysis
#[tauri::command]
pub async fn execute_scenario(
    file_path: String,
    scenario: ScenarioRequest,
) -> Result<ScenarioResult, String> {
    use flowsight_analysis::scenario::{
        Scenario, ScenarioExecutor, ScenarioOptions, SymbolicValue, ValueBinding,
    };

    let path = PathBuf::from(&file_path);

    // Parse file
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    // Read source for analysis
    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // Run analysis to get flow trees
    let mut analyzer = Analyzer::new();
    let analysis = analyzer
        .analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    // Find the flow tree for the entry function
    let entry_tree = analysis
        .flow_trees
        .iter()
        .find(|tree| tree.name == scenario.entry_function);

    let Some(entry_tree) = entry_tree else {
        return Ok(ScenarioResult {
            success: false,
            path: vec![],
            annotated_flow_tree: None,
            error: Some(format!(
                "Entry function '{}' not found in flow trees",
                scenario.entry_function
            )),
        });
    };

    // Convert bindings
    let bindings: Vec<ValueBinding> = scenario
        .bindings
        .iter()
        .map(|b| ValueBinding {
            path: b.path.clone(),
            value: SymbolicValue::parse(&b.value, &b.value_type),
        })
        .collect();

    // Build scenario
    let opts = scenario.options.as_ref();
    let options = ScenarioOptions {
        follow_async: opts.and_then(|o| o.follow_async).unwrap_or(true),
        show_kernel_api: opts.and_then(|o| o.show_kernel_api).unwrap_or(true),
        max_depth: opts.and_then(|o| o.max_depth).unwrap_or(10),
        multi_path: false,
        max_paths: 10,
        propagate_constraints: true,
        record_traces: true,
    };

    let scenario_config = Scenario {
        name: scenario.name,
        entry_function: scenario.entry_function,
        bindings,
        options: options.clone(),
    };

    // Execute scenario
    let mut executor = ScenarioExecutor::new(options);
    let result = executor.execute(&scenario_config, entry_tree);

    // Convert path steps to states
    let states: Vec<ScenarioState> = result
        .primary_path
        .steps
        .iter()
        .map(|s| ScenarioState {
            function: s.function.clone(),
            line: s.location.as_ref().map(|l| l.line).unwrap_or(0),
            variables: std::collections::HashMap::new(),
        })
        .collect();

    Ok(ScenarioResult {
        success: true,
        path: states,
        annotated_flow_tree: result.annotated_tree,
        error: None,
    })
}

// ============================================================================
// ExecutionFlow API (Phase 2)
// ============================================================================

/// Options for building ExecutionFlow
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionFlowOptions {
    /// Maximum depth to traverse
    pub max_depth: Option<usize>,
    /// Whether to include kernel call chains from knowledge base
    pub include_kernel_chains: Option<bool>,
    /// Whether to expand async callbacks
    pub expand_async: Option<bool>,
}

/// Flattened execution flow for frontend visualization (ReactFlow compatible)
#[derive(Debug, Serialize)]
pub struct FlatExecutionFlow {
    /// Entry function name
    pub entry_function: String,
    /// Flat list of nodes
    pub nodes: Vec<FlowGraphNode>,
    /// Flat list of edges
    pub edges: Vec<FlowGraphEdge>,
    /// Analysis metadata
    pub analysis_info: FlowAnalysisInfo,
}

/// Node in the flat graph structure
#[derive(Debug, Serialize)]
pub struct FlowGraphNode {
    /// Unique node ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Node type: "entry", "function", "async", "callback", "kernel", "separator"
    pub node_type: String,
    /// Source line number (0 if unknown)
    pub line: u32,
    /// Optional description
    pub description: Option<String>,
    /// Execution context
    pub context: Option<String>,
    /// Can this function sleep?
    pub can_sleep: Option<bool>,
    /// Is this a kernel internal function?
    pub is_kernel: bool,
}

/// Edge in the flat graph structure
#[derive(Debug, Clone, Serialize)]
pub struct FlowGraphEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Edge type: "sync", "async"
    pub edge_type: String,
    /// Optional label for the edge
    pub label: Option<String>,
}

/// Simplified analysis info for frontend
#[derive(Debug, Serialize)]
pub struct FlowAnalysisInfo {
    pub source_file: Option<String>,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub async_boundaries: usize,
    pub warnings: Vec<String>,
}

/// Build an ExecutionFlow for a specific function
///
/// This command uses the new FlowBuilder to construct a complete
/// ExecutionFlow with async boundaries, confidence levels, and
/// knowledge base injection.
///
/// Returns a flattened structure suitable for ReactFlow visualization.
#[tauri::command]
pub async fn build_execution_flow(
    file_path: String,
    entry_function: String,
    options: Option<ExecutionFlowOptions>,
) -> Result<FlatExecutionFlow, String> {
    let path = PathBuf::from(&file_path);

    // Parse file
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    // Read source for analysis
    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // Run analysis to get async bindings and function info
    let mut analyzer = Analyzer::new();
    let _analysis = analyzer
        .analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    // Build FlowBuilder and register functions
    let mut builder = FlowBuilder::new();

    // Extract async bindings from source
    builder.extract_async_bindings(&source);

    // Register all parsed functions
    for (name, func) in &parse_result.functions {
        builder.register_function(BuilderFunctionInfo {
            name: name.clone(),
            location: func.location.clone(),
            calls: func.calls.clone(),
            is_kernel: func.attributes.contains(&"__init".to_string())
                || func.attributes.contains(&"__exit".to_string())
                || name.starts_with("__"),
        });
    }

    // Build options
    let opts = options.as_ref();
    let build_opts = BuildOptions {
        max_depth: opts.and_then(|o| o.max_depth).unwrap_or(50),
        include_kernel_chains: opts.and_then(|o| o.include_kernel_chains).unwrap_or(true),
        expand_async: opts.and_then(|o| o.expand_async).unwrap_or(true),
    };

    // Build execution flow (tree structure)
    let mut flow = builder.build(&entry_function, &build_opts);

    // Update source file in analysis info
    flow.analysis_info.source_file = Some(file_path);

    // Convert to flat structure for frontend
    let flat_flow = flatten_execution_flow(&flow);

    Ok(flat_flow)
}

/// Convert tree-structured ExecutionFlow to flat nodes/edges for ReactFlow
fn flatten_execution_flow(flow: &ExecutionFlow) -> FlatExecutionFlow {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut visited = std::collections::HashSet::new();

    // Recursively flatten the tree
    flatten_node(
        &flow.root,
        &mut nodes,
        &mut edges,
        &mut visited,
        None,
        false,
    );

    // Collect warnings as strings
    let warnings: Vec<String> = flow
        .analysis_info
        .warnings
        .iter()
        .map(|w| w.message.clone())
        .collect();

    FlatExecutionFlow {
        entry_function: flow.entry_function.clone(),
        nodes,
        edges: edges.clone(),
        analysis_info: FlowAnalysisInfo {
            source_file: flow.analysis_info.source_file.clone(),
            total_nodes: flow.analysis_info.total_nodes,
            total_edges: edges.len(),
            async_boundaries: flow.async_boundaries.len(),
            warnings,
        },
    }
}

/// Recursively flatten a FlowNode into nodes and edges
fn flatten_node(
    node: &flowsight_core::FlowNode,
    nodes: &mut Vec<FlowGraphNode>,
    edges: &mut Vec<FlowGraphEdge>,
    visited: &mut std::collections::HashSet<String>,
    parent_id: Option<&str>,
    is_async_edge: bool,
) {
    // Avoid duplicates
    if visited.contains(&node.id) {
        // Still add edge if there's a parent
        if let Some(parent) = parent_id {
            edges.push(FlowGraphEdge {
                source: parent.to_string(),
                target: node.id.clone(),
                edge_type: if is_async_edge { "async" } else { "sync" }.to_string(),
                label: None,
            });
        }
        return;
    }
    visited.insert(node.id.clone());

    // Determine node type string
    let node_type = match &node.node_type {
        flowsight_core::FlowNodeType::EntryPoint => "entry",
        flowsight_core::FlowNodeType::Function => "function",
        flowsight_core::FlowNodeType::AsyncCallback { .. } => "async",
        flowsight_core::FlowNodeType::KernelApi => "kernel",
        flowsight_core::FlowNodeType::External => "external",
        flowsight_core::FlowNodeType::Separator { .. } => "separator",
        flowsight_core::FlowNodeType::Branch { .. } => "branch",
    };

    // Determine execution context string
    let context = node.execution_context.as_ref().map(|ctx| {
        match ctx {
            flowsight_core::ExecutionContext::Process => "Process Context",
            flowsight_core::ExecutionContext::SoftIrq => "SoftIRQ Context",
            flowsight_core::ExecutionContext::HardIrq => "HardIRQ Context",
            flowsight_core::ExecutionContext::Unknown => "Unknown Context",
        }
        .to_string()
    });

    // Create the node
    let graph_node = FlowGraphNode {
        id: node.id.clone(),
        label: node.display_name.clone(),
        node_type: node_type.to_string(),
        line: node.location.as_ref().map(|l| l.line).unwrap_or(0),
        description: node.description.clone(),
        context,
        can_sleep: node.can_sleep,
        is_kernel: node.is_kernel_internal,
    };
    nodes.push(graph_node);

    // Add edge from parent
    if let Some(parent) = parent_id {
        edges.push(FlowGraphEdge {
            source: parent.to_string(),
            target: node.id.clone(),
            edge_type: if is_async_edge { "async" } else { "sync" }.to_string(),
            label: None,
        });
    }

    // Process children
    let mut next_is_async = false;
    for child in &node.children {
        // Check if this is a separator indicating async boundary
        if matches!(
            child.node_type,
            flowsight_core::FlowNodeType::Separator { .. }
        ) {
            next_is_async = true;
            // Still add the separator node
            flatten_node(child, nodes, edges, visited, Some(&node.id), false);
        } else {
            flatten_node(child, nodes, edges, visited, Some(&node.id), next_is_async);
            next_is_async = false;
        }
    }
}

/// Get the raw tree-structured ExecutionFlow (for advanced use cases)
#[tauri::command]
pub async fn build_execution_flow_tree(
    file_path: String,
    entry_function: String,
    options: Option<ExecutionFlowOptions>,
) -> Result<ExecutionFlow, String> {
    let path = PathBuf::from(&file_path);

    // Parse file
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    // Read source for analysis
    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // Run analysis
    let mut analyzer = Analyzer::new();
    let _analysis = analyzer
        .analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    // Build FlowBuilder and register functions
    let mut builder = FlowBuilder::new();
    builder.extract_async_bindings(&source);

    for (name, func) in &parse_result.functions {
        builder.register_function(BuilderFunctionInfo {
            name: name.clone(),
            location: func.location.clone(),
            calls: func.calls.clone(),
            is_kernel: func.attributes.contains(&"__init".to_string())
                || func.attributes.contains(&"__exit".to_string())
                || name.starts_with("__"),
        });
    }

    let opts = options.as_ref();
    let build_opts = BuildOptions {
        max_depth: opts.and_then(|o| o.max_depth).unwrap_or(50),
        include_kernel_chains: opts.and_then(|o| o.include_kernel_chains).unwrap_or(true),
        expand_async: opts.and_then(|o| o.expand_async).unwrap_or(true),
    };

    let mut flow = builder.build(&entry_function, &build_opts);
    flow.analysis_info.source_file = Some(file_path);

    Ok(flow)
}

// ============================================================================
// 执行流格式化命令 (AI 辅助)
// ============================================================================

/// 格式化选项
#[derive(Debug, Serialize, Deserialize)]
pub struct FormatOptions {
    /// 输出格式: "mermaid", "markdown", "ascii", "json"
    pub format: String,
    /// 是否包含内核内部节点
    pub include_kernel_internal: Option<bool>,
    /// 最大显示深度
    pub max_depth: Option<usize>,
}

/// 格式化后的执行流数据
#[derive(Debug, Serialize, Deserialize)]
pub struct FormattedFlow {
    /// 格式类型
    pub format: String,
    /// 格式化后的内容
    pub content: String,
    /// 入口函数
    pub entry_function: String,
    /// 摘要
    pub summary: String,
}

/// 将执行流格式化为指定格式
#[tauri::command]
pub async fn format_execution_flow(
    file_path: String,
    entry_function: String,
    options: FormatOptions,
) -> Result<FormattedFlow, String> {
    use flowsight_ai::FlowFormatter;

    // 先构建执行流
    let flow = build_execution_flow_tree(file_path, entry_function.clone(), None).await?;

    // 创建格式化器
    let mut formatter = FlowFormatter::new();

    if let Some(include) = options.include_kernel_internal {
        formatter = formatter.with_kernel_internal(include);
    }

    if let Some(depth) = options.max_depth {
        formatter = formatter.with_max_depth(depth);
    }

    // 根据格式生成输出
    let (content, summary) = match options.format.as_str() {
        "mermaid" => {
            let mermaid = formatter.to_mermaid(&flow);
            let display = formatter.to_display_json(&flow);
            (mermaid, display.summary)
        }
        "markdown" => {
            let markdown = formatter.to_markdown_table(&flow);
            let display = formatter.to_display_json(&flow);
            (markdown, display.summary)
        }
        "ascii" => {
            let ascii = formatter.to_ascii_tree(&flow);
            let display = formatter.to_display_json(&flow);
            (ascii, display.summary)
        }
        "json" => {
            let display = formatter.to_display_json(&flow);
            let json = serde_json::to_string_pretty(&display).map_err(|e| e.to_string())?;
            (json, display.summary)
        }
        _ => {
            return Err(format!("Unsupported format: {}", options.format));
        }
    };

    Ok(FormattedFlow {
        format: options.format,
        content,
        entry_function,
        summary,
    })
}

/// 获取执行流的展示数据 (用于前端渲染)
#[tauri::command]
pub async fn get_flow_display_data(
    file_path: String,
    entry_function: String,
) -> Result<flowsight_ai::DisplayFlowData, String> {
    use flowsight_ai::FlowFormatter;

    // 构建执行流
    let flow = build_execution_flow_tree(file_path, entry_function, None).await?;

    // 生成展示数据
    let formatter = FlowFormatter::new();
    let display_data = formatter.to_display_json(&flow);

    Ok(display_data)
}

/// Get list of entry points (callbacks, module init/exit) for a file
#[tauri::command]
pub async fn get_entry_points(file_path: String) -> Result<Vec<EntryPointInfo>, String> {
    let path = PathBuf::from(&file_path);

    // Parse file
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    // Read source for analysis
    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // Run analysis
    let mut analyzer = Analyzer::new();
    let analysis = analyzer
        .analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    // Build entry point info
    let entry_points: Vec<EntryPointInfo> = analysis
        .entry_points
        .iter()
        .map(|name| {
            let func = parse_result.functions.get(name);
            EntryPointInfo {
                name: name.clone(),
                kind: if let Some(f) = func {
                    if f.callback_context.is_some() {
                        f.callback_context.clone().unwrap_or("callback".into())
                    } else if f.attributes.contains(&"__init".to_string()) {
                        "module_init".into()
                    } else if f.attributes.contains(&"__exit".to_string()) {
                        "module_exit".into()
                    } else {
                        "function".into()
                    }
                } else {
                    "unknown".into()
                },
                line: func
                    .and_then(|f| f.location.as_ref().map(|l| l.line))
                    .unwrap_or(0),
            }
        })
        .collect();

    Ok(entry_points)
}

#[derive(Debug, Serialize)]
pub struct EntryPointInfo {
    pub name: String,
    pub kind: String,
    pub line: u32,
}

/// Get async bindings detected in a file
#[tauri::command]
pub async fn get_async_bindings(file_path: String) -> Result<Vec<AsyncBindingInfo>, String> {
    let path = PathBuf::from(&file_path);

    // Parse file
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    // Read source for analysis
    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // Run analysis
    let mut analyzer = Analyzer::new();
    let analysis = analyzer
        .analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    // Convert async bindings
    let bindings: Vec<AsyncBindingInfo> = analysis
        .async_bindings
        .iter()
        .map(|b| AsyncBindingInfo {
            variable: b.variable.clone(),
            handler: b.handler.clone(),
            mechanism: format!("{:?}", b.mechanism),
            context: format!("{:?}", b.context),
            bind_line: b.bind_location.as_ref().map(|l| l.line),
            trigger_lines: b.trigger_locations.iter().map(|l| l.line).collect(),
        })
        .collect();

    Ok(bindings)
}

#[derive(Debug, Serialize)]
pub struct AsyncBindingInfo {
    pub variable: String,
    pub handler: String,
    pub mechanism: String,
    pub context: String,
    pub bind_line: Option<u32>,
    pub trigger_lines: Vec<u32>,
}

// ============================================================
// AI Inference Commands
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AiExplanationResult {
    pub trigger_condition: String,
    pub business_meaning: String,
    pub execution_result: String,
    pub related_functions: Vec<String>,
    pub common_errors: Vec<String>,
    pub context_type: Option<String>,
    pub can_sleep: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiContextAnnotation {
    pub context_type: String,
    pub can_sleep: bool,
    pub timing: String,
    pub notes: Vec<String>,
}

/// Explain function business semantics using AI or knowledge base
#[tauri::command]
pub async fn explain_function(
    file_path: String,
    function_name: String,
) -> Result<AiExplanationResult, String> {
    // First try to match against knowledge base
    let common_explanations = get_common_explanation(&function_name);
    if let Some(explanation) = common_explanations {
        return Ok(explanation);
    }

    // Read source code
    let source = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;

    // Find function in source
    let function_code =
        extract_function_code(&source, &function_name).unwrap_or_else(|| source.clone());

    // For now, return a template-based explanation
    // TODO: Integrate with actual AI model when available
    Ok(AiExplanationResult {
        trigger_condition: format!("当 {} 被调用时", function_name),
        business_meaning: format!("函数 {} 执行其定义的操作", function_name),
        execution_result: "执行函数体中的代码逻辑".to_string(),
        related_functions: extract_called_functions(&function_code),
        common_errors: vec!["检查返回值".to_string(), "注意错误处理".to_string()],
        context_type: detect_context_type(&function_code),
        can_sleep: detect_can_sleep(&function_code),
    })
}

/// Get context annotation for async handler
#[tauri::command]
pub async fn get_context_annotation(
    mechanism: String,
    _handler_code: String,
) -> Result<AiContextAnnotation, String> {
    // Match based on mechanism type
    let (context_type, can_sleep, timing, notes) = match mechanism.as_str() {
        "WorkQueue" | "workqueue" => (
            "进程上下文 (Process)".to_string(),
            true,
            "内核调度器选择合适的时机".to_string(),
            vec![
                "可以睡眠".to_string(),
                "可以调用可能阻塞的函数".to_string(),
                "执行时间不应过长".to_string(),
            ],
        ),
        "Timer" | "timer" => (
            "软中断上下文 (SoftIRQ)".to_string(),
            false,
            "定时器到期时在软中断上下文中执行".to_string(),
            vec![
                "不可睡眠".to_string(),
                "快速执行完成".to_string(),
                "注意与其他软中断的并发".to_string(),
            ],
        ),
        "IRQ" | "irq" | "HardIRQ" => (
            "硬中断上下文 (HardIRQ)".to_string(),
            false,
            "硬件中断触发时立即执行".to_string(),
            vec![
                "禁止睡眠".to_string(),
                "尽快处理完成".to_string(),
                "不要调用可能阻塞的函数".to_string(),
                "使用 schedule_work() 延迟处理".to_string(),
            ],
        ),
        "Tasklet" | "tasklet" => (
            "软中断上下文 (SoftIRQ)".to_string(),
            false,
            "由软中断调度执行".to_string(),
            vec![
                "不可睡眠".to_string(),
                "同一 tasklet 不会并发执行".to_string(),
            ],
        ),
        "ThreadedIRQ" | "threaded_irq" => (
            "进程上下文 (Process)".to_string(),
            true,
            "由内核线程执行".to_string(),
            vec!["可以睡眠".to_string(), "适合复杂的中断处理".to_string()],
        ),
        _ => (
            "未知上下文".to_string(),
            false,
            "根据具体情况确定".to_string(),
            vec!["请查阅相关文档".to_string()],
        ),
    };

    Ok(AiContextAnnotation {
        context_type,
        can_sleep,
        timing,
        notes,
    })
}

/// Translate constraint condition to business meaning
#[tauri::command]
pub async fn translate_condition(
    _code: String,
    constraint: String,
    function_name: String,
) -> Result<String, String> {
    // Simple template-based translation
    // TODO: Integrate with AI model for complex cases

    let translation =
        if constraint.contains("NULL") || constraint.contains("!") && constraint.contains("ptr") {
            format!("检查 {} 中的指针是否有效", function_name)
        } else if constraint.contains("< 0") || constraint.contains("ret") {
            "检查操作是否成功（负值表示错误）".to_string()
        } else if constraint.contains("== 0") {
            "检查条件是否满足（零值通常表示成功或假）".to_string()
        } else if constraint.contains("&&") || constraint.contains("||") {
            "复合条件检查，需要同时满足多个条件".to_string()
        } else {
            format!("条件: {}", constraint)
        };

    Ok(translation)
}

// Helper functions

fn get_common_explanation(function_name: &str) -> Option<AiExplanationResult> {
    // Common kernel function patterns
    if function_name.ends_with("_probe") || function_name.contains("probe") {
        return Some(AiExplanationResult {
            trigger_condition: "设备与驱动匹配时".to_string(),
            business_meaning: "驱动程序的设备探测函数，负责初始化设备".to_string(),
            execution_result: "分配设备资源，初始化硬件，注册设备".to_string(),
            related_functions: vec![
                "devm_kzalloc".to_string(),
                "platform_get_resource".to_string(),
                "devm_request_irq".to_string(),
            ],
            common_errors: vec![
                "内存分配失败".to_string(),
                "资源获取失败".to_string(),
                "中断注册失败".to_string(),
            ],
            context_type: Some("进程上下文 (Process)".to_string()),
            can_sleep: Some(true),
        });
    }

    if function_name.ends_with("_remove") {
        return Some(AiExplanationResult {
            trigger_condition: "设备移除或驱动卸载时".to_string(),
            business_meaning: "驱动程序的设备移除函数，负责清理资源".to_string(),
            execution_result: "释放设备资源，注销设备".to_string(),
            related_functions: vec!["device_unregister".to_string(), "free_irq".to_string()],
            common_errors: vec!["资源释放顺序错误".to_string(), "遗漏资源释放".to_string()],
            context_type: Some("进程上下文 (Process)".to_string()),
            can_sleep: Some(true),
        });
    }

    if function_name.contains("irq") || function_name.contains("interrupt") {
        return Some(AiExplanationResult {
            trigger_condition: "硬件中断发生时".to_string(),
            business_meaning: "中断处理函数，响应硬件事件".to_string(),
            execution_result: "处理中断，可能调度后续工作".to_string(),
            related_functions: vec!["schedule_work".to_string(), "tasklet_schedule".to_string()],
            common_errors: vec![
                "处理时间过长".to_string(),
                "调用了可能睡眠的函数".to_string(),
            ],
            context_type: Some("硬中断上下文 (HardIRQ)".to_string()),
            can_sleep: Some(false),
        });
    }

    if function_name.contains("work") || function_name.ends_with("_fn") {
        return Some(AiExplanationResult {
            trigger_condition: "工作队列调度执行时".to_string(),
            business_meaning: "延迟执行的工作函数，处理复杂任务".to_string(),
            execution_result: "执行延迟处理的任务".to_string(),
            related_functions: vec!["schedule_work".to_string(), "queue_work".to_string()],
            common_errors: vec!["访问已释放的资源".to_string(), "并发访问问题".to_string()],
            context_type: Some("进程上下文 (Process)".to_string()),
            can_sleep: Some(true),
        });
    }

    None
}

fn extract_function_code(source: &str, function_name: &str) -> Option<String> {
    // Simple extraction - find function and extract until closing brace
    let pattern = format!(
        r"(?s)(\w+\s+)?{}\s*\([^)]*\)\s*\{{",
        regex::escape(function_name)
    );
    let re = regex::Regex::new(&pattern).ok()?;

    if let Some(mat) = re.find(source) {
        let start = mat.start();
        let mut brace_count = 0;
        let mut end = start;

        for (i, c) in source[start..].char_indices() {
            match c {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        return Some(source[start..end].to_string());
    }

    None
}

fn extract_called_functions(code: &str) -> Vec<String> {
    // Extract function calls from code
    let re = regex::Regex::new(r"(\w+)\s*\(").unwrap();
    let mut functions: Vec<String> = re
        .captures_iter(code)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter(|name| {
            !["if", "while", "for", "switch", "return", "sizeof"].contains(&name.as_str())
        })
        .collect();

    functions.sort();
    functions.dedup();
    functions.truncate(10); // Limit to 10 functions
    functions
}

fn detect_context_type(code: &str) -> Option<String> {
    if code.contains("irqreturn_t") || code.contains("request_irq") {
        Some("硬中断上下文 (HardIRQ)".to_string())
    } else if code.contains("work_struct") || code.contains("INIT_WORK") {
        Some("进程上下文 (Process)".to_string())
    } else if code.contains("tasklet") {
        Some("软中断上下文 (SoftIRQ)".to_string())
    } else if code.contains("timer_list") || code.contains("timer_setup") {
        Some("软中断上下文 (SoftIRQ)".to_string())
    } else {
        None
    }
}

fn detect_can_sleep(code: &str) -> Option<bool> {
    // Check for sleep-incompatible patterns
    if code.contains("irqreturn_t") || code.contains("in_interrupt") {
        return Some(false);
    }
    if code.contains("spin_lock") && !code.contains("spin_unlock_irqrestore") {
        return Some(false);
    }
    if code.contains("work_struct") || code.contains("kthread") {
        return Some(true);
    }
    None
}

// ============================================================
// Knowledge Base API
// ============================================================

/// Knowledge info for a symbol (function or API)
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeInfo {
    /// Symbol name
    pub name: String,
    /// Description from knowledge base
    pub description: Option<String>,
    /// Execution context (process/softirq/hardirq)
    pub context: Option<String>,
    /// Whether this function can sleep
    pub can_sleep: Option<bool>,
    /// Trigger condition (for callbacks)
    pub trigger: Option<String>,
    /// Function signature
    pub signature: Option<String>,
    /// Call chain from kernel entry to user code
    pub call_chain: Option<Vec<CallChainNodeInfo>>,
    /// Related examples
    pub examples: Option<Vec<String>>,
    /// Framework name (e.g., "usb_driver", "file_operations")
    pub framework: Option<String>,
    /// Callback name within framework
    pub callback_type: Option<String>,
    /// Developer notes/warnings
    pub notes: Option<Vec<String>>,
}

/// Single node in a call chain
#[derive(Debug, Serialize, Deserialize)]
pub struct CallChainNodeInfo {
    /// Function name
    pub function: String,
    /// Source file path
    pub file: Option<String>,
    /// Execution context at this point
    pub context: String,
    /// Description of this step
    pub description: Option<String>,
    /// Whether this is the user code entry point
    pub is_user_entry: bool,
}

/// Async pattern info
#[derive(Debug, Serialize, Deserialize)]
pub struct AsyncPatternInfo {
    /// Pattern name (e.g., "work_struct", "timer_list")
    pub name: String,
    /// Description
    pub description: String,
    /// Execution context
    pub context: String,
    /// Whether handler can sleep
    pub can_sleep: bool,
    /// Handler signature
    pub handler_signature: Option<String>,
    /// Bind patterns (regex)
    pub bind_patterns: Vec<String>,
    /// Trigger patterns (regex)
    pub trigger_patterns: Vec<String>,
    /// Call chain for handler execution
    pub handler_call_chain: Option<Vec<CallChainNodeInfo>>,
}

/// Get knowledge info for a symbol (function, API, callback)
///
/// Looks up the symbol in the knowledge base and returns
/// comprehensive information including execution context,
/// call chains, and developer notes.
#[tauri::command]
pub async fn get_knowledge_info(
    symbol: String,
    code_context: Option<String>,
) -> Result<Option<KnowledgeInfo>, String> {
    use flowsight_knowledge::{ExecutionContext, KnowledgeBase};

    let kb = KnowledgeBase::builtin();
    let code_ctx = code_context.unwrap_or_default();

    // 1. Check if it's a kernel API
    if let Some(api) = kb.get_api(&symbol) {
        return Ok(Some(KnowledgeInfo {
            name: symbol.clone(),
            description: Some(api.description.clone()),
            context: Some(if api.can_sleep { "process" } else { "any" }.into()),
            can_sleep: Some(api.can_sleep),
            trigger: None,
            signature: None,
            call_chain: None,
            examples: None,
            framework: None,
            callback_type: None,
            notes: if api.can_fail {
                Some(vec!["This function can fail - check return value".into()])
            } else {
                None
            },
        }));
    }

    // 2. Try to identify as framework callback
    if let Some((fw_name, cb_name, callback)) = kb.identify_callback(&symbol, &code_ctx) {
        let call_chain = callback.call_chain.as_ref().map(|chain| {
            chain
                .nodes
                .iter()
                .map(|node| CallChainNodeInfo {
                    function: node.function.clone(),
                    file: node.file.clone(),
                    context: format!("{:?}", node.context),
                    description: node.description.clone(),
                    is_user_entry: node.is_user_entry,
                })
                .collect()
        });

        return Ok(Some(KnowledgeInfo {
            name: symbol.clone(),
            description: Some(callback.description.clone()),
            context: Some(match callback.context {
                ExecutionContext::Process => "process".into(),
                ExecutionContext::SoftIrq => "softirq".into(),
                ExecutionContext::HardIrq => "hardirq".into(),
                ExecutionContext::User => "user".into(),
                ExecutionContext::Unknown => "unknown".into(),
            }),
            can_sleep: Some(callback.context.can_sleep()),
            trigger: Some(callback.trigger.clone()),
            signature: callback.signature.clone(),
            call_chain,
            examples: None,
            framework: Some(fw_name.to_string()),
            callback_type: Some(cb_name.to_string()),
            notes: None,
        }));
    }

    // 3. Check frameworks by common patterns
    let frameworks_to_check = [
        ("usb_driver", "probe"),
        ("usb_driver", "disconnect"),
        ("file_operations", "open"),
        ("file_operations", "read"),
        ("file_operations", "write"),
        ("file_operations", "release"),
        ("platform_driver", "probe"),
        ("platform_driver", "remove"),
    ];

    for (fw_name, cb_name) in &frameworks_to_check {
        if symbol.contains(cb_name) {
            if let Some(callback) = kb.get_callback(fw_name, cb_name) {
                let call_chain = callback.call_chain.as_ref().map(|chain| {
                    chain
                        .nodes
                        .iter()
                        .map(|node| CallChainNodeInfo {
                            function: node.function.clone(),
                            file: node.file.clone(),
                            context: format!("{:?}", node.context),
                            description: node.description.clone(),
                            is_user_entry: node.is_user_entry,
                        })
                        .collect()
                });

                return Ok(Some(KnowledgeInfo {
                    name: symbol.clone(),
                    description: Some(callback.description.clone()),
                    context: Some(match callback.context {
                        ExecutionContext::Process => "process".into(),
                        ExecutionContext::SoftIrq => "softirq".into(),
                        ExecutionContext::HardIrq => "hardirq".into(),
                        ExecutionContext::User => "user".into(),
                        ExecutionContext::Unknown => "unknown".into(),
                    }),
                    can_sleep: Some(callback.context.can_sleep()),
                    trigger: Some(callback.trigger.clone()),
                    signature: callback.signature.clone(),
                    call_chain,
                    examples: None,
                    framework: Some(fw_name.to_string()),
                    callback_type: Some(cb_name.to_string()),
                    notes: None,
                }));
            }
        }
    }

    Ok(None)
}

/// Get async pattern information
#[tauri::command]
pub async fn get_async_pattern_info(
    pattern_name: String,
) -> Result<Option<AsyncPatternInfo>, String> {
    use flowsight_knowledge::KnowledgeBase;

    let kb = KnowledgeBase::builtin();

    if let Some(pattern) = kb.get_async_pattern(&pattern_name) {
        let handler_call_chain = pattern.handler_call_chain.as_ref().map(|chain| {
            chain
                .nodes
                .iter()
                .map(|node| CallChainNodeInfo {
                    function: node.function.clone(),
                    file: node.file.clone(),
                    context: format!("{:?}", node.context),
                    description: node.description.clone(),
                    is_user_entry: node.is_user_entry,
                })
                .collect()
        });

        return Ok(Some(AsyncPatternInfo {
            name: pattern_name,
            description: pattern.description.clone(),
            context: format!("{:?}", pattern.context),
            can_sleep: pattern.context.can_sleep(),
            handler_signature: pattern.handler_signature.clone(),
            bind_patterns: pattern.bind_patterns.clone(),
            trigger_patterns: pattern.trigger_patterns.clone(),
            handler_call_chain,
        }));
    }

    Ok(None)
}

/// List all available frameworks in knowledge base
#[tauri::command]
pub async fn list_frameworks() -> Result<Vec<FrameworkSummary>, String> {
    use flowsight_knowledge::KnowledgeBase;

    let kb = KnowledgeBase::builtin();

    let frameworks: Vec<FrameworkSummary> = kb
        .frameworks
        .iter()
        .map(|(name, fw)| FrameworkSummary {
            name: name.clone(),
            description: fw.description.clone(),
            header: fw.header.clone(),
            callback_count: fw.callbacks.len(),
            callbacks: fw.callbacks.keys().cloned().collect(),
        })
        .collect();

    Ok(frameworks)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameworkSummary {
    pub name: String,
    pub description: String,
    pub header: Option<String>,
    pub callback_count: usize,
    pub callbacks: Vec<String>,
}

/// List all async patterns
#[tauri::command]
pub async fn list_async_patterns() -> Result<Vec<AsyncPatternSummary>, String> {
    use flowsight_knowledge::KnowledgeBase;

    let kb = KnowledgeBase::builtin();

    let patterns: Vec<AsyncPatternSummary> = kb
        .async_patterns
        .iter()
        .map(|(name, pattern)| AsyncPatternSummary {
            name: name.clone(),
            description: pattern.description.clone(),
            context: format!("{:?}", pattern.context),
            can_sleep: pattern.context.can_sleep(),
        })
        .collect();

    Ok(patterns)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AsyncPatternSummary {
    pub name: String,
    pub description: String,
    pub context: String,
    pub can_sleep: bool,
}

// ============================================================
// Extended Function Detail API
// ============================================================

/// Get detailed function information from file
///
/// This command parses the file directly and extracts comprehensive
/// function information including local variables, complexity, and doc comments.
#[tauri::command]
pub async fn get_function_detail_from_file(
    file_path: String,
    function_name: String,
) -> Result<Option<FunctionDetailExt>, String> {
    let path = PathBuf::from(&file_path);

    // Read source file
    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // Parse file
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path).map_err(|e| e.to_string())?;

    // Run analysis to get async info
    let mut analyzer = Analyzer::new();
    let _ = analyzer
        .analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    // Find the target function
    let func = match parse_result.functions.get(&function_name) {
        Some(f) => f,
        None => return Ok(None),
    };

    // Extract function code for additional analysis
    let (func_code, start_line, end_line) =
        extract_function_code_with_lines(&source, &function_name).unwrap_or((
            String::new(),
            func.location.as_ref().map(|l| l.line).unwrap_or(0),
            0,
        ));

    // Calculate end_line if not found
    let end_line = if end_line > 0 {
        end_line
    } else {
        start_line + count_lines(&func_code)
    };

    // Find called_by - functions in this file that call our target function
    let called_by: Vec<String> = parse_result
        .functions
        .iter()
        .filter(|(name, f)| *name != &function_name && f.calls.contains(&function_name))
        .map(|(name, _)| name.clone())
        .collect();

    // Extract local variables from function body
    let local_variables = extract_local_variables(&func_code);

    // Calculate cyclomatic complexity
    let complexity = calculate_complexity(&func_code);

    // Extract doc comment
    let doc_comment = extract_doc_comment(&source, start_line);

    Ok(Some(FunctionDetailExt {
        name: func.name.clone(),
        return_type: func.return_type.clone(),
        params: func
            .params
            .iter()
            .map(|p| ParamInfo {
                name: p.name.clone(),
                type_name: p.type_name.clone(),
            })
            .collect(),
        file: file_path,
        line: start_line,
        end_line,
        is_callback: func.is_callback,
        callback_context: func.callback_context.clone(),
        calls: func.calls.clone(),
        called_by,
        local_variables,
        complexity,
        doc_comment,
    }))
}

/// Extract function code with start and end line numbers
fn extract_function_code_with_lines(
    source: &str,
    function_name: &str,
) -> Option<(String, u32, u32)> {
    let pattern = format!(
        r"(?s)(\w+\s+)?{}\s*\([^)]*\)\s*\{{",
        regex::escape(function_name)
    );
    let re = regex::Regex::new(&pattern).ok()?;

    if let Some(mat) = re.find(source) {
        let start = mat.start();
        let mut brace_count = 0;
        let mut end = start;

        for (i, c) in source[start..].char_indices() {
            match c {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        // Calculate line numbers
        let start_line = source[..start].lines().count() as u32 + 1;
        let end_line = source[..end].lines().count() as u32;

        return Some((source[start..end].to_string(), start_line, end_line));
    }

    None
}

/// Count lines in a string
fn count_lines(s: &str) -> u32 {
    s.lines().count() as u32
}

/// Extract local variables from function code
fn extract_local_variables(func_code: &str) -> Vec<LocalVarInfo> {
    let mut vars = Vec::new();

    // Pattern for common variable declarations
    // Matches: type name; or type name = ...; or type *name; etc.
    let re = regex::Regex::new(
        r"(?m)^\s*((?:const\s+|static\s+|volatile\s+|unsigned\s+|signed\s+|long\s+|short\s+)*(?:int|char|void|bool|size_t|ssize_t|u8|u16|u32|u64|s8|s16|s32|s64|__[a-z0-9_]+|struct\s+\w+))\s*(\**)(\w+)\s*(?:=|;|\[)"
    ).ok();

    if let Some(re) = re {
        for cap in re.captures_iter(func_code) {
            if let (Some(type_match), Some(ptr_match), Some(name_match)) =
                (cap.get(1), cap.get(2), cap.get(3))
            {
                let type_name = format!("{}{}", type_match.as_str().trim(), ptr_match.as_str());
                let name = name_match.as_str().to_string();

                // Skip common non-variable patterns
                if ![
                    "if", "while", "for", "switch", "return", "goto", "sizeof", "typeof",
                ]
                .contains(&name.as_str())
                {
                    vars.push(LocalVarInfo { name, type_name });
                }
            }
        }
    }

    vars
}

/// Calculate cyclomatic complexity (McCabe)
/// Complexity = E - N + 2P = decision points + 1
fn calculate_complexity(func_code: &str) -> u32 {
    let mut complexity: u32 = 1; // Base complexity

    // Count decision points
    let decision_patterns = [
        r"\bif\s*\(",     // if statements
        r"\belse\s+if\b", // else if (don't double count)
        r"\bwhile\s*\(",  // while loops
        r"\bfor\s*\(",    // for loops
        r"\bcase\s+",     // case labels
        r"\bdefault\s*:", // default label
        r"\bcatch\s*\(",  // catch blocks (if any)
        r"\?\s*[^:]+:",   // ternary operators
        r"\|\|",          // logical OR (short-circuit)
        r"&&",            // logical AND (short-circuit)
    ];

    for pattern in &decision_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            complexity += re.find_iter(func_code).count() as u32;
        }
    }

    // Note: else if is already handled by the pattern list
    // The "\belse\s+if\b" pattern counts it once, and "\bif\s*\("
    // doesn't match "else if" due to the word boundary

    complexity
}

/// Extract documentation comment before a function
fn extract_doc_comment(source: &str, func_start_line: u32) -> Option<String> {
    if func_start_line == 0 {
        return None;
    }

    let lines: Vec<&str> = source.lines().collect();
    let start_idx = (func_start_line as usize).saturating_sub(1);

    if start_idx == 0 {
        return None;
    }

    let mut comment_lines = Vec::new();
    let mut in_block_comment = false;

    // Scan backwards from function to find comment
    for i in (0..start_idx).rev() {
        let line = lines.get(i)?.trim();

        // Check for end of block comment (scanning backwards)
        if line.ends_with("*/") {
            in_block_comment = true;
            let content = line.trim_end_matches("*/").trim();
            if !content.is_empty() {
                comment_lines.push(content.to_string());
            }
            continue;
        }

        if in_block_comment {
            // Check for start of block comment
            if line.starts_with("/*") || line.starts_with("/**") {
                let content = line
                    .trim_start_matches("/**")
                    .trim_start_matches("/*")
                    .trim();
                if !content.is_empty() {
                    comment_lines.push(content.to_string());
                }
                break;
            }

            // Middle line of block comment
            let content = line.trim_start_matches('*').trim();
            comment_lines.push(content.to_string());
            continue;
        }

        // Single line comment
        if line.starts_with("//") {
            let content = line.trim_start_matches('/').trim();
            comment_lines.push(content.to_string());
            continue;
        }

        // Non-comment, non-empty line - stop scanning
        if !line.is_empty() {
            break;
        }
    }

    if comment_lines.is_empty() {
        return None;
    }

    // Reverse since we scanned backwards
    comment_lines.reverse();

    // Filter out empty lines at start/end and join
    let result: Vec<&str> = comment_lines
        .iter()
        .map(|s| s.as_str())
        .skip_while(|s| s.is_empty())
        .collect();

    let result: String = result.join("\n").trim().to_string();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

// ============================================================
// LLVM IR Commands
// ============================================================

/// LLVM IR function for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct LlvmIrFunction {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<LlvmIrParameter>,
    pub blocks: Vec<LlvmIrBasicBlock>,
    pub is_callback: bool,
    pub callback_context: Option<String>,
}

/// LLVM IR parameter
#[derive(Debug, Serialize, Deserialize)]
pub struct LlvmIrParameter {
    pub name: String,
    pub type_str: String,
}

/// LLVM IR basic block
#[derive(Debug, Serialize, Deserialize)]
pub struct LlvmIrBasicBlock {
    pub name: String,
    pub instructions: Vec<LlvmIrInstruction>,
    pub predecessors: Vec<String>,
    pub successors: Vec<String>,
    pub terminator: Option<LlvmIrInstruction>,
}

/// LLVM IR instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmIrInstruction {
    pub opcode: String,
    pub dest: Option<String>,
    pub type_str: String,
    pub operands: Vec<String>,
    pub location: Option<SourceLocation>,
}

/// Source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
}

/// LLVM IR parse result for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct LlvmIrResult {
    pub module_name: String,
    pub functions: std::collections::HashMap<String, LlvmIrFunction>,
}

/// Generate LLVM IR from a C source file using clang
///
/// This command compiles a C file to LLVM IR using clang and returns the parsed result.
/// Requires clang to be installed on the system.
#[tauri::command]
pub async fn generate_llvm_ir(file_path: String) -> Result<LlvmIrResult, String> {
    use std::process::Command;

    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Create temp file for IR output
    let temp_dir = std::env::temp_dir();
    let ir_file = temp_dir.join(format!(
        "{}.ll",
        path.file_stem().unwrap_or_default().to_string_lossy()
    ));

    // Find clang - try common paths
    let clang_paths = [
        "clang",
        "/usr/bin/clang",
        "/usr/local/bin/clang",
        "/opt/homebrew/bin/clang",
        "/opt/homebrew/opt/llvm/bin/clang",
    ];

    let mut clang_found = None;
    for clang in &clang_paths {
        if Command::new(clang).arg("--version").output().is_ok() {
            clang_found = Some(*clang);
            break;
        }
    }

    let clang = clang_found.ok_or_else(|| {
        "clang not found. Please install LLVM/clang to enable LLVM IR generation.".to_string()
    })?;

    // Compile to LLVM IR
    // Use -emit-llvm -S to get text IR (.ll file)
    // Note: Full kernel compilation requires kconfig, generated headers, etc.
    // We use minimal flags to extract function structure without full compilation.
    let kernel_base = "/Users/sky/linux-kernel/linux";

    let output = Command::new(clang)
        .args([
            "-emit-llvm",
            "-S",
            "-O0",                      // No optimization to preserve structure
            "-g",                       // Debug info for source locations
            "-fno-discard-value-names", // Preserve variable names
            "-Wno-everything",          // Suppress warnings for kernel code
            "-nostdinc",                // Don't use standard includes
            "-isystem",
            "/opt/homebrew/opt/llvm/lib/clang/19/include", // Clang builtins only
            // Kernel include paths (order matters!)
            "-I",
            &format!("{}/arch/x86/include", kernel_base),
            "-I",
            &format!("{}/arch/x86/include/generated", kernel_base),
            "-I",
            &format!("{}/include", kernel_base),
            "-I",
            &format!("{}/arch/x86/include/uapi", kernel_base),
            "-I",
            &format!("{}/include/uapi", kernel_base),
            "-I",
            &format!("{}/include/generated/uapi", kernel_base),
            // Kernel defines
            "-D",
            "__KERNEL__",
            "-D",
            "MODULE",
            "-D",
            "CONFIG_X86_64",
            "-D",
            "__x86_64__",
            // Output
            "-o",
            ir_file.to_str().unwrap(),
            file_path.as_str(),
        ])
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !output.status.success() {
        // Return a mock result with error info when clang fails
        // This allows the UI to still function
        return Ok(LlvmIrResult {
            module_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            functions: std::collections::HashMap::from([(
                "__compilation_error__".to_string(),
                LlvmIrFunction {
                    name: "__compilation_error__".to_string(),
                    return_type: "void".to_string(),
                    parameters: vec![],
                    blocks: vec![LlvmIrBasicBlock {
                        name: "error".to_string(),
                        instructions: vec![LlvmIrInstruction {
                            opcode: "error".to_string(),
                            dest: None,
                            type_str: "".to_string(),
                            operands: vec![format!(
                                "Clang compilation failed:\n{}",
                                String::from_utf8_lossy(&output.stderr)
                            )],
                            location: None,
                        }],
                        predecessors: vec![],
                        successors: vec![],
                        terminator: None,
                    }],
                    is_callback: false,
                    callback_context: None,
                },
            )]),
        });
    }

    // Parse the generated IR file
    let parser = flowsight_llvm::LlvmParser::new();
    let parse_result = parser
        .parse_file(&ir_file)
        .map_err(|e| format!("Failed to parse LLVM IR: {}", e))?;

    // Convert to frontend format
    let frontend_result = flowsight_llvm::to_frontend_format(&parse_result);

    // Clean up temp file
    let _ = std::fs::remove_file(&ir_file);

    // Convert to our serializable types
    let functions: std::collections::HashMap<String, LlvmIrFunction> = frontend_result
        .functions
        .into_iter()
        .map(|(name, func)| {
            (
                name,
                LlvmIrFunction {
                    name: func.name,
                    return_type: func.return_type,
                    parameters: func
                        .parameters
                        .into_iter()
                        .map(|p| LlvmIrParameter {
                            name: p.name,
                            type_str: p.type_str,
                        })
                        .collect(),
                    blocks: func
                        .blocks
                        .into_iter()
                        .map(|b| LlvmIrBasicBlock {
                            name: b.name,
                            instructions: b
                                .instructions
                                .into_iter()
                                .map(|i| LlvmIrInstruction {
                                    opcode: i.opcode,
                                    dest: i.dest,
                                    type_str: i.type_str,
                                    operands: i.operands,
                                    location: i.location.map(|l| SourceLocation {
                                        file: l.file,
                                        line: l.line,
                                    }),
                                })
                                .collect(),
                            predecessors: b.predecessors,
                            successors: b.successors,
                            terminator: b.terminator.map(|t| LlvmIrInstruction {
                                opcode: t.opcode,
                                dest: t.dest,
                                type_str: t.type_str,
                                operands: t.operands,
                                location: t.location.map(|l| SourceLocation {
                                    file: l.file,
                                    line: l.line,
                                }),
                            }),
                        })
                        .collect(),
                    is_callback: func.is_callback,
                    callback_context: func.callback_context,
                },
            )
        })
        .collect();

    Ok(LlvmIrResult {
        module_name: frontend_result.module_name,
        functions,
    })
}

/// Parse existing LLVM IR file (.ll or .bc)
#[tauri::command]
pub async fn parse_llvm_ir_file(file_path: String) -> Result<LlvmIrResult, String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let parser = flowsight_llvm::LlvmParser::new();
    let parse_result = parser
        .parse_file(&path)
        .map_err(|e| format!("Failed to parse LLVM IR: {}", e))?;

    let frontend_result = flowsight_llvm::to_frontend_format(&parse_result);

    // Convert to our serializable types
    let functions: std::collections::HashMap<String, LlvmIrFunction> = frontend_result
        .functions
        .into_iter()
        .map(|(name, func)| {
            (
                name,
                LlvmIrFunction {
                    name: func.name,
                    return_type: func.return_type,
                    parameters: func
                        .parameters
                        .into_iter()
                        .map(|p| LlvmIrParameter {
                            name: p.name,
                            type_str: p.type_str,
                        })
                        .collect(),
                    blocks: func
                        .blocks
                        .into_iter()
                        .map(|b| LlvmIrBasicBlock {
                            name: b.name,
                            instructions: b
                                .instructions
                                .into_iter()
                                .map(|i| LlvmIrInstruction {
                                    opcode: i.opcode,
                                    dest: i.dest,
                                    type_str: i.type_str,
                                    operands: i.operands,
                                    location: i.location.map(|l| SourceLocation {
                                        file: l.file,
                                        line: l.line,
                                    }),
                                })
                                .collect(),
                            predecessors: b.predecessors,
                            successors: b.successors,
                            terminator: b.terminator.map(|t| LlvmIrInstruction {
                                opcode: t.opcode,
                                dest: t.dest,
                                type_str: t.type_str,
                                operands: t.operands,
                                location: t.location.map(|l| SourceLocation {
                                    file: l.file,
                                    line: l.line,
                                }),
                            }),
                        })
                        .collect(),
                    is_callback: func.is_callback,
                    callback_context: func.callback_context,
                },
            )
        })
        .collect();

    Ok(LlvmIrResult {
        module_name: frontend_result.module_name,
        functions,
    })
}
