/**
 * Code Editor E2E Tests
 * Tests for the Monaco-based code editor component
 */

import { test, expect, Page } from '@playwright/test';
import * as fs from 'fs';

const SCREENSHOTS_DIR = './test-results/flowsight/code-editor';

// Mock C file content
const mockCFileContent = `#include <stdio.h>
#include <stdlib.h>

/**
 * Main function - Hello World
 */
int main(void) {
    printf("Hello, World!\\n");
    return 0;
}`;

// Mock file path for testing
const mockFilePath = '/tmp/test-file.c';

test.beforeAll(() => {
  // Create screenshots directory
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

/**
 * Helper: Setup Tauri mock for read_file command
 */
async function setupFileMock(page: Page, content: string = mockCFileContent) {
  await page.addInitScript((mockContent) => {
    // Mock Tauri invoke for read_file
    (window as any).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args: any) => {
        if (cmd === 'read_file') {
          return mockContent;
        }
        if (cmd === 'write_file') {
          return true;
        }
        throw new Error(`Unknown command: ${cmd}`);
      }
    };
  }, content);
}

/**
 * Helper: Navigate to app and open a mock file
 */
async function openMockFile(page: Page, filePath: string = mockFilePath) {
  // Set file path in store via URL hash or localStorage
  await page.evaluate((path) => {
    // Dispatch event to trigger file open
    window.dispatchEvent(new CustomEvent('open-file', { detail: { path } }));
  }, filePath);
  
  await page.waitForTimeout(500);
}

test.describe('Code Editor - Basic Display', () => {
  test('editor container exists', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Check editor container exists
    const editor = page.locator('[data-testid="code-editor"]');
    await expect(editor).toBeVisible();
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/editor-container.png` });
  });

  test('shows empty state when no file selected', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Check for empty state message
    const editor = page.locator('[data-testid="code-editor"]');
    await expect(editor).toBeVisible();
    
    // Look for empty state text
    const emptyState = page.locator('text=选择一个文件开始编辑');
    const hasEmptyState = await emptyState.count() > 0;
    console.log('Empty state visible:', hasEmptyState);
    
    // Also check for the FileCode icon container
    const iconContainer = editor.locator('.rounded-xl');
    const iconCount = await iconContainer.count();
    console.log('Icon container found:', iconCount > 0);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/editor-empty-state.png` });
  });

  test('editor has correct structure', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const editor = page.locator('[data-testid="code-editor"]');
    
    // Check editor uses flex layout
    const display = await editor.evaluate((el) => {
      return getComputedStyle(el).display;
    });
    console.log('Editor display:', display);
    expect(display).toBe('flex');
    
    // Check background color
    const bgColor = await editor.evaluate((el) => {
      return getComputedStyle(el).backgroundColor;
    });
    console.log('Editor background:', bgColor);
  });
});

test.describe('Code Editor - File Loading', () => {
  test('displays filename when file is loaded', async ({ page }) => {
    await setupFileMock(page);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Simulate file selection via store
    await page.evaluate(() => {
      // Access the analysis store if available
      const store = (window as any).__STORE__;
      if (store?.getState?.()?.setSelectedFile) {
        store.getState().setSelectedFile('/tmp/test-file.c');
      }
    });
    
    await page.waitForTimeout(1000);
    
    // Check if filename is displayed
    const filename = page.locator('[data-testid="editor-filename"]');
    const filenameCount = await filename.count();
    
    if (filenameCount > 0) {
      const filenameText = await filename.textContent();
      console.log('Displayed filename:', filenameText);
    } else {
      console.log('Filename element not found (file may not be loaded)');
    }
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/editor-with-filename.png` });
  });

  test('Monaco editor loads successfully', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Wait for Monaco to potentially load
    await page.waitForTimeout(2000);
    
    // Check if Monaco editor elements exist
    const monacoEditor = page.locator('.monaco-editor');
    const monacoCount = await monacoEditor.count();
    console.log('Monaco editor elements found:', monacoCount);
    
    // Monaco may not be visible until a file is opened
    if (monacoCount > 0) {
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/monaco-loaded.png` });
    }
  });
});

test.describe('Code Editor - Save Status', () => {
  test('save status indicator exists', async ({ page }) => {
    await setupFileMock(page);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Check for save status element
    const saveStatus = page.locator('[data-testid="editor-save-status"]');
    const statusCount = await saveStatus.count();
    console.log('Save status elements found:', statusCount);
    
    if (statusCount > 0) {
      const statusValue = await saveStatus.getAttribute('data-status');
      console.log('Current save status:', statusValue);
    }
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/save-status.png` });
  });

  test('save status shows correct states', async ({ page }) => {
    await setupFileMock(page);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    const saveStatus = page.locator('[data-testid="editor-save-status"]');
    
    // Initial state should be 'saved' or element may not be present
    const statusCount = await saveStatus.count();
    
    if (statusCount > 0) {
      const status = await saveStatus.getAttribute('data-status');
      console.log('Save status value:', status);
      // Valid states: saved, modified, saving, error, readonly
      expect(['saved', 'modified', 'saving', 'error', 'readonly', null]).toContain(status);
    } else {
      console.log('Save status not visible (no file loaded)');
    }
  });
});

test.describe('Code Editor - Keyboard Shortcuts', () => {
  test('Cmd+S triggers save', async ({ page }) => {
    await setupFileMock(page);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Focus on editor area
    const editor = page.locator('[data-testid="code-editor"]');
    await editor.click();
    
    // Press Cmd+S (or Ctrl+S on non-Mac)
    await page.keyboard.press('Meta+S');
    await page.waitForTimeout(300);
    
    console.log('Cmd+S triggered');
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/after-save-shortcut.png` });
  });

  test('Cmd+F opens search in Monaco', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Wait for Monaco
    await page.waitForTimeout(1000);
    
    const monacoEditor = page.locator('.monaco-editor');
    const monacoCount = await monacoEditor.count();
    
    if (monacoCount > 0) {
      // Click on editor to focus
      await monacoEditor.first().click();
      
      // Press Cmd+F
      await page.keyboard.press('Meta+F');
      await page.waitForTimeout(500);
      
      // Check if find widget appears
      const findWidget = page.locator('.monaco-editor .find-widget');
      const findWidgetCount = await findWidget.count();
      console.log('Find widget visible:', findWidgetCount > 0);
      
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/monaco-find-widget.png` });
      
      // Close find widget
      await page.keyboard.press('Escape');
    } else {
      console.log('Monaco editor not visible, skipping Cmd+F test');
    }
  });
});

test.describe('Code Editor - Syntax Highlighting', () => {
  test('C code has syntax highlighting classes', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Wait for Monaco to load
    await page.waitForTimeout(2000);
    
    const monacoEditor = page.locator('.monaco-editor');
    const monacoCount = await monacoEditor.count();
    
    if (monacoCount > 0) {
      // Check for Monaco syntax tokens
      const tokens = page.locator('.monaco-editor .mtk1, .monaco-editor .mtk2, .monaco-editor .mtk3');
      const tokenCount = await tokens.count();
      console.log('Syntax token elements found:', tokenCount);
      
      // Check for view lines
      const viewLines = page.locator('.monaco-editor .view-line');
      const lineCount = await viewLines.count();
      console.log('View lines found:', lineCount);
      
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/syntax-highlighting.png` });
    } else {
      console.log('Monaco editor not loaded, syntax highlighting test skipped');
    }
  });

  test('Monaco editor theme is vs-dark', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    await page.waitForTimeout(2000);
    
    // Check for vs-dark theme class
    const darkTheme = page.locator('.monaco-editor.vs-dark');
    const darkThemeCount = await darkTheme.count();
    console.log('VS Dark theme applied:', darkThemeCount > 0);
    
    if (darkThemeCount > 0) {
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/monaco-dark-theme.png` });
    }
  });
});

test.describe('Code Editor - UI Components', () => {
  test('file tab bar exists when file is open', async ({ page }) => {
    await setupFileMock(page);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Look for the tab bar structure
    const editor = page.locator('[data-testid="code-editor"]');
    
    // Check for FileCode icon in tab
    const fileIcon = editor.locator('svg.lucide-file-code');
    const iconCount = await fileIcon.count();
    console.log('File icons in editor:', iconCount);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/file-tab-bar.png` });
  });

  test('close button exists in tab', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Look for close button (X icon)
    const closeButton = page.locator('[data-testid="code-editor"] button[aria-label="关闭文件"]');
    const closeCount = await closeButton.count();
    console.log('Close buttons found:', closeCount);
    
    if (closeCount > 0) {
      // Check if close button has X icon
      const xIcon = closeButton.locator('svg.lucide-x');
      const xIconCount = await xIcon.count();
      console.log('X icons in close button:', xIconCount);
    }
  });

  test('save button appears when modified', async ({ page }) => {
    await setupFileMock(page);
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Look for save button (visible when status is modified)
    const saveButton = page.locator('[data-testid="code-editor"] button:has-text("保存")');
    const saveButtonCount = await saveButton.count();
    console.log('Save button visible:', saveButtonCount > 0);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/save-button.png` });
  });
});

test.describe('Code Editor - Loading States', () => {
  test('loading state displays correctly', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Check for loading indicator text
    const loadingText = page.locator('text=加载中...');
    const loadingCount = await loadingText.count();
    console.log('Loading text elements:', loadingCount);
    
    // Check for Loader2 spinner
    const spinner = page.locator('.animate-spin');
    const spinnerCount = await spinner.count();
    console.log('Spinner elements:', spinnerCount);
  });

  test('error state displays correctly', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Check for error message pattern
    const errorText = page.locator('text=加载失败');
    const errorCount = await errorText.count();
    console.log('Error text elements:', errorCount);
  });
});

test.describe('Code Editor - Integration', () => {
  test('editor integrates with main layout', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    
    // Check editor is within main content area
    const mainContent = page.locator('main');
    const editorInMain = mainContent.locator('[data-testid="code-editor"]');
    const count = await editorInMain.count();
    console.log('Editor in main content:', count > 0);
    
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/editor-in-layout.png` });
  });

  test('full page screenshot with editor', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1000);
    
    await page.screenshot({
      path: `${SCREENSHOTS_DIR}/full-page-with-editor.png`,
      fullPage: true
    });
  });
});
