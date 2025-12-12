import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: 'tests',
  use: {
    baseURL: 'http://localhost:4173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure'
  },
  webServer: {
    command: 'npm run preview',
    port: 4173,
    timeout: 120_000,
    reuseExistingServer: true,
  },
  reporter: [['list']]
})
