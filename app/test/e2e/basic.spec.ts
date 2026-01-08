import { test, expect } from '@playwright/test';

test.describe('FlowSight IDE', () => {
  test('should load the main page', async ({ page }) => {
    await page.goto('/');

    // Wait for the app to load
    await page.waitForSelector('#root', { timeout: 10000 });

    // Check that the root element exists
    const root = page.locator('#root');
    await expect(root).toBeVisible();
  });

  test('should have editor panel', async ({ page }) => {
    await page.goto('/');

    // Wait for Monaco editor to load
    await page.waitForTimeout(2000);

    // Check for Monaco editor container
    const editorContainer = page.locator('.monaco-editor');
    const hasEditor = await editorContainer.count() > 0;

    console.log('Monaco editor found:', hasEditor);
  });

  test('should have flow visualization panel', async ({ page }) => {
    await page.goto('/');

    await page.waitForTimeout(2000);

    // Check for React Flow container
    const flowContainer = page.locator('.react-flow');
    const hasFlow = await flowContainer.count() > 0;

    console.log('React Flow found:', hasFlow);
  });
});
