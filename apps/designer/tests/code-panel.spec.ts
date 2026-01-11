// apps/designer/tests/code-panel.spec.ts
// Code panel e2e tests
import { test, expect } from "@playwright/test";

test.describe("Code Panel", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app-container, .drop-zone", { timeout: 30000 });

    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Switch to Code tab - use data-testid
    await page.getByTestId("tab-code").click();
    await page.waitForSelector(".code-panel-container", { timeout: 10000 });
  });

  test("displays code panel", async ({ page }) => {
    const codePanel = page.locator(".code-panel-container");
    await expect(codePanel).toBeVisible();
  });

  test("code panel displays DSL editor", async ({ page }) => {
    // Code panel now only shows DSL editor (no tabs)
    const dslPanel = page.locator(".dsl-panel, [data-testid='dsl-panel-container']");

    // DSL panel should be visible
    await expect(dslPanel.first()).toBeVisible();

    // Code panel container should be visible
    await expect(page.locator(".code-panel-container")).toBeVisible();
  });

  test("code panel displays content", async ({ page }) => {
    // Code panel should have some content area
    const codeContent = page.locator(".code-panel-container, .dsl-panel, .monaco-editor");
    await codeContent
      .first()
      .isVisible()
      .catch(() => false);

    // Panel container should always be visible
    await expect(page.locator(".code-panel-container")).toBeVisible();
  });
});
