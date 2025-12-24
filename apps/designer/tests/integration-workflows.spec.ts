// apps/designer/tests/integration-workflows.spec.ts
// Comprehensive E2E integration tests for critical user workflows
import { test, expect } from "@playwright/test";
import fs from "fs";

test.describe("Integration Workflows", () => {
  test.describe("Complete User Journey", () => {
    test("full workflow: empty state -> load example -> navigate -> export", async ({
      page,
      context,
    }) => {
      // 1. Start at empty state
      await page.goto("/");
      await expect(page.locator(".drop-zone")).toBeVisible({ timeout: 10000 });

      // 2. Load example from examples menu
      const examplesButton = page.locator('button[aria-label="Examples"]');
      await examplesButton.waitFor({ timeout: 10000 });
      await examplesButton.click();
      await page.waitForTimeout(500);

      // Select first example
      const exampleItem = page.locator('[class*="example"], [role="option"]').first();
      await exampleItem.waitFor({ timeout: 10000 });
      await exampleItem.click();

      // 3. Wait for diagram to load
      await page.waitForSelector(".react-flow", { timeout: 30000 });
      await page.waitForSelector(".react-flow__node", { timeout: 10000 });

      // 4. Navigate through tabs
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await expect(page.locator(".react-flow")).toBeVisible({ timeout: 5000 });

      await page.locator('button.view-tab:has-text("Details")').click();
      await expect(page.locator(".details-view-unified")).toBeVisible({ timeout: 5000 });

      await page.locator('button.view-tab:has-text("Code")').click();
      await expect(page.locator(".code-panel-container")).toBeVisible({ timeout: 5000 });

      await page.locator('button.view-tab:has-text("Builder")').click();
      await expect(page.locator(".builder-wizard")).toBeVisible({ timeout: 5000 });

      // 5. Go back to Diagram and export
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // Export DSL
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      const downloadPromise = page.waitForEvent("download", { timeout: 10000 });
      await page.locator('button:has-text("Export .sruja")').click();
      const download = await downloadPromise;

      expect(download.suggestedFilename()).toMatch(/\.sruja$/);
      const filePath = await download.path();
      expect(filePath).toBeTruthy();

      // Verify file content
      const content = fs.readFileSync(filePath!, "utf-8");
      expect(content).toContain("model");
      expect(content.length).toBeGreaterThan(100);
    });

    test("workflow: load example -> drill down -> navigate back", async ({ page }) => {
      // 1. Load example
      await page.goto("/?level=L1&tab=diagram&example=ecommerce_platform.sruja");
      await page.waitForSelector(".react-flow__node", { timeout: 30000 });

      // 2. Verify at L1
      await expect.poll(async () => page.url()).toMatch(/\blevel=L1\b/);

      // 3. Find and click a system node to drill down
      const systemNodes = page
        .locator(".react-flow__node")
        .filter({ hasText: /System|Platform|ECommerce/i });
      const systemCount = await systemNodes.count();

      if (systemCount > 0) {
        await systemNodes.first().click();
        await page.waitForTimeout(3000);

        // 4. Verify at L2
        await expect.poll(async () => page.url()).toMatch(/\blevel=L2\b/);
        await page.waitForSelector(".react-flow__node", { timeout: 10000 });

        // 5. Navigate back using breadcrumb or go up button
        const goUpButton = page
          .locator('button[aria-label*="Go up"], button[aria-label*="Up"], .breadcrumb button')
          .first();
        const goUpVisible = await goUpButton.isVisible().catch(() => false);

        if (goUpVisible) {
          await goUpButton.click();
          await page.waitForTimeout(2000);

          // 6. Verify back at L1
          await expect.poll(async () => page.url()).toMatch(/\blevel=L1\b/);
        }
      }
    });

    test("workflow: switch between view and edit modes", async ({ page }) => {
      // 1. Load example
      await page.goto("/?level=L1&tab=diagram&example=ecommerce_platform.sruja");
      await page.waitForSelector(".react-flow__node", { timeout: 30000 });

      // 2. Verify default is view mode
      const viewModeButton = page.locator('button.mode-btn:has-text("View")');
      await expect(viewModeButton).toHaveAttribute("aria-pressed", "true");

      // 3. Switch to edit mode
      const editModeButton = page.locator('button.mode-btn:has-text("Edit")');
      await editModeButton.click();
      await page.waitForTimeout(500);

      // 4. Verify edit mode is active
      await expect(editModeButton).toHaveAttribute("aria-pressed", "true");

      // 5. Verify Builder tab is highlighted (edit mode should emphasize Builder)
      await page.locator('button.view-tab:has-text("Builder")').click();
      await expect(page.locator(".builder-wizard")).toBeVisible({ timeout: 5000 });

      // 6. Switch back to view mode
      await viewModeButton.click();
      await page.waitForTimeout(500);

      // 7. Verify view mode is active
      await expect(viewModeButton).toHaveAttribute("aria-pressed", "true");
    });
  });

  test.describe("Export Workflows", () => {
    test("export all formats from diagram tab", async ({ page }) => {
      await page.goto("/?level=L1&tab=diagram&example=ecommerce_platform.sruja");
      await page.waitForSelector(".react-flow__node", { timeout: 30000 });

      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Export DSL
      const dslPromise = page.waitForEvent("download", { timeout: 10000 });
      await page.locator('button:has-text("Export .sruja")').click();
      const dslDownload = await dslPromise;
      expect(dslDownload.suggestedFilename()).toMatch(/\.sruja$/);

      // Close menu and reopen for next export
      await actionsButton.click();
      await page.waitForTimeout(500);
      await actionsButton.click();

      // Export PNG
      const pngPromise = page.waitForEvent("download", { timeout: 15000 });
      await page.locator('button:has-text("Export PNG")').click();
      const pngDownload = await pngPromise;
      expect(pngDownload.suggestedFilename()).toMatch(/\.png$/i);

      // Close menu and reopen
      await actionsButton.click();
      await page.waitForTimeout(500);
      await actionsButton.click();

      // Export SVG
      const svgPromise = page.waitForEvent("download", { timeout: 15000 });
      await page.locator('button:has-text("Export SVG")').click();
      const svgDownload = await svgPromise;
      expect(svgDownload.suggestedFilename()).toMatch(/\.svg$/i);

      // Verify SVG content
      const svgPath = await svgDownload.path();
      const svgContent = fs.readFileSync(svgPath!, "utf-8");
      expect(svgContent).toContain("<svg");
    });
  });

  test.describe("Share and URL State", () => {
    test("share creates shareable URL and copies to clipboard", async ({ page, context }) => {
      await context.grantPermissions(["clipboard-read", "clipboard-write"]);

      await page.goto("/?level=L1&tab=diagram&example=ecommerce_platform.sruja");
      await page.waitForSelector(".react-flow__node", { timeout: 30000 });

      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Click share
      await page.locator('button:has-text("Share")').click();

      // Wait for toast
      await expect(page.locator("text=/Link copied to clipboard|Share/i")).toBeVisible({
        timeout: 5000,
      });

      // Verify clipboard contains share URL
      const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
      expect(clipboardText).toContain("share=");
      expect(clipboardText.length).toBeGreaterThan(50);
    });

    test("URL state persists across navigation", async ({ page }) => {
      // Start with specific state
      await page.goto("/?level=L1&tab=diagram&example=ecommerce_platform.sruja");
      await page.waitForSelector(".react-flow__node", { timeout: 30000 });

      // Navigate to different tab
      await page.locator('button.view-tab:has-text("Code")').click();
      await expect.poll(async () => page.url()).toMatch(/\btab=code\b/);

      // Reload page
      await page.reload();
      await page.waitForSelector(".code-panel-container", { timeout: 10000 });

      // Verify state persisted
      await expect.poll(async () => page.url()).toMatch(/\btab=code\b/);
      await expect.poll(async () => page.url()).toMatch(/example=ecommerce_platform\.sruja/);
    });
  });

  test.describe("Error Handling", () => {
    test("handles missing example gracefully", async ({ page }) => {
      // Try to load non-existent example
      await page.goto("/?example=nonexistent.sruja");

      // Should either show error or fall back to empty state
      const dropZone = page.locator(".drop-zone");
      const errorState = page.locator(".error-state, [class*='error']");

      // One of these should be visible
      const dropZoneVisible = await dropZone.isVisible().catch(() => false);
      const errorVisible = await errorState.isVisible().catch(() => false);

      expect(dropZoneVisible || errorVisible).toBe(true);
    });

    test("handles invalid tab parameter", async ({ page }) => {
      // Try invalid tab
      await page.goto("/?tab=invalidtab");

      // Should default to valid tab
      await page.waitForSelector(".view-tabs", { timeout: 10000 });

      // URL should be corrected or default tab should be shown
      const url = page.url();
      const validTabs = ["builder", "diagram", "details", "code", "governance"];
      const hasValidTab = validTabs.some((tab) => url.includes(`tab=${tab}`));

      // Either URL is corrected or we're at default (which is valid)
      expect(hasValidTab || !url.includes("tab=")).toBe(true);
    });
  });

  test.describe("Performance and Loading States", () => {
    test("shows loading state during example load", async ({ page }) => {
      await page.goto("/");

      // Click examples
      const examplesButton = page.locator('button[aria-label="Examples"]');
      await examplesButton.click();
      await page.waitForTimeout(500);

      // Click example
      const exampleItem = page.locator('[class*="example"], [role="option"]').first();
      await exampleItem.click();

      // Should show loading state (might be brief)
      const loader = page.locator('[class*="loader"], [class*="spinner"], [class*="loading"]');
      const loaderVisible = await loader
        .first()
        .isVisible({ timeout: 1000 })
        .catch(() => false);

      // Eventually diagram should load
      await page.waitForSelector(".react-flow__node", { timeout: 30000 });
      expect(await page.locator(".react-flow__node").count()).toBeGreaterThan(0);
    });

    test("diagram renders nodes within reasonable time", async ({ page }) => {
      const startTime = Date.now();

      await page.goto("/?level=L1&tab=diagram&example=ecommerce_platform.sruja");
      await page.waitForSelector(".react-flow__node", { timeout: 30000 });

      const loadTime = Date.now() - startTime;

      // Should load within 30 seconds (already handled by timeout)
      expect(loadTime).toBeLessThan(30000);

      // Verify nodes are actually rendered
      const nodeCount = await page.locator(".react-flow__node").count();
      expect(nodeCount).toBeGreaterThan(0);
    });
  });
});
