/**
 * Tauri API Mock for Playwright Tests
 * 
 * 在 Playwright 测试中模拟 Tauri 原生 API，使测试可以在浏览器环境中运行。
 * 
 * 使用方法:
 * ```typescript
 * import { tauriMockScript, createTauriMock } from './mocks/tauri-api';
 * 
 * test('my test', async ({ page }) => {
 *   await page.addInitScript(tauriMockScript);
 *   await page.goto('http://localhost:5173/');
 * });
 * ```
 */

// ============================================================================
// Type Definitions (匹配 Rust 后端返回的数据结构)
// ============================================================================

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[];
  extension?: string;
}

export interface ProjectInfo {
  path: string;
  files_count: number;
  functions_count: number;
  structs_count: number;
  indexed: boolean;
}

export interface IndexStats {
  functions: number;
  structs: number;
  files: number;
}

export interface FunctionInfo {
  name: string;
  return_type: string;
  line: number;
  is_callback: boolean;
  callback_context?: string;
}

export interface SearchResult {
  name: string;
  kind: string;
  file: string | null;
  line: number | null;
  is_callback: boolean;
}

export interface AnalysisResult {
  file: string;
  functions_count: number;
  structs_count: number;
  async_handlers_count: number;
  entry_points: string[];
  flow_trees: FlowTreeNode[];
}

export interface FlowTreeNode {
  id: string;
  name: string;
  display_name: string;
  location?: Location;
  node_type: string;
  children: FlowTreeNode[];
  description?: string;
}

export interface Location {
  file: string;
  line: number;
  column: number;
}

export interface EntryPointInfo {
  name: string;
  kind: string;
  line: number;
}

export interface AsyncBindingInfo {
  variable: string;
  handler: string;
  mechanism: string;
  context: string;
  bind_line: number | null;
  trigger_lines: number[];
}

export interface ExecutionFlow {
  entry_function: string;
  entry_location?: Location;
  root: FlowTreeNode;
  async_boundaries: AsyncBoundary[];
  analysis_info: AnalysisInfo;
}

export interface AsyncBoundary {
  id: string;
  mechanism: string;
  trigger_call: string;
  trigger_location?: Location;
  handler_function: string;
  handler_location?: Location;
  context: string;
}

export interface AnalysisInfo {
  total_nodes: number;
  max_depth: number;
  external_calls: number;
  async_calls: number;
  warnings: string[];
}

// ============================================================================
// Mock Data Factory
// ============================================================================

/**
 * 创建 Mock 数据工厂
 */
export function createMockDataFactory() {
  return {
    // 创建 Mock 文件树
    createFileTree(basePath: string = '/mock/project'): FileNode[] {
      return [
        {
          name: 'src',
          path: `${basePath}/src`,
          is_dir: true,
          children: [
            {
              name: 'main.c',
              path: `${basePath}/src/main.c`,
              is_dir: false,
              extension: 'c',
            },
            {
              name: 'driver.c',
              path: `${basePath}/src/driver.c`,
              is_dir: false,
              extension: 'c',
            },
            {
              name: 'utils.h',
              path: `${basePath}/src/utils.h`,
              is_dir: false,
              extension: 'h',
            },
          ],
        },
        {
          name: 'include',
          path: `${basePath}/include`,
          is_dir: true,
          children: [
            {
              name: 'types.h',
              path: `${basePath}/include/types.h`,
              is_dir: false,
              extension: 'h',
            },
          ],
        },
        {
          name: 'Makefile',
          path: `${basePath}/Makefile`,
          is_dir: false,
        },
        {
          name: 'README.md',
          path: `${basePath}/README.md`,
          is_dir: false,
          extension: 'md',
        },
      ];
    },

    // 创建 Mock 函数列表
    createFunctions(): FunctionInfo[] {
      return [
        { name: 'main', return_type: 'int', line: 10, is_callback: false },
        { name: 'init_driver', return_type: 'int', line: 25, is_callback: false },
        { name: 'probe_handler', return_type: 'int', line: 50, is_callback: true, callback_context: 'probe' },
        { name: 'work_handler', return_type: 'void', line: 80, is_callback: true, callback_context: 'workqueue' },
        { name: 'irq_handler', return_type: 'irqreturn_t', line: 100, is_callback: true, callback_context: 'interrupt' },
        { name: 'cleanup', return_type: 'void', line: 130, is_callback: false },
      ];
    },

    // 创建 Mock 分析结果
    createAnalysisResult(filePath: string): AnalysisResult {
      return {
        file: filePath,
        functions_count: 6,
        structs_count: 2,
        async_handlers_count: 3,
        entry_points: ['main', 'init_driver'],
        flow_trees: [
          {
            id: 'node-1',
            name: 'main',
            display_name: 'main()',
            location: { file: filePath, line: 10, column: 0 },
            node_type: 'EntryPoint',
            children: [
              {
                id: 'node-2',
                name: 'init_driver',
                display_name: 'init_driver()',
                location: { file: filePath, line: 25, column: 0 },
                node_type: 'Function',
                children: [
                  {
                    id: 'node-3',
                    name: 'probe_handler',
                    display_name: 'probe_handler()',
                    location: { file: filePath, line: 50, column: 0 },
                    node_type: 'Function',
                    children: [],
                    description: 'Device probe callback',
                  },
                ],
              },
            ],
          },
        ],
      };
    },

    // 创建 Mock 执行流
    createExecutionFlow(entryFunction: string, filePath: string): ExecutionFlow {
      return {
        entry_function: entryFunction,
        entry_location: { file: filePath, line: 10, column: 0 },
        root: {
          id: 'root',
          name: entryFunction,
          display_name: `${entryFunction}()`,
          location: { file: filePath, line: 10, column: 0 },
          node_type: 'EntryPoint',
          children: [
            {
              id: 'child-1',
              name: 'schedule_work',
              display_name: 'schedule_work()',
              node_type: 'KernelApi',
              children: [],
              description: 'Schedule work queue handler',
            },
          ],
        },
        async_boundaries: [
          {
            id: 'async-1',
            mechanism: 'WorkQueue',
            trigger_call: 'schedule_work',
            trigger_location: { file: filePath, line: 35, column: 4 },
            handler_function: 'work_handler',
            handler_location: { file: filePath, line: 80, column: 0 },
            context: 'Process',
          },
        ],
        analysis_info: {
          total_nodes: 5,
          max_depth: 3,
          external_calls: 2,
          async_calls: 1,
          warnings: [],
        },
      };
    },

    // 创建 Mock 入口点列表
    createEntryPoints(): EntryPointInfo[] {
      return [
        { name: 'main', kind: 'main_function', line: 10 },
        { name: 'init_driver', kind: 'module_init', line: 25 },
        { name: 'probe_handler', kind: 'probe', line: 50 },
      ];
    },

    // 创建 Mock 异步绑定
    createAsyncBindings(): AsyncBindingInfo[] {
      return [
        {
          variable: 'my_work',
          handler: 'work_handler',
          mechanism: 'WorkQueue',
          context: 'Process',
          bind_line: 30,
          trigger_lines: [35, 45],
        },
        {
          variable: 'my_timer',
          handler: 'timer_callback',
          mechanism: 'Timer',
          context: 'SoftIrq',
          bind_line: 40,
          trigger_lines: [55],
        },
      ];
    },

    // 创建 Mock 文件内容
    createFileContent(filePath: string): string {
      if (filePath.endsWith('.c')) {
        return `/**
 * Mock C Source File
 * Path: ${filePath}
 */

#include <linux/module.h>
#include <linux/workqueue.h>

static struct work_struct my_work;

static void work_handler(struct work_struct *work)
{
    pr_info("Work handler called\\n");
}

static int __init init_driver(void)
{
    INIT_WORK(&my_work, work_handler);
    schedule_work(&my_work);
    return 0;
}

static void __exit cleanup_driver(void)
{
    cancel_work_sync(&my_work);
}

module_init(init_driver);
module_exit(cleanup_driver);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Mock Driver");
`;
      }
      if (filePath.endsWith('.h')) {
        return `/**
 * Mock Header File
 * Path: ${filePath}
 */

#ifndef _MOCK_HEADER_H
#define _MOCK_HEADER_H

struct mock_data {
    int value;
    char name[32];
};

int mock_function(struct mock_data *data);

#endif /* _MOCK_HEADER_H */
`;
      }
      return `Mock content for ${filePath}`;
    },
  };
}

// ============================================================================
// Mock Configuration
// ============================================================================

export interface TauriMockConfig {
  /** 默认项目路径 */
  defaultProjectPath: string;
  /** 是否启用详细日志 */
  verbose: boolean;
  /** 自定义 invoke 处理器 */
  customInvokeHandlers?: Record<string, (args?: any) => any>;
  /** Dialog 返回值配置 */
  dialogResponses?: {
    openProject?: string | null;
    openFile?: string | null;
    saveFile?: string | null;
  };
}

const defaultConfig: TauriMockConfig = {
  defaultProjectPath: '/mock/project',
  verbose: false,
  dialogResponses: {
    openProject: '/mock/project',
    openFile: '/mock/project/src/main.c',
    saveFile: '/mock/project/output/result.txt',
  },
};

// ============================================================================
// Create Tauri Mock
// ============================================================================

/**
 * 创建 Tauri API Mock 对象
 */
export function createTauriMock(config: Partial<TauriMockConfig> = {}) {
  const cfg = { ...defaultConfig, ...config };
  const factory = createMockDataFactory();

  // 存储状态
  let currentProject: ProjectInfo | null = null;
  let fileContents: Map<string, string> = new Map();

  const log = (msg: string, ...args: any[]) => {
    if (cfg.verbose) {
      console.log(`[TauriMock] ${msg}`, ...args);
    }
  };

  // invoke 命令处理器
  const invokeHandlers: Record<string, (args?: any) => any> = {
    // 项目操作
    open_project: async (args: { path: string }): Promise<ProjectInfo> => {
      log('open_project', args);
      currentProject = {
        path: args.path,
        files_count: 42,
        functions_count: 128,
        structs_count: 15,
        indexed: true,
      };
      return currentProject;
    },

    get_index_stats: async (): Promise<IndexStats> => {
      log('get_index_stats');
      return {
        functions: currentProject?.functions_count || 0,
        structs: currentProject?.structs_count || 0,
        files: currentProject?.files_count || 0,
      };
    },

    // 文件操作
    list_directory: async (args: { path: string; recursive?: boolean }): Promise<FileNode[]> => {
      log('list_directory', args);
      return factory.createFileTree(args.path);
    },

    read_file: async (args: { path: string }): Promise<string> => {
      log('read_file', args);
      // 检查是否有缓存的内容
      if (fileContents.has(args.path)) {
        return fileContents.get(args.path)!;
      }
      return factory.createFileContent(args.path);
    },

    write_file: async (args: { path: string; contents: string }): Promise<void> => {
      log('write_file', args);
      fileContents.set(args.path, args.contents);
    },

    delete_file_or_dir: async (args: { path: string }): Promise<void> => {
      log('delete_file_or_dir', args);
      fileContents.delete(args.path);
    },

    rename_file: async (args: { oldPath: string; newPath: string }): Promise<void> => {
      log('rename_file', args);
      const content = fileContents.get(args.oldPath);
      if (content) {
        fileContents.delete(args.oldPath);
        fileContents.set(args.newPath, content);
      }
    },

    create_file: async (args: { path: string }): Promise<void> => {
      log('create_file', args);
      fileContents.set(args.path, '');
    },

    create_directory: async (args: { path: string }): Promise<void> => {
      log('create_directory', args);
      // 目录不需要存储内容
    },

    // 分析操作
    get_functions: async (args: { path: string }): Promise<FunctionInfo[]> => {
      log('get_functions', args);
      return factory.createFunctions();
    },

    analyze_file: async (args: { path: string }): Promise<AnalysisResult> => {
      log('analyze_file', args);
      return factory.createAnalysisResult(args.path);
    },

    search_symbols: async (args: { query: string }): Promise<SearchResult[]> => {
      log('search_symbols', args);
      const functions = factory.createFunctions();
      return functions
        .filter(f => f.name.toLowerCase().includes(args.query.toLowerCase()))
        .map(f => ({
          name: f.name,
          kind: 'function',
          file: '/mock/project/src/main.c',
          line: f.line,
          is_callback: f.is_callback,
        }));
    },

    // ExecutionFlow 操作
    build_execution_flow: async (args: {
      filePath: string;
      entryFunction: string;
      options?: any;
    }): Promise<ExecutionFlow> => {
      log('build_execution_flow', args);
      return factory.createExecutionFlow(args.entryFunction, args.filePath);
    },

    get_entry_points: async (args: { filePath: string }): Promise<EntryPointInfo[]> => {
      log('get_entry_points', args);
      return factory.createEntryPoints();
    },

    get_async_bindings: async (args: { filePath: string }): Promise<AsyncBindingInfo[]> => {
      log('get_async_bindings', args);
      return factory.createAsyncBindings();
    },

    // 导出操作
    export_flow_text: async (args: { path: string; content: string }): Promise<void> => {
      log('export_flow_text', args);
      fileContents.set(args.path, args.content);
    },

    // 合并自定义处理器
    ...cfg.customInvokeHandlers,
  };

  // Dialog Mock
  const dialogMock = {
    open: async (options?: {
      directory?: boolean;
      multiple?: boolean;
      title?: string;
      filters?: Array<{ name: string; extensions: string[] }>;
    }): Promise<string | string[] | null> => {
      log('dialog.open', options);
      if (options?.directory) {
        return cfg.dialogResponses?.openProject || null;
      }
      return cfg.dialogResponses?.openFile || null;
    },

    save: async (options?: {
      title?: string;
      defaultPath?: string;
      filters?: Array<{ name: string; extensions: string[] }>;
    }): Promise<string | null> => {
      log('dialog.save', options);
      return cfg.dialogResponses?.saveFile || null;
    },

    message: async (message: string, options?: { title?: string; kind?: string }): Promise<void> => {
      log('dialog.message', message, options);
    },

    ask: async (message: string, options?: { title?: string; kind?: string }): Promise<boolean> => {
      log('dialog.ask', message, options);
      return true;
    },

    confirm: async (message: string, options?: { title?: string }): Promise<boolean> => {
      log('dialog.confirm', message, options);
      return true;
    },
  };

  // Core API Mock
  const coreMock = {
    invoke: async <T>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
      const handler = invokeHandlers[cmd];
      if (handler) {
        return handler(args) as Promise<T>;
      }
      console.warn(`[TauriMock] Unknown command: ${cmd}`);
      throw new Error(`Unknown command: ${cmd}`);
    },

    convertFileSrc: (path: string): string => {
      return `asset://localhost/${path}`;
    },

    transformCallback: (callback: Function): number => {
      return 0;
    },
  };

  // Event API Mock
  const eventMock = {
    listen: async (event: string, handler: Function): Promise<() => void> => {
      log('event.listen', event);
      return () => {};
    },

    once: async (event: string, handler: Function): Promise<() => void> => {
      log('event.once', event);
      return () => {};
    },

    emit: async (event: string, payload?: unknown): Promise<void> => {
      log('event.emit', event, payload);
    },
  };

  // Window API Mock
  const windowMock = {
    appWindow: {
      setTitle: async (title: string): Promise<void> => {
        log('window.setTitle', title);
      },
      minimize: async (): Promise<void> => {
        log('window.minimize');
      },
      maximize: async (): Promise<void> => {
        log('window.maximize');
      },
      close: async (): Promise<void> => {
        log('window.close');
      },
      isMaximized: async (): Promise<boolean> => {
        return false;
      },
      isMinimized: async (): Promise<boolean> => {
        return false;
      },
    },
    getCurrentWindow: () => windowMock.appWindow,
    Window: class {
      constructor(label: string) {}
    },
  };

  // Path API Mock
  const pathMock = {
    join: async (...paths: string[]): Promise<string> => {
      return paths.join('/');
    },
    dirname: async (path: string): Promise<string> => {
      return path.split('/').slice(0, -1).join('/');
    },
    basename: async (path: string): Promise<string> => {
      return path.split('/').pop() || '';
    },
    extname: async (path: string): Promise<string> => {
      const parts = path.split('.');
      return parts.length > 1 ? `.${parts.pop()}` : '';
    },
    resolve: async (...paths: string[]): Promise<string> => {
      return paths.join('/');
    },
    appDataDir: async (): Promise<string> => '/mock/app-data',
    appConfigDir: async (): Promise<string> => '/mock/app-config',
    homeDir: async (): Promise<string> => '/mock/home',
    desktopDir: async (): Promise<string> => '/mock/home/Desktop',
    documentDir: async (): Promise<string> => '/mock/home/Documents',
  };

  // FS API Mock
  const fsMock = {
    readTextFile: async (path: string): Promise<string> => {
      return coreMock.invoke('read_file', { path });
    },
    writeTextFile: async (path: string, contents: string): Promise<void> => {
      return coreMock.invoke('write_file', { path, contents });
    },
    readDir: async (path: string): Promise<FileNode[]> => {
      return coreMock.invoke('list_directory', { path, recursive: false });
    },
    createDir: async (path: string): Promise<void> => {
      return coreMock.invoke('create_directory', { path });
    },
    removeFile: async (path: string): Promise<void> => {
      return coreMock.invoke('delete_file_or_dir', { path });
    },
    removeDir: async (path: string): Promise<void> => {
      return coreMock.invoke('delete_file_or_dir', { path });
    },
    exists: async (path: string): Promise<boolean> => {
      return true;
    },
    copyFile: async (source: string, destination: string): Promise<void> => {
      const content = await coreMock.invoke<string>('read_file', { path: source });
      await coreMock.invoke('write_file', { path: destination, contents: content });
    },
    renameFile: async (oldPath: string, newPath: string): Promise<void> => {
      return coreMock.invoke('rename_file', { oldPath, newPath });
    },
  };

  return {
    dialog: dialogMock,
    core: coreMock,
    event: eventMock,
    window: windowMock,
    path: pathMock,
    fs: fsMock,
    // 辅助方法
    setFileContent: (path: string, content: string) => {
      fileContents.set(path, content);
    },
    getFileContent: (path: string) => {
      return fileContents.get(path);
    },
    clearFileContents: () => {
      fileContents.clear();
    },
    getCurrentProject: () => currentProject,
  };
}

// ============================================================================
// Injection Script Generator
// ============================================================================

/**
 * 生成用于 page.addInitScript() 的注入脚本
 */
export function generateTauriMockScript(config: Partial<TauriMockConfig> = {}): string {
  const configJson = JSON.stringify(config);
  
  return `
// Tauri API Mock - Auto-generated injection script
(function() {
  const config = ${configJson};
  
  // Mock Data Factory (simplified for injection)
  const mockData = {
    fileTree: [
      { name: 'src', path: '/mock/project/src', is_dir: true, children: [
        { name: 'main.c', path: '/mock/project/src/main.c', is_dir: false, extension: 'c' },
        { name: 'driver.c', path: '/mock/project/src/driver.c', is_dir: false, extension: 'c' },
      ]},
      { name: 'include', path: '/mock/project/include', is_dir: true },
      { name: 'Makefile', path: '/mock/project/Makefile', is_dir: false },
    ],
    functions: [
      { name: 'main', return_type: 'int', line: 10, is_callback: false },
      { name: 'init_driver', return_type: 'int', line: 25, is_callback: false },
      { name: 'probe_handler', return_type: 'int', line: 50, is_callback: true },
      { name: 'work_handler', return_type: 'void', line: 80, is_callback: true },
    ],
    projectInfo: {
      path: config.defaultProjectPath || '/mock/project',
      files_count: 42,
      functions_count: 128,
      structs_count: 15,
      indexed: true,
    },
  };
  
  // File content storage
  const fileContents = new Map();
  
  // Invoke handlers
  const handlers = {
    open_project: (args) => mockData.projectInfo,
    get_index_stats: () => ({ functions: 128, structs: 15, files: 42 }),
    list_directory: (args) => mockData.fileTree,
    read_file: (args) => fileContents.get(args.path) || '// Mock file content for ' + args.path,
    write_file: (args) => { fileContents.set(args.path, args.contents); },
    get_functions: (args) => mockData.functions,
    analyze_file: (args) => ({
      file: args.path,
      functions_count: 6,
      structs_count: 2,
      async_handlers_count: 3,
      entry_points: ['main', 'init_driver'],
      flow_trees: [{
        id: 'node-1',
        name: 'main',
        display_name: 'main()',
        location: { file: args.path, line: 10, column: 0 },
        node_type: 'EntryPoint',
        children: [],
      }],
    }),
    search_symbols: (args) => mockData.functions
      .filter(f => f.name.includes(args.query))
      .map(f => ({ name: f.name, kind: 'function', file: '/mock/project/src/main.c', line: f.line, is_callback: f.is_callback })),
    build_execution_flow: (args) => ({
      entry_function: args.entryFunction,
      entry_location: { file: args.filePath, line: 10, column: 0 },
      root: { id: 'root', name: args.entryFunction, display_name: args.entryFunction + '()', node_type: 'EntryPoint', children: [] },
      async_boundaries: [],
      analysis_info: { total_nodes: 1, max_depth: 1, external_calls: 0, async_calls: 0, warnings: [] },
    }),
    get_entry_points: () => [
      { name: 'main', kind: 'main_function', line: 10 },
      { name: 'init_driver', kind: 'module_init', line: 25 },
    ],
    get_async_bindings: () => [
      { variable: 'my_work', handler: 'work_handler', mechanism: 'WorkQueue', context: 'Process', bind_line: 30, trigger_lines: [35] },
    ],
    delete_file_or_dir: (args) => { fileContents.delete(args.path); },
    rename_file: (args) => { 
      const c = fileContents.get(args.oldPath); 
      if(c) { fileContents.delete(args.oldPath); fileContents.set(args.newPath, c); }
    },
    create_file: (args) => { fileContents.set(args.path, ''); },
    create_directory: () => {},
    export_flow_text: (args) => { fileContents.set(args.path, args.content); },
  };
  
  // Create mock APIs
  const createInvoke = () => async (cmd, args) => {
    if (config.verbose) console.log('[TauriMock] invoke:', cmd, args);
    const handler = handlers[cmd];
    if (handler) return handler(args);
    console.warn('[TauriMock] Unknown command:', cmd);
    throw new Error('Unknown command: ' + cmd);
  };
  
  const createDialog = () => ({
    open: async (options) => {
      if (config.verbose) console.log('[TauriMock] dialog.open:', options);
      if (options?.directory) return config.dialogResponses?.openProject || '/mock/project';
      return config.dialogResponses?.openFile || '/mock/project/src/main.c';
    },
    save: async (options) => {
      if (config.verbose) console.log('[TauriMock] dialog.save:', options);
      return config.dialogResponses?.saveFile || '/mock/output/result.txt';
    },
    message: async () => {},
    ask: async () => true,
    confirm: async () => true,
  });
  
  // Install mocks
  window.__TAURI__ = {
    core: { invoke: createInvoke() },
    dialog: createDialog(),
    event: {
      listen: async () => () => {},
      once: async () => () => {},
      emit: async () => {},
    },
    window: {
      appWindow: {
        setTitle: async () => {},
        minimize: async () => {},
        maximize: async () => {},
        close: async () => {},
        isMaximized: async () => false,
        isMinimized: async () => false,
      },
      getCurrentWindow: () => window.__TAURI__.window.appWindow,
    },
    path: {
      join: async (...paths) => paths.join('/'),
      dirname: async (p) => p.split('/').slice(0, -1).join('/'),
      basename: async (p) => p.split('/').pop() || '',
      extname: async (p) => { const parts = p.split('.'); return parts.length > 1 ? '.' + parts.pop() : ''; },
    },
    fs: {
      readTextFile: async (path) => handlers.read_file({ path }),
      writeTextFile: async (path, contents) => handlers.write_file({ path, contents }),
      readDir: async (path) => handlers.list_directory({ path }),
      createDir: async (path) => {},
      removeFile: async (path) => handlers.delete_file_or_dir({ path }),
      exists: async () => true,
    },
  };
  
  // Also install for @tauri-apps/api imports
  window.__TAURI_INTERNALS__ = {
    invoke: createInvoke(),
    transformCallback: () => 0,
    convertFileSrc: (path) => 'asset://localhost/' + path,
  };
  
  // Mock the plugin-dialog
  window.__TAURI_PLUGIN_DIALOG__ = createDialog();
  
  console.log('[TauriMock] Tauri API mocks installed');
})();
`;
}

// ============================================================================
// Pre-built Injection Script (Default Config)
// ============================================================================

/**
 * 默认配置的注入脚本，可直接用于 page.addInitScript()
 */
export const tauriMockScript = generateTauriMockScript({
  verbose: false,
  defaultProjectPath: '/mock/project',
  dialogResponses: {
    openProject: '/mock/project',
    openFile: '/mock/project/src/main.c',
    saveFile: '/mock/output/result.txt',
  },
});

/**
 * 详细日志版本的注入脚本
 */
export const tauriMockScriptVerbose = generateTauriMockScript({
  verbose: true,
  defaultProjectPath: '/mock/project',
});

// ============================================================================
// Playwright Helper Functions
// ============================================================================

/**
 * 在 Playwright 页面中设置 Tauri Mock
 */
export async function setupTauriMock(
  page: any, // Playwright Page type
  config?: Partial<TauriMockConfig>
): Promise<void> {
  const script = generateTauriMockScript(config);
  await page.addInitScript(script);
}

/**
 * 配置 Mock Dialog 返回特定路径
 */
export function createDialogMockScript(responses: {
  openProject?: string | null;
  openFile?: string | null;
  saveFile?: string | null;
}): string {
  return generateTauriMockScript({
    dialogResponses: responses,
  });
}

/**
 * 创建带有自定义文件内容的 Mock 脚本
 */
export function createMockWithFiles(files: Record<string, string>): string {
  const filesJson = JSON.stringify(files);
  return `
${tauriMockScript}
(function() {
  const customFiles = ${filesJson};
  Object.entries(customFiles).forEach(([path, content]) => {
    window.__TAURI_MOCK_FILES__ = window.__TAURI_MOCK_FILES__ || new Map();
    window.__TAURI_MOCK_FILES__.set(path, content);
  });
})();
`;
}
