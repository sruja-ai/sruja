// apps/designer/tests/app.spec.ts
// Core e2e tests for the designer app - tests run against built code
import { test, expect } from "@playwright/test";

test.describe("Designer App - Core Functionality", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    // Wait for app container or drop zone to appear
    await page.waitForSelector(".app-container, .drop-zone", { timeout: 30000 });
  });

  test("app loads and shows empty state", async ({ page }) => {
    // Should show drop zone or demo button when no architecture is loaded
    const dropZone = page.locator(".drop-zone");
    await expect(dropZone).toBeVisible({ timeout: 10000 });
    await expect(page.locator("button.demo-btn")).toBeVisible();
  });

  test("loads demo architecture", async ({ page }) => {
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      // Wait for ReactFlow to appear (SrujaCanvas uses ReactFlow)
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Verify diagram is rendered - SrujaCanvas uses ReactFlow
    await expect(page.locator(".react-flow")).toBeVisible();
    // Wait for nodes to appear
    await page.waitForSelector(".react-flow__node", { timeout: 10000 });
    const nodeCount = await page.locator(".react-flow__node").count();
    expect(nodeCount).toBeGreaterThan(0);
  });

  test("view tabs are visible and functional", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Verify all tabs are present
    await expect(page.locator('button.view-tab:has-text("Builder")')).toBeVisible();
    await expect(page.locator('button.view-tab:has-text("Diagram")')).toBeVisible();
    await expect(page.locator('button.view-tab:has-text("Details")')).toBeVisible();
    await expect(page.locator('button.view-tab:has-text("Code")')).toBeVisible();
    await expect(page.locator('button.view-tab:has-text("Governance")')).toBeVisible();
  });

  test("navigates between view tabs", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Test Builder tab
    await page.locator('button.view-tab:has-text("Builder")').click();
    await expect(page.locator(".builder-wizard")).toBeVisible({ timeout: 5000 });

    // Test Diagram tab
    await page.locator('button.view-tab:has-text("Diagram")').click();
    await expect(page.locator(".react-flow")).toBeVisible({ timeout: 5000 });

    // Test Details tab
    await page.locator('button.view-tab:has-text("Details")').click();
    await expect(page.locator(".details-view-unified")).toBeVisible({ timeout: 5000 });

    // Test Code tab
    await page.locator('button.view-tab:has-text("Code")').click();
    await expect(page.locator(".code-panel-container")).toBeVisible({ timeout: 5000 });

    // Test Governance tab
    await page.locator('button.view-tab:has-text("Governance")').click();
    await expect(page.locator('[role="tabpanel"][id="tabpanel-governance"]')).toBeVisible({
      timeout: 5000,
    });
  });

  test("URL state persists tab selection", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
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

    // Wait for examples menu to appear
    await page.waitForTimeout(500);

    // Look for example items - could be in various formats
    const exampleItem = page
      .locator('[class*="example"], [role="option"], [class*="Example"]')
      .first();
    await exampleItem.waitFor({ timeout: 10000 });
    await exampleItem.click();

    // Verify diagram loads - wait for ReactFlow
    await page.waitForSelector(".react-flow", { timeout: 30000 });
    await page.waitForSelector(".react-flow__node", { timeout: 10000 });
    const nodeCount = await page.locator(".react-flow__node").count();
    expect(nodeCount).toBeGreaterThan(0);
  });
});
