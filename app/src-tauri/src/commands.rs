//! Tauri Commands

use flowsight_analysis::Analyzer;
use flowsight_index::SymbolIndex;
use flowsight_parser::get_parser;
use flowsight_parser::parallel::{ParallelParser, ProgressPhase};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
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

/// Global index state
static INDEX: Lazy<Mutex<SymbolIndex>> = Lazy::new(|| Mutex::new(SymbolIndex::new()));

/// Project information
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub files_count: usize,
    pub functions_count: usize,
    pub structs_count: usize,
    pub indexed: bool,
}

/// Search result
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub kind: String, // "function" or "struct"
    pub file: Option<String>,
    pub line: Option<u32>,
    pub is_callback: bool,
}

/// Open a project directory - returns immediately, indexing happens in background
#[tauri::command]
pub async fn open_project(path: String, app_handle: tauri::AppHandle) -> Result<ProjectInfo, String> {
    let project_path = PathBuf::from(&path);

    if !project_path.is_dir() {
        return Err("Path is not a directory".into());
    }

    // Clear previous index
    {
        let mut index = INDEX.lock().map_err(|e| e.to_string())?;
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
    let _ = app_handle.emit("index-progress", serde_json::json!({
        "phase": "scanning",
        "current": 0,
        "total": 0,
        "message": "Scanning files..."
    }));

    // Scan files
    let mut c_files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&project_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map(|ext| ext == "c" || ext == "h").unwrap_or(false) {
            c_files.push(entry.path().to_path_buf());
            if c_files.len() % 2000 == 0 {
                let _ = app_handle.emit("index-progress", serde_json::json!({
                    "phase": "scanning",
                    "current": c_files.len(),
                    "total": 0,
                    "message": format!("Found {} files...", c_files.len())
                }));
            }
        }
    }

    let total = c_files.len();
    let _ = app_handle.emit("index-progress", serde_json::json!({
        "phase": "parsing",
        "current": 0,
        "total": total,
        "message": format!("Parsing {} files...", total)
    }));

    // Parse in parallel
    let parallel_parser = ParallelParser::new();
    let results = parallel_parser.parse_files(&c_files);

    let _ = app_handle.emit("index-progress", serde_json::json!({
        "phase": "indexing",
        "current": 0,
        "total": total,
        "message": "Building index..."
    }));

    // Build index
    if let Ok(mut index) = INDEX.lock() {
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
                let _ = app_handle.emit("index-progress", serde_json::json!({
                    "phase": "indexing",
                    "current": i,
                    "total": total,
                    "message": format!("Indexed {}/{}", i, total)
                }));
            }
        }

        let stats = index.stats();
        let _ = app_handle.emit("index-progress", serde_json::json!({
            "phase": "done",
            "current": total,
            "total": total,
            "files": total,
            "functions": stats.total_functions,
            "structs": stats.total_structs,
            "message": format!("Done! {} files, {} functions", total, stats.total_functions)
        }));
    }
}

/// Search for symbols in the index
#[tauri::command]
pub async fn search_symbols(query: String) -> Result<Vec<SearchResult>, String> {
    let index = INDEX.lock().map_err(|e| e.to_string())?;
    let query_lower = query.to_lowercase();

    let mut results = Vec::new();

    // Search functions
    for (name, func) in &index.functions {
        if name.to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                name: name.clone(),
                kind: "function".into(),
                file: func.location.as_ref().map(|l| l.file.clone()),
                line: func.location.as_ref().map(|l| l.line),
                is_callback: func.is_callback,
            });
        }
    }

    // Search structs
    for (name, st) in &index.structs {
        if name.to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                name: name.clone(),
                kind: "struct".into(),
                file: st.location.as_ref().map(|l| l.file.clone()),
                line: st.location.as_ref().map(|l| l.line),
                is_callback: false,
            });
        }
    }

    // Limit results
    results.truncate(50);

    Ok(results)
}

/// Get index statistics
#[tauri::command]
pub async fn get_index_stats() -> Result<IndexStats, String> {
    let index = INDEX.lock().map_err(|e| e.to_string())?;
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

/// Get function detail from index
#[tauri::command]
pub async fn get_function_detail(name: String) -> Result<Option<FunctionDetail>, String> {
    let index = INDEX.lock().map_err(|e| e.to_string())?;

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
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
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
    let index = INDEX.lock().map_err(|e| e.to_string())?;
    
    let mut callers = Vec::new();
    
    // Search through all functions to find callers
    for (name, func) in &index.functions {
        // Check if this function calls the target
        if func.calls.contains(&function_name) {
            let call_type = if func.is_callback {
                "async"
            } else {
                "direct"
            };
            
            callers.push(CallerInfo {
                name: name.clone(),
                file: func.location.as_ref().map(|l| l.file.clone()).unwrap_or_default(),
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
    use flowsight_analysis::scenario::{Scenario, ScenarioExecutor, ScenarioOptions, SymbolicValue, ValueBinding};
    
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
    let entry_tree = analysis.flow_trees.iter()
        .find(|tree| tree.name == scenario.entry_function);
    
    let Some(entry_tree) = entry_tree else {
        return Ok(ScenarioResult {
            success: false,
            path: vec![],
            annotated_flow_tree: None,
            error: Some(format!("Entry function '{}' not found in flow trees", scenario.entry_function)),
        });
    };
    
    // Convert bindings
    let bindings: Vec<ValueBinding> = scenario.bindings.iter()
        .map(|b| ValueBinding {
            path: b.path.clone(),
            value: SymbolicValue::parse(&b.value, &b.value_type),
        })
        .collect();
    
    // Build scenario
    let options = scenario.options.as_ref().map(|o| ScenarioOptions {
        follow_async: o.follow_async.unwrap_or(true),
        show_kernel_api: o.show_kernel_api.unwrap_or(true),
        max_depth: o.max_depth.unwrap_or(10),
    }).unwrap_or_default();
    
    let scenario_config = Scenario {
        name: scenario.name,
        entry_function: scenario.entry_function,
        bindings,
        options: options.clone(),
    };
    
    // Execute scenario
    let mut executor = ScenarioExecutor::new(options);
    let result = executor.execute(&scenario_config, entry_tree);
    
    // Convert states
    let states: Vec<ScenarioState> = result.states.iter()
        .map(|s| ScenarioState {
            function: s.function.clone(),
            line: s.location.line,
            variables: s.variables.iter()
                .map(|(k, v)| (k.clone(), v.display()))
                .collect(),
        })
        .collect();
    
    Ok(ScenarioResult {
        success: true,
        path: states,
        annotated_flow_tree: result.flow_tree,
        error: None,
    })
}
