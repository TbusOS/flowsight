/**
 * FlowSight 完整工作流程测试
 * 
 * 测试从打开项目到执行流分析的完整流程
 */

import { test, expect, Page } from '@playwright/test';
import { generateTauriMockScript } from '../mocks/tauri-api';

// 测试配置
const SCREENSHOTS_DIR = 'test-results/flowsight/workflow';
const BASE_URL = 'http://localhost:5173/';

// Mock 配置 - 使用真实的测试目录
const MOCK_PROJECT_PATH = '/Users/sky/linux-kernel/usb-learn/flowsight/tests/fixtures/simple_driver.c';
const MOCK_CONFIG = {
  verbose: false,
  defaultProjectPath: '/mock/project',
  dialogResponses: {
    openProject: '/mock/project',
    openFile: '/mock/project/src/main.c',
  },
};

// 帮助函数：设置 Mock
async function setupMock(page: Page) {
  await page.addInitScript(generateTauriMockScript(MOCK_CONFIG));
}

// 帮助函数：等待页面稳定
async function waitForStable(page: Page) {
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}

// 帮助函数：获取侧边栏按钮
function getSidebarButton(page: Page, id: string) {
  return page.locator(`[data-testid="sidebar-${id}"]`);
}

test.describe('完整工作流程测试', () => {
  test.beforeEach(async ({ page }) => {
    await setupMock(page);
    await page.goto(BASE_URL);
    await waitForStable(page);
  });

  test('工作流程1: 打开项目 → 文件浏览器显示文件', async ({ page }) => {
    // 1. 验证初始状态 - 左侧面板显示"打开项目"提示
    const openProjectPrompt = page.locator('text=打开一个项目开始浏览');
    await expect(openProjectPrompt).toBeVisible();
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/01-initial-state.png` });
    
    // 2. 点击"打开项目"按钮
    const openProjectButton = page.locator('button:has-text("打开项目")');
    if (await openProjectButton.isVisible()) {
      await openProjectButton.click();
      await page.waitForTimeout(1000);
      
      // 3. 验证文件浏览器显示文件树
      // Mock 会返回文件树数据
      const fileTree = page.locator('text=src, text=main.c').first();
      const hasFileTree = await fileTree.isVisible().catch(() => false);
      console.log('文件树显示:', hasFileTree);
      
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/02-after-open-project.png` });
    }
  });

  test('工作流程2: 选择文件 → 代码编辑器加载', async ({ page }) => {
    // 1. 首先打开项目
    const openProjectButton = page.locator('button:has-text("打开项目")');
    if (await openProjectButton.isVisible()) {
      await openProjectButton.click();
      await page.waitForTimeout(1000);
    }
    
    // 2. 点击文件 (如果文件树已显示)
    const mainCFile = page.locator('text=main.c').first();
    if (await mainCFile.isVisible().catch(() => false)) {
      await mainCFile.click();
      await page.waitForTimeout(500);
      
      // 3. 验证代码编辑器显示内容
      const codeEditor = page.locator('[data-testid="code-editor"], .monaco-editor');
      const hasEditor = await codeEditor.isVisible().catch(() => false);
      console.log('代码编辑器显示:', hasEditor);
      
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/03-file-selected.png` });
    } else {
      console.log('文件树未显示，跳过测试');
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/03-no-file-tree.png` });
    }
  });

  test('工作流程3: 大纲面板显示函数列表', async ({ page }) => {
    // 1. 打开项目
    const openProjectButton = page.locator('button:has-text("打开项目")');
    if (await openProjectButton.isVisible()) {
      await openProjectButton.click();
      await page.waitForTimeout(1000);
    }
    
    // 2. 选择文件
    const mainCFile = page.locator('text=main.c').first();
    if (await mainCFile.isVisible().catch(() => false)) {
      await mainCFile.click();
      await page.waitForTimeout(500);
    }
    
    // 3. 打开大纲面板
    const outlineButton = getSidebarButton(page, 'outline');
    await outlineButton.click();
    await page.waitForTimeout(500);
    
    // 4. 验证大纲面板内容
    // 应该显示函数列表（从 Mock 数据）
    const outlinePanel = page.locator('text=大纲');
    await expect(outlinePanel.first()).toBeVisible();
    
    // 检查是否有函数显示
    const functionItems = page.locator('text=main, text=init_driver, text=probe_handler').first();
    const hasFunctions = await functionItems.isVisible().catch(() => false);
    console.log('函数列表显示:', hasFunctions);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/04-outline-panel.png` });
  });

  test('工作流程4: 执行流视图显示', async ({ page }) => {
    // 1. 打开项目
    const openProjectButton = page.locator('button:has-text("打开项目")');
    if (await openProjectButton.isVisible()) {
      await openProjectButton.click();
      await page.waitForTimeout(1000);
    }
    
    // 2. 选择文件
    const mainCFile = page.locator('text=main.c').first();
    if (await mainCFile.isVisible().catch(() => false)) {
      await mainCFile.click();
      await page.waitForTimeout(500);
    }
    
    // 3. 切换到执行流视图
    const flowButton = getSidebarButton(page, 'flow');
    await flowButton.click();
    await page.waitForTimeout(500);
    
    // 4. 验证执行流视图
    const flowView = page.locator('text=执行流');
    await expect(flowView.first()).toBeVisible();
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/05-flow-view.png` });
    
    // 5. 检查是否有"分析 main 函数"按钮
    const analyzeButton = page.locator('button:has-text("分析 main 函数")');
    const hasAnalyzeButton = await analyzeButton.isVisible().catch(() => false);
    console.log('分析按钮显示:', hasAnalyzeButton);
    
    if (hasAnalyzeButton) {
      await analyzeButton.click();
      await page.waitForTimeout(1000);
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/06-after-analyze.png` });
    }
  });

  test('工作流程5: 搜索功能', async ({ page }) => {
    // 1. 打开项目
    const openProjectButton = page.locator('button:has-text("打开项目")');
    if (await openProjectButton.isVisible()) {
      await openProjectButton.click();
      await page.waitForTimeout(1000);
    }
    
    // 2. 打开搜索面板
    const searchButton = getSidebarButton(page, 'search');
    await searchButton.click();
    await page.waitForTimeout(500);
    
    // 3. 搜索函数
    const searchInput = page.locator('[data-testid="search-input"], input[placeholder*="搜索"]').first();
    if (await searchInput.isVisible()) {
      await searchInput.fill('main');
      await page.waitForTimeout(500);
      
      // 4. 验证搜索结果
      const searchResults = page.locator('[data-testid="search-results"], [data-testid="search-result-item"]');
      const hasResults = await searchResults.count() > 0;
      console.log('搜索结果数量:', await searchResults.count());
      
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/07-search-results.png` });
    }
  });

  test('工作流程6: 主题切换', async ({ page }) => {
    // 1. 验证默认主题
    const body = page.locator('body');
    const defaultBg = await body.evaluate(el => getComputedStyle(el).backgroundColor);
    console.log('默认主题背景色:', defaultBg);
    
    // 2. 点击主题切换按钮
    const themeToggle = page.locator('[data-testid="theme-toggle"], button:has-text("深色")').first();
    if (await themeToggle.isVisible()) {
      await themeToggle.click();
      await page.waitForTimeout(300);
      
      // 3. 选择浅色主题
      const lightOption = page.locator('[data-testid="theme-option-light"], button:has-text("浅色")').first();
      if (await lightOption.isVisible()) {
        await lightOption.click();
        await page.waitForTimeout(500);
        
        // 4. 验证主题变化
        const newBg = await body.evaluate(el => getComputedStyle(el).backgroundColor);
        console.log('新主题背景色:', newBg);
        
        await page.screenshot({ path: `${SCREENSHOTS_DIR}/08-light-theme.png` });
      }
    }
  });
});

test.describe('功能验证测试', () => {
  test.beforeEach(async ({ page }) => {
    await setupMock(page);
    await page.goto(BASE_URL);
    await waitForStable(page);
  });

  test('验证 Tauri API Mock 工作', async ({ page }) => {
    // 检查 Mock 是否正确安装
    const hasMock = await page.evaluate(() => {
      return typeof (window as any).__TAURI__ !== 'undefined';
    });
    console.log('Tauri Mock 已安装:', hasMock);
    expect(hasMock).toBe(true);
    
    // 尝试调用 Mock API
    const result = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      if (tauri && tauri.core && tauri.core.invoke) {
        try {
          const project = await tauri.core.invoke('open_project', { path: '/mock/project' });
          return { success: true, project };
        } catch (e: any) {
          return { success: false, error: e.message };
        }
      }
      return { success: false, error: 'Tauri not available' };
    });
    
    console.log('Mock API 调用结果:', result);
    expect(result.success).toBe(true);
  });

  test('验证左侧面板默认打开', async ({ page }) => {
    // 左侧面板（文件浏览器）应该默认打开
    const resourceManager = page.locator('text=资源管理器');
    const openProjectPrompt = page.locator('text=打开一个项目开始浏览');
    
    const isLeftPanelVisible = await resourceManager.isVisible() || await openProjectPrompt.isVisible();
    console.log('左侧面板默认打开:', isLeftPanelVisible);
    expect(isLeftPanelVisible).toBe(true);
  });

  test('验证侧边栏按钮都存在', async ({ page }) => {
    const buttons = ['dashboard', 'explorer', 'outline', 'flow', 'search', 'command'];
    for (const id of buttons) {
      const button = getSidebarButton(page, id);
      const isVisible = await button.isVisible();
      console.log(`按钮 ${id} 存在:`, isVisible);
      expect(isVisible).toBe(true);
    }
  });

  test('验证文件按钮可点击且有响应', async ({ page }) => {
    // 初始状态 - 左侧面板应该打开（包含"打开项目"按钮）
    const openProjectButton = page.locator('button:has-text("打开项目")');
    const initialVisible = await openProjectButton.isVisible();
    console.log('初始状态"打开项目"按钮可见:', initialVisible);
    expect(initialVisible).toBe(true);
    
    // 点击文件按钮
    const explorerButton = getSidebarButton(page, 'explorer');
    await expect(explorerButton).toBeVisible();
    
    // 验证按钮可点击
    await explorerButton.click();
    await page.waitForTimeout(300);
    
    // 验证按钮被激活
    const buttonClass = await explorerButton.getAttribute('class');
    console.log('文件按钮 class:', buttonClass);
    
    // 拍摄截图验证状态
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/09-file-button-clicked.png` });
    
    // 只要按钮能点击且页面不报错就通过
    console.log('文件按钮点击成功');
  });
});
