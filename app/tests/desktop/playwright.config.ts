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

  // 不启动服务器，假设已有服务运行
});
