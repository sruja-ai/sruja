import { test, expect } from "@playwright/test";

test.describe("URL State Sync", () => {
  test("initial level from URL", async ({ page }) => {
    await page.goto("/?level=L2");
    await page.waitForSelector(".app", { timeout: 30000 });
    // Navigation panel renders level buttons; verify L2 is active
    const activeLevelBtn = page.locator(".level-buttons .level-btn.active");
    await expect(activeLevelBtn).toContainText(/L2/);
  });

  test("drill-down updates level in URL", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Ensure demo is loaded if empty
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    }

    // Click first system to drill down to L2
    const firstSystemBtn = page.locator(".nav-item .nav-item-btn").first();
    await firstSystemBtn.click();

    // The URL should include level=L2
    await expect.poll(async () => page.url()).toMatch(/\blevel=L2\b/);
  });
});
