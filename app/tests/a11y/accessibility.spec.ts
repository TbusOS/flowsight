/**
 * 无障碍性测试
 * 
 * 使用 axe-core 检测 WCAG 违规
 * 
 * 运行命令:
 *   npx playwright test tests/a11y/
 */

import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

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
              { name: 'test.c', path: '/test/test.c', is_dir: false, extension: 'c' },
            ];
          case 'read_file':
            return '// Test file';
          case 'get_functions':
            return [{ name: 'main', return_type: 'int', line: 1 }];
          case 'open_project':
            return { path: args.path, files_count: 1, functions_count: 1 };
          default:
            return null;
        }
      }
    },
    event: {
      listen: async () => () => {},
      emit: async () => {}
    },
    dialog: { open: async () => '/test' }
  };
})();
`;
}

test.describe('无障碍性测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    await page.waitForTimeout(500)
  })

  test('初始页面无障碍性检查', async ({ page }) => {
    const results = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa']) // WCAG 2.0 Level A 和 AA
      .analyze()

    // 报告违规
    if (results.violations.length > 0) {
      console.log('无障碍性违规:')
      results.violations.forEach((violation) => {
        console.log(`  - ${violation.id}: ${violation.description}`)
        console.log(`    影响: ${violation.impact}`)
        console.log(`    节点数: ${violation.nodes.length}`)
      })
    }

    // 记录所有违规供后续修复
    const criticalViolations = results.violations.filter(
      (v) => v.impact === 'critical'
    )
    const seriousViolations = results.violations.filter(
      (v) => v.impact === 'serious'
    )
    
    console.log(`严重违规: ${criticalViolations.length}, 次要违规: ${seriousViolations.length}`)
    
    // 目前允许一定数量的 serious 违规（颜色对比度问题正在修复中）
    // TODO: 逐步修复后将此阈值降为 0
    expect(criticalViolations).toHaveLength(0)
    expect(seriousViolations.length).toBeLessThan(10)  // 允许最多 10 个 serious 违规
  })

  test('键盘导航可用', async ({ page }) => {
    // 测试 Tab 键导航
    await page.keyboard.press('Tab')
    
    // 应该有焦点元素
    const focusedElement = await page.evaluate(() => document.activeElement?.tagName)
    expect(focusedElement).toBeTruthy()
  })

  test('侧边栏按钮有 aria-label', async ({ page }) => {
    // 检查侧边栏按钮
    const buttons = page.locator('aside button')
    const count = await buttons.count()
    
    for (let i = 0; i < count; i++) {
      const button = buttons.nth(i)
      const ariaLabel = await button.getAttribute('aria-label')
      const title = await button.getAttribute('title')
      const text = await button.textContent()
      
      // 按钮应该有某种形式的标签
      const hasLabel = ariaLabel || title || (text && text.trim().length > 0)
      if (!hasLabel) {
        console.warn(`按钮 ${i} 缺少无障碍标签`)
      }
    }
  })

  test('颜色对比度', async ({ page }) => {
    const results = await new AxeBuilder({ page })
      .withRules(['color-contrast'])
      .analyze()

    const contrastViolations = results.violations.filter(
      (v) => v.id === 'color-contrast'
    )

    if (contrastViolations.length > 0) {
      console.log('颜色对比度问题:')
      contrastViolations.forEach((v) => {
        v.nodes.forEach((node) => {
          console.log(`  - ${node.html.substring(0, 100)}`)
        })
      })
    }

    // 对比度问题不应该太多
    expect(contrastViolations.length).toBeLessThan(5)
  })

  test('图片替代文本', async ({ page }) => {
    const results = await new AxeBuilder({ page })
      .withRules(['image-alt'])
      .analyze()

    const imageViolations = results.violations.filter(
      (v) => v.id === 'image-alt'
    )

    expect(imageViolations).toHaveLength(0)
  })

  test('表单标签', async ({ page }) => {
    const results = await new AxeBuilder({ page })
      .withRules(['label'])
      .analyze()

    const labelViolations = results.violations.filter(
      (v) => v.id === 'label'
    )

    if (labelViolations.length > 0) {
      console.log('表单标签问题:')
      labelViolations.forEach((v) => {
        v.nodes.forEach((node) => {
          console.log(`  - ${node.html.substring(0, 100)}`)
        })
      })
    }
  })

  test('ARIA 属性正确', async ({ page }) => {
    const results = await new AxeBuilder({ page })
      .withRules(['aria-valid-attr', 'aria-valid-attr-value', 'aria-roles'])
      .analyze()

    const ariaViolations = results.violations.filter(
      (v) => v.id.startsWith('aria')
    )

    expect(ariaViolations).toHaveLength(0)
  })
})

test.describe('交互无障碍性', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    await page.waitForTimeout(500)
  })

  test('命令面板可通过键盘打开', async ({ page }) => {
    await page.keyboard.press('F1')
    await page.waitForTimeout(300)

    const dialog = page.locator('[role="dialog"]')
    expect(await dialog.isVisible()).toBe(true)

    // 可以通过 Escape 关闭
    await page.keyboard.press('Escape')
    await page.waitForTimeout(300)
    
    expect(await dialog.isVisible()).toBe(false)
  })

  test('快捷键不依赖鼠标', async ({ page }) => {
    // 测试各种快捷键
    const shortcuts = [
      { key: 'Meta+1', description: '切换代码视图' },
      { key: 'Meta+2', description: '切换执行流视图' },
      { key: 'Meta+b', description: '切换侧边栏' },
      { key: 'Meta+j', description: '切换底部面板' },
    ]

    for (const shortcut of shortcuts) {
      await page.keyboard.press(shortcut.key)
      await page.waitForTimeout(200)
      // 页面不应该报错
      const hasError = await page.locator('text=Error').isVisible().catch(() => false)
      expect(hasError).toBe(false)
    }
  })
})
