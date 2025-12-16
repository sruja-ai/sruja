import { test, expect } from "@playwright/test";

test.describe("Settings and Mode", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }
  });

  test("toggle edit mode", async ({ page }) => {
    const editButton = page.locator(".mode-btn").nth(1);
    await editButton.click();
    await expect(page.locator(".center-panel.edit-mode")).toBeVisible();
    const viewButton = page.locator(".mode-btn").nth(0);
    await viewButton.click();
    await expect(page.locator(".center-panel.edit-mode")).toHaveCount(0);
  });

  test("open feature settings dialog", async ({ page }) => {
    await page.locator('button[aria-label="Feature Settings"]').click();
    await expect(page.getByText("Feature Settings")).toBeVisible();
    await page.getByRole("button", { name: "Close dialog" }).click();
    await expect(page.getByText("Feature Settings")).toHaveCount(0);
  });
});
