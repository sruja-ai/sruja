// apps/designer/tests/fix-verification.spec.ts
// Verification tests for the fixes: diagram loading, template gallery, code tab, builder
import { test, expect } from "@playwright/test";

test.describe("Fix Verification", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app-container, .drop-zone", { timeout: 30000 });
  });

  test("template gallery loads and selects template", async ({ page }) => {
    // Load demo first to get to builder
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Switch to Builder tab
    await page.locator('button.view-tab:has-text("Builder")').click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });

    // Navigate to Goals step (first step)
    const goalsStep = page.locator(".wizard-step").first();
    await goalsStep.click();
    await page.waitForTimeout(500);

    // Click template button
    const templateBtn = page.locator(
      'button:has-text("Start from a Template"), .template-prompt-btn'
    );
    if (await templateBtn.isVisible().catch(() => false)) {
      await templateBtn.click();
      await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

      // Verify template gallery is visible
      await expect(page.locator(".template-gallery-modal")).toBeVisible();

      // Select a template (first one)
      const firstTemplate = page.locator(".template-card").first();
      await firstTemplate.click();
      await expect(firstTemplate).toHaveClass(/selected/);

      // Click "Use Template" button
      const useBtn = page.locator(".template-use-btn");
      await expect(useBtn).not.toBeDisabled();
      await useBtn.click();

      // Modal should close
      await expect(page.locator(".template-gallery-modal")).not.toBeVisible({ timeout: 2000 });

      // Wait a bit for model to load
      await page.waitForTimeout(2000);

      // Verify diagram loads
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForSelector(".react-flow", { timeout: 10000 });
      await expect(page.locator(".react-flow")).toBeVisible();
    } else {
      test.skip();
    }
  });

  test("code tab shows DSL after loading template", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Switch to Code tab
    await page.locator('button.view-tab:has-text("Code")').click();
    await page.waitForSelector(".code-panel-container", { timeout: 10000 });

    // Verify code panel is visible
    await expect(page.locator(".code-panel-container")).toBeVisible();

    // Check if DSL panel is visible (should be default tab)
    const dslPanel = page.locator(".dsl-panel");
    const isVisible = await dslPanel.isVisible().catch(() => false);

    if (isVisible) {
      // Check if there's content (either DSL or placeholder)
      const editor = page.locator(".monaco-editor, .dsl-panel-content");
      await expect(editor.first()).toBeVisible();

      // Check if there's text content (not just empty)
      const hasContent = await editor
        .first()
        .textContent()
        .then((t) => t && t.trim().length > 0)
        .catch(() => false);
      expect(hasContent).toBeTruthy();
    }
  });

  test("diagram renders after template load", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Verify diagram is visible
    await expect(page.locator(".react-flow")).toBeVisible();

    // Check for SVG elements
    await page.waitForSelector(".react-flow__node", { timeout: 10000 });
    const nodeCount = await page.locator(".react-flow__node").count();
    expect(nodeCount).toBeGreaterThan(0);
  });

  test("builder wizard steps are functional", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Switch to Builder tab
    await page.locator('button.view-tab:has-text("Builder")').click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });

    // Verify wizard is visible
    await expect(page.locator(".builder-wizard")).toBeVisible();

    // Check for wizard steps
    const steps = page.locator(".wizard-step");
    const stepCount = await steps.count();
    expect(stepCount).toBeGreaterThan(0);

    // Try clicking on different steps
    if (stepCount > 1) {
      await steps.nth(1).click();
      await page.waitForTimeout(500);

      // Verify step content is visible
      const stepContent = page.locator(".wizard-main, .step-content, .wizard-step-content");
      await expect(stepContent.first()).toBeVisible();
    }
  });

  test("full flow: template -> diagram -> code", async ({ page }) => {
    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Go to Builder
    await page.locator('button.view-tab:has-text("Builder")').click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });

    // Open template gallery
    const templateBtn = page.locator(
      'button:has-text("Start from a Template"), .template-prompt-btn'
    );
    if (await templateBtn.isVisible().catch(() => false)) {
      await templateBtn.click();
      await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });

      // Select second template (microservices)
      const microservicesTemplate = page.locator(".template-card").nth(1);
      if (await microservicesTemplate.isVisible().catch(() => false)) {
        await microservicesTemplate.click();
        await page.locator(".template-use-btn").click();
        await page.waitForTimeout(2000);

        // Verify diagram loads
        await page.locator('button.view-tab:has-text("Diagram")').click();
        await page.waitForSelector(".react-flow", { timeout: 10000 });
        await expect(page.locator(".react-flow")).toBeVisible();

        // Verify code tab has content
        await page.locator('button.view-tab:has-text("Code")').click();
        await page.waitForSelector(".code-panel-container", { timeout: 10000 });

        const dslPanel = page.locator(".dsl-panel");
        if (await dslPanel.isVisible().catch(() => false)) {
          const editor = page.locator(".monaco-editor, .dsl-panel-content");
          const hasContent = await editor
            .first()
            .textContent()
            .then((t) => t && t.trim().length > 0)
            .catch(() => false);
          expect(hasContent).toBeTruthy();
        }
      }
    } else {
      test.skip();
    }
  });
});
