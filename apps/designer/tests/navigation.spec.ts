// apps/designer/tests/navigation.spec.ts
// Navigation panel and header e2e tests
import { test, expect } from "@playwright/test";

test.describe("Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    }
  });

  test("header is visible", async ({ page }) => {
    const header = page.locator("header, .header");
    await expect(header.first()).toBeVisible();
  });

  test("examples button is accessible", async ({ page }) => {
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await expect(examplesButton).toBeVisible();
  });

  test("opens examples menu", async ({ page }) => {
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await examplesButton.click();

    // Examples menu should appear
    const examplesMenu = page.locator(".example-item, [role='menu']");
    await expect(examplesMenu.first()).toBeVisible({ timeout: 5000 });
  });

  test("navigation panel can be toggled", async ({ page }) => {
    // Look for navigation toggle button
    const navToggle = page.locator('button[aria-label*="Navigation"], button[aria-label*="Menu"]').first();
    
    if (await navToggle.isVisible().catch(() => false)) {
      await navToggle.click();
      
      // Navigation panel should be visible
      const navPanel = page.locator(".navigation-panel, .navigation-panel-wrapper");
      await expect(navPanel.first()).toBeVisible({ timeout: 2000 });
    }
  });
});

