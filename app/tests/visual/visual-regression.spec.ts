/**
 * 视觉回归测试
 * 
 * 使用 Playwright 的截图对比功能检测 UI 变化
 * 
 * 首次运行会生成基线图片，后续运行会与基线对比
 * 
 * 运行命令:
 *   npx playwright test tests/visual/ --update-snapshots  # 更新基线
 *   npx playwright test tests/visual/                      # 对比测试
 */

import { test, expect } from '@playwright/test'

// Mock 脚本
function generateMock() {
  return `
(function() {
  window.__TAURI__ = {
    core: {
      invoke: async (cmd, args) => {
        switch (cmd) {
          case 'list_directory':
            return [
              { name: 'gpio-dwapb.c', path: '/test/gpio-dwapb.c', is_dir: false, extension: 'c' },
              { name: 'drivers', path: '/test/drivers', is_dir: true },
            ];
          case 'read_file':
            return '// GPIO Driver\\nstatic int dwapb_gpio_probe(struct platform_device *pdev) {\\n  return 0;\\n}';
          case 'get_functions':
            return [
              { name: 'dwapb_gpio_probe', return_type: 'int', line: 2 },
            ];
          case 'open_project':
            return { path: args.path, files_count: 10, functions_count: 50 };
          case 'build_execution_flow':
            return {
              entry_function: 'probe',
              nodes: [
                { id: 'n1', label: 'probe', node_type: 'entry', line: 10 },
                { id: 'n2', label: 'init', node_type: 'function', line: 20 },
                { id: 'n3', label: 'handler', node_type: 'callback', line: 30 },
              ],
              edges: [
                { source: 'n1', target: 'n2', edge_type: 'sync' },
                { source: 'n2', target: 'n3', edge_type: 'async' },
              ]
            };
          default:
            return null;
        }
      }
    },
    event: {
      listen: async () => () => {},
      emit: async () => {}
    },
    dialog: { open: async () => '/test/project' }
  };
})();
`;
}

test.describe('视觉回归测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    await page.waitForTimeout(1000)
  })

  test('初始页面布局', async ({ page }) => {
    // 截图对比初始页面
    await expect(page).toHaveScreenshot('01-initial-layout.png', {
      threshold: 0.1, // 允许 10% 差异（颜色微调等）
      maxDiffPixels: 1000,
    })
  })

  test('侧边栏展开状态', async ({ page }) => {
    // 确保侧边栏可见
    const sidebar = page.locator('[data-testid="sidebar"]').first()
    if (await sidebar.isVisible()) {
      await expect(page).toHaveScreenshot('02-sidebar-expanded.png', {
        threshold: 0.1,
      })
    }
  })

  test('深色主题', async ({ page }) => {
    // 应用已经是深色主题
    await expect(page).toHaveScreenshot('03-dark-theme.png', {
      threshold: 0.1,
    })
  })

  test('打开项目后的文件树', async ({ page }) => {
    // 点击打开项目
    const openButton = page.locator('button:has-text("打开项目")')
    if (await openButton.isVisible()) {
      await openButton.click()
      await page.waitForTimeout(500)
    }

    await expect(page).toHaveScreenshot('04-file-tree.png', {
      threshold: 0.15, // 文件树可能有动态内容
    })
  })

  test('执行流视图', async ({ page }) => {
    // 切换到执行流视图
    await page.keyboard.press('Meta+2')
    await page.waitForTimeout(500)

    // 如果有分析按钮，点击分析
    const analyzeButton = page.locator('button:has-text("分析")').first()
    if (await analyzeButton.isVisible()) {
      await analyzeButton.click()
      await page.waitForTimeout(1000)
    }

    await expect(page).toHaveScreenshot('05-flow-view.png', {
      threshold: 0.2, // 流程图位置可能有微小变化
      maxDiffPixelRatio: 0.1,
    })
  })

  test('命令面板', async ({ page }) => {
    // 打开命令面板
    await page.keyboard.press('F1')
    await page.waitForTimeout(300)

    const commandPalette = page.locator('[role="dialog"]')
    if (await commandPalette.isVisible()) {
      await expect(page).toHaveScreenshot('06-command-palette.png', {
        threshold: 0.1,
      })
    }
  })

  test('底部面板展开', async ({ page }) => {
    // 打开终端面板
    await page.keyboard.press('Meta+j')
    await page.waitForTimeout(300)

    await expect(page).toHaveScreenshot('07-bottom-panel.png', {
      threshold: 0.15,
    })
  })
})

test.describe('组件级视觉测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    await page.waitForTimeout(500)
  })

  test('状态栏样式', async ({ page }) => {
    const statusBar = page.locator('[data-testid="status-bar"]').first()
    if (await statusBar.isVisible()) {
      await expect(statusBar).toHaveScreenshot('component-status-bar.png', {
        threshold: 0.1,
      })
    }
  })

  test('头部导航样式', async ({ page }) => {
    const header = page.locator('header').first()
    if (await header.isVisible()) {
      await expect(header).toHaveScreenshot('component-header.png', {
        threshold: 0.1,
      })
    }
  })
})
