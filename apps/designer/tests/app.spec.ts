// apps/designer/tests/app.spec.ts
// Core e2e tests for the designer app - tests run against built code
import { test, expect } from "@playwright/test";

test.describe("Designer App - Core Functionality", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
  });

  test("app loads and shows empty state", async ({ page }) => {
    // Should show drop zone or demo button when no architecture is loaded
    const dropZone = page.locator(".drop-zone");
    await expect(dropZone).toBeVisible();
    await expect(page.locator("button.demo-btn")).toBeVisible();
  });

  test("loads demo architecture", async ({ page }) => {
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    }

    // Verify diagram is rendered
    await expect(page.locator(".likec4-canvas")).toBeVisible();
    await page.waitForSelector(".likec4-diagram-container svg", { timeout: 10000 });
    const svgCount = await page.locator(".likec4-diagram-container svg").count();
    expect(svgCount).toBeGreaterThan(0);
  });

  test("view tabs are visible and functional", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    }

    // Verify all tabs are present
    await expect(page.locator('button.view-tab:has-text("Builder")')).toBeVisible();
    await expect(page.locator('button.view-tab:has-text("Diagram")')).toBeVisible();
    await expect(page.locator('button.view-tab:has-text("Details")')).toBeVisible();
    await expect(page.locator('button.view-tab:has-text("Code")')).toBeVisible();
  });

  test("navigates between view tabs", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    }

    // Test Builder tab
    await page.locator('button.view-tab:has-text("Builder")').click();
    await expect(page.locator(".builder-wizard")).toBeVisible();

    // Test Diagram tab
    await page.locator('button.view-tab:has-text("Diagram")').click();
    await expect(page.locator(".likec4-canvas")).toBeVisible();

    // Test Details tab
    await page.locator('button.view-tab:has-text("Details")').click();
    await expect(page.locator(".details-view")).toBeVisible();

    // Test Code tab
    await page.locator('button.view-tab:has-text("Code")').click();
    await expect(page.locator(".code-panel-container")).toBeVisible();
  });

  test("URL state persists tab selection", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    }

    // Switch to Builder tab
    await page.locator('button.view-tab:has-text("Builder")').click();
    await expect.poll(async () => page.url()).toMatch(/\btab=builder\b/);

    // Switch to Diagram tab
    await page.locator('button.view-tab:has-text("Diagram")').click();
    await expect.poll(async () => page.url()).toMatch(/\btab=diagram\b/);

    // Switch to Details tab
    await page.locator('button.view-tab:has-text("Details")').click();
    await expect.poll(async () => page.url()).toMatch(/\btab=details\b/);

    // Switch to Code tab
    await page.locator('button.view-tab:has-text("Code")').click();
    await expect.poll(async () => page.url()).toMatch(/\btab=code\b/);
  });

  test("loads example from examples menu", async ({ page }) => {
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await examplesButton.waitFor({ timeout: 10000 });
    await examplesButton.click();

    const firstExample = page.locator(".example-item").first();
    await firstExample.waitFor({ timeout: 10000 });
    await firstExample.click();

    // Verify diagram loads
    await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    await page.waitForSelector(".likec4-diagram-container svg", { timeout: 10000 });
    const svgCount = await page.locator(".likec4-diagram-container svg").count();
    expect(svgCount).toBeGreaterThan(0);
  });
});

