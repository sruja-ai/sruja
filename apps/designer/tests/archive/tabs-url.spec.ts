import { test, expect } from "@playwright/test";

test.describe("Tabs URL Persistence", () => {
  test("switching tabs updates ?tab param", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
    // Ensure demo is loaded to render tabs
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    }

    // In edit mode, the first tab is Builder (Overview merged)
    await page.locator('button.view-tab:has-text("Builder")').click();
    await expect.poll(async () => page.url()).toMatch(/\btab=builder\b/);

    await page.locator('button.view-tab:has-text("Diagram")').click();
    await expect.poll(async () => page.url()).toMatch(/\btab=diagram\b/);

    await page.locator('button.view-tab:has-text("Details")').click();
    await expect.poll(async () => page.url()).toMatch(/\btab=details\b/);

    await page.locator('button.view-tab:has-text("Code")').click();
    await expect.poll(async () => page.url()).toMatch(/\btab=code\b/);
  });
});
