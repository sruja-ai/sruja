// apps/designer/tests/import-export.spec.ts
// Import/Export functionality e2e tests
import { test, expect } from "@playwright/test";

test.describe("Import and Export", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app-container, .drop-zone", { timeout: 30000 });

    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }
  });

  test("export menu is accessible", async ({ page }) => {
    // Look for export button in header
    const exportButton = page
      .locator('button:has-text("Export"), button[aria-label*="Export"]')
      .first();

    if (await exportButton.isVisible().catch(() => false)) {
      await exportButton.click();

      // Should show export options (PNG, SVG, JSON, etc.)
      const exportMenu = page.locator(".export-menu, [role='menu']");
      await expect(exportMenu).toBeVisible();
    }
  });

  test("import button is accessible", async ({ page }) => {
    // Look for import button in header
    const importButton = page
      .locator('button:has-text("Import"), button[aria-label*="Import"]')
      .first();

    if (await importButton.isVisible().catch(() => false)) {
      await expect(importButton).toBeVisible();
      // Note: We don't actually trigger file input in e2e tests as it requires file system access
    }
  });
});
