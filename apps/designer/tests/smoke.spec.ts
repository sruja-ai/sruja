import { test, expect } from "@playwright/test";

test.describe("Playground Smoke", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
  });

  test("loads demo and renders diagram", async ({ page }) => {
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
    }

    await page.waitForSelector(".react-flow", { timeout: 30000 });
    await page.waitForSelector(".level-indicator", { timeout: 10000 });

    const nodeCount = await page.locator(".react-flow__node").count();
    expect(nodeCount).toBeGreaterThan(0);
  });

  test("opens Examples menu and loads first example", async ({ page }) => {
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await examplesButton.waitFor({ timeout: 10000 });
    await examplesButton.click();

    const firstExample = page.locator(".example-item").first();
    await firstExample.waitFor({ timeout: 10000 });
    await firstExample.click();

    await page.waitForSelector(".react-flow", { timeout: 30000 });
    const nodes = await page.locator(".react-flow__node").count();
    expect(nodes).toBeGreaterThan(0);
  });

  test("switches layout engine via selector", async ({ page }) => {
    // Ensure there is a diagram
    const hasCanvas = await page
      .locator(".react-flow")
      .isVisible()
      .catch(() => false);
    if (!hasCanvas) {
      const examplesButton = page.locator('button[aria-label="Examples"]');
      if (await examplesButton.isVisible().catch(() => false)) {
        await examplesButton.click();
        const firstExample = page.locator(".example-item").first();
        await firstExample.waitFor({ timeout: 10000 });
        await firstExample.click();
      } else {
        const dropZone = page.locator(".drop-zone");
        if (await dropZone.isVisible().catch(() => false)) {
          await page.locator("button.demo-btn").click();
        }
      }
    }

    await page.waitForSelector(".react-flow", { timeout: 30000 });
    const selector = page.locator(".layout-selector select").first();
    await selector.waitFor({ timeout: 10000 });

    await selector.selectOption("c4level").catch(() => {});
    await page.waitForTimeout(1000);
    await selector.selectOption("sruja").catch(() => {});
    await page.waitForTimeout(1000);

    // Basic assertion: diagram remains rendered
    const nodeCount = await page.locator(".react-flow__node").count();
    expect(nodeCount).toBeGreaterThan(0);
  });

  test("navigates between view tabs", async ({ page }) => {
    // Ensure some data is loaded
    const hasData = await page
      .locator(".react-flow")
      .isVisible()
      .catch(() => false);
    if (!hasData) {
      const examplesButton = page.locator('button[aria-label="Examples"]');
      if (await examplesButton.isVisible().catch(() => false)) {
        await examplesButton.click();
        const firstExample = page.locator(".example-item").first();
        await firstExample.waitFor({ timeout: 10000 });
        await firstExample.click();
      } else {
        const dropZone = page.locator(".drop-zone");
        if (await dropZone.isVisible().catch(() => false)) {
          await page.locator("button.demo-btn").click();
        }
      }
    }

    const overviewTab = page.locator('button.view-tab:has-text("Builder")');
    const diagramTab = page.locator('button.view-tab:has-text("Diagram")');
    const detailsTab = page.locator('button.view-tab:has-text("Details")');
    const codeTab = page.locator('button.view-tab:has-text("Code")');

    await overviewTab.click();
    await expect(page.locator(".builder-wizard")).toBeVisible();

    await diagramTab.click();
    await expect(page.locator(".react-flow")).toBeVisible();

    await detailsTab.click();
    await expect(page.locator(".details-view")).toBeVisible();

    await codeTab.click();
    await expect(page.locator(".code-panel-container")).toBeVisible();
  });
});
