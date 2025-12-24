// apps/designer/tests/dsl-sync.spec.ts
// Test for DSL sync issue when switching tabs
import { test, expect } from "@playwright/test";

test.describe("DSL Sync on Tab Switch", () => {
  test("DSL remains visible after switching tabs", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app-container, .drop-zone", { timeout: 30000 });

    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Go to Code tab first time
    await page.locator('button.view-tab:has-text("Code")').click();
    await page.waitForSelector(".code-panel-container", { timeout: 10000 });
    await page.waitForSelector(".dsl-panel", { timeout: 5000 });

    // Verify DSL content is visible
    const editor = page.locator(".monaco-editor, .dsl-panel-content");
    const initialContent = await editor
      .first()
      .textContent()
      .catch(() => "");
    expect(initialContent).toBeTruthy();
    expect(initialContent?.trim().length || 0).toBeGreaterThan(0);

    // Switch to Diagram tab
    await page.locator('button.view-tab:has-text("Diagram")').click();
    await page.waitForSelector(".react-flow", { timeout: 10000 });
    await page.waitForTimeout(500); // Allow component to unmount

    // Switch back to Code tab
    await page.locator('button.view-tab:has-text("Code")').click();
    await page.waitForSelector(".code-panel-container", { timeout: 10000 });
    await page.waitForSelector(".dsl-panel", { timeout: 5000 });

    // Verify DSL content is still visible (should not be blank)
    const editorAfterSwitch = page.locator(".monaco-editor, .dsl-panel-content");
    await expect(editorAfterSwitch.first()).toBeVisible();

    // Wait a bit for content to load
    await page.waitForTimeout(1000);

    const contentAfterSwitch = await editorAfterSwitch
      .first()
      .textContent()
      .catch(() => "");
    expect(contentAfterSwitch).toBeTruthy();
    expect(contentAfterSwitch?.trim().length || 0).toBeGreaterThan(0);

    // Content should be the same (or at least not empty)
    if (initialContent && contentAfterSwitch) {
      // Either same content or both have content
      expect(contentAfterSwitch.trim().length).toBeGreaterThan(0);
    }
  });

  test("DSL syncs correctly when loading example", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app-container, .drop-zone", { timeout: 30000 });

    // Load example from examples menu
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await examplesButton.waitFor({ timeout: 10000 });
    await examplesButton.click();

    const firstExample = page.locator(".example-item").first();
    await firstExample.waitFor({ timeout: 10000 });
    await firstExample.click();

    // Wait for diagram to load
    await page.waitForSelector(".react-flow", { timeout: 30000 });

    // Go to Code tab
    await page.locator('button.view-tab:has-text("Code")').click();
    await page.waitForSelector(".dsl-panel", { timeout: 10000 });

    // Verify DSL is visible
    const editor = page.locator(".monaco-editor, .dsl-panel-content");
    await expect(editor.first()).toBeVisible();

    await page.waitForTimeout(1000);
    const content = await editor
      .first()
      .textContent()
      .catch(() => "");
    expect(content).toBeTruthy();
    expect(content?.trim().length || 0).toBeGreaterThan(0);

    // Switch away and back
    await page.locator('button.view-tab:has-text("Diagram")').click();
    await page.waitForTimeout(500);
    await page.locator('button.view-tab:has-text("Code")').click();
    await page.waitForSelector(".dsl-panel", { timeout: 10000 });
    await page.waitForTimeout(1000);

    // Verify DSL is still visible
    const contentAfterSwitch = await editor
      .first()
      .textContent()
      .catch(() => "");
    expect(contentAfterSwitch).toBeTruthy();
    expect(contentAfterSwitch?.trim().length || 0).toBeGreaterThan(0);
  });
});
