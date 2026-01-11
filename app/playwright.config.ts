import { defineConfig, devices } from '@playwright/test';

/**
 * FlowSight E2E 测试配置
 *
 * 测试场景：
 * 1. 内核调用链注入 - 验证 USB probe 等函数显示完整调用链
 * 2. 执行上下文显示 - 验证节点显示 can_sleep 等信息
 * 3. 节点交互 - 验证展开/折叠功能
 */
export default defineConfig({
  testDir: './tests/e2e',
  timeout: 60000,
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [['html'], ['list']],

  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
