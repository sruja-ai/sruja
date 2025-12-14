import { test, expect } from "@playwright/test";

test.describe("Examples Loading", () => {
  test("open menu and load first example", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await examplesButton.click();
    const firstExample = page.locator(".example-item").first();
    await firstExample.waitFor({ timeout: 10000 });
    await firstExample.click();
    await page.waitForSelector(".react-flow", { timeout: 30000 });
    const nodes = await page.locator(".react-flow__node").count();
    expect(nodes).toBeGreaterThan(0);
  });
});
