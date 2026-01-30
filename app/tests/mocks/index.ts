/**
 * Tauri API Mocks Entry Point
 * 
 * 导出所有 Mock 相关的类型、函数和脚本。
 */

export {
  // Types
  type FileNode,
  type ProjectInfo,
  type IndexStats,
  type FunctionInfo,
  type SearchResult,
  type AnalysisResult,
  type FlowTreeNode,
  type Location,
  type EntryPointInfo,
  type AsyncBindingInfo,
  type ExecutionFlow,
  type AsyncBoundary,
  type AnalysisInfo,
  type TauriMockConfig,
  
  // Factory
  createMockDataFactory,
  
  // Mock Creator
  createTauriMock,
  
  // Script Generators
  generateTauriMockScript,
  
  // Pre-built Scripts
  tauriMockScript,
  tauriMockScriptVerbose,
  
  // Playwright Helpers
  setupTauriMock,
  createDialogMockScript,
  createMockWithFiles,
} from './tauri-api';
