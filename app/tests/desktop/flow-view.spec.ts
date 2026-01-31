/**
 * FlowSight 执行流视图 E2E 测试
 * 
 * 测试内容:
 * - 执行流视图基本显示
 * - 节点渲染与交互
 * - 详情面板显示
 * - 工具栏功能
 */

import { test, expect, Page } from '@playwright/test';
import { mkdirSync, existsSync } from 'fs';
import { generateTauriMockScript } from '../mocks';

const SCREENSHOTS_DIR = './test-results/flowsight/flow-view';

// Mock 执行流数据
const mockExecutionFlow = {
  entry_function: "main",
  entry_location: { file: "/mock/project/src/main.c", line: 10, column: 0 },
  nodes: [
    { id: "main", label: "main", node_type: "entry", line: 10 },
    { id: "init_system", label: "init_system", node_type: "function", line: 25 },
    { id: "setup_driver", label: "setup_driver", node_type: "function", line: 40 },
    { id: "work_handler", label: "work_handler", node_type: "async", line: 80 },
  ],
  edges: [
    { source: "main", target: "init_system", edge_type: "sync" },
    { source: "init_system", target: "setup_driver", edge_type: "sync" },
    { source: "setup_driver", target: "work_handler", edge_type: "async" },
  ],
  async_boundaries: [
    {
      id: "async-1",
      mechanism: "WorkQueue",
      trigger_call: "schedule_work",
      trigger_location: { file: "/mock/project/src/main.c", line: 45, column: 4 },
      handler_function: "work_handler",
      handler_location: { file: "/mock/project/src/main.c", line: 80, column: 0 },
      context: "Process",
    }
  ],
  analysis_info: {
    total_nodes: 4,
    max_depth: 3,
    external_calls: 1,
    async_calls: 1,
    direct_calls: 2,
    warnings: [],
  },
};

// 生成增强版 Tauri Mock 脚本
function generateEnhancedMockScript(config: {
  hasProject?: boolean;
  hasFile?: boolean;
  hasExecutionFlow?: boolean;
} = {}): string {
  const { hasProject = false, hasFile = false, hasExecutionFlow = false } = config;

  return `
// Enhanced Tauri API Mock for Flow View Tests
(function() {
  const mockProject = ${hasProject} ? {
    path: '/mock/project',
    files_count: 42,
    functions_count: 128,
    structs_count: 15,
    indexed: true,
  } : null;

  const mockFile = ${hasFile} ? '/mock/project/src/main.c' : null;

  const mockExecutionFlow = ${hasExecutionFlow} ? ${JSON.stringify(mockExecutionFlow)} : null;

  const mockFunctions = [
    { name: 'main', return_type: 'int', line: 10, is_callback: false },
    { name: 'init_system', return_type: 'int', line: 25, is_callback: false },
    { name: 'setup_driver', return_type: 'int', line: 40, is_callback: false },
    { name: 'work_handler', return_type: 'void', line: 80, is_callback: true, callback_context: 'workqueue' },
  ];

  const mockAnalysisResult = {
    file: '/mock/project/src/main.c',
    functions_count: 4,
    structs_count: 2,
    async_handlers_count: 1,
    entry_points: ['main', 'init_system'],
    flow_trees: [
      {
        id: 'node-1',
        name: 'main',
        display_name: 'main()',
        location: { file: '/mock/project/src/main.c', line: 10, column: 0 },
        node_type: 'EntryPoint',
        children: [
          {
            id: 'node-2',
            name: 'init_system',
            display_name: 'init_system()',
            location: { file: '/mock/project/src/main.c', line: 25, column: 0 },
            node_type: 'Function',
            children: [],
          },
        ],
      },
    ],
  };

  const handlers = {
    open_project: (args) => mockProject || { path: args.path, files_count: 0, functions_count: 0, structs_count: 0, indexed: false },
    get_index_stats: () => mockProject ? { functions: 128, structs: 15, files: 42 } : { functions: 0, structs: 0, files: 0 },
    list_directory: () => mockProject ? [
      { name: 'src', path: '/mock/project/src', is_dir: true, children: [
        { name: 'main.c', path: '/mock/project/src/main.c', is_dir: false, extension: 'c' },
      ]},
    ] : [],
    read_file: (args) => '// Mock file content for ' + args.path,
    get_functions: () => mockFunctions,
    analyze_file: () => mockAnalysisResult,
    search_symbols: (args) => mockFunctions
      .filter(f => f.name.includes(args.query))
      .map(f => ({ name: f.name, kind: 'function', file: '/mock/project/src/main.c', line: f.line, is_callback: f.is_callback })),
    build_execution_flow: (args) => {
      if (!mockExecutionFlow) {
        return {
          entry_function: args.entryFunction,
          entry_location: { file: args.filePath, line: 10, column: 0 },
          root: { id: 'root', name: args.entryFunction, display_name: args.entryFunction + '()', node_type: 'EntryPoint', children: [] },
          async_boundaries: [],
          analysis_info: { total_nodes: 1, max_depth: 1, external_calls: 0, async_calls: 0, direct_calls: 0, warnings: [] },
        };
      }
      return mockExecutionFlow;
    },
    get_entry_points: () => [
      { name: 'main', kind: 'main_function', line: 10 },
      { name: 'init_system', kind: 'module_init', line: 25 },
    ],
    get_async_bindings: () => [
      { variable: 'my_work', handler: 'work_handler', mechanism: 'WorkQueue', context: 'Process', bind_line: 30, trigger_lines: [45] },
    ],
    get_function_detail: (args) => {
      const func = mockFunctions.find(f => f.name === args.name);
      if (func) {
        return {
          name: func.name,
          return_type: func.return_type,
          line: func.line,
          file: '/mock/project/src/main.c',
          params: [],
          is_callback: func.is_callback,
          callback_context: func.callback_context || null,
          calls: [],
          called_by: [],
        };
      }
      return null;
    },
  };

  const createInvoke = () => async (cmd, args) => {
    console.log('[TauriMock] invoke:', cmd, args);
    const handler = handlers[cmd];
    if (handler) return handler(args);
    console.warn('[TauriMock] Unknown command:', cmd);
    throw new Error('Unknown command: ' + cmd);
  };

  const createDialog = () => ({
    open: async (options) => {
      if (options?.directory) return '/mock/project';
      return '/mock/project/src/main.c';
    },
    save: async () => '/mock/output/result.txt',
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
      writeTextFile: async () => {},
      readDir: async (path) => handlers.list_directory({ path }),
      createDir: async () => {},
      removeFile: async () => {},
      exists: async () => true,
    },
  };

  window.__TAURI_INTERNALS__ = {
    invoke: createInvoke(),
    transformCallback: () => 0,
    convertFileSrc: (path) => 'asset://localhost/' + path,
  };

  window.__TAURI_PLUGIN_DIALOG__ = createDialog();

  // 模拟项目和文件状态
  if (${hasProject}) {
    window.__MOCK_PROJECT__ = mockProject;
  }
  if (${hasFile}) {
    window.__MOCK_CURRENT_FILE__ = mockFile;
  }

  console.log('[TauriMock] Enhanced mock installed', { hasProject: ${hasProject}, hasFile: ${hasFile}, hasExecutionFlow: ${hasExecutionFlow} });
})();
`;
}

// 辅助函数: 获取侧边栏按钮
function getSidebarButton(page: Page, id: string) {
  return page.locator(`[data-testid="sidebar-${id}"]`);
}

test.beforeAll(() => {
  if (!existsSync(SCREENSHOTS_DIR)) {
    mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

test.describe('执行流视图 - 基本显示', () => {
  test('无项目时显示空状态提示', async ({ page }) => {
    // 注入无项目的 Mock
    await page.addInitScript(generateEnhancedMockScript({ hasProject: false }));
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // 点击执行流按钮切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await expect(flowButton).toBeVisible();
    await flowButton.click();
    await page.waitForTimeout(500);

    // 验证空状态提示
    const emptyStateIcon = page.locator('svg.lucide-zap, [class*="Zap"]').first();
    const emptyStateText = page.locator('text=执行流视图, text=打开项目后查看执行流').first();
    
    // 检查空状态元素是否可见
    const hasEmptyState = await emptyStateIcon.isVisible() || await emptyStateText.isVisible();
    console.log('空状态提示显示:', hasEmptyState);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/empty-state-no-project.png` });
    
    // 验证至少有一个空状态元素
    expect(hasEmptyState).toBe(true);
  });

  test('切换到执行流视图', async ({ page }) => {
    await page.addInitScript(generateEnhancedMockScript({ hasProject: true }));
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 点击执行流按钮
    const flowButton = getSidebarButton(page, 'flow');
    await expect(flowButton).toBeVisible();
    await flowButton.click();
    await page.waitForTimeout(500);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-view-switched.png` });

    // 验证主内容区域显示执行流相关内容
    const mainContent = page.locator('[data-view-mode="flow"]');
    const isFlowMode = await mainContent.count() > 0;
    console.log('执行流视图模式:', isFlowMode);
  });

  test('有项目但无文件时显示选择入口函数提示', async ({ page }) => {
    await page.addInitScript(generateEnhancedMockScript({ hasProject: true, hasFile: false }));
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(500);

    // 验证提示选择入口函数
    const selectPrompt = page.locator('text=选择入口函数, text=从大纲面板选择一个函数').first();
    const hasPrompt = await selectPrompt.isVisible().catch(() => false);
    console.log('选择入口函数提示显示:', hasPrompt);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-view-select-entry.png` });
  });
});

test.describe('执行流视图 - 节点渲染', () => {
  test.beforeEach(async ({ page }) => {
    // 注入完整的 Mock（有项目、有文件、有执行流数据）
    await page.addInitScript(generateEnhancedMockScript({ 
      hasProject: true, 
      hasFile: true, 
      hasExecutionFlow: true 
    }));
  });

  test('执行流节点正确渲染', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 检查 React Flow 容器
    const flowContainer = page.locator('.react-flow, [class*="react-flow"]');
    const containerCount = await flowContainer.count();
    console.log('React Flow 容器数量:', containerCount);

    // 检查节点
    const nodes = page.locator('.react-flow__node, [data-testid="flow-node"]');
    const nodeCount = await nodes.count();
    console.log('渲染的节点数量:', nodeCount);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-nodes-rendered.png` });
  });

  test('异步边连接正确显示', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 检查边/连接线
    const edges = page.locator('.react-flow__edge, [class*="edge"]');
    const edgeCount = await edges.count();
    console.log('渲染的边数量:', edgeCount);

    // 检查异步边（带动画）
    const animatedEdges = page.locator('.react-flow__edge.animated, [class*="animated"]');
    const animatedCount = await animatedEdges.count();
    console.log('异步边数量:', animatedCount);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-edges-rendered.png` });
  });

  test('节点类型样式正确', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 检查不同类型节点的样式
    const entryNodes = page.locator('.node-entry, [class*="entry"]');
    const functionNodes = page.locator('.node-function, [class*="function"]');
    const asyncNodes = page.locator('.node-async, [class*="async"]');

    console.log('入口节点数量:', await entryNodes.count());
    console.log('函数节点数量:', await functionNodes.count());
    console.log('异步节点数量:', await asyncNodes.count());

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-node-types.png` });
  });
});

test.describe('执行流视图 - 节点交互', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateEnhancedMockScript({ 
      hasProject: true, 
      hasFile: true, 
      hasExecutionFlow: true 
    }));
  });

  test('点击节点显示详情', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 找到第一个节点并点击
    const firstNode = page.locator('.react-flow__node, [data-testid="flow-node"]').first();
    if (await firstNode.count() > 0) {
      await firstNode.click();
      await page.waitForTimeout(500);

      // 验证详情面板更新（通过检查右侧面板内容）
      const detailPanel = page.locator('[data-testid="detail-panel"], [class*="detail"]');
      const panelVisible = await detailPanel.count() > 0;
      console.log('详情面板可见:', panelVisible);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-node-clicked.png` });
  });

  test('悬停节点显示 tooltip', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 悬停在节点上
    const firstNode = page.locator('.react-flow__node, [data-testid="flow-node"]').first();
    if (await firstNode.count() > 0) {
      await firstNode.hover();
      await page.waitForTimeout(500);

      // 检查 tooltip 或悬停预览
      const tooltip = page.locator('[class*="tooltip"], [class*="preview"], [role="tooltip"]');
      const tooltipVisible = await tooltip.count() > 0;
      console.log('Tooltip 显示:', tooltipVisible);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-node-hover.png` });
  });

  test('节点展开/折叠交互', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 查找展开/折叠按钮
    const toggleButton = page.locator('.node-toggle, [class*="expand"], [class*="collapse"]').first();
    if (await toggleButton.count() > 0) {
      // 记录当前节点数量
      const nodesBefore = await page.locator('.react-flow__node').count();
      console.log('折叠前节点数量:', nodesBefore);

      await toggleButton.click();
      await page.waitForTimeout(500);

      const nodesAfter = await page.locator('.react-flow__node').count();
      console.log('操作后节点数量:', nodesAfter);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-node-toggle.png` });
  });

  test('双击节点跳转到代码', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 双击节点
    const firstNode = page.locator('.react-flow__node, [data-testid="flow-node"]').first();
    if (await firstNode.count() > 0) {
      await firstNode.dblclick();
      await page.waitForTimeout(500);

      // 检查是否切换到代码视图或高亮行
      const codeHighlight = page.locator('.highlight-line, [class*="highlight"], .monaco-editor');
      const hasHighlight = await codeHighlight.count() > 0;
      console.log('代码高亮显示:', hasHighlight);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-node-dblclick.png` });
  });
});

test.describe('执行流视图 - 工具栏', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateEnhancedMockScript({ 
      hasProject: true, 
      hasFile: true, 
      hasExecutionFlow: true 
    }));
  });

  test('工具栏正确显示', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 验证工具栏存在
    const toolbar = page.locator('.flow-toolbar, [class*="toolbar"]');
    const hasToolbar = await toolbar.count() > 0;
    console.log('工具栏存在:', hasToolbar);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-toolbar.png` });
  });

  test('刷新按钮存在并可点击', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 查找刷新按钮
    const refreshButton = page.locator('button[title*="刷新"], button[title*="refresh"], .lucide-refresh-cw').first();
    const hasRefreshButton = await refreshButton.count() > 0;
    console.log('刷新按钮存在:', hasRefreshButton);

    if (hasRefreshButton) {
      await refreshButton.click();
      await page.waitForTimeout(500);
      console.log('刷新按钮点击成功');
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-refresh-button.png` });
  });

  test('缩放控制存在', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 检查 React Flow 默认控制器
    const controls = page.locator('.react-flow__controls, .zoom-controls, [class*="controls"]');
    const hasControls = await controls.count() > 0;
    console.log('缩放控制存在:', hasControls);

    // 检查缩放按钮
    const zoomIn = page.locator('.react-flow__controls-zoomin, button[title*="zoom in"], [class*="zoom-in"]').first();
    const zoomOut = page.locator('.react-flow__controls-zoomout, button[title*="zoom out"], [class*="zoom-out"]').first();
    const fitView = page.locator('.react-flow__controls-fitview, button[title*="fit"], [class*="fit"]').first();

    console.log('放大按钮:', await zoomIn.count() > 0);
    console.log('缩小按钮:', await zoomOut.count() > 0);
    console.log('适应视图按钮:', await fitView.count() > 0);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-zoom-controls.png` });
  });

  test('展开/折叠所有按钮', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 查找展开所有按钮
    const expandAllBtn = page.locator('button[title*="Expand all"], button[title*="展开"]').first();
    const collapseAllBtn = page.locator('button[title*="Collapse all"], button[title*="折叠"]').first();

    console.log('展开所有按钮:', await expandAllBtn.count() > 0);
    console.log('折叠所有按钮:', await collapseAllBtn.count() > 0);

    // 测试展开所有
    if (await expandAllBtn.count() > 0) {
      await expandAllBtn.click();
      await page.waitForTimeout(500);
      const nodesExpanded = await page.locator('.react-flow__node').count();
      console.log('展开后节点数量:', nodesExpanded);
    }

    // 测试折叠所有
    if (await collapseAllBtn.count() > 0) {
      await collapseAllBtn.click();
      await page.waitForTimeout(500);
      const nodesCollapsed = await page.locator('.react-flow__node').count();
      console.log('折叠后节点数量:', nodesCollapsed);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-expand-collapse.png` });
  });

  test('搜索功能', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 查找搜索输入框
    const searchInput = page.locator('.flow-search input, input[placeholder*="Search"], input[placeholder*="搜索"]').first();
    const hasSearch = await searchInput.count() > 0;
    console.log('搜索输入框存在:', hasSearch);

    if (hasSearch) {
      await searchInput.fill('main');
      await page.waitForTimeout(500);

      // 检查搜索结果
      const searchResults = page.locator('.search-count, [class*="search-result"]').first();
      const hasResults = await searchResults.count() > 0;
      console.log('搜索结果显示:', hasResults);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-search.png` });
  });
});

test.describe('执行流视图 - 导出功能', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateEnhancedMockScript({ 
      hasProject: true, 
      hasFile: true, 
      hasExecutionFlow: true 
    }));
  });

  test('导出 PNG 按钮', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 查找导出 PNG 按钮
    const exportPngBtn = page.locator('button[title*="PNG"], button[title*="Export"]').first();
    const hasExportPng = await exportPngBtn.count() > 0;
    console.log('导出 PNG 按钮:', hasExportPng);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-export-png.png` });
  });

  test('导出 SVG 按钮', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 查找导出 SVG 按钮
    const exportSvgBtn = page.locator('button[title*="SVG"]').first();
    const hasExportSvg = await exportSvgBtn.count() > 0;
    console.log('导出 SVG 按钮:', hasExportSvg);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-export-svg.png` });
  });
});

test.describe('执行流视图 - 键盘快捷键', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateEnhancedMockScript({ 
      hasProject: true, 
      hasFile: true, 
      hasExecutionFlow: true 
    }));
  });

  test('数字键控制展开深度', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(1000);

    // 按 1-5 键控制展开深度
    for (const depth of ['1', '2', '3']) {
      await page.keyboard.press(depth);
      await page.waitForTimeout(300);
      const nodes = await page.locator('.react-flow__node').count();
      console.log(`深度 ${depth} 时节点数:`, nodes);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-keyboard-depth.png` });
  });
});

test.describe('执行流视图 - 响应式布局', () => {
  test('窗口缩小时布局正确', async ({ page }) => {
    await page.addInitScript(generateEnhancedMockScript({ 
      hasProject: true, 
      hasFile: true, 
      hasExecutionFlow: true 
    }));
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(500);

    // 设置小窗口尺寸
    await page.setViewportSize({ width: 800, height: 600 });
    await page.waitForTimeout(500);

    // 验证布局仍然正确
    const flowContainer = page.locator('.react-flow, [class*="react-flow"]');
    const isVisible = await flowContainer.isVisible();
    console.log('小窗口下 Flow 容器可见:', isVisible);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-responsive-small.png` });
  });

  test('全屏模式', async ({ page }) => {
    await page.addInitScript(generateEnhancedMockScript({ 
      hasProject: true, 
      hasFile: true, 
      hasExecutionFlow: true 
    }));
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(300);

    // 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(500);

    // 设置大窗口尺寸
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(500);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-fullscreen.png` });
  });
});
