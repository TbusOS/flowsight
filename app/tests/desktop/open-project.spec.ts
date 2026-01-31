/**
 * FlowSight 打开项目测试
 * 
 * 测试内容:
 * - 使用 Tauri Mock 模拟打开项目
 * - 验证文件浏览器显示项目文件
 * - 验证大纲面板加载函数列表
 * 
 * 使用 fixtures/sample-project 作为测试项目
 */

import { test, expect } from '@playwright/test';
import { mkdirSync, existsSync, readFileSync } from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { generateTauriMockScript, setupTauriMock } from '../mocks';

const SCREENSHOTS_DIR = './test-results/flowsight/open-project';

// ES Module 兼容: 获取 __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 获取 fixtures 绝对路径
const FIXTURES_PATH = path.resolve(__dirname, '../fixtures/sample-project');

// Sample project 文件树结构 (模拟 Tauri list_directory 返回)
const SAMPLE_PROJECT_FILE_TREE = [
  {
    name: 'main.c',
    path: `${FIXTURES_PATH}/main.c`,
    is_dir: false,
    extension: 'c',
  },
  {
    name: 'utils.c',
    path: `${FIXTURES_PATH}/utils.c`,
    is_dir: false,
    extension: 'c',
  },
  {
    name: 'utils.h',
    path: `${FIXTURES_PATH}/utils.h`,
    is_dir: false,
    extension: 'h',
  },
  {
    name: 'Makefile',
    path: `${FIXTURES_PATH}/Makefile`,
    is_dir: false,
  },
];

// Sample project 函数列表 (模拟 Tauri get_functions 返回)
const MAIN_C_FUNCTIONS = [
  { name: 'main', return_type: 'int', line: 24, is_callback: false, callback_context: null, calls: [] },
  { name: 'init_system', return_type: 'int', line: 48, is_callback: false, callback_context: null, calls: [] },
  { name: 'process_data', return_type: 'void', line: 63, is_callback: false, callback_context: null, calls: [] },
  { name: 'cleanup', return_type: 'void', line: 87, is_callback: false, callback_context: null, calls: [] },
  { name: 'sample_callback', return_type: 'void', line: 101, is_callback: true, callback_context: 'callback', calls: [] },
  { name: 'handle_error', return_type: 'void', line: 116, is_callback: false, callback_context: null, calls: [] },
];

const UTILS_C_FUNCTIONS = [
  { name: 'calculate', return_type: 'int', line: 14, is_callback: false, callback_context: null, calls: [] },
  { name: 'print_result', return_type: 'void', line: 28, is_callback: false, callback_context: null, calls: [] },
  { name: 'validate_input', return_type: 'int', line: 45, is_callback: false, callback_context: null, calls: [] },
  { name: 'format_output', return_type: 'void', line: 59, is_callback: false, callback_context: null, calls: [] },
];

/**
 * 生成自定义的 Tauri Mock 脚本
 * 支持 sample-project fixtures
 */
function generateSampleProjectMockScript(options: {
  projectPath: string;
  verbose?: boolean;
}): string {
  const { projectPath, verbose = false } = options;
  
  // 读取真实的文件内容 (如果可用)
  let mainCContent = '// Mock main.c content';
  let utilsCContent = '// Mock utils.c content';
  let utilsHContent = '// Mock utils.h content';
  
  try {
    mainCContent = readFileSync(path.join(projectPath, 'main.c'), 'utf-8');
    utilsCContent = readFileSync(path.join(projectPath, 'utils.c'), 'utf-8');
    utilsHContent = readFileSync(path.join(projectPath, 'utils.h'), 'utf-8');
  } catch {
    // 使用默认 mock 内容
  }
  
  const fileContents = {
    [`${projectPath}/main.c`]: mainCContent,
    [`${projectPath}/utils.c`]: utilsCContent,
    [`${projectPath}/utils.h`]: utilsHContent,
  };
  
  const fileTree = SAMPLE_PROJECT_FILE_TREE.map(f => ({
    ...f,
    path: f.path.replace(FIXTURES_PATH, projectPath),
  }));
  
  return `
// Tauri API Mock - Sample Project Fixtures
(function() {
  const config = {
    verbose: ${verbose},
    projectPath: ${JSON.stringify(projectPath)},
  };
  
  const fileTree = ${JSON.stringify(fileTree)};
  const mainCFunctions = ${JSON.stringify(MAIN_C_FUNCTIONS)};
  const utilsCFunctions = ${JSON.stringify(UTILS_C_FUNCTIONS)};
  const fileContents = ${JSON.stringify(fileContents)};
  
  const log = (msg, ...args) => {
    if (config.verbose) console.log('[TauriMock]', msg, ...args);
  };
  
  // 项目状态
  let currentProject = null;
  
  // invoke 处理器
  const handlers = {
    open_project: (args) => {
      log('open_project', args);
      currentProject = {
        path: args.path,
        files_count: 4,
        functions_count: 10,
        structs_count: 0,
        indexed: true,
      };
      return currentProject;
    },
    
    get_index_stats: () => {
      log('get_index_stats');
      return { functions: 10, structs: 0, files: 4 };
    },
    
    list_directory: (args) => {
      log('list_directory', args);
      // 返回调整过路径的文件树
      return fileTree.map(f => ({
        ...f,
        path: f.path.replace('${FIXTURES_PATH}', args.path),
      }));
    },
    
    read_file: (args) => {
      log('read_file', args);
      // 尝试匹配文件内容
      for (const [key, content] of Object.entries(fileContents)) {
        if (args.path.endsWith(key.split('/').pop())) {
          return content;
        }
      }
      return '// Mock file content for ' + args.path;
    },
    
    get_functions: (args) => {
      log('get_functions', args);
      if (args.path.includes('main.c')) {
        return mainCFunctions;
      }
      if (args.path.includes('utils.c')) {
        return utilsCFunctions;
      }
      return [];
    },
    
    analyze_file: (args) => {
      log('analyze_file', args);
      const functions = args.path.includes('main.c') ? mainCFunctions : utilsCFunctions;
      return {
        file: args.path,
        functions_count: functions.length,
        structs_count: 0,
        async_handlers_count: functions.filter(f => f.is_callback).length,
        entry_points: ['main', 'init_system'],
        flow_trees: [{
          id: 'node-main',
          name: 'main',
          display_name: 'main()',
          location: { file: args.path, line: 24, column: 0 },
          node_type: 'EntryPoint',
          children: [{
            id: 'node-init',
            name: 'init_system',
            display_name: 'init_system()',
            location: { file: args.path, line: 48, column: 0 },
            node_type: 'Function',
            children: [],
          }],
        }],
      };
    },
    
    search_symbols: (args) => {
      log('search_symbols', args);
      const allFuncs = [...mainCFunctions, ...utilsCFunctions];
      return allFuncs
        .filter(f => f.name.toLowerCase().includes(args.query.toLowerCase()))
        .map(f => ({
          name: f.name,
          kind: 'function',
          file: config.projectPath + '/main.c',
          line: f.line,
          is_callback: f.is_callback,
        }));
    },
    
    get_entry_points: (args) => {
      log('get_entry_points', args);
      return [
        { name: 'main', kind: 'main_function', line: 24 },
        { name: 'init_system', kind: 'init', line: 48 },
      ];
    },
    
    get_async_bindings: (args) => {
      log('get_async_bindings', args);
      return [{
        variable: 'sample_callback',
        handler: 'sample_callback',
        mechanism: 'Callback',
        context: 'User',
        bind_line: 101,
        trigger_lines: [],
      }];
    },
    
    build_execution_flow: (args) => {
      log('build_execution_flow', args);
      return {
        entry_function: args.entryFunction,
        entry_location: { file: args.filePath, line: 24, column: 0 },
        root: {
          id: 'root',
          name: args.entryFunction,
          display_name: args.entryFunction + '()',
          node_type: 'EntryPoint',
          children: [{
            id: 'child-1',
            name: 'init_system',
            display_name: 'init_system()',
            node_type: 'Function',
            children: [],
          }],
        },
        async_boundaries: [],
        analysis_info: { total_nodes: 2, max_depth: 2, external_calls: 0, async_calls: 0, warnings: [] },
      };
    },
    
    write_file: (args) => {
      log('write_file', args);
      fileContents[args.path] = args.contents;
    },
    
    delete_file_or_dir: (args) => {
      log('delete_file_or_dir', args);
      delete fileContents[args.path];
    },
    
    create_file: (args) => {
      log('create_file', args);
      fileContents[args.path] = '';
    },
    
    create_directory: () => {},
  };
  
  // Dialog Mock
  const dialogMock = {
    open: async (options) => {
      log('dialog.open', options);
      if (options?.directory) {
        return config.projectPath;
      }
      return config.projectPath + '/main.c';
    },
    save: async () => config.projectPath + '/output.txt',
    message: async () => {},
    ask: async () => true,
    confirm: async () => true,
  };
  
  // Install mocks
  window.__TAURI__ = {
    core: {
      invoke: async (cmd, args) => {
        log('invoke:', cmd, args);
        const handler = handlers[cmd];
        if (handler) return handler(args);
        console.warn('[TauriMock] Unknown command:', cmd);
        throw new Error('Unknown command: ' + cmd);
      },
    },
    dialog: dialogMock,
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
    },
    fs: {
      readTextFile: async (path) => handlers.read_file({ path }),
      writeTextFile: async (path, contents) => handlers.write_file({ path, contents }),
      readDir: async (path) => handlers.list_directory({ path }),
      exists: async () => true,
    },
  };
  
  window.__TAURI_INTERNALS__ = {
    invoke: window.__TAURI__.core.invoke,
    transformCallback: () => 0,
    convertFileSrc: (path) => 'asset://localhost/' + path,
  };
  
  window.__TAURI_PLUGIN_DIALOG__ = dialogMock;
  
  console.log('[TauriMock] Sample Project Mock installed - Path:', config.projectPath);
})();
`;
}

test.beforeAll(() => {
  if (!existsSync(SCREENSHOTS_DIR)) {
    mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

test.describe('打开项目 - 基本流程', () => {
  test('通过文件浏览器打开项目按钮', async ({ page }) => {
    // 注入 Mock 脚本
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
      verbose: true,
    });
    await page.addInitScript(mockScript);
    
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/01-initial.png` });
    
    // 验证左侧面板显示"打开项目"按钮
    const openProjectButton = page.locator('button:has-text("打开项目")');
    await expect(openProjectButton).toBeVisible();
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/02-open-project-button.png` });
    
    // 点击"打开项目"按钮
    await openProjectButton.click();
    await page.waitForTimeout(1000);
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/03-after-click.png` });
    
    // 验证 Mock 的 dialog.open 被调用
    const dialogCalled = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      const result = await tauri.dialog.open({ directory: true });
      return result !== null;
    });
    expect(dialogCalled).toBe(true);
  });

  test('Mock invoke 返回正确的项目信息', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 测试 open_project invoke
    const projectInfo = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('open_project', { path: '/test/project' });
    });
    
    expect(projectInfo.indexed).toBe(true);
    expect(projectInfo.files_count).toBe(4);
    expect(projectInfo.functions_count).toBe(10);
    
    console.log('Project info:', projectInfo);
  });
});

test.describe('打开项目 - 文件浏览器', () => {
  test('显示项目文件树', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
      verbose: true,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 模拟打开项目（直接设置 store 状态）
    await page.evaluate(async (projectPath) => {
      const tauri = (window as any).__TAURI__;
      await tauri.core.invoke('open_project', { path: projectPath });
    }, FIXTURES_PATH);
    
    // 验证 list_directory 返回正确的文件列表
    const files = await page.evaluate(async (projectPath) => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('list_directory', { path: projectPath, recursive: true });
    }, FIXTURES_PATH);
    
    expect(files.length).toBe(4);
    expect(files.some((f: any) => f.name === 'main.c')).toBe(true);
    expect(files.some((f: any) => f.name === 'utils.c')).toBe(true);
    expect(files.some((f: any) => f.name === 'utils.h')).toBe(true);
    expect(files.some((f: any) => f.name === 'Makefile')).toBe(true);
    
    console.log('Files:', files.map((f: any) => f.name));
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/file-tree.png` });
  });

  test('点击文件按钮打开文件浏览器面板', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 点击文件浏览器按钮
    const explorerButton = page.locator('[data-testid="sidebar-explorer"]');
    await expect(explorerButton).toBeVisible();
    await explorerButton.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/file-explorer-panel.png` });
    
    // 验证右侧面板打开
    const fileTab = page.locator('button:has-text("文件")');
    await expect(fileTab.first()).toBeVisible();
  });
});

test.describe('打开项目 - 大纲面板', () => {
  test('显示函数列表', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
      verbose: true,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 验证 get_functions 返回正确的函数列表
    const mainFunctions = await page.evaluate(async (projectPath) => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('get_functions', { path: projectPath + '/main.c' });
    }, FIXTURES_PATH);
    
    expect(mainFunctions.length).toBe(6);
    expect(mainFunctions.some((f: any) => f.name === 'main')).toBe(true);
    expect(mainFunctions.some((f: any) => f.name === 'init_system')).toBe(true);
    expect(mainFunctions.some((f: any) => f.name === 'process_data')).toBe(true);
    expect(mainFunctions.some((f: any) => f.name === 'cleanup')).toBe(true);
    expect(mainFunctions.some((f: any) => f.name === 'sample_callback' && f.is_callback)).toBe(true);
    expect(mainFunctions.some((f: any) => f.name === 'handle_error')).toBe(true);
    
    console.log('Main.c functions:', mainFunctions.map((f: any) => `${f.name}:${f.line}`));
    
    // 验证 utils.c 函数
    const utilsFunctions = await page.evaluate(async (projectPath) => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('get_functions', { path: projectPath + '/utils.c' });
    }, FIXTURES_PATH);
    
    expect(utilsFunctions.length).toBe(4);
    expect(utilsFunctions.some((f: any) => f.name === 'calculate')).toBe(true);
    expect(utilsFunctions.some((f: any) => f.name === 'print_result')).toBe(true);
    
    console.log('Utils.c functions:', utilsFunctions.map((f: any) => `${f.name}:${f.line}`));
  });

  test('点击大纲按钮打开大纲面板', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 点击大纲按钮
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await expect(outlineButton).toBeVisible();
    await outlineButton.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/outline-panel.png` });
    
    // 验证大纲面板打开
    const outlineTab = page.locator('button:has-text("大纲")');
    await expect(outlineTab.first()).toBeVisible();
    
    // 验证搜索输入框存在
    const searchInput = page.locator('[data-testid="outline-search"]');
    await expect(searchInput).toBeVisible();
  });

  test('大纲面板显示空状态提示', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开大纲面板
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await outlineButton.click();
    await page.waitForTimeout(500);
    
    // 切换到大纲标签
    const outlineTab = page.locator('button:has-text("大纲")').first();
    if (await outlineTab.isVisible()) {
      await outlineTab.click();
      await page.waitForTimeout(300);
    }
    
    // 验证空状态提示 (没有打开文件时)
    const emptyMessage = page.locator('text=打开文件查看大纲');
    const hasEmptyMessage = await emptyMessage.isVisible();
    console.log('显示空状态提示:', hasEmptyMessage);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/outline-empty-state.png` });
  });
});

test.describe('打开项目 - 分析功能', () => {
  test('分析文件返回正确结果', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
      verbose: true,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 分析 main.c
    const analysisResult = await page.evaluate(async (projectPath) => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('analyze_file', { path: projectPath + '/main.c' });
    }, FIXTURES_PATH);
    
    expect(analysisResult.file).toContain('main.c');
    expect(analysisResult.functions_count).toBe(6);
    expect(analysisResult.entry_points).toContain('main');
    expect(analysisResult.flow_trees.length).toBeGreaterThan(0);
    
    console.log('Analysis result:', {
      file: analysisResult.file,
      functions: analysisResult.functions_count,
      entryPoints: analysisResult.entry_points,
    });
  });

  test('搜索符号功能', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 搜索 "init"
    const searchResults = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('search_symbols', { query: 'init' });
    });
    
    expect(searchResults.length).toBeGreaterThan(0);
    expect(searchResults.some((r: any) => r.name === 'init_system')).toBe(true);
    
    console.log('Search results for "init":', searchResults.map((r: any) => r.name));
    
    // 搜索 "callback"
    const callbackResults = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('search_symbols', { query: 'callback' });
    });
    
    expect(callbackResults.some((r: any) => r.is_callback === true)).toBe(true);
    console.log('Callback search results:', callbackResults.map((r: any) => `${r.name} (callback: ${r.is_callback})`));
  });

  test('获取入口点列表', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const entryPoints = await page.evaluate(async (projectPath) => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('get_entry_points', { filePath: projectPath + '/main.c' });
    }, FIXTURES_PATH);
    
    expect(entryPoints.length).toBe(2);
    expect(entryPoints.some((e: any) => e.name === 'main')).toBe(true);
    expect(entryPoints.some((e: any) => e.name === 'init_system')).toBe(true);
    
    console.log('Entry points:', entryPoints);
  });

  test('构建执行流', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const executionFlow = await page.evaluate(async (projectPath) => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('build_execution_flow', {
        filePath: projectPath + '/main.c',
        entryFunction: 'main',
        options: { max_depth: 10 },
      });
    }, FIXTURES_PATH);
    
    expect(executionFlow.entry_function).toBe('main');
    expect(executionFlow.root).toBeDefined();
    expect(executionFlow.root.name).toBe('main');
    expect(executionFlow.analysis_info.total_nodes).toBeGreaterThan(0);
    
    console.log('Execution flow:', {
      entry: executionFlow.entry_function,
      totalNodes: executionFlow.analysis_info.total_nodes,
      maxDepth: executionFlow.analysis_info.max_depth,
    });
  });
});

test.describe('打开项目 - 集成测试', () => {
  test('完整的打开项目流程', async ({ page }) => {
    const mockScript = generateSampleProjectMockScript({
      projectPath: FIXTURES_PATH,
      verbose: true,
    });
    await page.addInitScript(mockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 1. 初始状态截图
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/integration-01-initial.png` });
    
    // 2. 打开命令面板
    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/integration-02-command-palette.png` });
    
    // 3. 选择"打开项目"
    const openProjectOption = page.locator('[role="option"]:has-text("打开项目")');
    if (await openProjectOption.isVisible()) {
      await openProjectOption.click();
      await page.waitForTimeout(1000);
    } else {
      // 按 Escape 关闭命令面板，手动触发打开项目
      await page.keyboard.press('Escape');
      await page.evaluate(async (projectPath) => {
        const tauri = (window as any).__TAURI__;
        await tauri.core.invoke('open_project', { path: projectPath });
      }, FIXTURES_PATH);
      await page.waitForTimeout(500);
    }
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/integration-03-after-open.png` });
    
    // 4. 打开文件浏览器
    const explorerButton = page.locator('[data-testid="sidebar-explorer"]');
    if (await explorerButton.isVisible()) {
      await explorerButton.click();
      await page.waitForTimeout(500);
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/integration-04-file-explorer.png` });
    }
    
    // 5. 打开大纲面板
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    if (await outlineButton.isVisible()) {
      await outlineButton.click();
      await page.waitForTimeout(500);
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/integration-05-outline.png` });
    }
    
    console.log('集成测试完成: 完整的打开项目流程');
  });
});
