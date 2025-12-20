// apps/designer/tests/smoke.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Designer App Smoke Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
  });

  test("loads demo and renders LikeC4 diagram", async ({ page }) => {
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
    }

    await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    // LikeC4 renders SVG elements, check for diagram content
    await page.waitForSelector(".likec4-diagram-container svg", { timeout: 10000 });

    // Check for rendered diagram elements (LikeC4 uses SVG)
    const diagramContent = await page.locator(".likec4-diagram-container svg").count();
    expect(diagramContent).toBeGreaterThan(0);
  });

  test("opens Examples menu and loads first example", async ({ page }) => {
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await examplesButton.waitFor({ timeout: 10000 });
    await examplesButton.click();

    const firstExample = page.locator(".example-item").first();
    await firstExample.waitFor({ timeout: 10000 });
    await firstExample.click();

    await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    // Check for LikeC4 diagram content
    await page.waitForSelector(".likec4-diagram-container svg", { timeout: 10000 });
    const diagramContent = await page.locator(".likec4-diagram-container svg").count();
    expect(diagramContent).toBeGreaterThan(0);
  });

  test("LikeC4 diagram renders correctly", async ({ page }) => {
    // Ensure there is a diagram
    const hasCanvas = await page
      .locator(".likec4-canvas")
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

    await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    
    // Verify LikeC4 diagram container is present
    const likec4Container = page.locator(".likec4-diagram-container");
    await expect(likec4Container).toBeVisible();
    
    // Verify SVG elements are rendered (LikeC4 uses SVG for diagrams)
    await page.waitForSelector(".likec4-diagram-container svg", { timeout: 10000 });
    const diagramContent = await page.locator(".likec4-diagram-container svg").count();
    expect(diagramContent).toBeGreaterThan(0);
  });

  test("navigates between view tabs", async ({ page }) => {
    // Ensure some data is loaded
    const hasData = await page
      .locator(".likec4-canvas")
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
    await expect(page.locator(".likec4-canvas")).toBeVisible();

    await detailsTab.click();
    await expect(page.locator(".details-view")).toBeVisible();

    await codeTab.click();
    await expect(page.locator(".code-panel-container")).toBeVisible();
  });
});
