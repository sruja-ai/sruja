import { test, expect } from "@playwright/test";

test.describe("Builder Wizard", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

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

  test("displays wizard stepper with 5 steps", async ({ page }) => {
    const steps = page.locator(".wizard-step");
    await expect(steps).toHaveCount(5);

    // Verify step labels
    await expect(page.locator(".wizard-step").nth(0)).toContainText("Define");
    await expect(page.locator(".wizard-step").nth(1)).toContainText("Context");
    await expect(page.locator(".wizard-step").nth(2)).toContainText("Containers");
    await expect(page.locator(".wizard-step").nth(3)).toContainText("Components");
    await expect(page.locator(".wizard-step").nth(4)).toContainText("Flows");
  });

  test("navigates between steps using stepper", async ({ page }) => {
    // Click on Context step
    await page.locator(".wizard-step").nth(1).click();
    await expect(page.locator(".step-header h2")).toContainText("System Context");

    // Click back to Define step
    await page.locator(".wizard-step").nth(0).click();
    await expect(page.locator(".step-header h2")).toContainText("Goals");
  });

  test("shows DSL preview in sidebar", async ({ page }) => {
    // Enable preview sidebar
    const toggleBtn = page.locator(".preview-toggle-btn");
    await toggleBtn.click();
    const dslPreview = page.locator(".dsl-preview");
    await expect(dslPreview).toBeVisible();

    // Should contain architecture keyword
    await expect(dslPreview).toContainText("architecture");
  });

  test("shows validation panel in sidebar", async ({ page }) => {
    // Ensure preview sidebar is visible
    const toggleBtn = page.locator(".preview-toggle-btn");
    await toggleBtn.click();
    const validationPanel = page.locator(".validation-panel");
    await expect(validationPanel).toBeVisible();

    // Should show score
    const score = page.locator(".validation-panel .score, .validation-score");
    await expect(score).toBeVisible();
  });

  test("shows Share button in sidebar", async ({ page }) => {
    // Ensure preview sidebar is visible
    const toggleBtn = page.locator(".preview-toggle-btn");
    await toggleBtn.click();
    const shareBtn = page.locator(".share-btn");
    await expect(shareBtn).toBeVisible();
    await expect(shareBtn).toContainText("Share");
  });

  test("GoalsStep allows adding requirements", async ({ page }) => {
    // Ensure we're on Goals step
    await page.locator(".wizard-step").nth(0).click();

    // Find requirements section
    const reqSection = page
      .locator(".requirements-section, .step-section")
      .filter({ hasText: "Requirements" });

    if (await reqSection.isVisible().catch(() => false)) {
      // Fill in requirement form
      const idInput = reqSection
        .locator('input[placeholder*="ID"], input[placeholder*="R"]')
        .first();
      const titleInput = reqSection
        .locator('input[placeholder*="title"], input[placeholder*="Title"]')
        .first();

      if (await idInput.isVisible().catch(() => false)) {
        await idInput.fill("TEST-REQ-1");
        await titleInput.fill("Test Requirement");

        // Click add button
        const addBtn = reqSection.locator('button:has-text("Add")');
        await addBtn.click();

        // Verify it was added
        await expect(page.locator(".item-list, .requirements-list")).toContainText("TEST-REQ-1");
      }
    }
  });

  test("SystemContextStep shows actors and systems", async ({ page }) => {
    // Navigate to Context step
    await page.locator(".wizard-step").nth(1).click();
    await page.waitForSelector(".step-header", { timeout: 5000 });

    // Should have actors section
    const actorsSection = page.locator(".step-section").filter({ hasText: /Actors|Persons/ });
    await expect(actorsSection).toBeVisible();

    // Should have systems section
    const systemsSection = page.locator('.step-section:has(h3:has-text("Systems"))');
    await expect(systemsSection).toBeVisible();
  });

  test("ContainersStep shows when system exists", async ({ page }) => {
    // Navigate to Containers step
    await page.locator(".wizard-step").nth(2).click();

    // Should show containers content or placeholder
    const stepContent = page.locator(".wizard-main");
    await expect(stepContent).toBeVisible();
  });

  test("step completion indicators update", async ({ page }) => {
    // Goals step should be complete (demo has data)
    const goalsStep = page.locator(".wizard-step").nth(0);

    // Check if it has complete class or checkmark
    const isComplete = (await goalsStep.locator(".wizard-step-indicator svg").count()) > 0;
    expect(isComplete).toBeTruthy();
  });

  test("progress bar reflects completion", async ({ page }) => {
    await page.locator(".wizard-step").nth(1).click();
    const progressFill = page.locator(".wizard-progress-fill");

    // Progress should be > 0% since demo has data
    const width = await progressFill.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.width) || 0;
    });
    expect(width).toBeGreaterThan(0);
  });
});
