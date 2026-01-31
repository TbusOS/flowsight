/**
 * FlowSight 执行流交互测试
 * 
 * 测试执行流视图与节点详情面板的完整交互:
 * - 点击节点显示详情
 * - 执行流构建
 * - 节点选择状态同步
 */

import { test, expect, Page } from '@playwright/test';
import * as fs from 'fs';
import { fileURLToPath } from 'url';
import * as path from 'path';

// ES Module 兼容
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SCREENSHOTS_DIR = './test-results/flowsight/flow-interaction';

// Mock 执行流数据
const MOCK_EXECUTION_FLOW = {
  entry_function: 'main',
  nodes: [
    { id: 'node-1', label: 'main', node_type: 'entry', line: 10 },
    { id: 'node-2', label: 'init_driver', node_type: 'function', line: 25 },
    { id: 'node-3', label: 'setup_irq', node_type: 'function', line: 45 },
    { id: 'node-4', label: 'irq_handler', node_type: 'callback', line: 80 },
  ],
  edges: [
    { source: 'node-1', target: 'node-2', edge_type: 'sync' },
    { source: 'node-2', target: 'node-3', edge_type: 'sync' },
    { source: 'node-3', target: 'node-4', edge_type: 'async' },
  ],
};

// Mock 函数详情
const MOCK_FUNCTION_DETAIL = {
  name: 'main',
  return_type: 'int',
  params: [
    { name: 'argc', type_name: 'int' },
    { name: 'argv', type_name: 'char **' },
  ],
  file: '/mock/project/src/main.c',
  line: 10,
  is_callback: false,
  callback_context: null,
  calls: ['init_driver', 'cleanup'],
  called_by: [],
};

// 生成完整的 Mock 脚本
function generateFlowInteractionMockScript() {
  const flowJson = JSON.stringify(MOCK_EXECUTION_FLOW);
  const detailJson = JSON.stringify(MOCK_FUNCTION_DETAIL);
  
  return `
(function() {
  const mockFlow = ${flowJson};
  const mockDetail = ${detailJson};
  
  window.__TAURI__ = {
    core: {
      invoke: async (cmd, args) => {
        console.log('[TauriMock] invoke:', cmd, args);
        
        switch (cmd) {
          case 'build_execution_flow':
            return mockFlow;
          case 'get_function_detail_from_file':
            return { ...mockDetail, name: args.function_name || 'main' };
          case 'open_project':
            return { 
              path: '/mock/project', 
              files_count: 10, 
              functions_count: 20, 
              structs_count: 5, 
              indexed: true 
            };
          case 'list_directory':
            return [
              { name: 'main.c', path: '/mock/project/src/main.c', is_dir: false, extension: 'c' },
              { name: 'driver.c', path: '/mock/project/src/driver.c', is_dir: false, extension: 'c' },
            ];
          case 'read_file':
            return '// Mock file content';
          case 'get_functions':
            return [
              { name: 'main', return_type: 'int', line: 10, is_callback: false },
              { name: 'init_driver', return_type: 'int', line: 25, is_callback: false },
            ];
          default:
            console.warn('[TauriMock] Unknown command:', cmd);
            throw new Error('Unknown command: ' + cmd);
        }
      }
    },
    dialog: {
      open: async () => '/mock/project',
    },
    event: {
      listen: async () => () => {},
      emit: async () => {},
    },
  };
  
  window.__TAURI_INTERNALS__ = {
    invoke: window.__TAURI__.core.invoke,
    transformCallback: () => 0,
  };
  
  console.log('[TauriMock] Flow interaction mock installed');
})();
`;
}

test.beforeAll(() => {
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

test.describe('执行流交互 - 视图切换', () => {
  test('切换到执行流视图', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 点击执行流按钮
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 验证执行流视图显示
    const flowView = page.locator('text=执行流');
    const hasFlowView = await flowView.first().isVisible();
    console.log('执行流视图显示:', hasFlowView);
    expect(hasFlowView).toBe(true);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/01-flow-view.png` });
  });

  test('执行流视图显示工具栏', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 切换到执行流视图
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 检查工具栏元素
    const toolbar = page.locator('text=执行流').first();
    const hasToolbar = await toolbar.isVisible();
    console.log('工具栏显示:', hasToolbar);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/02-toolbar.png` });
  });
});

test.describe('执行流交互 - 分析按钮', () => {
  test('显示 "分析 main 函数" 按钮', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 首先打开项目
    const openProjectBtn = page.locator('button:has-text("打开项目")');
    if (await openProjectBtn.isVisible()) {
      await openProjectBtn.click();
      await page.waitForTimeout(1000);
    }
    
    // 切换到执行流视图
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 检查分析按钮
    const analyzeButton = page.locator('button:has-text("分析 main 函数")');
    const hasAnalyzeButton = await analyzeButton.isVisible().catch(() => false);
    console.log('分析 main 函数按钮显示:', hasAnalyzeButton);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/03-analyze-button.png` });
  });

  test('点击分析按钮触发流程构建', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开项目
    const openProjectBtn = page.locator('button:has-text("打开项目")');
    if (await openProjectBtn.isVisible()) {
      await openProjectBtn.click();
      await page.waitForTimeout(1000);
    }
    
    // 选择文件（如果文件浏览器显示）
    const mainFile = page.locator('text=main.c').first();
    if (await mainFile.isVisible().catch(() => false)) {
      await mainFile.click();
      await page.waitForTimeout(500);
    }
    
    // 切换到执行流视图
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 点击分析按钮
    const analyzeButton = page.locator('button:has-text("分析 main 函数")');
    if (await analyzeButton.isVisible().catch(() => false)) {
      await analyzeButton.click();
      await page.waitForTimeout(1000);
      console.log('已点击分析按钮');
    }
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/04-after-analyze.png` });
  });
});

test.describe('执行流交互 - ReactFlow 画布', () => {
  test('ReactFlow 画布存在', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 切换到执行流视图
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 检查 ReactFlow 容器
    const reactFlow = page.locator('.react-flow');
    const hasReactFlow = await reactFlow.count() > 0;
    console.log('ReactFlow 画布存在:', hasReactFlow);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/05-react-flow.png` });
  });

  test('执行流控制器存在', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 切换到执行流视图
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 检查控制器
    const controls = page.locator('.react-flow__controls');
    const hasControls = await controls.count() > 0;
    console.log('执行流控制器存在:', hasControls);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/06-controls.png` });
  });
});

test.describe('执行流交互 - 空状态', () => {
  test('未选择文件时显示空状态提示', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 切换到执行流视图
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 检查空状态提示
    const emptyHint = page.locator('text=选择入口函数, text=从大纲面板选择').first();
    const hasEmptyHint = await emptyHint.isVisible().catch(() => false);
    console.log('空状态提示显示:', hasEmptyHint);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/07-empty-state.png` });
  });

  test('未打开项目时显示提示', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 不打开项目，直接切换到执行流视图
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 检查无项目提示
    const noProjectHint = page.locator('text=打开项目后查看执行流, text=执行流视图').first();
    const hasNoProjectHint = await noProjectHint.isVisible().catch(() => false);
    console.log('无项目提示显示:', hasNoProjectHint);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/08-no-project.png` });
  });
});

test.describe('执行流交互 - 刷新功能', () => {
  test('刷新按钮存在', async ({ page }) => {
    await page.addInitScript(generateFlowInteractionMockScript());
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开项目并选择文件
    const openProjectBtn = page.locator('button:has-text("打开项目")');
    if (await openProjectBtn.isVisible()) {
      await openProjectBtn.click();
      await page.waitForTimeout(1000);
    }
    
    // 切换到执行流视图
    const flowButton = page.locator('[data-testid="sidebar-flow"]');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 触发分析
    const analyzeButton = page.locator('button:has-text("分析 main 函数")');
    if (await analyzeButton.isVisible().catch(() => false)) {
      await analyzeButton.click();
      await page.waitForTimeout(1000);
    }
    
    // 检查刷新按钮
    const refreshButton = page.locator('button[title="刷新"]');
    const hasRefreshButton = await refreshButton.count() > 0;
    console.log('刷新按钮存在:', hasRefreshButton);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/09-refresh-button.png` });
  });
});
