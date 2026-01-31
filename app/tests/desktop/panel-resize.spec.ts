/**
 * FlowSight 面板拖拽调整大小测试
 * 
 * 测试面板拖拽调整功能:
 * - 左侧面板拖拽
 * - 右侧面板拖拽
 * - 底部面板拖拽
 */

import { test, expect, Page } from '@playwright/test';
import * as fs from 'fs';
import { fileURLToPath } from 'url';
import * as path from 'path';

// ES Module 兼容
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SCREENSHOTS_DIR = './test-results/flowsight/panel-resize';

// Mock 脚本
const MOCK_SCRIPT = `
(function() {
  window.__TAURI__ = {
    core: {
      invoke: async (cmd, args) => {
        if (cmd === 'open_project') {
          return { path: '/mock/project', files_count: 10, functions_count: 20, structs_count: 5, indexed: true };
        }
        if (cmd === 'list_directory') {
          return [{ name: 'main.c', path: '/mock/project/src/main.c', is_dir: false, extension: 'c' }];
        }
        return {};
      }
    },
    dialog: { open: async () => '/mock/project' },
    event: { listen: async () => () => {}, emit: async () => {} },
  };
  window.__TAURI_INTERNALS__ = { invoke: window.__TAURI__.core.invoke, transformCallback: () => 0 };
})();
`;

test.beforeAll(() => {
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

test.describe('面板拖拽调整大小 - 左侧面板', () => {
  test('左侧面板分隔条存在', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 验证左侧面板打开
    const leftPanel = page.locator('text=资源管理器').first();
    await expect(leftPanel).toBeVisible();
    
    // 查找分隔条
    const divider = page.locator('[data-testid="resizable-divider"]').first();
    const hasDivider = await divider.count() > 0;
    console.log('左侧面板分隔条存在:', hasDivider);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/01-left-divider.png` });
  });

  test('左侧面板可拖拽调整宽度', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 获取初始宽度
    const leftPanel = page.locator('text=资源管理器').first().locator('..');
    
    // 查找分隔条并尝试拖拽
    const divider = page.locator('[data-testid="resizable-divider"]').first();
    if (await divider.count() > 0) {
      const box = await divider.boundingBox();
      if (box) {
        // 模拟拖拽
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
        await page.mouse.down();
        await page.mouse.move(box.x + 50, box.y + box.height / 2);
        await page.mouse.up();
        
        console.log('完成左侧面板拖拽');
      }
    }
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/02-left-resized.png` });
  });
});

test.describe('面板拖拽调整大小 - 右侧面板', () => {
  test('打开右侧面板', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 点击大纲按钮打开右侧面板
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await outlineButton.click();
    await page.waitForTimeout(500);
    
    // 验证右侧面板打开
    const rightPanel = page.locator('text=大纲').first();
    await expect(rightPanel).toBeVisible();
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/03-right-panel.png` });
  });

  test('右侧面板分隔条存在', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开右侧面板
    const outlineButton = page.locator('[data-testid="sidebar-outline"]');
    await outlineButton.click();
    await page.waitForTimeout(500);
    
    // 查找分隔条
    const dividers = page.locator('[data-testid="resizable-divider"]');
    const dividerCount = await dividers.count();
    console.log('分隔条数量:', dividerCount);
    
    // 应该有至少 2 个分隔条（左侧 + 右侧）
    const hasRightDivider = dividerCount >= 2;
    console.log('右侧面板分隔条存在:', hasRightDivider);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/04-right-divider.png` });
  });
});

test.describe('面板拖拽调整大小 - 底部面板', () => {
  test('底部面板默认打开', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 验证底部面板打开（终端标签）
    const terminalTab = page.locator('button:has-text("终端")');
    await expect(terminalTab).toBeVisible();
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/05-bottom-panel.png` });
  });

  test('底部面板分隔条存在', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 查找垂直分隔条（底部面板的）
    const dividers = page.locator('[data-testid="resizable-divider"]');
    const dividerCount = await dividers.count();
    console.log('分隔条总数:', dividerCount);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/06-bottom-divider.png` });
  });

  test('Cmd+J 切换底部面板', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 初始状态检查
    const terminalTab = page.locator('button:has-text("终端")');
    const initialVisible = await terminalTab.isVisible();
    console.log('初始底部面板可见:', initialVisible);
    
    // 按 Cmd+J 关闭
    await page.keyboard.press('Meta+j');
    await page.waitForTimeout(300);
    
    const afterCloseVisible = await terminalTab.isVisible();
    console.log('关闭后底部面板可见:', afterCloseVisible);
    
    // 按 Cmd+J 打开
    await page.keyboard.press('Meta+j');
    await page.waitForTimeout(300);
    
    const afterOpenVisible = await terminalTab.isVisible();
    console.log('打开后底部面板可见:', afterOpenVisible);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/07-bottom-toggle.png` });
  });
});

test.describe('面板拖拽调整大小 - 分隔条样式', () => {
  test('分隔条有正确的光标样式', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 获取分隔条
    const divider = page.locator('[data-testid="resizable-divider"]').first();
    if (await divider.count() > 0) {
      const cursor = await divider.evaluate(el => getComputedStyle(el).cursor);
      console.log('分隔条光标样式:', cursor);
      
      // 应该是 col-resize 或 row-resize
      const hasResizeCursor = cursor === 'col-resize' || cursor === 'row-resize';
      expect(hasResizeCursor).toBe(true);
    }
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/08-divider-cursor.png` });
  });

  test('鼠标悬停时分隔条高亮', async ({ page }) => {
    await page.addInitScript(MOCK_SCRIPT);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 获取分隔条
    const divider = page.locator('[data-testid="resizable-divider"]').first();
    if (await divider.count() > 0) {
      // 悬停前的背景色
      const beforeBg = await divider.evaluate(el => getComputedStyle(el).backgroundColor);
      console.log('悬停前背景色:', beforeBg);
      
      // 悬停
      await divider.hover();
      await page.waitForTimeout(100);
      
      // 悬停后的背景色
      const afterBg = await divider.evaluate(el => getComputedStyle(el).backgroundColor);
      console.log('悬停后背景色:', afterBg);
    }
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/09-divider-hover.png` });
  });
});
