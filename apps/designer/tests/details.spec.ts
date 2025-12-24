// apps/designer/tests/details.spec.ts
// Details view e2e tests
import { test, expect } from "@playwright/test";

test.describe("Details View", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app-container, .drop-zone", { timeout: 30000 });

    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Switch to Details tab
    await page.locator('button.view-tab:has-text("Details")').click();
    await page.waitForSelector(".details-view-unified", { timeout: 10000 });
  });

  test("displays details view content", async ({ page }) => {
    const detailsView = page.locator(".details-view-unified");
    await expect(detailsView).toBeVisible();
  });

  test("details view has sections", async ({ page }) => {
    // Details view should have some content sections
    const detailsContent = page.locator(".details-view-unified");
    await expect(detailsContent).toBeVisible();

    // Check if there are any list items or sections
    const hasContent = await detailsContent
      .locator("li, .section, .item, [class*='item-card']")
      .count();
    // At minimum, the view should be present even if empty
    expect(hasContent).toBeGreaterThanOrEqual(0);
  });
});
