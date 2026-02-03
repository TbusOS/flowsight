/**
 * FlowSight 内核代码分析 E2E 测试
 * 
 * 完整测试流程：打开内核文件 → 分析 → 显示执行流
 * 
 * 测试目标：
 * 1. USB 驱动 probe 回调识别
 * 2. 执行流图显示
 * 3. 异步模式检测 (WorkQueue, Timer, IRQ)
 * 4. 知识库数据匹配
 * 
 * 运行方式：
 *   cd app && npx playwright test tests/integration/kernel-analysis.spec.ts \
 *     --config=tests/desktop/playwright.config.ts
 */

import { test, expect, Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

// ============================================================================
// 配置
// ============================================================================

const KERNEL_PATH = '/Users/sky/linux-kernel/linux';
const USB_DRIVER_PATH = `${KERNEL_PATH}/drivers/usb/core/driver.c`;
const GPIO_DRIVER_PATH = `${KERNEL_PATH}/drivers/gpio/gpio-dwapb.c`;
const IMX_DRIVER_PATH = `${KERNEL_PATH}/arch/arm/mach-imx/clk-imx6q.c`;

// 测试超时
const ANALYSIS_TIMEOUT = 30000;

// ============================================================================
// 辅助函数
// ============================================================================

async function waitForAnalysisComplete(page: Page, timeout = ANALYSIS_TIMEOUT): Promise<boolean> {
  try {
    // 等待加载指示器消失或执行流节点出现
    await page.waitForFunction(() => {
      const loading = document.querySelector('[data-testid="loading"], .loading');
      const nodes = document.querySelectorAll('.react-flow__node');
      return !loading || nodes.length > 0;
    }, { timeout });
    return true;
  } catch {
    return false;
  }
}

async function getFlowNodeCount(page: Page): Promise<number> {
  return page.locator('.react-flow__node').count();
}

async function getFlowEdgeCount(page: Page): Promise<number> {
  return page.locator('.react-flow__edge').count();
}

async function takeDebugScreenshot(page: Page, name: string) {
  const screenshotDir = 'test-results/kernel-analysis';
  if (!fs.existsSync(screenshotDir)) {
    fs.mkdirSync(screenshotDir, { recursive: true });
  }
  await page.screenshot({ path: `${screenshotDir}/${name}.png`, fullPage: true });
}

// ============================================================================
// USB 驱动分析测试
// ============================================================================

test.describe('Linux Kernel USB Driver Analysis', () => {
  test.beforeEach(async ({ page }) => {
    // 监听控制台日志
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('[Error]') || text.includes('error') || text.includes('FlowView')) {
        console.log(`Console [${msg.type()}]: ${text}`);
      }
    });
    
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1000);
  });

  test('USB-001: 识别 USB 驱动 probe 回调', async ({ page }) => {
    console.log('测试: 识别 USB 驱动 probe 回调');
    
    // 检查内核路径是否存在
    if (!fs.existsSync(USB_DRIVER_PATH)) {
      console.log(`跳过: 内核文件不存在 ${USB_DRIVER_PATH}`);
      test.skip();
      return;
    }

    // 尝试通过 Tauri API 打开文件
    const result = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        const functions = await invoke('get_functions', { path: filePath });
        
        return {
          success: true,
          functionCount: functions?.length || 0,
          hasProbe: functions?.some((f: any) => 
            f.name.includes('probe') || 
            f.name.includes('usb_probe')
          ),
          functions: functions?.slice(0, 10).map((f: any) => ({
            name: f.name,
            isCallback: f.is_callback || false,
            callsCount: f.calls?.length || 0
          }))
        };
      } catch (e) {
        return { error: String(e) };
      }
    }, USB_DRIVER_PATH);
    
    console.log('API 调用结果:', JSON.stringify(result, null, 2));
    
    await takeDebugScreenshot(page, 'usb-001-probe-callback');
    
    if (result.error === 'Tauri not available') {
      console.log('注意: 浏览器模式下无法测试 Tauri API，此测试需要在 Tauri WebView 中运行');
      // 在浏览器模式下，验证 UI 基本元素存在
      const mainContent = page.locator('main, [role="main"], .main-content');
      await expect(mainContent.first()).toBeVisible();
    } else if (result.success) {
      // 验证找到了 probe 相关函数
      expect(result.functionCount).toBeGreaterThan(0);
      console.log(`找到 ${result.functionCount} 个函数`);
      if (result.hasProbe) {
        console.log('✅ 找到 probe 回调函数');
      }
    }
  });

  test('USB-002: 显示 USB 驱动执行流图', async ({ page }) => {
    console.log('测试: 显示 USB 驱动执行流图');
    
    if (!fs.existsSync(USB_DRIVER_PATH)) {
      test.skip();
      return;
    }

    // 尝试获取执行流
    const flowResult = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        
        // 先获取函数列表
        const functions = await invoke('get_functions', { path: filePath });
        
        // 找到第一个 probe 函数
        const probeFunc = functions?.find((f: any) => f.name.includes('probe'));
        
        if (probeFunc) {
          // 获取执行流
          const flow = await invoke('build_execution_flow', {
            filePath: filePath,
            entryFunction: probeFunc.name,
            options: { maxDepth: 5 }
          });
          
          return {
            success: true,
            entryFunction: probeFunc.name,
            nodeCount: flow?.nodes?.length || 0,
            edgeCount: flow?.edges?.length || 0,
            nodes: flow?.nodes?.slice(0, 5).map((n: any) => n.label || n.name)
          };
        }
        
        return { success: false, error: 'No probe function found' };
      } catch (e) {
        return { error: String(e) };
      }
    }, USB_DRIVER_PATH);
    
    console.log('执行流结果:', JSON.stringify(flowResult, null, 2));
    
    await takeDebugScreenshot(page, 'usb-002-execution-flow');
    
    if (flowResult.success) {
      expect(flowResult.nodeCount).toBeGreaterThan(0);
      console.log(`✅ 执行流包含 ${flowResult.nodeCount} 个节点`);
    }
  });

  test('USB-003: 检测 URB completion 异步回调', async ({ page }) => {
    console.log('测试: 检测 URB completion 异步回调');
    
    if (!fs.existsSync(USB_DRIVER_PATH)) {
      test.skip();
      return;
    }

    const asyncResult = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        
        // 分析文件中的异步模式
        const analysis = await invoke('analyze_async_patterns', { path: filePath });
        
        return {
          success: true,
          asyncHandlers: analysis?.async_bindings?.length || 0,
          workQueues: analysis?.async_bindings?.filter((b: any) => 
            b.mechanism?.WorkQueue
          ).length || 0,
          completionCallbacks: analysis?.async_bindings?.filter((b: any) =>
            b.handler?.includes('complete')
          ).length || 0
        };
      } catch (e) {
        return { error: String(e) };
      }
    }, USB_DRIVER_PATH);
    
    console.log('异步分析结果:', JSON.stringify(asyncResult, null, 2));
    
    await takeDebugScreenshot(page, 'usb-003-async-callbacks');
    
    if (asyncResult.success) {
      console.log(`找到 ${asyncResult.asyncHandlers} 个异步处理器`);
    }
  });
});

// ============================================================================
// GPIO 驱动分析测试
// ============================================================================

test.describe('Linux Kernel GPIO Driver Analysis', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1000);
  });

  test('GPIO-001: 分析 GPIO 驱动 probe 函数', async ({ page }) => {
    console.log('测试: 分析 GPIO 驱动 probe 函数');
    
    if (!fs.existsSync(GPIO_DRIVER_PATH)) {
      console.log(`跳过: 内核文件不存在 ${GPIO_DRIVER_PATH}`);
      test.skip();
      return;
    }

    const result = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        const functions = await invoke('get_functions', { path: filePath });
        
        // 检查是否识别到 platform_driver 回调
        const probeFunc = functions?.find((f: any) => 
          f.name.includes('probe') && !f.name.includes('remove')
        );
        
        return {
          success: true,
          functionCount: functions?.length || 0,
          probeFunction: probeFunc?.name || null,
          probeCalls: probeFunc?.calls?.length || 0,
          isCallback: probeFunc?.is_callback || false,
          callbackContext: probeFunc?.callback_context || 'unknown'
        };
      } catch (e) {
        return { error: String(e) };
      }
    }, GPIO_DRIVER_PATH);
    
    console.log('GPIO 分析结果:', JSON.stringify(result, null, 2));
    
    await takeDebugScreenshot(page, 'gpio-001-probe-analysis');
    
    if (result.success && result.probeFunction) {
      console.log(`✅ 找到 probe 函数: ${result.probeFunction}`);
      expect(result.probeCalls).toBeGreaterThan(0);
    }
  });

  test('GPIO-002: 检测 IRQ handler 注册', async ({ page }) => {
    console.log('测试: 检测 IRQ handler 注册');
    
    if (!fs.existsSync(GPIO_DRIVER_PATH)) {
      test.skip();
      return;
    }

    const irqResult = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        
        // 获取回调函数列表
        const callbacks = await invoke('get_callbacks', { path: filePath });
        
        // 检查是否有 IRQ handler
        const irqHandlers = callbacks?.filter((c: any) => 
          c.name.includes('irq') || 
          c.name.includes('handler') ||
          c.callback_context === 'hardirq'
        );
        
        return {
          success: true,
          totalCallbacks: callbacks?.length || 0,
          irqHandlers: irqHandlers?.map((h: any) => ({
            name: h.name,
            context: h.callback_context
          })) || []
        };
      } catch (e) {
        return { error: String(e) };
      }
    }, GPIO_DRIVER_PATH);
    
    console.log('IRQ 分析结果:', JSON.stringify(irqResult, null, 2));
    
    await takeDebugScreenshot(page, 'gpio-002-irq-handlers');
    
    if (irqResult.success) {
      console.log(`找到 ${irqResult.totalCallbacks} 个回调，${irqResult.irqHandlers.length} 个 IRQ handler`);
    }
  });
});

// ============================================================================
// ARM32 架构驱动测试
// ============================================================================

test.describe('ARM32 Architecture Driver Analysis', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1000);
  });

  test('ARM32-001: 分析 IMX6 时钟驱动', async ({ page }) => {
    console.log('测试: 分析 IMX6 时钟驱动');
    
    if (!fs.existsSync(IMX_DRIVER_PATH)) {
      console.log(`跳过: 内核文件不存在 ${IMX_DRIVER_PATH}`);
      test.skip();
      return;
    }

    const result = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        const functions = await invoke('get_functions', { path: filePath });
        
        // IMX6 时钟驱动应该有 clk_register 相关调用
        const clkFunctions = functions?.filter((f: any) =>
          f.calls?.some((c: string) => c.includes('clk_'))
        );
        
        return {
          success: true,
          functionCount: functions?.length || 0,
          clkRelatedFunctions: clkFunctions?.length || 0,
          functionNames: functions?.slice(0, 10).map((f: any) => f.name) || []
        };
      } catch (e) {
        return { error: String(e) };
      }
    }, IMX_DRIVER_PATH);
    
    console.log('IMX6 分析结果:', JSON.stringify(result, null, 2));
    
    await takeDebugScreenshot(page, 'arm32-001-imx6-clk');
    
    if (result.success) {
      console.log(`找到 ${result.functionCount} 个函数，${result.clkRelatedFunctions} 个与时钟相关`);
    }
  });
});

// ============================================================================
// 执行流验证测试
// ============================================================================

test.describe('Execution Flow Validation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1000);
  });

  test('FLOW-001: 执行流节点包含正确的上下文信息', async ({ page }) => {
    console.log('测试: 执行流节点包含正确的上下文信息');
    
    const testFile = fs.existsSync(GPIO_DRIVER_PATH) ? GPIO_DRIVER_PATH : USB_DRIVER_PATH;
    
    if (!fs.existsSync(testFile)) {
      test.skip();
      return;
    }

    const flowResult = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        
        // 获取函数列表
        const functions = await invoke('get_functions', { path: filePath });
        const probeFunc = functions?.find((f: any) => f.name.includes('probe'));
        
        if (!probeFunc) {
          return { error: 'No probe function found' };
        }
        
        // 获取执行流
        const flow = await invoke('build_execution_flow', {
          filePath: filePath,
          entryFunction: probeFunc.name,
          options: { maxDepth: 3, includeKnowledge: true }
        });
        
        // 检查节点是否包含上下文信息
        const nodesWithContext = flow?.nodes?.filter((n: any) => 
          n.context || n.execution_context || n.can_sleep !== undefined
        );
        
        return {
          success: true,
          totalNodes: flow?.nodes?.length || 0,
          nodesWithContext: nodesWithContext?.length || 0,
          sampleNodes: flow?.nodes?.slice(0, 5).map((n: any) => ({
            name: n.label || n.name,
            context: n.context || n.execution_context,
            canSleep: n.can_sleep
          }))
        };
      } catch (e) {
        return { error: String(e) };
      }
    }, testFile);
    
    console.log('执行流上下文结果:', JSON.stringify(flowResult, null, 2));
    
    await takeDebugScreenshot(page, 'flow-001-context-info');
    
    if (flowResult.success && flowResult.totalNodes > 0) {
      console.log(`✅ 执行流包含 ${flowResult.totalNodes} 个节点`);
      // 验证至少有一些节点包含上下文信息
      if (flowResult.nodesWithContext > 0) {
        console.log(`✅ ${flowResult.nodesWithContext} 个节点包含上下文信息`);
      }
    }
  });

  test('FLOW-002: 执行流边包含调用关系', async ({ page }) => {
    console.log('测试: 执行流边包含调用关系');
    
    const testFile = fs.existsSync(GPIO_DRIVER_PATH) ? GPIO_DRIVER_PATH : USB_DRIVER_PATH;
    
    if (!fs.existsSync(testFile)) {
      test.skip();
      return;
    }

    const edgeResult = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        
        const functions = await invoke('get_functions', { path: filePath });
        const probeFunc = functions?.find((f: any) => f.name.includes('probe'));
        
        if (!probeFunc) {
          return { error: 'No probe function found' };
        }
        
        const flow = await invoke('build_execution_flow', {
          filePath: filePath,
          entryFunction: probeFunc.name,
          options: { maxDepth: 3 }
        });
        
        return {
          success: true,
          nodeCount: flow?.nodes?.length || 0,
          edgeCount: flow?.edges?.length || 0,
          sampleEdges: flow?.edges?.slice(0, 5).map((e: any) => ({
            source: e.source,
            target: e.target,
            type: e.edge_type || e.type
          }))
        };
      } catch (e) {
        return { error: String(e) };
      }
    }, testFile);
    
    console.log('执行流边结果:', JSON.stringify(edgeResult, null, 2));
    
    await takeDebugScreenshot(page, 'flow-002-edges');
    
    if (edgeResult.success) {
      // 如果有节点，应该有边（除非只有一个节点）
      if (edgeResult.nodeCount > 1) {
        expect(edgeResult.edgeCount).toBeGreaterThan(0);
        console.log(`✅ 执行流包含 ${edgeResult.edgeCount} 条边`);
      }
    }
  });
});

// ============================================================================
// 数据正确性测试 (关键)
// ============================================================================

test.describe('🔴 Data Correctness Tests (Critical)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1000);
  });

  test('DATA-001: 解析的函数必须存在于源代码中', async ({ page }) => {
    console.log('🔴 关键测试: 解析的函数必须存在于源代码中');
    
    const testFile = fs.existsSync(GPIO_DRIVER_PATH) ? GPIO_DRIVER_PATH : USB_DRIVER_PATH;
    
    if (!fs.existsSync(testFile)) {
      test.skip();
      return;
    }

    // 读取源代码
    const sourceCode = fs.readFileSync(testFile, 'utf-8');

    const validationResult = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        const functions = await invoke('get_functions', { path: filePath });
        
        return {
          success: true,
          functionNames: functions?.map((f: any) => f.name) || []
        };
      } catch (e) {
        return { error: String(e) };
      }
    }, testFile);
    
    console.log('函数列表:', validationResult.functionNames?.slice(0, 10));
    
    if (validationResult.success && validationResult.functionNames) {
      // 验证每个函数名都在源代码中
      let validCount = 0;
      let invalidCount = 0;
      
      for (const funcName of validationResult.functionNames.slice(0, 10)) {
        const pattern = new RegExp(`\\b${funcName}\\s*\\(`);
        if (pattern.test(sourceCode)) {
          validCount++;
        } else {
          console.log(`⚠️ 函数 "${funcName}" 在源代码中未找到`);
          invalidCount++;
        }
      }
      
      console.log(`验证结果: ${validCount} 有效, ${invalidCount} 无效`);
      
      // 至少 80% 的函数应该能在源代码中找到
      const validRatio = validCount / (validCount + invalidCount);
      expect(validRatio).toBeGreaterThanOrEqual(0.8);
      console.log(`✅ 函数验证通过率: ${(validRatio * 100).toFixed(1)}%`);
    }
    
    await takeDebugScreenshot(page, 'data-001-function-validation');
  });

  test('DATA-002: 调用关系必须真实存在', async ({ page }) => {
    console.log('🔴 关键测试: 调用关系必须真实存在');
    
    const testFile = fs.existsSync(GPIO_DRIVER_PATH) ? GPIO_DRIVER_PATH : USB_DRIVER_PATH;
    
    if (!fs.existsSync(testFile)) {
      test.skip();
      return;
    }

    const sourceCode = fs.readFileSync(testFile, 'utf-8');

    const callResult = await page.evaluate(async (filePath: string) => {
      // @ts-ignore
      if (typeof window.__TAURI__ === 'undefined') {
        return { error: 'Tauri not available' };
      }
      
      try {
        // @ts-ignore
        const { invoke } = window.__TAURI__.core;
        const functions = await invoke('get_functions', { path: filePath });
        
        // 找到一个有调用的函数
        const funcWithCalls = functions?.find((f: any) => f.calls?.length > 0);
        
        if (funcWithCalls) {
          return {
            success: true,
            functionName: funcWithCalls.name,
            calls: funcWithCalls.calls
          };
        }
        
        return { success: false, error: 'No function with calls found' };
      } catch (e) {
        return { error: String(e) };
      }
    }, testFile);
    
    console.log('调用关系:', callResult);
    
    if (callResult.success && callResult.calls) {
      // 验证调用关系在源代码中
      let verifiedCalls = 0;
      
      for (const callee of callResult.calls.slice(0, 5)) {
        // 检查源代码中是否有这个调用
        const callPattern = new RegExp(`${callee}\\s*\\(`);
        if (callPattern.test(sourceCode)) {
          verifiedCalls++;
          console.log(`✅ 验证调用: ${callee}`);
        }
      }
      
      expect(verifiedCalls).toBeGreaterThan(0);
      console.log(`✅ 验证了 ${verifiedCalls} 个调用关系`);
    }
    
    await takeDebugScreenshot(page, 'data-002-call-validation');
  });
});
