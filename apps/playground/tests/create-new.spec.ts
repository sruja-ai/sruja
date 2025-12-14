import { test, expect } from "@playwright/test";

test.describe("Create New Architecture", () => {
  test("create new from empty state", async ({ page }) => {
    await page.goto("/?share=disabled");
    await page.waitForSelector(".app", { timeout: 30000 });
    await page.waitForSelector(".drop-zone", { timeout: 30000 });
    await page.locator('button:has-text("Create New")').click();
    await page.waitForSelector(".view-tabs", { timeout: 30000 });
    const tabCount = await page.locator('button.view-tab:has-text("Diagram")').count();
    if (tabCount > 0) {
      await page.locator('button.view-tab:has-text("Diagram")').click();
    }
    await page.waitForSelector(".react-flow", { timeout: 30000 });
    const nodes = await page.locator(".react-flow__node").count();
    expect(nodes).toBeGreaterThan(0);
  });
});
