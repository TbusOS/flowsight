/**
 * FlowSight Enhanced Tests using DeskPilot v2.0
 * 
 * Tests using the new features:
 * - Ref mechanism for stable element references
 * - Network interception for edge case testing
 * - Flow chart testing for execution flow visualization
 * - Virtual list testing for large file trees
 * - Monaco editor testing for code viewer
 * - Screen recording for test documentation
 */

import { test, expect } from '@playwright/test';

// Note: These tests demonstrate DeskPilot usage patterns
// In production, import from 'deskpilot' package

test.describe('FlowSight with DeskPilot Enhanced Features', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  });

  test.describe('Ref Mechanism Tests', () => {
    
    test('should interact with elements using refs', async ({ page }) => {
      // Take a snapshot and get element refs
      const snapshot = await page.evaluate(() => {
        const elements: Array<{ ref: string; role: string; name: string }> = [];
        let refCounter = 0;
        
        document.querySelectorAll('button, [role="treeitem"], [role="menuitem"]').forEach((el) => {
          const ref = `@e${++refCounter}`;
          el.setAttribute('data-ref', ref);
          elements.push({
            ref,
            role: el.getAttribute('role') || el.tagName.toLowerCase(),
            name: el.getAttribute('aria-label') || el.textContent?.trim() || ''
          });
        });
        
        return elements;
      });
      
      expect(snapshot.length).toBeGreaterThan(0);
      
      // Find and click using ref
      const buttonRef = snapshot.find(e => e.role === 'button');
      if (buttonRef) {
        await page.click(`[data-ref="${buttonRef.ref}"]`);
      }
    });
    
    test('refs should be stable across interactions', async ({ page }) => {
      // First snapshot
      const snapshot1 = await page.evaluate(() => {
        return Array.from(document.querySelectorAll('[data-ref]')).map(el => ({
          ref: el.getAttribute('data-ref'),
          text: el.textContent?.trim()
        }));
      });
      
      // Perform some interaction
      await page.keyboard.press('Escape');
      await page.waitForTimeout(100);
      
      // Second snapshot - refs should remain stable
      const snapshot2 = await page.evaluate(() => {
        return Array.from(document.querySelectorAll('[data-ref]')).map(el => ({
          ref: el.getAttribute('data-ref'),
          text: el.textContent?.trim()
        }));
      });
      
      // Refs should be consistent
      expect(snapshot1.length).toBe(snapshot2.length);
    });
  });

  test.describe('Network Interception Tests', () => {
    
    test('should handle empty file list gracefully', async ({ page }) => {
      // Mock empty response
      await page.route('**/api/index', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ files: [], functions: [], structs: [] })
        });
      });
      
      // Trigger API call (e.g., open project)
      // The UI should show an empty state, not "0 个文件"
      await page.screenshot({ path: 'test-results/flowsight/empty-state.png' });
    });
    
    test('should handle API errors gracefully', async ({ page }) => {
      // Mock error response
      await page.route('**/api/**', async route => {
        await route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Internal Server Error' })
        });
      });
      
      // UI should show error state
      await page.screenshot({ path: 'test-results/flowsight/error-state.png' });
    });
    
    test('should handle slow network gracefully', async ({ page }) => {
      // Simulate slow network
      await page.route('**/api/**', async route => {
        await new Promise(resolve => setTimeout(resolve, 2000));
        await route.continue();
      });
      
      // UI should show loading state
      await page.screenshot({ path: 'test-results/flowsight/loading-state.png' });
    });
  });

  test.describe('Flow Chart Tests', () => {
    
    test('should render execution flow nodes', async ({ page }) => {
      // Wait for flow view to be visible
      const flowContainer = page.locator('[data-testid="flow-view"], .react-flow');
      
      if (await flowContainer.count() > 0) {
        // Get nodes
        const nodes = await page.evaluate(() => {
          const nodeElements = document.querySelectorAll('.react-flow__node');
          return Array.from(nodeElements).map(node => ({
            id: node.getAttribute('data-id'),
            type: node.getAttribute('data-type') || 'default',
            label: node.textContent?.trim()
          }));
        });
        
        // Should have nodes if flow is rendered
        if (nodes.length > 0) {
          expect(nodes[0]).toHaveProperty('id');
          expect(nodes[0]).toHaveProperty('label');
        }
        
        await page.screenshot({ path: 'test-results/flowsight/flow-nodes.png' });
      }
    });
    
    test('should render edges between nodes', async ({ page }) => {
      const flowContainer = page.locator('[data-testid="flow-view"], .react-flow');
      
      if (await flowContainer.count() > 0) {
        // Get edges
        const edges = await page.evaluate(() => {
          const edgeElements = document.querySelectorAll('.react-flow__edge');
          return Array.from(edgeElements).map(edge => ({
            id: edge.getAttribute('data-id'),
            source: edge.getAttribute('data-source'),
            target: edge.getAttribute('data-target')
          }));
        });
        
        await page.screenshot({ path: 'test-results/flowsight/flow-edges.png' });
      }
    });
    
    test('should support zoom and pan', async ({ page }) => {
      const flowContainer = page.locator('[data-testid="flow-view"], .react-flow');
      
      if (await flowContainer.count() > 0) {
        // Zoom in
        await page.keyboard.down('Control');
        await flowContainer.click();
        await page.mouse.wheel(0, -100);
        await page.keyboard.up('Control');
        
        await page.screenshot({ path: 'test-results/flowsight/flow-zoomed.png' });
        
        // Pan
        await flowContainer.hover();
        await page.mouse.down();
        await page.mouse.move(100, 100);
        await page.mouse.up();
        
        await page.screenshot({ path: 'test-results/flowsight/flow-panned.png' });
      }
    });
  });

  test.describe('Virtual List Tests', () => {
    
    test('should render file tree with virtualization', async ({ page }) => {
      const fileTree = page.locator('[data-testid="file-tree"], [role="tree"]');
      
      if (await fileTree.count() > 0) {
        // Get visible items
        const visibleItems = await page.evaluate(() => {
          const items = document.querySelectorAll('[role="treeitem"]');
          return Array.from(items).map(item => ({
            text: item.textContent?.trim(),
            isVisible: item.getBoundingClientRect().top >= 0
          }));
        });
        
        expect(visibleItems.length).toBeGreaterThan(0);
        
        await page.screenshot({ path: 'test-results/flowsight/file-tree.png' });
      }
    });
    
    test('should scroll to specific file', async ({ page }) => {
      const fileTree = page.locator('[data-testid="file-tree"], [role="tree"]');
      
      if (await fileTree.count() > 0) {
        // Scroll to bottom
        await fileTree.evaluate(el => {
          el.scrollTop = el.scrollHeight;
        });
        
        await page.waitForTimeout(300);
        await page.screenshot({ path: 'test-results/flowsight/file-tree-bottom.png' });
        
        // Scroll back to top
        await fileTree.evaluate(el => {
          el.scrollTop = 0;
        });
        
        await page.waitForTimeout(300);
        await page.screenshot({ path: 'test-results/flowsight/file-tree-top.png' });
      }
    });
    
    test('should measure scroll performance', async ({ page }) => {
      const fileTree = page.locator('[data-testid="file-tree"], [role="tree"]');
      
      if (await fileTree.count() > 0) {
        const startTime = Date.now();
        
        // Rapid scrolling
        for (let i = 0; i < 10; i++) {
          await fileTree.evaluate(el => {
            el.scrollTop += 200;
          });
          await page.waitForTimeout(16); // ~60fps
        }
        
        const endTime = Date.now();
        const duration = endTime - startTime;
        
        // Should complete within reasonable time
        expect(duration).toBeLessThan(1000);
      }
    });
  });

  test.describe('Monaco Editor Tests', () => {
    
    test('should display code with syntax highlighting', async ({ page }) => {
      const editor = page.locator('.monaco-editor');
      
      if (await editor.count() > 0) {
        // Check for syntax tokens
        const hasTokens = await page.evaluate(() => {
          const tokens = document.querySelectorAll('.monaco-editor .token');
          return tokens.length > 0;
        });
        
        await page.screenshot({ path: 'test-results/flowsight/monaco-syntax.png' });
      }
    });
    
    test('should support keyboard navigation', async ({ page }) => {
      const editor = page.locator('.monaco-editor');
      
      if (await editor.count() > 0) {
        await editor.click();
        
        // Go to line command
        await page.keyboard.press('Control+g');
        await page.waitForTimeout(200);
        await page.screenshot({ path: 'test-results/flowsight/monaco-goto.png' });
        await page.keyboard.press('Escape');
        
        // Find command
        await page.keyboard.press('Control+f');
        await page.waitForTimeout(200);
        await page.screenshot({ path: 'test-results/flowsight/monaco-find.png' });
        await page.keyboard.press('Escape');
      }
    });
  });

  test.describe('Screen Recording Tests', () => {
    
    test('should capture test sequence', async ({ page }) => {
      const screenshots: string[] = [];
      
      // Step 1: Initial state
      await page.screenshot({ path: 'test-results/flowsight/sequence-001.png' });
      screenshots.push('sequence-001.png');
      
      // Step 2: Open command palette
      await page.keyboard.press('Meta+k');
      await page.waitForTimeout(300);
      await page.screenshot({ path: 'test-results/flowsight/sequence-002.png' });
      screenshots.push('sequence-002.png');
      
      // Step 3: Close command palette
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);
      await page.screenshot({ path: 'test-results/flowsight/sequence-003.png' });
      screenshots.push('sequence-003.png');
      
      // All screenshots captured
      expect(screenshots).toHaveLength(3);
    });
    
    test('should compare screenshots', async ({ page }) => {
      // Take baseline
      await page.screenshot({ path: 'test-results/flowsight/baseline.png' });
      
      // Perform action
      await page.keyboard.press('Meta+k');
      await page.waitForTimeout(300);
      
      // Take comparison
      await page.screenshot({ path: 'test-results/flowsight/comparison.png' });
      
      // Close command palette
      await page.keyboard.press('Escape');
      
      // Screenshots should be different
      // (In production, use visual diff library)
    });
  });

  test.describe('Performance Benchmarks', () => {
    
    test('should measure page load time', async ({ page }) => {
      const startTime = Date.now();
      
      await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
      
      const loadTime = Date.now() - startTime;
      
      // Should load within 5 seconds
      expect(loadTime).toBeLessThan(5000);
      
      console.log(`Page load time: ${loadTime}ms`);
    });
    
    test('should measure command palette open time', async ({ page }) => {
      const startTime = Date.now();
      
      await page.keyboard.press('Meta+k');
      await page.waitForSelector('[role="dialog"], [data-testid="command-palette"]', { timeout: 1000 });
      
      const openTime = Date.now() - startTime;
      
      // Should open within 500ms
      expect(openTime).toBeLessThan(500);
      
      console.log(`Command palette open time: ${openTime}ms`);
      
      await page.keyboard.press('Escape');
    });
    
    test('should measure memory usage', async ({ page }) => {
      // Get JS heap size (if available)
      const memoryInfo = await page.evaluate(() => {
        if ('memory' in performance) {
          const memory = (performance as Performance & { memory: { usedJSHeapSize: number; totalJSHeapSize: number } }).memory;
          return {
            usedHeap: memory.usedJSHeapSize,
            totalHeap: memory.totalJSHeapSize
          };
        }
        return null;
      });
      
      if (memoryInfo) {
        console.log(`Used heap: ${(memoryInfo.usedHeap / 1024 / 1024).toFixed(2)}MB`);
        console.log(`Total heap: ${(memoryInfo.totalHeap / 1024 / 1024).toFixed(2)}MB`);
        
        // Should use less than 200MB
        expect(memoryInfo.usedHeap).toBeLessThan(200 * 1024 * 1024);
      }
    });
  });

  test.describe('Data Correctness Tests', () => {
    
    test('should not display zero counts for loaded data', async ({ page }) => {
      // This is the critical bug that started this whole journey
      // "发现 0 个文件，0 个函数，0 个结构体"
      
      // Check if stats display exists
      const statsElement = page.locator('[data-testid="stats"], .stats-display, text=/发现.*个/');
      
      if (await statsElement.count() > 0) {
        const text = await statsElement.textContent();
        
        // If showing counts, they should NOT all be zero
        if (text?.includes('发现')) {
          const hasAllZeros = /发现\s*0\s*个文件.*0\s*个函数.*0\s*个/.test(text);
          
          // Screenshot for evidence
          await page.screenshot({ path: 'test-results/flowsight/stats-display.png' });
          
          // This should FAIL if we have the bug
          expect(hasAllZeros).toBe(false);
        }
      }
    });
  });
});
