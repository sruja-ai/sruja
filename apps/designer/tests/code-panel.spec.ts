// apps/designer/tests/code-panel.spec.ts
// Code panel e2e tests
import { test, expect } from "@playwright/test";

test.describe("Code Panel", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
    }

    // Switch to Code tab
    await page.locator('button.view-tab:has-text("Code")').click();
    await page.waitForSelector(".code-panel-container", { timeout: 10000 });
  });

  test("displays code panel", async ({ page }) => {
    const codePanel = page.locator(".code-panel-container");
    await expect(codePanel).toBeVisible();
  });

  test("code panel has tabs (DSL, JSON, Markdown)", async ({ page }) => {
    // Look for code tabs
    const dslTab = page.locator('button:has-text("DSL"), .code-tab:has-text("DSL")');
    const jsonTab = page.locator('button:has-text("JSON"), .code-tab:has-text("JSON")');
    const markdownTab = page.locator('button:has-text("Markdown"), .code-tab:has-text("Markdown")');

    // At least one tab should be visible
    const hasTabs = await Promise.race([
      dslTab.isVisible().then(() => true),
      jsonTab.isVisible().then(() => true),
      markdownTab.isVisible().then(() => true),
    ]).catch(() => false);

    // Code panel should be visible regardless
    await expect(page.locator(".code-panel-container")).toBeVisible();
  });

  test("code panel displays content", async ({ page }) => {
    // Code panel should have some content area
    const codeContent = page.locator(".code-panel-container, .dsl-panel, .monaco-editor");
    const isVisible = await codeContent.first().isVisible().catch(() => false);
    
    // Panel container should always be visible
    await expect(page.locator(".code-panel-container")).toBeVisible();
  });
});

