//! FlowSight Tauri Application

mod commands;

pub use commands::{
    AnalysisResult, AsyncBindingInfo, AsyncPatternInfo, AsyncPatternSummary, CallChainNodeInfo,
    EntryPointInfo, FileNode, FormatOptions, FormattedFlow, FrameworkSummary, FunctionDetail,
    FunctionDetailExt, FunctionInfo, FunctionLocation, IndexStats, KnowledgeInfo, LocalVarInfo,
    ParamInfo, ProjectInfo, SearchOptions, SearchResult,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Configure rayon thread pool with larger stack size (8MB per thread)
    rayon::ThreadPoolBuilder::new()
        .stack_size(8 * 1024 * 1024)
        .build_global()
        .ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_devtools::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::analyze_file,
            commands::get_functions,
            commands::read_file,
            commands::write_file,
            commands::write_file_base64,
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
            // Phase 2: ExecutionFlow API
            commands::build_execution_flow,
            commands::build_execution_flow_tree,
            commands::get_entry_points,
            commands::get_async_bindings,
            // Flow Formatting Commands (AI-assisted)
            commands::format_execution_flow,
            commands::get_flow_display_data,
            // AI Inference Commands
            commands::explain_function,
            commands::get_context_annotation,
            commands::translate_condition,
            // Extended Function Detail API
            commands::get_function_detail_from_file,
            // LLVM IR Commands
            commands::generate_llvm_ir,
            commands::parse_llvm_ir_file,
            // Knowledge Base API
            commands::get_knowledge_info,
            commands::get_async_pattern_info,
            commands::list_frameworks,
            commands::list_async_patterns,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
