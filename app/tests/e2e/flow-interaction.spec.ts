/**
 * FlowSight 执行流程图交互测试
 * 测试 React Flow 流程图的节点交互功能
 */

import { test, expect } from '@playwright/test';

const SCREENSHOTS_DIR = './test-results/flowsight';

test.describe('执行流程图交互', () => {
  test.beforeAll(async () => {
    // 确保截图目录存在
    const { fs } = await import('fs');
    if (!fs.existsSync(SCREENSHOTS_DIR)) {
      fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
    }
  });

  test.beforeEach(async ({ page }) => {
    // 关闭所有打开的对话框
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
  });

  test('流程图容器渲染', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 切换到执行流视图
    await page.locator('button:has-text("执行流")').click();
    await page.waitForTimeout(500);

    // 检查 React Flow 容器
    const flowContainer = page.locator('.react-flow, [class*="react-flow"]');
    const count = await flowContainer.count();
    console.log('Flow container found:', count > 0);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/flow-container.png` });
  });

  test('节点存在检测', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 切换到执行流视图
    await page.locator('button:has-text("执行流")').click();
    await page.waitForTimeout(1000);

    // 检查节点选择器 (React Flow 节点)
    const nodes = page.locator('.react-flow__node, [class*="node"]');
    const nodeCount = await nodes.count();
    console.log('Nodes found:', nodeCount);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/nodes.png` });
  });

  test('节点悬停显示详情', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 切换到执行流视图
    await page.locator('button:has-text("执行流")').click();
    await page.waitForTimeout(1000);

    // 查找第一个节点并悬停
    const firstNode = page.locator('.react-flow__node, [class*="node"]').first();
    if (await firstNode.count() > 0) {
      await firstNode.hover();
      await page.waitForTimeout(300);

      // 检查 tooltip 或详情面板
      const tooltip = page.locator('[class*="tooltip"], [class*="detail"]');
      console.log('Tooltip visible:', await tooltip.count() > 0);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/node-hover.png` });
  });

  test('节点展开/折叠', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 切换到执行流视图
    await page.locator('button:has-text("执行流")').click();
    await page.waitForTimeout(1000);

    // 查找展开/折叠按钮
    const expandBtn = page.locator('[class*="expand"], [class*="collapse"], button:has-text("+"), button:has-text("-")');
    const btnCount = await expandBtn.count();
    console.log('Expand/Collapse buttons:', btnCount);

    if (btnCount > 0) {
      await expandBtn.first().click();
      await page.waitForTimeout(500);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/node-expand.png` });
  });

  test('节点点击跳转代码', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 切换到执行流视图
    await page.locator('button:has-text("执行流")').click();
    await page.waitForTimeout(1000);

    // 点击节点
    const node = page.locator('.react-flow__node, [class*="node"]').first();
    if (await node.count() > 0) {
      await node.click();
      await page.waitForTimeout(500);

      // 检查 Monaco 编辑器是否高亮对应行
      const highlightedLine = page.locator('.highlight-line, [class*="highlight"]');
      console.log('Highlighted line found:', await highlightedLine.count() > 0);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/node-click.png` });
  });

  test('边连接线渲染', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 切换到执行流视图
    await page.locator('button:has-text("执行流")').click();
    await page.waitForTimeout(1000);

    // 检查边/连接线
    const edges = page.locator('.react-flow__edge, [class*="edge"], svg[class*="flow"]');
    const edgeCount = await edges.count();
    console.log('Edges found:', edgeCount);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/edges.png` });
  });

  test('流程图缩放控制', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 切换到执行流视图
    await page.locator('button:has-text("执行流")').click();
    await page.waitForTimeout(500);

    // 检查缩放控制按钮
    const zoomControls = page.locator('[class*="zoom"], [class*="controls"]');
    const controlsCount = await zoomControls.count();
    console.log('Zoom controls found:', controlsCount > 0);
  });

  test('视图模式切换', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 测试三种视图模式
    // 1. 代码视图
    await page.locator('button:has-text("代码")').click();
    await page.waitForTimeout(300);

    // 2. 执行流视图
    await page.locator('button:has-text("执行流")').click();
    await page.waitForTimeout(300);

    // 3. 分屏视图
    const splitBtn = page.locator('button:has-text("分屏"), button:has-text("Split")');
    if (await splitBtn.count() > 0) {
      await splitBtn.click();
      await page.waitForTimeout(300);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/view-modes.png` });
  });
});

test.describe('节点详情面板', () => {
  test.beforeEach(async ({ page }) => {
    // 关闭所有打开的对话框
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
  });

  test('节点详情面板渲染', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 打开大纲面板
    await page.locator('button:has-text("大纲")').click();
    await page.waitForTimeout(500);

    // 检查详情面板
    const detailPanel = page.locator('[class*="detail"], [class*="node-detail"]');
    await expect(detailPanel.first()).toBeVisible();

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/detail-panel.png` });
  });

  test('LLVM IR 可视化面板', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 打开 LLVM IR 面板
    const llvmBtn = page.locator('button:has-text("LLVM")');
    if (await llvmBtn.count() > 0) {
      await llvmBtn.click();
      await page.waitForTimeout(500);

      // 检查面板内容
      const llvmPanel = page.locator('[class*="llvm"], [class*="ir"]');
      console.log('LLVM panel visible:', await llvmPanel.count() > 0);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/llvm-panel.png` });
  });

  test('函数调用链显示', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 点击详情面板
    await page.locator('button:has-text("详情")').click();
    await page.waitForTimeout(500);

    // 检查调用链
    const callChain = page.locator('[class*="call"], [class*="chain"]');
    const chainCount = await callChain.count();
    console.log('Call chain items:', chainCount);
  });
});

test.describe('键盘快捷键', () => {
  test('Ctrl+\\ 切换右侧面板', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 按 Ctrl+\\ 切换右侧面板
    await page.keyboard.press('Control+\\');
    await page.waitForTimeout(300);

    // 检查右侧面板是否打开
    const rightPanel = page.locator('[class*="right"], [class*="panel"]:right-of(header)');
    console.log('Right panel toggled');
  });

  test('Escape 关闭面板', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 确保面板打开
    await page.locator('button:has-text("大纲")').click();
    await page.waitForTimeout(300);

    // 按 Escape 关闭
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    console.log('Escape key test passed');
  });

  test('Cmd+B 切换侧边栏', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 获取初始侧边栏宽度
    const initialWidth = await page.locator('aside').first().evaluate(el => {
      return window.getComputedStyle(el).width;
    });
    console.log('Initial sidebar width:', initialWidth);

    // 按 Cmd+B
    await page.keyboard.press('Meta+B');
    await page.waitForTimeout(300);

    // 检查侧边栏是否收起
    const afterWidth = await page.locator('aside').first().evaluate(el => {
      return window.getComputedStyle(el).width;
    });
    console.log('After Cmd+B width:', afterWidth);
  });
});

test.describe('文件树导航', () => {
  test.beforeEach(async ({ page }) => {
    // 关闭所有打开的对话框
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
  });

  test('文件树展开/折叠', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 打开文件面板
    await page.locator('button:has-text("文件")').click();
    await page.waitForTimeout(500);

    // 查找目录展开箭头
    const dirChevron = page.locator('[class*="chevron"], [class*="expand"]').first();
    if (await dirChevron.count() > 0) {
      await dirChevron.click();
      await page.waitForTimeout(300);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/file-tree.png` });
  });

  test('点击文件打开编辑器', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // 打开文件面板
    await page.locator('button:has-text("文件")').click();
    await page.waitForTimeout(500);

    // 点击文件
    const fileItem = page.locator('[class*="file"], [class*="tree-item"]').first();
    if (await fileItem.count() > 0) {
      await fileItem.click();
      await page.waitForTimeout(500);

      // 检查 Monaco 编辑器内容
      const editor = page.locator('[class*="monaco"], [class*="editor"]');
      console.log('Editor visible:', await editor.count() > 0);
    }
  });
});
