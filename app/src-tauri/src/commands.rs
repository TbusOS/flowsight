//! Tauri Commands

use flowsight_analysis::flow_builder::{BuildOptions, FlowBuilder, FunctionInfo as BuilderFunctionInfo};
use flowsight_analysis::Analyzer;
use flowsight_core::ExecutionFlow;
use flowsight_index::SymbolIndex;
use flowsight_parser::get_parser;
use flowsight_parser::parallel::ParallelParser;
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
    let states: Vec<ScenarioState> = result.primary_path.steps.iter()
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
    flatten_node(&flow.root, &mut nodes, &mut edges, &mut visited, None, false);
    
    // Collect warnings as strings
    let warnings: Vec<String> = flow.analysis_info.warnings
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
        }.to_string()
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
        if matches!(child.node_type, flowsight_core::FlowNodeType::Separator { .. }) {
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
    let entry_points: Vec<EntryPointInfo> = analysis.entry_points
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
                line: func.and_then(|f| f.location.as_ref().map(|l| l.line)).unwrap_or(0),
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
    let bindings: Vec<AsyncBindingInfo> = analysis.async_bindings
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
    let function_code = extract_function_code(&source, &function_name)
        .unwrap_or_else(|| source.clone());
    
    // For now, return a template-based explanation
    // TODO: Integrate with actual AI model when available
    Ok(AiExplanationResult {
        trigger_condition: format!("当 {} 被调用时", function_name),
        business_meaning: format!("函数 {} 执行其定义的操作", function_name),
        execution_result: "执行函数体中的代码逻辑".to_string(),
        related_functions: extract_called_functions(&function_code),
        common_errors: vec![
            "检查返回值".to_string(),
            "注意错误处理".to_string(),
        ],
        context_type: detect_context_type(&function_code),
        can_sleep: detect_can_sleep(&function_code),
    })
}

/// Get context annotation for async handler
#[tauri::command]
pub async fn get_context_annotation(
    mechanism: String,
    handler_code: String,
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
            vec![
                "可以睡眠".to_string(),
                "适合复杂的中断处理".to_string(),
            ],
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
    code: String,
    constraint: String,
    function_name: String,
) -> Result<String, String> {
    // Simple template-based translation
    // TODO: Integrate with AI model for complex cases
    
    let translation = if constraint.contains("NULL") || constraint.contains("!") && constraint.contains("ptr") {
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
            related_functions: vec![
                "device_unregister".to_string(),
                "free_irq".to_string(),
            ],
            common_errors: vec![
                "资源释放顺序错误".to_string(),
                "遗漏资源释放".to_string(),
            ],
            context_type: Some("进程上下文 (Process)".to_string()),
            can_sleep: Some(true),
        });
    }
    
    if function_name.contains("irq") || function_name.contains("interrupt") {
        return Some(AiExplanationResult {
            trigger_condition: "硬件中断发生时".to_string(),
            business_meaning: "中断处理函数，响应硬件事件".to_string(),
            execution_result: "处理中断，可能调度后续工作".to_string(),
            related_functions: vec![
                "schedule_work".to_string(),
                "tasklet_schedule".to_string(),
            ],
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
            related_functions: vec![
                "schedule_work".to_string(),
                "queue_work".to_string(),
            ],
            common_errors: vec![
                "访问已释放的资源".to_string(),
                "并发访问问题".to_string(),
            ],
            context_type: Some("进程上下文 (Process)".to_string()),
            can_sleep: Some(true),
        });
    }
    
    None
}

fn extract_function_code(source: &str, function_name: &str) -> Option<String> {
    // Simple extraction - find function and extract until closing brace
    let pattern = format!(r"(?s)(\w+\s+)?{}\s*\([^)]*\)\s*\{{", regex::escape(function_name));
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
        .filter(|name| !["if", "while", "for", "switch", "return", "sizeof"].contains(&name.as_str()))
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
