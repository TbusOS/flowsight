//! Tauri Commands

use flowsight_parser::get_parser;
use flowsight_analysis::Analyzer;
use flowsight_index::SymbolIndex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use once_cell::sync::Lazy;
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

/// Open a project directory and index it
#[tauri::command]
pub async fn open_project(path: String) -> Result<ProjectInfo, String> {
    let project_path = PathBuf::from(&path);
    
    if !project_path.is_dir() {
        return Err("Path is not a directory".into());
    }
    
    // Find all C files
    let c_files: Vec<PathBuf> = WalkDir::new(&project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "c" || ext == "h")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let parser = get_parser();
    let mut index = INDEX.lock().map_err(|e| e.to_string())?;
    
    // Clear previous index
    *index = SymbolIndex::new();
    
    // Index all files
    for file in &c_files {
        if let Ok(result) = parser.parse_file(file) {
            for (_, func) in result.functions {
                index.add_function(func, file);
            }
            for (_, st) in result.structs {
                index.add_struct(st);
            }
        }
    }

    Ok(ProjectInfo {
        path,
        files_count: c_files.len(),
        functions_count: index.stats().total_functions,
        structs_count: index.stats().total_structs,
        indexed: true,
    })
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
            params: func.params.iter().map(|p| ParamInfo {
                name: p.name.clone(),
                type_name: p.type_name.clone(),
            }).collect(),
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
    let mut parse_result = parser.parse_file(&path)
        .map_err(|e| e.to_string())?;

    let source = std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())?;
    
    let mut analyzer = Analyzer::new();
    let _ = analyzer.analyze(&source, &mut parse_result)
        .map_err(|e| e.to_string())?;

    let locations: Vec<FunctionLocation> = parse_result.functions
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
    
    fn read_dir_entries(path: &Path, recursive: bool, depth: usize) -> Result<Vec<FileNode>, String> {
        if depth > 10 {
            return Ok(vec![]); // Limit depth
        }
        
        let mut entries: Vec<FileNode> = std::fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .filter(|e| {
                // Skip hidden files and common non-source directories
                let name = e.file_name().to_string_lossy().to_string();
                !name.starts_with('.') && 
                !["node_modules", "target", "build", "dist", "__pycache__", ".git"].contains(&name.as_str())
            })
            .map(|e| {
                let path = e.path();
                let name = e.file_name().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                let extension = if !is_dir {
                    path.extension().map(|ext| ext.to_string_lossy().to_string())
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
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
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
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

