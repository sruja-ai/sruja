import { test, expect } from "@playwright/test";

test.describe("Details Panel", () => {
  test("open details view tab", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    }
    await page.locator('button.view-tab:has-text("Details")').click();
    await expect(page.locator(".details-view")).toBeVisible();
  });
});
