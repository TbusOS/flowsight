//! FlowSight Tauri Application

mod commands;

pub use commands::{
    AnalysisResult, FileNode, FunctionDetail, FunctionInfo, FunctionLocation, IndexStats,
    ProjectInfo, SearchResult,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Configure rayon thread pool with larger stack size (8MB per thread)
    rayon::ThreadPoolBuilder::new()
        .stack_size(8 * 1024 * 1024)
        .build_global()
        .ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::analyze_file,
            commands::get_functions,
            commands::read_file,
            commands::open_project,
            commands::search_symbols,
            commands::get_index_stats,
            commands::get_function_detail,
            commands::get_function_locations,
            commands::list_directory,
            commands::expand_directory,
            commands::export_flow_text,
            commands::execute_scenario,
            commands::get_function_callers,
            commands::create_file,
            commands::create_directory,
            commands::rename_file,
            commands::delete_file_or_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
