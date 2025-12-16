import { test, expect } from "@playwright/test";

test.describe("Examples Search", () => {
  test("filter examples list", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await examplesButton.click();
    const menu = page.locator(".examples-menu");
    await expect(menu).toBeVisible();
    const input = page.locator('input[type="text"]');
    await input.fill("E-Commerce");
    const items = page.locator(".example-item");
    const count = await items.count();
    expect(count).toBeGreaterThan(0);
  });
});
