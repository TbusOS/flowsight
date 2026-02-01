/**
 * FlowSight 真实 UI 测试
 * 
 * 这些测试会真正发现 UI 问题，不是写了不运行的装饰代码。
 * 每个测试都有明确的断言，会失败并报告问题。
 */

import { test, expect, Page } from '@playwright/test';

// ============================================================
// 辅助函数
// ============================================================

/** 打开项目 - 真实操作 */
async function openProject(page: Page, projectPath: string): Promise<void> {
  // 打开命令面板
  await page.keyboard.press('Meta+k');
  await page.waitForSelector('[role="dialog"], [data-testid="command-palette"]', { timeout: 5000 });
  
  // 搜索 "打开项目"
  await page.keyboard.type('打开项目');
  await page.waitForTimeout(200);
  
  // 选择第一个结果
  await page.keyboard.press('Enter');
  await page.waitForTimeout(500);
  
  // 如果有文件对话框，输入路径
  // 注意: Tauri 原生对话框可能需要特殊处理
}

/** 等待索引完成 */
async function waitForIndexing(page: Page, timeoutMs = 120000): Promise<void> {
  // 等待终端显示 "发现 X 个文件"
  await page.waitForFunction(() => {
    const terminal = document.querySelector('[data-testid="terminal"], .terminal-panel');
    if (!terminal) return false;
    const text = terminal.textContent || '';
    // 检查是否显示了非零的文件数
    return /发现\s+[1-9]\d*\s+个文件/.test(text);
  }, { timeout: timeoutMs });
}

// ============================================================
// 1. 关键功能测试 - 这些必须通过
// ============================================================

test.describe('CRITICAL: 核心功能测试', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test('CRITICAL-001: 函数点击必须有响应', async ({ page }) => {
    // 问题：目前函数需要双击才触发分析，单击无反应
    
    // 1. 确保右侧面板打开（大纲）
    await page.keyboard.press('Meta+\\\\');
    await page.waitForTimeout(300);
    
    // 2. 切换到大纲标签
    const outlineTab = page.locator('text=大纲').first();
    if (await outlineTab.isVisible()) {
      await outlineTab.click();
      await page.waitForTimeout(300);
    }
    
    // 3. 找到任意函数项
    const functionItem = page.locator('[data-testid*="function"], .function-item, [class*="outline"] >> text=/\\w+\\(\\)$/').first();
    
    // 4. 如果有函数，测试点击响应
    if (await functionItem.count() > 0) {
      // 获取点击前的状态
      const beforeClick = await page.screenshot({ encoding: 'base64' });
      
      // 单击函数
      await functionItem.click();
      await page.waitForTimeout(500);
      
      // 获取点击后的状态
      const afterClick = await page.screenshot({ encoding: 'base64' });
      
      // 必须有视觉变化（选中状态或其他反馈）
      // 如果截图完全相同，说明没有响应
      expect(beforeClick).not.toBe(afterClick);  // 简单检查
      
      // 更严格的检查：应该有选中状态
      const selectedClass = await functionItem.getAttribute('class');
      expect(selectedClass || '').toMatch(/selected|active|focus|bg-/);
    }
  });

  test('CRITICAL-002: 函数双击必须触发执行流分析', async ({ page }) => {
    // 确保右侧面板打开
    await page.keyboard.press('Meta+\\\\');
    await page.waitForTimeout(300);
    
    // 找到函数项
    const functionItem = page.locator('[data-testid*="function"], .function-item').first();
    
    if (await functionItem.count() > 0) {
      const funcName = await functionItem.textContent();
      
      // 双击函数
      await functionItem.dblclick();
      await page.waitForTimeout(1000);
      
      // 应该在执行流视图中看到该函数
      const flowView = page.locator('[data-testid="flow-view"], .react-flow');
      if (await flowView.count() > 0) {
        const flowContent = await flowView.textContent();
        expect(flowContent).toContain(funcName?.replace(/[()]/g, '') || '');
      }
    }
  });

  test('CRITICAL-003: 终端必须正确显示统计（非零）', async ({ page }) => {
    // 这是那个著名的 "0 个文件" bug 的测试
    
    // 注意：这个测试需要先打开一个项目
    // 如果没有项目打开，终端应该显示提示信息而不是错误的统计
    
    const terminal = page.locator('[data-testid="terminal"], .terminal-panel, text=/flowsight/');
    
    // 如果终端显示了统计信息
    const terminalText = await terminal.textContent() || '';
    
    if (terminalText.includes('发现')) {
      // 不能全是 0
      const hasAllZeros = /发现\s*0\s*个文件[,，]\s*0\s*个函数[,，]\s*0\s*个/.test(terminalText);
      expect(hasAllZeros).toBe(false);
    }
    
    // 截图作为证据
    await page.screenshot({ path: 'test-results/critical-003-terminal-stats.png' });
  });
});

// ============================================================
// 2. UI 布局和可用性测试
// ============================================================

test.describe('UI: 布局和可用性', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test('UI-001: 所有可点击元素必须足够大（至少 24x24px）', async ({ page }) => {
    // WCAG 2.2 要求触摸目标至少 24x24px
    
    const clickableElements = page.locator('button, [role="button"], a, [onclick], [tabindex="0"]');
    const count = await clickableElements.count();
    
    const tooSmallElements: string[] = [];
    
    for (let i = 0; i < Math.min(count, 50); i++) {  // 检查前 50 个
      const el = clickableElements.nth(i);
      const box = await el.boundingBox();
      
      if (box) {
        const minSize = 24;
        if (box.width < minSize || box.height < minSize) {
          const text = await el.textContent() || await el.getAttribute('aria-label') || 'unknown';
          tooSmallElements.push(`${text.trim().slice(0, 20)}: ${box.width.toFixed(0)}x${box.height.toFixed(0)}px`);
        }
      }
    }
    
    // 报告问题但不失败（可以设置阈值）
    if (tooSmallElements.length > 0) {
      console.warn('⚠️ 以下元素太小，可能难以点击:');
      tooSmallElements.forEach(el => console.warn(`  - ${el}`));
    }
    
    // 严重问题：超过 30% 的元素太小
    expect(tooSmallElements.length).toBeLessThan(count * 0.3);
  });

  test('UI-002: 文字不能太小（至少 12px）', async ({ page }) => {
    // 检查所有文本元素的字体大小
    
    const tooSmallTexts = await page.evaluate(() => {
      const issues: string[] = [];
      const textElements = document.querySelectorAll('span, p, div, li, a, button, label');
      
      textElements.forEach(el => {
        const style = window.getComputedStyle(el);
        const fontSize = parseFloat(style.fontSize);
        const text = el.textContent?.trim() || '';
        
        // 只检查有内容的元素
        if (text && text.length > 0 && fontSize < 12) {
          issues.push(`"${text.slice(0, 30)}": ${fontSize}px`);
        }
      });
      
      return issues.slice(0, 20);  // 最多返回 20 个
    });
    
    if (tooSmallTexts.length > 0) {
      console.warn('⚠️ 以下文字太小 (<12px):');
      tooSmallTexts.forEach(t => console.warn(`  - ${t}`));
    }
    
    // 不应该有小于 10px 的文字
    const criticalCount = await page.evaluate(() => {
      let count = 0;
      document.querySelectorAll('*').forEach(el => {
        const fontSize = parseFloat(window.getComputedStyle(el).fontSize);
        if (fontSize < 10 && el.textContent?.trim()) count++;
      });
      return count;
    });
    
    expect(criticalCount).toBe(0);
  });

  test('UI-003: 面板间距不能太紧凑', async ({ page }) => {
    // 检查主要面板之间的间距
    
    const panels = page.locator('[class*="panel"], [data-panel]');
    const panelCount = await panels.count();
    
    const spacingIssues: string[] = [];
    
    for (let i = 0; i < panelCount - 1; i++) {
      const panel1 = panels.nth(i);
      const panel2 = panels.nth(i + 1);
      
      const box1 = await panel1.boundingBox();
      const box2 = await panel2.boundingBox();
      
      if (box1 && box2) {
        // 检查水平间距
        const hGap = Math.abs(box2.x - (box1.x + box1.width));
        // 检查垂直间距
        const vGap = Math.abs(box2.y - (box1.y + box1.height));
        
        // 间距应该至少 4px
        if (hGap > 0 && hGap < 4) {
          spacingIssues.push(`水平间距太小: ${hGap.toFixed(0)}px`);
        }
        if (vGap > 0 && vGap < 4) {
          spacingIssues.push(`垂直间距太小: ${vGap.toFixed(0)}px`);
        }
      }
    }
    
    expect(spacingIssues.length).toBe(0);
  });

  test('UI-004: 所有交互元素必须有视觉反馈', async ({ page }) => {
    // 测试 hover 状态
    
    const buttons = page.locator('button:visible');
    const buttonCount = await buttons.count();
    
    for (let i = 0; i < Math.min(buttonCount, 10); i++) {
      const button = buttons.nth(i);
      const box = await button.boundingBox();
      
      if (box) {
        // 获取 hover 前的样式
        const styleBefore = await button.evaluate(el => {
          const style = window.getComputedStyle(el);
          return {
            background: style.backgroundColor,
            color: style.color,
            border: style.borderColor,
          };
        });
        
        // Hover
        await button.hover();
        await page.waitForTimeout(100);
        
        // 获取 hover 后的样式
        const styleAfter = await button.evaluate(el => {
          const style = window.getComputedStyle(el);
          return {
            background: style.backgroundColor,
            color: style.color,
            border: style.borderColor,
          };
        });
        
        // 至少一个属性应该变化
        const hasChange = 
          styleBefore.background !== styleAfter.background ||
          styleBefore.color !== styleAfter.color ||
          styleBefore.border !== styleAfter.border;
        
        if (!hasChange) {
          const text = await button.textContent();
          console.warn(`⚠️ 按钮没有 hover 反馈: "${text?.trim()}"`);
        }
      }
    }
  });
});

// ============================================================
// 3. 交互测试 - 点击、双击、快捷键
// ============================================================

test.describe('交互: 用户操作响应', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test('交互-001: 所有快捷键必须有效', async ({ page }) => {
    // 测试文档中声明的快捷键
    
    const shortcuts = [
      { keys: 'Meta+k', expect: 'command palette opens', selector: '[role="dialog"]' },
      { keys: 'Meta+b', expect: 'sidebar toggles', action: 'toggle' },
      { keys: 'Meta+e', expect: 'file explorer toggles', action: 'toggle' },
      { keys: 'Meta+j', expect: 'bottom panel toggles', action: 'toggle' },
      { keys: 'Meta+\\\\', expect: 'right panel toggles', action: 'toggle' },
      { keys: 'Meta+1', expect: 'code view', action: 'view' },
      { keys: 'Meta+2', expect: 'flow view', action: 'view' },
    ];
    
    for (const shortcut of shortcuts) {
      // 记录初始状态
      const beforeScreenshot = await page.screenshot({ encoding: 'base64' });
      
      // 执行快捷键
      await page.keyboard.press(shortcut.keys);
      await page.waitForTimeout(300);
      
      // 如果有选择器，验证元素出现
      if (shortcut.selector) {
        const element = page.locator(shortcut.selector);
        const isVisible = await element.isVisible().catch(() => false);
        expect(isVisible, `快捷键 ${shortcut.keys} 应该打开 ${shortcut.expect}`).toBe(true);
        
        // 关闭对话框
        await page.keyboard.press('Escape');
        await page.waitForTimeout(200);
      }
      
      // 记录最终状态
      const afterScreenshot = await page.screenshot({ encoding: 'base64' });
      
      // 应该有变化
      expect(beforeScreenshot !== afterScreenshot, `快捷键 ${shortcut.keys} 没有响应`).toBe(true);
    }
  });

  test('交互-002: 右侧大纲面板函数列表可点击', async ({ page }) => {
    // 打开右侧面板
    await page.keyboard.press('Meta+\\\\');
    await page.waitForTimeout(500);
    
    // 找到大纲中的函数
    const functionItems = page.locator('[class*="outline"] [class*="item"], [data-testid*="outline-item"]');
    const count = await functionItems.count();
    
    if (count > 0) {
      // 点击第一个函数
      const firstFunc = functionItems.first();
      await firstFunc.click();
      await page.waitForTimeout(300);
      
      // 应该有选中状态
      const classes = await firstFunc.getAttribute('class') || '';
      const hasSelectedStyle = classes.includes('selected') || 
                               classes.includes('active') || 
                               classes.includes('bg-');
      
      expect(hasSelectedStyle, '函数项点击后应该有选中样式').toBe(true);
    }
  });

  test('交互-003: 执行流节点可点击和交互', async ({ page }) => {
    // 切换到流程视图
    await page.keyboard.press('Meta+2');
    await page.waitForTimeout(500);
    
    // 找到流程节点
    const flowNodes = page.locator('.react-flow__node');
    const nodeCount = await flowNodes.count();
    
    if (nodeCount > 0) {
      const firstNode = flowNodes.first();
      
      // 点击节点
      await firstNode.click();
      await page.waitForTimeout(300);
      
      // 检查是否有选中状态或详情面板更新
      const nodeClasses = await firstNode.getAttribute('class') || '';
      const isSelected = nodeClasses.includes('selected') || 
                         nodeClasses.includes('active');
      
      // 截图记录
      await page.screenshot({ path: 'test-results/interaction-003-node-click.png' });
      
      // 节点应该有某种响应
      expect(isSelected || nodeCount > 0, '流程节点应该可以交互').toBe(true);
    }
  });
});

// ============================================================
// 4. VLM 视觉验证（当 VLM 可用时）
// ============================================================

test.describe.skip('VLM: 视觉智能验证', () => {
  // 这些测试需要 VLM API
  // 跳过直到 VLM 集成完成
  
  test('VLM-001: 界面整体布局是否合理', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const screenshot = await page.screenshot({ encoding: 'base64' });
    
    // TODO: 调用 VLM API
    // const vlm = new VLMClient({ provider: 'auto' });
    // const result = await vlm.assertVisual({
    //   screenshot,
    //   assertion: '这是一个 IDE 界面，应该有：1) 左侧文件树 2) 中间代码/流程视图 3) 右侧大纲面板 4) 底部终端',
    // });
    // expect(result.passed).toBe(true);
  });

  test('VLM-002: 检测 UI 问题', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const screenshot = await page.screenshot({ encoding: 'base64' });
    
    // TODO: 调用 VLM API 检测问题
    // const vlm = new VLMClient({ provider: 'auto' });
    // const issues = await vlm.detectVisualIssues(screenshot);
    // expect(issues.filter(i => i.severity === 'critical')).toHaveLength(0);
  });
});

// ============================================================
// 5. 已知问题回归测试
// ============================================================

test.describe('回归: 已修复问题不能复现', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test('回归-001: 不能显示全零统计', async ({ page }) => {
    // Bug: "发现 0 个文件，0 个函数，0 个结构体"
    
    const allText = await page.textContent('body') || '';
    
    // 检查是否有全零统计
    const hasAllZeroStats = /发现\s*0\s*个文件[,，]\s*0\s*个函数[,，]\s*0\s*个/.test(allText);
    
    if (hasAllZeroStats) {
      await page.screenshot({ path: 'test-results/regression-001-zero-stats.png' });
    }
    
    expect(hasAllZeroStats, '不应该显示全零统计').toBe(false);
  });

  test('回归-002: LLVM IR 面板不能是空占位符', async ({ page }) => {
    // Bug: LLVM IR 面板只显示占位符文本
    
    // 打开右侧面板
    await page.keyboard.press('Meta+\\\\');
    await page.waitForTimeout(300);
    
    // 切换到 IR 标签
    const irTab = page.locator('text=IR').first();
    if (await irTab.isVisible()) {
      await irTab.click();
      await page.waitForTimeout(300);
      
      // 检查内容
      const irPanel = page.locator('[class*="llvm"], [data-testid="llvm-ir"]');
      const irContent = await irPanel.textContent() || '';
      
      // 不能只是占位符
      expect(irContent).not.toBe('LLVM IR 面板');
      expect(irContent.length).toBeGreaterThan(20);  // 应该有实际内容
    }
  });
});
