// apps/website/__tests__/e2e/viewer.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Viewer Page", () => {
  test("loads empty viewer", async ({ page }) => {
    await page.goto("/viewer");

    // Check if page loads
    await expect(page).toHaveTitle(/Sruja/i);

    // Check for editor element
    const editor = page.getByTestId("viewer-editor");
    await expect(editor).toBeVisible({ timeout: 15000 });
  });

  test("displays viewer interface", async ({ page }) => {
    await page.goto("/viewer");

    // Wait for viewer to load
    await page.waitForLoadState("networkidle");

    // Verify viewer container exists
    const viewerContainer = page
      .locator('[data-testid="viewer"]')
      .or(page.locator(".viewer-container"))
      .first();
    await expect(viewerContainer).toBeVisible({ timeout: 5000 });
  });
});
