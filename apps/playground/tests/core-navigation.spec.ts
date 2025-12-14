import { test, expect } from "@playwright/test";

test.describe("Core Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }
  });

  test("switch tabs by click", async ({ page }) => {
    await page.locator('button.view-tab:has-text("Overview")').click();
    await expect(page.locator(".overview-panel")).toBeVisible();
    await page.locator('button.view-tab:has-text("Diagram")').click();
    await expect(page.locator(".react-flow")).toBeVisible();
    await page.locator('button.view-tab:has-text("Details")').click();
    await expect(page.locator(".details-view")).toBeVisible();
    await page.locator('button.view-tab:has-text("Code")').click();
    await expect(page.locator(".code-panel-container")).toBeVisible();
  });
});
