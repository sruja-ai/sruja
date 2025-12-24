// apps/designer/tests/builder.spec.ts
// Builder wizard e2e tests
import { test, expect } from "@playwright/test";

test.describe("Builder Wizard", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app-container, .drop-zone", { timeout: 30000 });

    // Load demo first to ensure architecture exists
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Switch to Builder tab
    const builderTab = page.locator('button.view-tab:has-text("Builder")');
    await builderTab.click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });
  });

  test("displays wizard with steps", async ({ page }) => {
    const steps = page.locator(".wizard-step");
    const stepCount = await steps.count();
    expect(stepCount).toBeGreaterThan(0);
  });

  test("navigates between wizard steps", async ({ page }) => {
    const steps = page.locator(".wizard-step");
    const stepCount = await steps.count();

    if (stepCount > 1) {
      // Click on second step
      await steps.nth(1).click();
      await page.waitForTimeout(500); // Allow step transition

      // Verify step content is visible
      const stepContent = page.locator(".wizard-main, .step-content");
      await expect(stepContent.first()).toBeVisible();
    }
  });

  test("shows DSL preview when enabled", async ({ page }) => {
    const toggleBtn = page.locator(".preview-toggle-btn");
    if (await toggleBtn.isVisible().catch(() => false)) {
      await toggleBtn.click();
      const dslPreview = page.locator(".dsl-preview");
      await expect(dslPreview).toBeVisible();
      await expect(dslPreview).toContainText("architecture");
    }
  });
});
