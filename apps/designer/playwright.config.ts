import { defineConfig } from "@playwright/test";

// Allow overriding server command and baseURL to avoid port collisions and host restrictions.
// Defaults keep current behavior but can be changed via env when running locally or in CI:
// - PLAYWRIGHT_TEST_SERVER_CMD: custom server start command (e.g., "npm run preview -- --host 127.0.0.1 --port 0")
// - PLAYWRIGHT_TEST_BASE_URL: custom base URL if server runs elsewhere
// - PLAYWRIGHT_TEST_NO_SERVER: skip starting a webServer (expects externally running app)
const DEFAULT_PORT = process.env.PLAYWRIGHT_TEST_PORT || "4173";
const DEFAULT_HOST = process.env.PLAYWRIGHT_TEST_HOST || "127.0.0.1";
const baseURL = process.env.PLAYWRIGHT_TEST_BASE_URL || `http://${DEFAULT_HOST}:${DEFAULT_PORT}`;
const serverCommand =
  process.env.PLAYWRIGHT_TEST_SERVER_CMD ||
  (process.env.CI
    ? `npm run preview -- --host ${DEFAULT_HOST} --port ${DEFAULT_PORT}`
    : `npm run dev -- --host ${DEFAULT_HOST} --port ${DEFAULT_PORT}`);
const startServer = !process.env.PLAYWRIGHT_TEST_NO_SERVER;

export default defineConfig({
  testDir: "tests",
  use: {
    baseURL,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  webServer: startServer
    ? {
        command: serverCommand,
        url: baseURL,
        timeout: 120_000,
        reuseExistingServer: !process.env.CI,
      }
    : undefined,
  reporter: [["list"]],
});
