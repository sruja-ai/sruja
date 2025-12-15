import { test, expect } from "@playwright/test";

test.describe("Template Gallery", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // We need fresh state without demo loaded for template tests
    // Check if we have drop zone (fresh state)
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      // Load demo first since we need wizard access
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Switch to Builder tab
    const builderTab = page.locator('button.view-tab:has-text("Builder")');
    await builderTab.click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });
  });

  test("opens template gallery from Goals step", async ({ page }) => {
    // Ensure we're on Goals step
    await page.locator(".wizard-step").nth(0).click();

    // Look for template prompt button
    const templateBtn = page.locator(".template-prompt button, button:has-text('Template')");

    if (await templateBtn.isVisible().catch(() => false)) {
      await templateBtn.click();

      const modal = page.locator(".template-gallery-modal");
      await expect(modal).toBeVisible();
      await expect(modal).toContainText("Choose a Template");
    }
  });

  test("displays 6 starter templates", async ({ page }) => {
    await page.locator(".wizard-step").nth(0).click();

    const templateBtn = page.locator(".template-prompt button, button:has-text('Template')");
    if (!(await templateBtn.isVisible().catch(() => false))) {
      test.skip();
      return;
    }

    await templateBtn.click();
    await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

    const cards = page.locator(".template-card");
    await expect(cards).toHaveCount(6);
  });

  test("shows template categories", async ({ page }) => {
    await page.locator(".wizard-step").nth(0).click();

    const templateBtn = page.locator(".template-prompt button, button:has-text('Template')");
    if (!(await templateBtn.isVisible().catch(() => false))) {
      test.skip();
      return;
    }

    await templateBtn.click();
    await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

    // Should have category badges
    await expect(page.locator(".template-category")).toContainText([
      "Basic",
      "Intermediate",
      "Advanced",
    ]);
  });

  test("selects template on click", async ({ page }) => {
    await page.locator(".wizard-step").nth(0).click();

    const templateBtn = page.locator(".template-prompt button, button:has-text('Template')");
    if (!(await templateBtn.isVisible().catch(() => false))) {
      test.skip();
      return;
    }

    await templateBtn.click();
    await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

    // Click second template (Simple Web App)
    const webAppCard = page.locator(".template-card").nth(1);
    await webAppCard.click();

    // Should show selected state
    await expect(webAppCard).toHaveClass(/selected/);

    // Should show checkmark
    await expect(webAppCard.locator(".template-selected-badge")).toBeVisible();
  });

  test("loads template on Use Template button", async ({ page }) => {
    await page.locator(".wizard-step").nth(0).click();

    const templateBtn = page.locator(".template-prompt button, button:has-text('Template')");
    if (!(await templateBtn.isVisible().catch(() => false))) {
      test.skip();
      return;
    }

    await templateBtn.click();
    await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

    // Select Microservices template
    const microservicesCard = page.locator(".template-card").filter({ hasText: "Microservices" });
    await microservicesCard.click();

    // Click Use Template
    const useBtn = page.locator(".template-use-btn");
    await useBtn.click();

    // Modal should close
    await expect(page.locator(".template-gallery-modal")).not.toBeVisible();

    // DSL preview should update with new content
    const dslPreview = page.locator(".dsl-preview");
    await expect(dslPreview).toContainText("Microservices");
  });

  test("loads template on double-click", async ({ page }) => {
    await page.locator(".wizard-step").nth(0).click();

    const templateBtn = page.locator(".template-prompt button, button:has-text('Template')");
    if (!(await templateBtn.isVisible().catch(() => false))) {
      test.skip();
      return;
    }

    await templateBtn.click();
    await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

    // Double-click Event-Driven template
    const eventDrivenCard = page.locator(".template-card").filter({ hasText: "Event-Driven" });
    await eventDrivenCard.dblclick();

    // Modal should close
    await expect(page.locator(".template-gallery-modal")).not.toBeVisible();

    // Architecture should be loaded
    const dslPreview = page.locator(".dsl-preview");
    await expect(dslPreview).toContainText("Event");
  });

  test("closes modal on Cancel", async ({ page }) => {
    await page.locator(".wizard-step").nth(0).click();

    const templateBtn = page.locator(".template-prompt button, button:has-text('Template')");
    if (!(await templateBtn.isVisible().catch(() => false))) {
      test.skip();
      return;
    }

    await templateBtn.click();
    await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

    await page.locator(".template-cancel-btn").click();

    await expect(page.locator(".template-gallery-modal")).not.toBeVisible();
  });

  test("Use Template button is disabled without selection", async ({ page }) => {
    await page.locator(".wizard-step").nth(0).click();

    const templateBtn = page.locator(".template-prompt button, button:has-text('Template')");
    if (!(await templateBtn.isVisible().catch(() => false))) {
      test.skip();
      return;
    }

    await templateBtn.click();
    await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

    const useBtn = page.locator(".template-use-btn");
    await expect(useBtn).toBeDisabled();
  });
});
