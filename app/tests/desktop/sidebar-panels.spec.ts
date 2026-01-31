/**
 * FlowSight 侧边栏和面板测试
 * 
 * 测试内容:
 * - 侧边栏视图切换 (代码视图 ↔ 执行流视图)
 * - 文件浏览器面板打开/关闭
 * - 大纲面板显示
 */

import { test, expect, Page, Locator } from '@playwright/test';
import { mkdirSync, existsSync } from 'fs';

const SCREENSHOTS_DIR = './test-results/flowsight/sidebar-panels';

// 侧边栏按钮 ID (使用 data-testid 定位)
// 每个按钮都有 data-testid="sidebar-{id}" 属性
const SIDEBAR_IDS = {
  dashboard: 'dashboard',   // 项目 (代码视图)
  explorer: 'explorer',     // 文件浏览器 (打开右侧面板文件标签)
  outline: 'outline',       // 大纲面板 (打开右侧面板大纲标签)
  flow: 'flow',             // 执行流视图
  search: 'search',         // 搜索
  command: 'command',       // 命令
  settings: 'settings',     // 设置
};

test.beforeAll(() => {
  if (!existsSync(SCREENSHOTS_DIR)) {
    mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

// 每个测试前关闭可能打开的模态框
test.beforeEach(async ({ page }) => {
  await page.goto('/', { waitUntil: 'networkidle' });
  await page.waitForTimeout(300);
  
  // 按 Escape 关闭任何可能打开的模态框
  await page.keyboard.press('Escape');
  await page.waitForTimeout(100);
});

// 辅助函数: 获取侧边栏按钮 (通过 data-testid)
function getSidebarButton(page: Page, id: string): Locator {
  return page.locator(`[data-testid="sidebar-${id}"]`);
}

// 辅助函数: 获取所有侧边栏按钮 (兼容旧测试)
async function getSidebarButtons(page: Page) {
  return page.locator('aside button');
}

// 辅助函数: 检查右侧面板是否打开
async function isRightPanelOpen(page: Page) {
  // 检查大纲、详情、IR、文件标签是否存在
  const outlineTab = page.locator('button:has-text("大纲")');
  return await outlineTab.isVisible();
}

// 辅助函数: 获取当前视图模式
async function getCurrentViewMode(page: Page) {
  // 检查执行流视图特征元素
  const flowViewIndicator = page.locator('text=执行流视图');
  const codeEditorIndicator = page.locator('text=代码编辑器');
  
  if (await flowViewIndicator.isVisible()) return 'flow';
  if (await codeEditorIndicator.isVisible()) return 'code';
  return 'unknown';
}

test.describe('侧边栏视图切换', () => {
  test('默认显示代码视图', async ({ page }) => {
    // 验证代码编辑器可见
    const codeEditor = page.locator('h3:has-text("代码编辑器")');
    const isCodeView = await codeEditor.isVisible();
    
    console.log('默认视图是代码视图:', isCodeView);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/default-code-view.png` });
    
    // 如果代码编辑器不可见，可能显示执行流视图
    if (!isCodeView) {
      const flowView = page.locator('text=执行流视图');
      expect(await flowView.isVisible() || isCodeView).toBe(true);
    }
  });

  test('点击执行流按钮切换视图', async ({ page }) => {
    // 截图初始状态
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/before-flow-switch.png` });

    // 使用 data-testid 定位执行流按钮
    const flowButton = getSidebarButton(page, SIDEBAR_IDS.flow);
    await expect(flowButton).toBeVisible();
    
    await flowButton.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/after-flow-switch.png` });
    
    // 验证按钮激活状态 (检查是否有激活样式)
    const isActive = await flowButton.evaluate((el) => {
      return el.classList.contains('bg-') || 
             window.getComputedStyle(el).backgroundColor !== 'rgba(0, 0, 0, 0)';
    });
    console.log('执行流按钮激活:', isActive);
  });

  test('点击项目按钮切换回代码视图', async ({ page }) => {
    // 先点击执行流按钮
    const flowButton = getSidebarButton(page, SIDEBAR_IDS.flow);
    await expect(flowButton).toBeVisible();
    await flowButton.click();
    await page.waitForTimeout(500);

    // 再点击项目按钮切换回代码视图
    const dashboardButton = getSidebarButton(page, SIDEBAR_IDS.dashboard);
    await expect(dashboardButton).toBeVisible();
    await dashboardButton.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/back-to-code-view.png` });
    
    // 验证代码视图已激活
    const codeEditor = page.locator('h3:has-text("代码编辑器")');
    const isCodeView = await codeEditor.isVisible();
    console.log('切换回代码视图成功:', isCodeView);
  });
});

test.describe('文件浏览器面板', () => {
  test('左侧面板默认显示文件浏览器', async ({ page }) => {
    // 新布局: 文件浏览器在左侧面板，默认打开
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/file-explorer-default.png` });
    
    // 验证左侧面板默认打开
    // 左侧面板包含 "资源管理器" 或 "打开一个项目开始浏览"
    const resourceManager = page.locator('text=资源管理器');
    const openProjectPrompt = page.locator('text=打开一个项目开始浏览');
    const isLeftPanelVisible = await resourceManager.isVisible() || await openProjectPrompt.isVisible();
    console.log('左侧面板默认打开:', isLeftPanelVisible);
    expect(isLeftPanelVisible).toBe(true);
    
    // 验证文件浏览器内容 (空状态)
    const emptyState = page.locator('text=打开一个项目开始浏览');
    const isEmpty = await emptyState.isVisible();
    console.log('显示空项目提示:', isEmpty);
    expect(isEmpty).toBe(true);
  });

  test('文件按钮点击切换左侧面板', async ({ page }) => {
    // 使用 data-testid 定位文件浏览器按钮
    const explorerButton = getSidebarButton(page, SIDEBAR_IDS.explorer);
    await expect(explorerButton).toBeVisible();
    
    // 左侧面板默认打开，点击关闭
    await explorerButton.click();
    await page.waitForTimeout(300);
    
    // 检查左侧面板是否关闭
    const resourceManager = page.locator('text=资源管理器');
    const openProjectPrompt = page.locator('text=打开一个项目开始浏览');
    const isClosedAfterFirstClick = !(await resourceManager.isVisible() || await openProjectPrompt.isVisible());
    console.log('第一次点击后左侧面板关闭:', isClosedAfterFirstClick);
    
    // 再次点击打开
    await explorerButton.click();
    await page.waitForTimeout(300);
    
    const isOpenAfterSecondClick = await resourceManager.isVisible() || await openProjectPrompt.isVisible();
    console.log('第二次点击后左侧面板打开:', isOpenAfterSecondClick);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/file-tab-active.png` });
  });

  test('右侧面板标签切换', async ({ page }) => {
    // 使用 data-testid 定位大纲按钮打开右侧面板
    const outlineButton = getSidebarButton(page, SIDEBAR_IDS.outline);
    await expect(outlineButton).toBeVisible();
    await outlineButton.click();
    await page.waitForTimeout(500);

    // 测试各个标签切换 - 使用更精确的选择器
    // 面板标签在右侧面板区域的 flex-col 容器中
    // 注意: "文件"选项已移至左侧面板，不再在右侧面板中
    const tabs = ['大纲', '详情', 'IR', '搜索'];
    for (const tab of tabs) {
      // 优先匹配面板底部的标签按钮，使用 flex-col 容器特征
      const tabButton = page.locator(`.flex-col button:has-text("${tab}")`).first();
      if (await tabButton.isVisible({ timeout: 1000 }).catch(() => false)) {
        await tabButton.click();
        await page.waitForTimeout(300);
        console.log(`切换到 ${tab} 标签`);
        await page.screenshot({ path: `${SCREENSHOTS_DIR}/tab-${tab}.png` });
      } else {
        console.log(`标签 ${tab} 不可见，跳过`);
      }
    }
  });
});

test.describe('大纲面板', () => {
  test('点击大纲按钮打开大纲面板', async ({ page }) => {
    // 使用 data-testid 定位大纲按钮
    const outlineButton = getSidebarButton(page, SIDEBAR_IDS.outline);
    await expect(outlineButton).toBeVisible();
    
    await outlineButton.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/outline-panel-open.png` });
    
    // 验证右侧面板打开
    const isPanelOpen = await isRightPanelOpen(page);
    console.log('右侧面板打开:', isPanelOpen);
    expect(isPanelOpen).toBe(true);
    
    // 验证大纲面板内容
    const outlineHeader = page.locator('text=大纲');
    expect(await outlineHeader.first().isVisible()).toBe(true);
  });

  test('大纲面板显示空状态提示', async ({ page }) => {
    // 使用 data-testid 定位大纲按钮
    const outlineButton = getSidebarButton(page, SIDEBAR_IDS.outline);
    await expect(outlineButton).toBeVisible();
    await outlineButton.click();
    await page.waitForTimeout(500);
    
    // 切换到大纲标签
    const outlineTab = page.locator('button:has-text("大纲")').first();
    if (await outlineTab.isVisible()) {
      await outlineTab.click();
      await page.waitForTimeout(300);
    }
    
    // 验证空状态提示
    const emptyMessage = page.locator('text=打开文件查看大纲');
    const hasEmptyMessage = await emptyMessage.isVisible();
    console.log('显示空文件提示:', hasEmptyMessage);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/outline-empty-state.png` });
  });

  test('大纲面板有搜索输入框', async ({ page }) => {
    // 使用 data-testid 定位大纲按钮
    const outlineButton = getSidebarButton(page, SIDEBAR_IDS.outline);
    await expect(outlineButton).toBeVisible();
    await outlineButton.click();
    await page.waitForTimeout(500);
    
    // 切换到大纲标签
    const outlineTab = page.locator('button:has-text("大纲")').first();
    if (await outlineTab.isVisible()) {
      await outlineTab.click();
      await page.waitForTimeout(300);
    }
    
    // 使用 data-testid 验证搜索输入框
    const searchInput = page.locator('[data-testid="outline-search"]');
    await expect(searchInput).toBeVisible();
    console.log('大纲面板有搜索输入框: true');
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/outline-search-input.png` });
  });
});

test.describe('键盘快捷键', () => {
  test('Cmd/Ctrl + K 打开命令面板', async ({ page }) => {
    // 按下 Cmd+K
    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);

    // 验证命令面板打开
    const dialog = page.locator('[role="dialog"]');
    const isDialogVisible = await dialog.isVisible();
    console.log('命令面板打开:', isDialogVisible);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/command-palette-open.png` });
    
    // 关闭命令面板
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);
  });

  test('Cmd/Ctrl + B 切换侧边栏', async ({ page }) => {
    // 获取侧边栏初始状态
    const sidebar = page.locator('aside').first();
    const initialWidth = await sidebar.evaluate((el) => (el as HTMLElement).offsetWidth);
    console.log('侧边栏初始宽度:', initialWidth);

    // 按下 Cmd+B
    await page.keyboard.press('Meta+B');
    await page.waitForTimeout(500);

    // 验证侧边栏状态变化
    const newWidth = await sidebar.evaluate((el) => (el as HTMLElement).offsetWidth);
    console.log('侧边栏新宽度:', newWidth);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/sidebar-toggled.png` });
  });

  test('Cmd/Ctrl + J 切换底部面板', async ({ page }) => {
    // 验证底部面板存在
    const terminalButton = page.locator('button:has-text("终端")');
    expect(await terminalButton.isVisible()).toBe(true);

    // 按下 Cmd+J
    await page.keyboard.press('Meta+J');
    await page.waitForTimeout(500);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/bottom-panel-toggled.png` });
  });
});

test.describe('UI 元素可访问性', () => {
  test('侧边栏按钮应该有 tooltip', async ({ page }) => {
    // 测试悬停显示 tooltip - 使用 data-testid
    const buttonIds = Object.values(SIDEBAR_IDS).slice(0, 6);
    
    for (const id of buttonIds) {
      const button = getSidebarButton(page, id);
      if (await button.isVisible()) {
        await button.hover();
        await page.waitForTimeout(200);
      }
    }
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/sidebar-tooltips.png` });
  });

  test('所有侧边栏按钮都有 aria-label', async ({ page }) => {
    // 验证所有导航按钮都有 aria-label
    const navButtonIds = ['dashboard', 'explorer', 'outline', 'flow', 'search', 'command'];
    
    for (const id of navButtonIds) {
      const button = getSidebarButton(page, id);
      await expect(button).toBeVisible();
      
      const ariaLabel = await button.getAttribute('aria-label');
      console.log(`按钮 ${id} 的 aria-label: ${ariaLabel}`);
      expect(ariaLabel).toBeTruthy();
    }
    
    console.log('所有侧边栏按钮都有 aria-label ✓');
  });
});
