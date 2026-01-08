import { defineConfig } from '@playwright/test';
import path from 'path';

export default defineConfig({
  testDir: './test/e2e',
  timeout: 60000,
  retries: 0,
  use: {
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'tauri',
      use: {
        // For Tauri apps, we test via the web view
        baseURL: 'http://localhost:5173',
      },
    },
  ],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: true,
    timeout: 120000,
  },
});
