import { test, expect } from "@playwright/test";

test.describe("Navigation Panel", () => {
  test("open and close navigation panel", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
    await page.setViewportSize({ width: 600, height: 800 });
    await page.locator(".mobile-menu-btn").first().click();
    await page.waitForSelector(".navigation-panel-wrapper.open", { timeout: 10000 });
    await expect(page.locator(".navigation-panel-wrapper.open")).toBeVisible();
    await page.locator(".mobile-overlay").click();
    await expect(page.locator(".navigation-panel-wrapper.open")).toHaveCount(0);
  });
});
