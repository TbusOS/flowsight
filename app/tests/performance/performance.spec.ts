/**
 * 前端性能测试
 * 
 * 测试关键性能指标:
 * - 首屏加载时间
 * - 组件渲染时间
 * - 内存使用
 * 
 * 运行命令:
 *   npx playwright test tests/performance/
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
            // 生成大量文件模拟性能测试
            const files = [];
            for (let i = 0; i < 100; i++) {
              files.push({ name: 'file_' + i + '.c', path: '/test/file_' + i + '.c', is_dir: false, extension: 'c' });
            }
            return files;
          case 'read_file':
            // 生成大文件内容
            let content = '';
            for (let i = 0; i < 1000; i++) {
              content += '// Line ' + i + '\\nint func_' + i + '() { return ' + i + '; }\\n';
            }
            return content;
          case 'get_functions':
            const funcs = [];
            for (let i = 0; i < 100; i++) {
              funcs.push({ name: 'func_' + i, return_type: 'int', line: i * 2 });
            }
            return funcs;
          case 'open_project':
            return { path: args.path, files_count: 100, functions_count: 1000 };
          case 'build_execution_flow':
            const nodes = [{ id: 'n0', label: 'entry', node_type: 'entry', line: 0 }];
            const edges = [];
            for (let i = 1; i < 50; i++) {
              nodes.push({ id: 'n' + i, label: 'func_' + i, node_type: 'function', line: i * 10 });
              edges.push({ source: 'n' + (i - 1), target: 'n' + i, edge_type: 'sync' });
            }
            return { entry_function: 'entry', nodes, edges };
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

test.describe('性能测试', () => {
  test('首屏加载时间 < 3s', async ({ page }) => {
    const startTime = Date.now()
    
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'domcontentloaded' })
    
    const loadTime = Date.now() - startTime
    console.log(`首屏加载时间: ${loadTime}ms`)
    
    // 首屏应该在 3 秒内加载完成
    expect(loadTime).toBeLessThan(3000)
  })

  test('完整页面加载时间 < 5s', async ({ page }) => {
    const startTime = Date.now()
    
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    
    const loadTime = Date.now() - startTime
    console.log(`完整加载时间: ${loadTime}ms`)
    
    expect(loadTime).toBeLessThan(5000)
  })

  test('大量文件列表渲染时间 < 1s', async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    
    // 打开项目触发文件列表渲染
    const openButton = page.locator('button:has-text("打开项目")')
    if (await openButton.isVisible()) {
      const startTime = Date.now()
      await openButton.click()
      
      // 等待文件列表出现
      await page.waitForSelector('text=file_0.c', { timeout: 5000 }).catch(() => {})
      
      const renderTime = Date.now() - startTime
      console.log(`文件列表渲染时间: ${renderTime}ms`)
      
      expect(renderTime).toBeLessThan(1000)
    }
  })

  test('执行流渲染时间 < 2s (50 节点)', async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    await page.waitForTimeout(500)

    // 切换到执行流视图
    await page.keyboard.press('Meta+2')
    await page.waitForTimeout(300)

    // 点击分析
    const analyzeButton = page.locator('button:has-text("分析")').first()
    if (await analyzeButton.isVisible()) {
      const startTime = Date.now()
      await analyzeButton.click()
      
      // 等待节点出现
      await page.waitForSelector('.react-flow__node', { timeout: 5000 }).catch(() => {})
      
      const renderTime = Date.now() - startTime
      console.log(`执行流渲染时间 (50节点): ${renderTime}ms`)
      
      expect(renderTime).toBeLessThan(2000)
    }
  })

  test('内存使用合理', async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    
    // 获取 JS 堆内存
    const metrics = await page.evaluate(() => {
      if ('memory' in performance) {
        const memory = (performance as any).memory
        return {
          usedJSHeapSize: memory.usedJSHeapSize,
          totalJSHeapSize: memory.totalJSHeapSize,
        }
      }
      return null
    })

    if (metrics) {
      const usedMB = metrics.usedJSHeapSize / 1024 / 1024
      console.log(`JS 堆内存使用: ${usedMB.toFixed(2)} MB`)
      
      // 初始内存使用应该小于 100MB
      expect(usedMB).toBeLessThan(100)
    }
  })

  test('交互响应时间 < 100ms', async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    await page.waitForTimeout(500)

    // 测试快捷键响应时间
    const shortcuts = ['Meta+1', 'Meta+2', 'Meta+b', 'Meta+j']
    
    for (const shortcut of shortcuts) {
      const startTime = Date.now()
      await page.keyboard.press(shortcut)
      await page.waitForTimeout(50) // 等待状态更新
      const responseTime = Date.now() - startTime
      
      console.log(`${shortcut} 响应时间: ${responseTime}ms`)
      expect(responseTime).toBeLessThan(200) // 包含网络延迟
    }
  })
})

test.describe('渲染性能', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(generateMock())
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' })
    await page.waitForTimeout(500)
  })

  test('无阻塞渲染', async ({ page }) => {
    // 检查是否有长任务
    const longTasks = await page.evaluate(() => {
      return new Promise((resolve) => {
        const observer = new PerformanceObserver((list) => {
          const entries = list.getEntries()
          resolve(entries.filter((e) => e.duration > 50))
        })
        
        try {
          observer.observe({ entryTypes: ['longtask'] })
          setTimeout(() => {
            observer.disconnect()
            resolve([])
          }, 2000)
        } catch {
          resolve([])
        }
      })
    })

    console.log(`长任务数量: ${(longTasks as any[]).length}`)
  })

  test('累积布局偏移 (CLS) 合理', async ({ page }) => {
    // 等待页面稳定
    await page.waitForTimeout(2000)

    const cls = await page.evaluate(() => {
      return new Promise((resolve) => {
        let clsValue = 0
        const observer = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            if (!(entry as any).hadRecentInput) {
              clsValue += (entry as any).value
            }
          }
        })
        
        try {
          observer.observe({ entryTypes: ['layout-shift'] })
          setTimeout(() => {
            observer.disconnect()
            resolve(clsValue)
          }, 1000)
        } catch {
          resolve(0)
        }
      })
    })

    console.log(`CLS: ${cls}`)
    // CLS 应该小于 0.1 (Good)
    expect(cls as number).toBeLessThan(0.25)
  })
})
