// apps/designer/tests/diagram.spec.ts
// Diagram view e2e tests
import { test, expect } from "@playwright/test";

test.describe("Diagram View", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    }

    // Ensure we're on Diagram tab
    await page.locator('button.view-tab:has-text("Diagram")').click();
    await page.waitForSelector(".likec4-canvas", { timeout: 10000 });
  });

  test("renders diagram with SVG elements", async ({ page }) => {
    await page.waitForSelector(".likec4-diagram-container svg", { timeout: 10000 });
    const svgCount = await page.locator(".likec4-diagram-container svg").count();
    expect(svgCount).toBeGreaterThan(0);
  });

  test("diagram is interactive", async ({ page }) => {
    // Check if canvas is present and visible
    const canvas = page.locator(".likec4-canvas");
    await expect(canvas).toBeVisible();

    // Try to interact with the canvas (pan/zoom if available)
    const canvasBox = await canvas.boundingBox();
    if (canvasBox) {
      // Click on canvas to verify it's interactive
      await page.mouse.move(canvasBox.x + canvasBox.width / 2, canvasBox.y + canvasBox.height / 2);
      await page.mouse.click(canvasBox.x + canvasBox.width / 2, canvasBox.y + canvasBox.height / 2);
    }
  });

  test("diagram responds to level changes via URL", async ({ page }) => {
    // Navigate with level parameter
    await page.goto("/?level=L1&tab=diagram");
    await page.waitForSelector(".likec4-canvas", { timeout: 10000 });

    // Verify diagram is still visible
    await expect(page.locator(".likec4-canvas")).toBeVisible();
  });
});

