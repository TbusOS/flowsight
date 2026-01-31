/**
 * Playwright Tests for Theme Switching
 * Tests theme toggle functionality and persistence
 */

import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

const SCREENSHOTS_DIR = './test-results/flowsight/theme';

test.beforeAll(() => {
  // Create screenshots directory
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }
});

test.beforeEach(async ({ page, context }) => {
  // Clear localStorage before each test to ensure clean state
  await context.clearCookies();
  await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  // Wait for theme initialization
  await page.waitForTimeout(500);
});

test.describe('Theme Switching - Basic Functionality', () => {
  test('default theme is dark', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Verify document has dark theme attribute
    const theme = await page.evaluate(() => {
      return document.documentElement.getAttribute('data-theme');
    });
    expect(theme).toBe('dark');

    // Verify theme selector button shows "深色"
    const themeButton = page.locator('button:has-text("深色")');
    await expect(themeButton).toBeVisible();
  });

  test('theme toggle button exists', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Find theme selector button (shows current theme label)
    const themeButton = page.locator('button:has-text("深色")');
    await expect(themeButton).toBeVisible();

    // Verify button has theme color indicator
    const colorIndicator = themeButton.locator('div[style*="background-color"]');
    await expect(colorIndicator).toBeVisible();
  });

  test('click button opens theme menu', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Find and click theme selector button
    const themeButton = page.locator('button:has-text("深色")');
    await themeButton.click();
    await page.waitForTimeout(300);

    // Verify theme menu is visible
    const themeMenu = page.locator('text=选择主题').locator('..');
    await expect(themeMenu).toBeVisible();

    // Verify menu contains theme options
    const lightOption = page.locator('button:has-text("浅色")');
    await expect(lightOption).toBeVisible();
  });

  test('switch to light theme', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Open theme menu
    const themeButton = page.locator('button:has-text("深色")');
    await themeButton.click();
    await page.waitForTimeout(300);

    // Click light theme option
    const lightOption = page.locator('button:has-text("浅色")');
    await lightOption.click();
    await page.waitForTimeout(500);

    // Verify theme changed to light
    const theme = await page.evaluate(() => {
      return document.documentElement.getAttribute('data-theme');
    });
    expect(theme).toBe('light');

    // Verify button text updated
    const updatedButton = page.locator('button:has-text("浅色")');
    await expect(updatedButton.first()).toBeVisible();
  });

  test('CSS variables change correctly', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Get initial background color (dark theme)
    const darkBgColor = await page.evaluate(() => {
      return getComputedStyle(document.body).backgroundColor;
    });
    console.log('Dark theme background:', darkBgColor);

    // Switch to light theme
    const themeButton = page.locator('button:has-text("深色")');
    await themeButton.click();
    await page.waitForTimeout(300);

    const lightOption = page.locator('button:has-text("浅色")');
    await lightOption.click();
    await page.waitForTimeout(500);

    // Get light theme background color
    const lightBgColor = await page.evaluate(() => {
      return getComputedStyle(document.body).backgroundColor;
    });
    console.log('Light theme background:', lightBgColor);

    // Verify colors are different
    expect(lightBgColor).not.toBe(darkBgColor);

    // Verify CSS variable --bg-primary exists
    const bgPrimary = await page.evaluate(() => {
      return getComputedStyle(document.documentElement)
        .getPropertyValue('--bg-primary')
        .trim();
    });
    expect(bgPrimary).toBeTruthy();
  });
});

test.describe('Theme Switching - Persistence', () => {
  test('theme preference persists after page reload', async ({ page, context }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Switch to light theme
    const themeButton = page.locator('button:has-text("深色")');
    await themeButton.click();
    await page.waitForTimeout(300);

    const lightOption = page.locator('button:has-text("浅色")');
    await lightOption.click();
    await page.waitForTimeout(500);

    // Verify theme is light
    let theme = await page.evaluate(() => {
      return document.documentElement.getAttribute('data-theme');
    });
    expect(theme).toBe('light');

    // Reload page
    await page.reload({ waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Verify theme persisted
    theme = await page.evaluate(() => {
      return document.documentElement.getAttribute('data-theme');
    });
    expect(theme).toBe('light');

    // Verify localStorage has theme stored
    const storedTheme = await page.evaluate(() => {
      return localStorage.getItem('flowsight-theme');
    });
    expect(storedTheme).toBe('light');
  });

  test('localStorage stores theme preference', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Check initial localStorage (should be dark or null)
    let storedTheme = await page.evaluate(() => {
      return localStorage.getItem('flowsight-theme');
    });
    console.log('Initial stored theme:', storedTheme);

    // Switch to light theme
    const themeButton = page.locator('button:has-text("深色")');
    await themeButton.click();
    await page.waitForTimeout(300);

    const lightOption = page.locator('button:has-text("浅色")');
    await lightOption.click();
    await page.waitForTimeout(500);

    // Verify localStorage updated
    storedTheme = await page.evaluate(() => {
      return localStorage.getItem('flowsight-theme');
    });
    expect(storedTheme).toBe('light');
  });
});

test.describe('Theme Switching - Visual Verification', () => {
  test('dark theme screenshot', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1000);

    // Ensure dark theme is active
    const theme = await page.evaluate(() => {
      return document.documentElement.getAttribute('data-theme');
    });
    if (theme !== 'dark') {
      // Switch to dark if not already dark
      const themeButton = page.locator('button').filter({ hasText: /深色|浅色|Slate|Forest|Sunset|Lavender/ }).first();
      await themeButton.click();
      await page.waitForTimeout(300);
      const darkOption = page.locator('button:has-text("深色")');
      await darkOption.click();
      await page.waitForTimeout(500);
    }

    // Take screenshot
    await page.screenshot({
      path: `${SCREENSHOTS_DIR}/dark-theme.png`,
      fullPage: true
    });

    // Verify dark theme CSS variables
    const bgColor = await page.evaluate(() => {
      return getComputedStyle(document.body).backgroundColor;
    });
    console.log('Dark theme body background:', bgColor);
  });

  test('light theme screenshot', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Switch to light theme
    const themeButton = page.locator('button:has-text("深色")');
    await themeButton.click();
    await page.waitForTimeout(300);

    const lightOption = page.locator('button:has-text("浅色")');
    await lightOption.click();
    await page.waitForTimeout(1000);

    // Verify light theme is active
    const theme = await page.evaluate(() => {
      return document.documentElement.getAttribute('data-theme');
    });
    expect(theme).toBe('light');

    // Take screenshot
    await page.screenshot({
      path: `${SCREENSHOTS_DIR}/light-theme.png`,
      fullPage: true
    });

    // Verify light theme CSS variables
    const bgColor = await page.evaluate(() => {
      return getComputedStyle(document.body).backgroundColor;
    });
    console.log('Light theme body background:', bgColor);
  });

  test('theme menu screenshot', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    // Open theme menu
    const themeButton = page.locator('button:has-text("深色")');
    await themeButton.click();
    await page.waitForTimeout(300);

    // Take screenshot of menu
    const menu = page.locator('text=选择主题').locator('..');
    await menu.screenshot({
      path: `${SCREENSHOTS_DIR}/theme-menu.png`
    });
  });

  test('all themes can be switched', async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    const themes = ['dark', 'light', 'slate', 'forest', 'sunset', 'lavender'];
    const themeLabels: Record<string, string> = {
      dark: '深色',
      light: '浅色',
      slate: 'Slate',
      forest: 'Forest',
      sunset: 'Sunset',
      lavender: 'Lavender',
    };

    for (const theme of themes) {
      // Open theme menu
      const themeToggle = page.locator('[data-testid="theme-toggle"]');
      await themeToggle.click();
      await page.waitForTimeout(300);

      // Select theme using data-testid
      const themeOption = page.locator(`[data-testid="theme-option-${theme}"]`);
      await themeOption.click();
      await page.waitForTimeout(500);

      // Verify theme applied
      const appliedTheme = await page.evaluate(() => {
        return document.documentElement.getAttribute('data-theme');
      });
      expect(appliedTheme).toBe(theme);
      console.log(`✓ Theme "${theme}" applied successfully`);
    }
  });
});
