//! FlowSight Tauri Application

mod commands;

pub use commands::{AnalysisResult, FunctionInfo, ProjectInfo, SearchResult, IndexStats};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
