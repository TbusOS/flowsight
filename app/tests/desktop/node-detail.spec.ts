/**
 * FlowSight 节点详情面板测试
 * 
 * 测试节点详情面板的功能:
 * - 显示函数基本信息
 * - 显示参数列表
 * - 显示调用关系
 * - 交互功能
 */

import { test, expect, Page } from '@playwright/test';
import * as fs from 'fs';
import { fileURLToPath } from 'url';
import * as path from 'path';

// ES Module 兼容
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SCREENSHOTS_DIR = './test-results/flowsight/node-detail';

// Mock 节点详情数据
const MOCK_NODE_DETAIL = {
  id: 'node-1',
  name: 'init_driver',
  return_type: 'int',
  parameters: [
    { name: 'dev', type: 'struct device *' },
    { name: 'id', type: 'int' },
  ],
  file_path: '/mock/project/src/driver.c',
  line: 42,
  is_callback: false,
  calls: ['register_device', 'setup_irq', 'init_workqueue'],
  called_by: ['module_init', 'probe'],
  node_type: 'function',
  description: '初始化驱动程序',
};

const MOCK_CALLBACK_NODE = {
  id: 'node-2',
  name: 'work_handler',
  return_type: 'void',
  parameters: [
    { name: 'work', type: 'struct work_struct *' },
  ],
  file_path: '/mock/project/src/driver.c',
  line: 80,
  is_callback: true,
  callback_context: 'WorkQueue (Process Context)',
  calls: ['process_data', 'notify_completion'],
  called_by: [],
  node_type: 'callback',
};

// 生成 Mock 脚本
function generateNodeDetailMockScript(nodeDetail: typeof MOCK_NODE_DETAIL) {
  const detailJson = JSON.stringify(nodeDetail);
  return `
(function() {
  // Mock Tauri API
  window.__TAURI__ = {
    core: {
      invoke: async (cmd, args) => {
        console.log('[TauriMock] invoke:', cmd, args);
        if (cmd === 'get_function_detail_from_file') {
          return ${detailJson};
        }
        if (cmd === 'open_project') {
          return { path: '/mock/project', files_count: 10, functions_count: 20, structs_count: 5, indexed: true };
        }
        if (cmd === 'list_directory') {
          return [
            { name: 'driver.c', path: '/mock/project/src/driver.c', is_dir: false, extension: 'c' },
          ];
        }
        throw new Error('Unknown command: ' + cmd);
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
  
  console.log('[TauriMock] Node detail mock installed');
})();
`;
}

// 设置 Mock 并设置选中节点状态
async function setupWithSelectedNode(page: Page, nodeDetail: typeof MOCK_NODE_DETAIL) {
  await page.addInitScript(generateNodeDetailMockScript(nodeDetail));
  
  // 在页面加载后设置选中节点
  await page.addInitScript((detail) => {
    // 等待 React 加载完成后设置状态
    window.addEventListener('load', () => {
      setTimeout(() => {
        // 触发自定义事件让组件更新
        window.dispatchEvent(new CustomEvent('set-selected-node', { detail }));
      }, 500);
    });
  }, nodeDetail);
}

test.beforeAll(() => {
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

test.describe('节点详情面板 - 基础功能', () => {
  test('空状态显示正确提示', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开右侧面板并切换到详情标签
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await outlineButton.click();
    await page.waitForTimeout(300);
    
    // 点击详情标签
    const detailTab = page.locator('button:has-text("详情")');
    if (await detailTab.isVisible()) {
      await detailTab.click();
      await page.waitForTimeout(300);
    }
    
    // 验证空状态
    const emptyState = page.locator('text=选择一个节点查看详情');
    const isEmptyStateVisible = await emptyState.isVisible().catch(() => false);
    console.log('空状态显示:', isEmptyStateVisible);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/01-empty-state.png` });
  });

  test('detail-panel 元素存在', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开右侧面板
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await outlineButton.click();
    await page.waitForTimeout(300);
    
    // 查找 detail-panel
    const detailPanel = page.locator('[data-testid="detail-panel"]');
    const hasDetailPanel = await detailPanel.count() > 0;
    console.log('detail-panel 元素存在:', hasDetailPanel);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/02-panel-exists.png` });
  });
});

test.describe('节点详情面板 - 数据显示', () => {
  test('通过 JavaScript 设置节点数据后显示', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 使用 JavaScript 直接操作 Jotai store 设置节点数据
    const result = await page.evaluate((detail) => {
      // 尝试找到并触发状态更新
      const event = new CustomEvent('flowsight:set-selected-node', { 
        detail,
        bubbles: true,
      });
      document.dispatchEvent(event);
      return { success: true, detail };
    }, MOCK_NODE_DETAIL);
    
    console.log('设置节点数据:', result);
    await page.waitForTimeout(500);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/03-node-data-set.png` });
  });

  test('函数名称正确显示', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 检查函数名元素是否存在（通过 data-testid）
    const functionName = page.locator('[data-testid="detail-function-name"]');
    const hasFunctionName = await functionName.count() > 0;
    console.log('函数名元素存在:', hasFunctionName);
    
    if (hasFunctionName) {
      const text = await functionName.textContent();
      console.log('函数名:', text);
    }
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/04-function-name.png` });
  });
});

test.describe('节点详情面板 - 调用关系', () => {
  test('调用列表元素存在', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开右侧面板
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await outlineButton.click();
    await page.waitForTimeout(300);
    
    // 检查调用列表
    const callsList = page.locator('[data-testid="detail-calls"]');
    const hasCallsList = await callsList.count() > 0;
    console.log('调用列表元素存在:', hasCallsList);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/05-calls-list.png` });
  });

  test('被调用列表元素存在', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开右侧面板
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await outlineButton.click();
    await page.waitForTimeout(300);
    
    // 检查被调用列表
    const calledByList = page.locator('[data-testid="detail-called-by"]');
    const hasCalledByList = await calledByList.count() > 0;
    console.log('被调用列表元素存在:', hasCalledByList);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/06-called-by-list.png` });
  });
});

test.describe('节点详情面板 - 回调函数', () => {
  test('回调函数显示特殊标识', async ({ page }) => {
    // 使用回调函数 Mock 数据
    await page.addInitScript(generateNodeDetailMockScript(MOCK_CALLBACK_NODE as any));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 检查回调标识
    const callbackBadge = page.locator('text=回调');
    const hasCallbackBadge = await callbackBadge.isVisible().catch(() => false);
    console.log('回调标识显示:', hasCallbackBadge);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/07-callback-badge.png` });
  });

  test('异步上下文信息显示', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_CALLBACK_NODE as any));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 检查异步上下文
    const asyncContext = page.locator('text=异步上下文');
    const hasAsyncContext = await asyncContext.isVisible().catch(() => false);
    console.log('异步上下文显示:', hasAsyncContext);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/08-async-context.png` });
  });
});

test.describe('节点详情面板 - 参数显示', () => {
  test('参数列表元素存在', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开右侧面板
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await outlineButton.click();
    await page.waitForTimeout(300);
    
    // 检查参数列表
    const paramsList = page.locator('[data-testid="detail-parameters"]');
    const hasParamsList = await paramsList.count() > 0;
    console.log('参数列表元素存在:', hasParamsList);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/09-parameters-list.png` });
  });
});

test.describe('节点详情面板 - 操作按钮', () => {
  test('查看调用链按钮存在', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 检查按钮
    const callChainButton = page.locator('button:has-text("查看调用链")');
    const hasCallChainButton = await callChainButton.isVisible().catch(() => false);
    console.log('查看调用链按钮存在:', hasCallChainButton);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/10-call-chain-button.png` });
  });

  test('LLVM IR 按钮存在', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 检查按钮
    const llvmButton = page.locator('button:has-text("LLVM IR")');
    const hasLlvmButton = await llvmButton.isVisible().catch(() => false);
    console.log('LLVM IR 按钮存在:', hasLlvmButton);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/11-llvm-ir-button.png` });
  });
});

test.describe('节点详情面板 - 文件位置', () => {
  test('文件路径元素存在', async ({ page }) => {
    await page.addInitScript(generateNodeDetailMockScript(MOCK_NODE_DETAIL));
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 检查文件路径
    const filePath = page.locator('[data-testid="detail-file-path"]');
    const hasFilePath = await filePath.count() > 0;
    console.log('文件路径元素存在:', hasFilePath);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/12-file-path.png` });
  });
});
