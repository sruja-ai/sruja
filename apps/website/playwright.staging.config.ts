// apps/website/playwright.staging.config.ts
// Playwright configuration for testing the deployed staging website
import { defineConfig, devices } from "@playwright/test";

const STAGING_URL = process.env.STAGING_URL || "https://staging.sruja.ai";

export default defineConfig({
  testDir: "./__tests__/e2e/staging",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : undefined,
  reporter: process.env.CI ? [["github"], ["html"]] : "html",
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  use: {
    baseURL: STAGING_URL,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    // Increase navigation timeout for staging site
    navigationTimeout: 30_000,
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "firefox",
      use: { ...devices["Desktop Firefox"] },
    },
    {
      name: "webkit",
      use: { ...devices["Desktop Safari"] },
    },
  ],
  // No webServer needed - we're testing the deployed staging site
});
