import { defineConfig } from "@playwright/test";

// E2E tests run against built code by default (preview server).
// This ensures tests validate the production build, not just dev mode.
//
// Environment variables for customization:
// - PLAYWRIGHT_TEST_SERVER_CMD: custom server start command
// - PLAYWRIGHT_TEST_BASE_URL: custom base URL if server runs elsewhere
// - PLAYWRIGHT_TEST_NO_SERVER: skip starting a webServer (expects externally running app)
// - PLAYWRIGHT_TEST_USE_DEV: set to "true" to use dev server instead of preview
// - PLAYWRIGHT_TEST_PORT: custom port (default: 4173)
// - PLAYWRIGHT_TEST_HOST: custom host (default: 127.0.0.1)
const DEFAULT_PORT = process.env.PLAYWRIGHT_TEST_PORT || "4173";
const DEFAULT_HOST = process.env.PLAYWRIGHT_TEST_HOST || "127.0.0.1";
const baseURL = process.env.PLAYWRIGHT_TEST_BASE_URL || `http://${DEFAULT_HOST}:${DEFAULT_PORT}`;

// Default to preview server (built code) unless explicitly overridden
const useDevServer = process.env.PLAYWRIGHT_TEST_USE_DEV === "true";
// Preview server uses the built code (which should be built with BASE_PATH=/ for tests)
const serverCommand =
  process.env.PLAYWRIGHT_TEST_SERVER_CMD ||
  (useDevServer
    ? `npm run dev -- --host ${DEFAULT_HOST} --port ${DEFAULT_PORT}`
    : `npm run preview -- --host ${DEFAULT_HOST} --port ${DEFAULT_PORT}`);

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
  reporter: [["list"], ["html", { outputFolder: "playwright-report" }]],
  // Increase timeout for slower operations
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
});
