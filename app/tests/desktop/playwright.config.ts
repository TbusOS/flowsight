import { defineConfig, devices } from '@playwright/test';

/**
 * FlowSight Desktop UI 测试配置
 * 测试 Tauri WebView 中渲染的 React 组件
 */
export default defineConfig({
  testDir: '.',
  timeout: 60000,
  fullyParallel: false,
  retries: 0,
  workers: 1,
  reporter: [['list']],

  use: {
    baseURL: 'http://localhost:5173',
    trace: 'off',
    screenshot: 'on',
    video: 'off',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // 自动启动 Vite dev server
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:5173',
    reuseExistingServer: true,
    timeout: 120000,
  },
});
