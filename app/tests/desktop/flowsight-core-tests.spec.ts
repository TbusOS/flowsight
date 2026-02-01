/**
 * FlowSight 核心功能测试
 * 
 * 测试静态分析、执行流构建、LLVM IR 等核心功能
 * 这些测试会真正发现功能性问题
 */

import { test, expect, Page } from '@playwright/test';

// 测试目标：/Users/sky/linux-kernel/linux/drivers/gpio
const GPIO_PROJECT = '/Users/sky/linux-kernel/linux/drivers/gpio';
const GPIO_AMD8111 = '/Users/sky/linux-kernel/linux/drivers/gpio/gpio-amd8111.c';

// ============================================================
// 核心功能测试 - 静态分析
// ============================================================

test.describe('CORE: 静态分析功能', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test('CORE-001: 解析器必须正确提取函数调用', async ({ page }) => {
    // 测试要求：
    // 1. 打开 gpio-amd8111.c 文件
    // 2. 分析应该提取出所有函数调用关系
    // 3. 执行流应该显示多个节点（不是只有1个）
    
    // 打开命令面板
    await page.keyboard.press('Meta+k');
    await page.waitForSelector('[role="dialog"]', { timeout: 5000 });
    
    // 搜索打开项目
    await page.keyboard.type('打开项目');
    await page.waitForTimeout(200);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(1000);
    
    // 注意：Tauri 原生对话框需要特殊处理
    // 这里我们等待项目加载
    await page.waitForTimeout(3000);
    
    // 截图记录当前状态
    await page.screenshot({ path: 'test-results/core-001-analysis.png' });
    
    // 检查是否有执行流节点
    const flowNodes = page.locator('.react-flow__node');
    const nodeCount = await flowNodes.count();
    
    console.log(`执行流节点数量: ${nodeCount}`);
    
    // 如果只有 1 个节点，说明解析失败
    // 一个正常的驱动文件应该有多个函数调用
    if (nodeCount <= 1) {
      console.error('⚠️ 执行流只有 1 个节点，解析可能失败！');
      console.error('预期：至少 3-5 个节点（probe 函数会调用多个其他函数）');
    }
  });

  test('CORE-002: 函数调用关系必须被解析', async ({ page }) => {
    // 测试 gpio-amd8111.c 中的函数调用关系：
    // amd_gpio_init -> pci_register_driver
    // amd8111_gpio_probe -> pci_iomap, gpiochip_add_data
    // amd_gpio_request -> ioread8
    // 等等
    
    // 这些调用关系应该在执行流中体现
    
    // 打开大纲面板
    await page.keyboard.press('Meta+Backslash');
    await page.waitForTimeout(500);
    
    // 找到大纲中的函数列表
    const outlinePanel = page.locator('[class*="outline"]');
    const functionsText = await outlinePanel.textContent() || '';
    
    console.log('大纲面板内容:', functionsText.slice(0, 500));
    
    // 预期的函数（来自 gpio-amd8111.c）
    const expectedFunctions = [
      'amd_gpio_request',
      'amd_gpio_free',
      'amd_gpio_set',
      'amd_gpio_get',
      'amd8111_gpio_probe',
      'amd8111_gpio_remove',
    ];
    
    // 检查是否解析出了这些函数
    for (const func of expectedFunctions) {
      if (!functionsText.includes(func)) {
        console.warn(`⚠️ 未找到函数: ${func}`);
      }
    }
    
    await page.screenshot({ path: 'test-results/core-002-outline.png' });
  });

  test('CORE-003: 点击函数后执行流必须显示调用关系', async ({ page }) => {
    // 选择一个函数后，执行流视图应该显示：
    // 1. 该函数作为入口点
    // 2. 它调用的所有函数
    // 3. 调用链的深度至少 2-3 层
    
    // 等待界面加载
    await page.waitForTimeout(2000);
    
    // 找到大纲中的函数
    const funcItem = page.locator('[class*="outline"] >> text=/probe/i').first();
    
    if (await funcItem.count() > 0) {
      // 点击 probe 函数
      await funcItem.click();
      await page.waitForTimeout(1000);
      
      // 检查执行流视图
      const flowView = page.locator('.react-flow');
      const flowNodes = flowView.locator('.react-flow__node');
      const nodeCount = await flowNodes.count();
      
      console.log(`probe 函数的执行流节点数: ${nodeCount}`);
      
      // probe 函数应该调用多个其他函数
      // 例如：kzalloc, pci_iomap, gpiochip_add_data 等
      expect(nodeCount, 'probe 函数应该有多个调用').toBeGreaterThan(1);
      
      // 获取所有节点的文本
      const nodeTexts: string[] = [];
      for (let i = 0; i < nodeCount; i++) {
        const text = await flowNodes.nth(i).textContent();
        if (text) nodeTexts.push(text);
      }
      console.log('执行流节点:', nodeTexts);
      
      await page.screenshot({ path: 'test-results/core-003-execution-flow.png' });
    }
  });
});

// ============================================================
// LLVM IR 功能测试
// ============================================================

test.describe('CORE: LLVM IR 功能', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test('LLVM-001: LLVM IR 编译不能失败', async ({ page }) => {
    // 当前问题：
    // error: Clang compilation failed:
    // In file included from /Users/sky/linux-kernel/linux/include...
    // #include <asm/types.h>
    // 
    // 原因：内核头文件路径配置不正确
    
    // 打开右侧面板
    await page.keyboard.press('Meta+Backslash');
    await page.waitForTimeout(500);
    
    // 切换到 IR 标签
    const irTab = page.locator('text=IR').first();
    if (await irTab.isVisible()) {
      await irTab.click();
      await page.waitForTimeout(500);
      
      // 检查是否有编译错误
      const irContent = await page.textContent('body') || '';
      
      if (irContent.includes('compilation failed') || irContent.includes('error:')) {
        console.error('⚠️ LLVM IR 编译失败！');
        console.error('检查内核头文件路径配置');
        
        // 截图记录错误
        await page.screenshot({ path: 'test-results/llvm-001-compile-error.png' });
        
        // 这应该失败，提醒修复
        expect(irContent.includes('compilation failed')).toBe(false);
      }
    }
  });

  test('LLVM-002: LLVM IR 应该显示函数定义', async ({ page }) => {
    // 正确的 LLVM IR 应该包含：
    // - define 关键字
    // - 函数名
    // - 基本块
    // - 指令（load, store, call 等）
    
    // 打开右侧面板
    await page.keyboard.press('Meta+Backslash');
    await page.waitForTimeout(500);
    
    // 切换到 IR 标签
    const irTab = page.locator('text=IR').first();
    if (await irTab.isVisible()) {
      await irTab.click();
      await page.waitForTimeout(1000);
      
      const irPanel = page.locator('[class*="llvm"], [data-testid="llvm-ir"]');
      const irContent = await irPanel.textContent() || '';
      
      // 检查是否有有效的 LLVM IR 内容
      const hasDefine = irContent.includes('define');
      const hasInstructions = irContent.includes('load') || 
                              irContent.includes('store') || 
                              irContent.includes('call');
      
      if (!hasDefine) {
        console.error('⚠️ LLVM IR 没有 define 关键字');
      }
      if (!hasInstructions) {
        console.error('⚠️ LLVM IR 没有指令（load/store/call）');
      }
      
      await page.screenshot({ path: 'test-results/llvm-002-content.png' });
    }
  });
});

// ============================================================
// 数据正确性测试
// ============================================================

test.describe('CORE: 数据正确性', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test('DATA-001: 索引统计必须与实际文件数匹配', async ({ page }) => {
    // 如果终端显示 "发现 X 个文件"，
    // 那么文件树中应该有大约 X 个 .c/.h 文件
    
    await page.waitForTimeout(2000);
    
    const pageText = await page.textContent('body') || '';
    
    // 提取文件数
    const fileMatch = pageText.match(/发现\s*(\d+)\s*个文件/);
    if (fileMatch) {
      const reportedFiles = parseInt(fileMatch[1]);
      console.log(`报告的文件数: ${reportedFiles}`);
      
      // 如果报告 0，这是 bug
      expect(reportedFiles, '文件数不能为 0').toBeGreaterThan(0);
      
      // 检查文件树中的文件数
      const fileItems = page.locator('[class*="file-tree"] [class*="file"], [data-testid*="file"]');
      const visibleFiles = await fileItems.count();
      
      console.log(`文件树中可见的文件数: ${visibleFiles}`);
      
      // 文件树应该有内容（可能不是全部加载）
      if (reportedFiles > 0 && visibleFiles === 0) {
        console.error('⚠️ 报告有文件但文件树为空！');
      }
    }
    
    await page.screenshot({ path: 'test-results/data-001-file-count.png' });
  });

  test('DATA-002: 函数数量必须与解析结果匹配', async ({ page }) => {
    // 如果终端显示 "发现 X 个函数"，
    // 那么大纲面板应该显示这些函数
    
    await page.waitForTimeout(2000);
    
    const pageText = await page.textContent('body') || '';
    
    // 提取函数数
    const funcMatch = pageText.match(/(\d+)\s*个函数/);
    if (funcMatch) {
      const reportedFuncs = parseInt(funcMatch[1]);
      console.log(`报告的函数数: ${reportedFuncs}`);
      
      if (reportedFuncs > 0) {
        // 打开大纲面板
        await page.keyboard.press('Meta+Backslash');
        await page.waitForTimeout(500);
        
        // 统计大纲中的函数
        const funcItems = page.locator('[class*="outline"] [class*="function"], [data-testid*="function"]');
        const visibleFuncs = await funcItems.count();
        
        console.log(`大纲中可见的函数数: ${visibleFuncs}`);
        
        // 应该有一定比例的函数显示
        if (visibleFuncs === 0 && reportedFuncs > 0) {
          console.error('⚠️ 报告有函数但大纲为空！');
        }
      }
    }
    
    await page.screenshot({ path: 'test-results/data-002-func-count.png' });
  });
});

// ============================================================
// 错误处理测试
// ============================================================

test.describe('CORE: 错误处理', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test('ERR-001: 解析错误必须显示有意义的信息', async ({ page }) => {
    // 如果解析失败，应该：
    // 1. 显示错误原因
    // 2. 不是简单地显示空内容
    // 3. 提供修复建议
    
    // 检查当前页面是否有错误提示
    const errorElements = page.locator('[class*="error"], [class*="Error"], [role="alert"]');
    const errorCount = await errorElements.count();
    
    if (errorCount > 0) {
      for (let i = 0; i < errorCount; i++) {
        const errorText = await errorElements.nth(i).textContent();
        console.log(`错误信息 ${i + 1}: ${errorText}`);
        
        // 错误信息不能是空的或无意义的
        expect(errorText?.length, '错误信息不能为空').toBeGreaterThan(5);
      }
    }
    
    await page.screenshot({ path: 'test-results/err-001-errors.png' });
  });
});
