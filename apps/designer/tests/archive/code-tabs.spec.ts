import { test, expect } from "@playwright/test";

test.describe("Code Tabs", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?tab=code");
    await page.waitForSelector(".app", { timeout: 30000 });
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    }
  });

  test("switch code tabs", async ({ page }) => {
    await page.locator('button.view-tab:has-text("Code")').click();
    await expect(page.locator(".code-panel-container")).toBeVisible();
    await page.locator('.code-tab:has-text("Sruja DSL")').click();
    await expect(page.locator(".code-content")).toBeVisible();
    await page.locator('.code-tab:has-text("JSON")').click();
    await expect(page.locator(".code-content")).toBeVisible();
    await page.locator('.code-tab:has-text("Markdown")').click();
    await expect(page.locator(".code-content")).toBeVisible();
  });
});
