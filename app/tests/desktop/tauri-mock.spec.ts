/**
 * Tauri Mock Integration Tests
 * 
 * 演示如何使用 Tauri API Mock 进行 E2E 测试
 */

import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import {
  tauriMockScript,
  tauriMockScriptVerbose,
  setupTauriMock,
  createDialogMockScript,
  createMockWithFiles,
} from '../mocks';

const SCREENSHOTS_DIR = './test-results/tauri-mock';

test.beforeAll(() => {
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

test.describe('Tauri Mock - Basic Usage', () => {
  test('injects mock and loads app', async ({ page }) => {
    // 注入 Mock 脚本
    await page.addInitScript(tauriMockScript);
    
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 验证页面加载成功
    await expect(page.locator('header')).toBeVisible();
    
    // 验证 Mock 已安装
    const hasMock = await page.evaluate(() => {
      return typeof (window as any).__TAURI__ !== 'undefined';
    });
    expect(hasMock).toBe(true);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/mock-loaded.png` });
  });

  test('mock dialog returns configured path', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 测试 dialog.open 返回值
    const dialogResult = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.dialog.open({ directory: true });
    });
    
    expect(dialogResult).toBe('/mock/project');
    console.log('Dialog returned:', dialogResult);
  });

  test('mock invoke returns project info', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 测试 invoke('open_project')
    const projectInfo = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('open_project', { path: '/test/project' });
    });
    
    expect(projectInfo.indexed).toBe(true);
    expect(projectInfo.files_count).toBeGreaterThan(0);
    console.log('Project info:', projectInfo);
  });
});

test.describe('Tauri Mock - Command Palette Integration', () => {
  test('open project command with mock dialog', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开命令面板
    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);
    
    // 验证命令面板打开
    const dialog = page.locator('[role="dialog"]');
    await expect(dialog).toBeVisible();
    
    // 查找"打开项目"选项
    const openProjectOption = page.locator('[role="option"]:has-text("打开项目")');
    await expect(openProjectOption).toBeVisible();
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/command-palette-open.png` });
    
    // 点击"打开项目"
    await openProjectOption.click();
    await page.waitForTimeout(500);
    
    // Mock 的 dialog.open 会自动返回 /mock/project
    // 然后 invoke('open_project') 会被调用
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/after-open-project.png` });
  });

  test('open file command with mock dialog', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 打开命令面板
    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);
    
    // 查找"打开文件"选项
    const openFileOption = page.locator('[role="option"]:has-text("打开文件")');
    await expect(openFileOption).toBeVisible();
    
    // 点击"打开文件"
    await openFileOption.click();
    await page.waitForTimeout(500);
    
    // Mock 的 dialog.open 会返回 /mock/project/src/main.c
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/after-open-file.png` });
  });
});

test.describe('Tauri Mock - Custom Configuration', () => {
  test('custom project path', async ({ page }) => {
    // 使用自定义配置
    await setupTauriMock(page, {
      verbose: true,
      dialogResponses: {
        openProject: '/Users/sky/linux-kernel/linux',
        openFile: '/Users/sky/linux-kernel/linux/arch/arm/mach-imx/clk-imx6q.c',
      },
    });
    
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 验证自定义路径
    const dialogResult = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.dialog.open({ directory: true });
    });
    
    expect(dialogResult).toBe('/Users/sky/linux-kernel/linux');
    console.log('Custom dialog returned:', dialogResult);
  });

  test('custom file content', async ({ page }) => {
    // 创建带自定义文件的 Mock
    const script = createMockWithFiles({
      '/custom/test.c': `
#include <stdio.h>
int main() {
    printf("Custom test file\\n");
    return 0;
}
      `.trim(),
    });
    
    await page.addInitScript(script);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 读取自定义文件
    const content = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('read_file', { path: '/custom/test.c' });
    });
    
    expect(content).toContain('Custom test file');
    console.log('Custom file content:', content);
  });
});

test.describe('Tauri Mock - Analysis API', () => {
  test('get_functions returns mock data', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const functions = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('get_functions', { path: '/mock/main.c' });
    });
    
    expect(functions.length).toBeGreaterThan(0);
    expect(functions.some((f: any) => f.name === 'main')).toBe(true);
    expect(functions.some((f: any) => f.is_callback === true)).toBe(true);
    
    console.log('Functions:', functions.map((f: any) => f.name));
  });

  test('analyze_file returns flow trees', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const result = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('analyze_file', { path: '/mock/main.c' });
    });
    
    expect(result.file).toBe('/mock/main.c');
    expect(result.functions_count).toBeGreaterThan(0);
    expect(result.flow_trees.length).toBeGreaterThan(0);
    expect(result.entry_points).toContain('main');
    
    console.log('Analysis result:', {
      file: result.file,
      functions: result.functions_count,
      entryPoints: result.entry_points,
    });
  });

  test('build_execution_flow returns flow data', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const flow = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('build_execution_flow', {
        filePath: '/mock/main.c',
        entryFunction: 'main',
        options: { max_depth: 10 },
      });
    });
    
    expect(flow.entry_function).toBe('main');
    expect(flow.root).toBeDefined();
    expect(flow.root.name).toBe('main');
    expect(flow.analysis_info).toBeDefined();
    
    console.log('Execution flow:', {
      entry: flow.entry_function,
      totalNodes: flow.analysis_info.total_nodes,
      asyncBoundaries: flow.async_boundaries.length,
    });
  });

  test('search_symbols filters by query', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const results = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('search_symbols', { query: 'handler' });
    });
    
    expect(results.length).toBeGreaterThan(0);
    expect(results.every((r: any) => r.name.includes('handler'))).toBe(true);
    
    console.log('Search results:', results.map((r: any) => r.name));
  });
});

test.describe('Tauri Mock - File Operations', () => {
  test('read and write file', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 写入文件
    await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      await tauri.core.invoke('write_file', {
        path: '/test/new-file.c',
        contents: '// New file content',
      });
    });
    
    // 读取文件
    const content = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('read_file', { path: '/test/new-file.c' });
    });
    
    expect(content).toBe('// New file content');
  });

  test('list_directory returns file tree', async ({ page }) => {
    await page.addInitScript(tauriMockScript);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const tree = await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      return await tauri.core.invoke('list_directory', {
        path: '/mock/project',
        recursive: false,
      });
    });
    
    expect(tree.length).toBeGreaterThan(0);
    expect(tree.some((n: any) => n.is_dir)).toBe(true);
    expect(tree.some((n: any) => !n.is_dir)).toBe(true);
    
    console.log('File tree:', tree.map((n: any) => `${n.is_dir ? '[D]' : '[F]'} ${n.name}`));
  });
});

test.describe('Tauri Mock - Verbose Logging', () => {
  test('verbose mode logs to console', async ({ page }) => {
    // 收集控制台日志
    const logs: string[] = [];
    page.on('console', (msg) => {
      if (msg.text().includes('[TauriMock]')) {
        logs.push(msg.text());
      }
    });
    
    await page.addInitScript(tauriMockScriptVerbose);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // 执行一些操作
    await page.evaluate(async () => {
      const tauri = (window as any).__TAURI__;
      await tauri.core.invoke('get_functions', { path: '/test.c' });
      await tauri.dialog.open({ directory: true });
    });
    
    // 验证有日志输出
    expect(logs.length).toBeGreaterThan(0);
    console.log('Mock logs:', logs);
  });
});
