import { defineConfig, devices } from "@playwright/test";

/**
 * E2E tests for the Sruja book (e.g. "Show diagram" in ```sruja blocks).
 * Prerequisite: Book must be built and served (e.g. make book-serve or book/serve.sh).
 * Base URL: http://localhost:3000 (mdbook default).
 */
export default defineConfig({
  testDir: "e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: "html",
  use: {
    baseURL: process.env.BOOK_BASE_URL || "http://localhost:3000",
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  timeout: 30_000,
});
