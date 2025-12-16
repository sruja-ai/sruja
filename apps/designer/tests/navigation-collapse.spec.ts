import { test, expect } from "@playwright/test";

test.describe("Navigation Panel Collapse", () => {
  test("toggle collapse persists and updates UI state", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Ensure demo is loaded
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    await page.waitForSelector(".navigation-panel", { timeout: 10000 });
    const wrapper = page.locator(".navigation-panel");
    const collapseBtn = page.locator(".nav-collapse-btn");
    await expect(collapseBtn).toBeVisible({ timeout: 10000 });

    // Collapse
    await collapseBtn.click();
    await expect(wrapper).toHaveClass(/collapsed/);
    const collapsedFlag = await page.evaluate(() =>
      localStorage.getItem("navigation-panel-collapsed")
    );
    expect(collapsedFlag).toBe("true");

    // Expand
    await collapseBtn.click();
    await expect(wrapper).not.toHaveClass(/collapsed/);
    const expandedFlag = await page.evaluate(() =>
      localStorage.getItem("navigation-panel-collapsed")
    );
    expect(expandedFlag).toBe("false");
  });
});
