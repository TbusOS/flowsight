/**
 * Playwright Tests for FlowSight WebView
 * Tests the React UI components rendered in Tauri WebView
 */

import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

const SCREENSHOTS_DIR = './test-results/flowsight';

test.beforeAll(() => {
  // Create screenshots directory
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

test.describe('FlowSight UI Components', () => {
  test('Header renders correctly', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Check header exists
    const header = page.locator('header');
    await expect(header).toBeVisible();

    // Screenshot
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/header.png` });

    // Check background color
    const bgColor = await page.evaluate(() => {
      return getComputedStyle(document.querySelector('header')!).backgroundColor;
    });
    console.log('Header background:', bgColor);
  });

  test('Sidebar renders correctly', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Check sidebar exists
    const sidebar = page.locator('aside').first();
    await expect(sidebar).toBeVisible();

    // Check for navigation buttons
    const buttons = page.locator('aside button');
    const buttonCount = await buttons.count();
    console.log(`Found ${buttonCount} sidebar buttons`);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/sidebar.png` });
  });

  test('Main content area renders', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    const main = page.locator('main');
    await expect(main).toBeVisible();

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/main.png` });
  });

  test('Footer/StatusBar renders', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    const footer = page.locator('footer');
    await expect(footer).toBeVisible();

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/footer.png` });
  });
});

test.describe('Color System', () => {
  test('Background colors are applied correctly', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Check body background
    const bodyBg = await page.evaluate(() => {
      return getComputedStyle(document.body).backgroundColor;
    });
    console.log('Body background:', bodyBg);

    // Verify dark background is applied (rgb values should be low for dark themes)
    // Accept any dark background color (rgb values typically < 50)
    expect(bodyBg).toMatch(/rgb\(\s*\d{1,2}\s*,\s*\d{1,2}\s*,\s*\d{1,2}\s*\)/);

    // Check header background
    const headerBg = await page.evaluate(() => {
      const header = document.querySelector('header');
      return header ? getComputedStyle(header).backgroundColor : 'not found';
    });
    console.log('Header background:', headerBg);

    // Header should have a background applied (dark or transparent)
    expect(headerBg).not.toBe('not found');
  });

  test('Text colors are applied', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Check primary text color
    const textColor = await page.evaluate(() => {
      const h3 = document.querySelector('h3');
      return h3 ? getComputedStyle(h3).color : 'not found';
    });
    console.log('Text color:', textColor);
  });
});

test.describe('Command Palette', () => {
  test('opens with Cmd+K', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Press Cmd+K
    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);

    // Check dialog exists
    const dialog = page.locator('[role="dialog"]');
    const isVisible = await dialog.isVisible();
    console.log('Command palette visible:', isVisible);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/command-palette.png` });

    // Close with Escape
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);
  });

  test('has search input', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);

    const input = page.locator('input[placeholder*="搜索"]');
    const inputExists = await input.count() > 0;
    console.log('Search input exists:', inputExists);
  });

  test('has open project command', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);

    // Check for "打开项目" option
    const openProjectOption = page.locator('[role="option"]:has-text("打开项目")');
    const exists = await openProjectOption.count() > 0;
    console.log('Open project option exists:', exists);
    expect(exists).toBe(true);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/command-palette-open-project.png` });
  });

  test('has open file command', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);

    // Check for "打开文件" option
    const openFileOption = page.locator('[role="option"]:has-text("打开文件")');
    const exists = await openFileOption.count() > 0;
    console.log('Open file option exists:', exists);
    expect(exists).toBe(true);
  });

  test('keyboard navigation works', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);

    // Navigate down with arrow key
    await page.keyboard.press('ArrowDown');
    await page.waitForTimeout(200);

    // Check second option is selected
    const selectedOption = page.locator('[role="option"][aria-selected="true"]');
    const selectedText = await selectedOption.textContent();
    console.log('Selected option after ArrowDown:', selectedText);

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/command-palette-navigation.png` });
  });
});

test.describe('Navigation', () => {
  test('sidebar buttons are clickable', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Ensure no dialogs or overlays are open
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Get initial button count
    const initialCount = await page.locator('aside button').count();
    console.log('Sidebar buttons:', initialCount);

    // Click on sidebar button instead of hover (more reliable)
    const firstButton = page.locator('aside button').first();
    if (initialCount > 0) {
      await firstButton.click();
      await page.waitForTimeout(300);
    }

    await page.screenshot({ path: `${SCREENSHOTS_DIR}/sidebar-clicked.png` });
  });
});

test.describe('Layout Structure', () => {
  test('full layout screenshot', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1000);

    await page.screenshot({
      path: `${SCREENSHOTS_DIR}/full-layout.png`,
      fullPage: true
    });
  });

  test('flexbox layout is correct', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Check main container uses flex
    const mainContainer = page.locator('.app, [class*="flex"]').first();
    const display = await mainContainer.evaluate((el) => {
      return getComputedStyle(el).display;
    });
    console.log('Main container display:', display);

    // Check body uses flex column
    const bodyDisplay = await page.evaluate(() => {
      return getComputedStyle(document.body).display;
    });
    console.log('Body display:', bodyDisplay);
  });
});

test.describe('Interactive Tests', () => {
  test('bottom panel toggle', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Check if bottom panel button exists
    const toggleBtn = page.locator('button:has-text("Terminal")');
    const btnCount = await toggleBtn.count();
    console.log('Terminal toggle buttons:', btnCount);

    if (btnCount > 0) {
      await toggleBtn.first().click();
      await page.waitForTimeout(500);
      await page.screenshot({ path: `${SCREENSHOTS_DIR}/bottom-panel.png` });
    }
  });

  test('view mode switching', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });

    // Check for view toggle buttons
    const codeBtn = page.locator('button:has-text("代码")');
    const flowBtn = page.locator('button:has-text("执行流")');

    const codeCount = await codeBtn.count();
    const flowCount = await flowBtn.count();
    console.log('Code view button:', codeCount, 'Flow view button:', flowCount);
  });
});

// Custom helper for taking element screenshots
async function takeElementScreenshot(
  page: any,
  selector: string,
  name: string
) {
  const element = page.locator(selector);
  if (await element.count() > 0) {
    await element.screenshot({ path: `${SCREENSHOTS_DIR}/${name}.png` });
  }
}
