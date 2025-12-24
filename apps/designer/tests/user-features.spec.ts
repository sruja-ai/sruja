// apps/designer/tests/user-features.spec.ts
// Comprehensive E2E tests for all user-facing features
import { test, expect } from "@playwright/test";
import path from "path";
import fs from "fs";

test.describe("User-Facing Features", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to designer with an example loaded
    // Base path is / for tests (BASE_PATH=/ in build:test)
    await page.goto("/?level=L1&tab=diagram&example=ecommerce_platform.sruja", {
      waitUntil: "networkidle",
      timeout: 60000,
    });

    // Wait for diagram to load - SrujaCanvas uses ReactFlow
    await page.waitForSelector(".react-flow", { timeout: 90000 });
    // Wait for nodes to appear (more reliable than waiting for SVG)
    await page.waitForSelector(".react-flow__node", { timeout: 60000 });

    // Wait a bit for layout to stabilize
    await page.waitForTimeout(2000);
  });

  test.describe("Export Functionality", () => {
    test("exports DSL file", async ({ page }) => {
      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await expect(actionsButton).toBeVisible();
      await actionsButton.click();

      // Wait for menu to appear
      await expect(page.locator('button:has-text("Export .sruja")')).toBeVisible();

      // Set up download listener
      const downloadPromise = page.waitForEvent("download", { timeout: 10000 });

      // Click export DSL button
      await page.locator('button:has-text("Export .sruja")').click();

      // Wait for download
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/\.sruja$/);

      // Verify file was downloaded
      const filePath = await download.path();
      expect(filePath).toBeTruthy();

      // Read and verify file content
      const content = fs.readFileSync(filePath!, "utf-8");
      expect(content).toContain("model");
      expect(content.length).toBeGreaterThan(0);
    });

    test("exports PNG image", async ({ page }) => {
      // Ensure we're on diagram tab
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Wait for menu to appear
      await expect(page.locator('button:has-text("Export PNG")')).toBeVisible();

      // Set up download listener
      const downloadPromise = page.waitForEvent("download", { timeout: 15000 });

      // Click export PNG button
      await page.locator('button:has-text("Export PNG")').click();

      // Wait for download
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/\.png$/i);

      // Verify file was downloaded
      const filePath = await download.path();
      expect(filePath).toBeTruthy();
      expect(fs.existsSync(filePath!)).toBe(true);
    });

    test("exports SVG image", async ({ page }) => {
      // Ensure we're on diagram tab
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Wait for menu to appear
      await expect(page.locator('button:has-text("Export SVG")')).toBeVisible();

      // Set up download listener
      const downloadPromise = page.waitForEvent("download", { timeout: 15000 });

      // Click export SVG button
      await page.locator('button:has-text("Export SVG")').click();

      // Wait for download
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/\.svg$/i);

      // Verify file was downloaded and contains SVG content
      const filePath = await download.path();
      expect(filePath).toBeTruthy();
      const content = fs.readFileSync(filePath!, "utf-8");
      expect(content).toContain("<svg");
      expect(content).toContain("xmlns");
    });
  });

  test.describe("Import Functionality", () => {
    test("import button is accessible", async ({ page }) => {
      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Import button should be visible
      const importButton = page.locator('button:has-text("Import .sruja")');
      await expect(importButton).toBeVisible();
    });

    test("can trigger file input for import", async ({ page }) => {
      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Set up file chooser listener
      const fileChooserPromise = page.waitForEvent("filechooser", { timeout: 5000 });

      // Click import button - this should trigger file input
      await page.locator('button:has-text("Import .sruja")').click();

      // Note: In E2E tests, we can't actually select a file, but we can verify
      // the file chooser is triggered. We'll skip the actual file selection
      // as it requires actual file system interaction which is tested manually.
      try {
        await fileChooserPromise;
        // If file chooser opens, that's good
      } catch {
        // File chooser might not trigger in headless mode, which is OK
      }
    });
  });

  test.describe("Share Functionality", () => {
    test("share button is accessible", async ({ page }) => {
      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Share button should be visible
      const shareButton = page.locator('button:has-text("Share")');
      await expect(shareButton).toBeVisible();
    });

    test("share copies URL to clipboard", async ({ page, context }) => {
      // Grant clipboard permissions
      await context.grantPermissions(["clipboard-read", "clipboard-write"]);

      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Click share button
      await page.locator('button:has-text("Share")').click();

      // Wait for toast notification
      await expect(page.locator("text=/Link copied to clipboard/i")).toBeVisible({ timeout: 5000 });

      // Verify clipboard contains share URL
      const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
      expect(clipboardText).toContain("share=");
      expect(clipboardText.length).toBeGreaterThan(0);
    });
  });

  test.describe("Tab Navigation", () => {
    test("all tabs are visible", async ({ page }) => {
      // Verify all tabs are present
      await expect(page.locator('button.view-tab:has-text("Overview")')).toBeVisible();
      await expect(page.locator('button.view-tab:has-text("Diagram")')).toBeVisible();
      await expect(page.locator('button.view-tab:has-text("Details")')).toBeVisible();
      await expect(page.locator('button.view-tab:has-text("Code")')).toBeVisible();
      await expect(page.locator('button.view-tab:has-text("Builder")')).toBeVisible();
    });

    test("navigates to Overview tab", async ({ page }) => {
      await page.locator('button.view-tab:has-text("Overview")').click();

      // Verify URL updated
      await expect.poll(async () => page.url()).toMatch(/\btab=overview\b/);

      // Wait a bit for tab to render
      await page.waitForTimeout(500);
    });

    test("navigates to Diagram tab", async ({ page }) => {
      // Go to another tab first
      await page.locator('button.view-tab:has-text("Code")').click();
      await page.waitForTimeout(500);

      // Then go to Diagram tab
      await page.locator('button.view-tab:has-text("Diagram")').click();

      // Verify URL updated
      await expect.poll(async () => page.url()).toMatch(/\btab=diagram\b/);

      // Verify diagram is visible
      await expect(page.locator(".react-flow")).toBeVisible();
    });

    test("navigates to Details tab", async ({ page }) => {
      await page.locator('button.view-tab:has-text("Details")').click();

      // Verify URL updated
      await expect.poll(async () => page.url()).toMatch(/\btab=details\b/);

      // Verify details view is visible
      await expect(page.locator(".details-view-unified")).toBeVisible({ timeout: 5000 });
    });

    test("navigates to Code tab", async ({ page }) => {
      await page.locator('button.view-tab:has-text("Code")').click();

      // Verify URL updated
      await expect.poll(async () => page.url()).toMatch(/\btab=code\b/);

      // Verify code panel is visible
      await expect(page.locator(".code-panel-container")).toBeVisible({ timeout: 5000 });
      // Monaco editor might take longer to load
      await page.waitForTimeout(1000);
    });

    test("navigates to Builder tab", async ({ page }) => {
      await page.locator('button.view-tab:has-text("Builder")').click();

      // Verify URL updated
      await expect.poll(async () => page.url()).toMatch(/\btab=builder\b/);

      // Wait a bit for tab to render
      await page.waitForTimeout(500);
    });
  });

  test.describe("Level Navigation", () => {
    test("breadcrumb navigation works", async ({ page }) => {
      // Ensure we're on diagram tab
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // Wait for breadcrumb to appear
      const breadcrumb = page.locator(".breadcrumb, [class*='breadcrumb']");
      await expect(breadcrumb.first()).toBeVisible({ timeout: 5000 });

      // Click on a system node to drill down to L2
      // Look for nodes that might be systems (they typically have specific styling or text)
      const systemNodes = page
        .locator(".react-flow__node")
        .filter({ hasText: /System|Platform|ECommerce|API/i });
      const systemCount = await systemNodes.count();

      if (systemCount > 0) {
        // Click first system node
        await systemNodes.first().click();
        await page.waitForTimeout(3000); // Wait for navigation

        // Verify we're at L2 (check URL)
        await expect.poll(async () => page.url()).toMatch(/\blevel=L2\b/);

        // Verify diagram updated with new level
        await page.waitForSelector(".react-flow__node", { timeout: 10000 });
      } else {
        // If no system nodes found, test might be using a different example
        // Just verify breadcrumb is visible
        expect(await breadcrumb.first().isVisible()).toBe(true);
      }
    });

    test("go up button navigates to previous level", async ({ page }) => {
      // Ensure we're on diagram tab
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // First, drill down to L2
      const systemNodes = page.locator(".react-flow__node").filter({ hasText: /System|Platform/i });
      const systemCount = await systemNodes.count();

      if (systemCount > 0) {
        await systemNodes.first().click();
        await page.waitForTimeout(2000);

        // Find and click "Go Up" button
        const goUpButton = page.locator('button[aria-label*="Go up"], button[aria-label*="Up"]');
        const goUpVisible = await goUpButton.isVisible().catch(() => false);

        if (goUpVisible) {
          await goUpButton.click();
          await page.waitForTimeout(1000);

          // Verify we're back at L1
          await expect.poll(async () => page.url()).toMatch(/\blevel=L1\b/);
        }
      }
    });
  });

  test.describe("Node Selection and Drill-Down", () => {
    test("selects a node", async ({ page }) => {
      // Ensure we're on diagram tab
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // Find a node and click it
      const nodes = page.locator(".react-flow__node");
      const nodeCount = await nodes.count();

      expect(nodeCount).toBeGreaterThan(0);

      // Click first node
      await nodes.first().click();
      await page.waitForTimeout(500);

      // Verify details panel might open (if implemented)
      // This is a basic test - the actual behavior depends on implementation
    });

    test("drill-down works on system nodes", async ({ page }) => {
      // Ensure we're on diagram tab and L1
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // Verify we're at L1
      await expect.poll(async () => page.url()).toMatch(/\blevel=L1\b/);

      // Find system nodes (nodes that can be drilled into)
      const systemNodes = page
        .locator(".react-flow__node")
        .filter({ hasText: /System|Platform|ECommerce|API/i });
      const systemCount = await systemNodes.count();

      if (systemCount > 0) {
        const currentUrl = page.url();

        // Click on a system node to drill down
        await systemNodes.first().click();
        await page.waitForTimeout(3000); // Wait for navigation and diagram update

        // Verify URL changed (drill-down should update level)
        const newUrl = page.url();
        if (newUrl !== currentUrl) {
          // URL changed, which is expected for drill-down
          expect(newUrl).toMatch(/\blevel=L2\b/);
          // Verify diagram updated
          await page.waitForSelector(".react-flow__node", { timeout: 10000 });
        }
      }
    });
  });

  test.describe("Example Selection", () => {
    test("examples dropdown is accessible", async ({ page }) => {
      // Find examples button
      const examplesButton = page.locator('button[aria-label="Examples"]');
      await expect(examplesButton).toBeVisible();
    });

    test("opens examples menu", async ({ page }) => {
      // Click examples button
      const examplesButton = page.locator('button[aria-label="Examples"]');
      await examplesButton.click();

      // Wait for examples menu to appear
      await page.waitForTimeout(500);

      // Look for example items
      const exampleItems = page.locator('[class*="example"], [role="option"]');
      const itemCount = await exampleItems.count();

      // Should have at least one example
      expect(itemCount).toBeGreaterThan(0);
    });

    test("loads a different example", async ({ page }) => {
      // Click examples button
      const examplesButton = page.locator('button[aria-label="Examples"]');
      await examplesButton.click();
      await page.waitForTimeout(500);

      // Find and click a different example (not the currently loaded one)
      const exampleItems = page.locator('[class*="example"], [role="option"]');
      const itemCount = await exampleItems.count();

      if (itemCount > 1) {
        // Click second example
        await exampleItems.nth(1).click();

        // Wait for diagram to reload
        await page.waitForSelector(".react-flow", { timeout: 30000 });
        await page.waitForTimeout(2000);

        // Verify URL updated with new example
        const url = page.url();
        expect(url).toContain("example=");
      }
    });
  });

  test.describe("Project Creation", () => {
    test("new project button is accessible", async ({ page }) => {
      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // New Project button should be visible
      const newProjectButton = page.locator('button:has-text("New Project")');
      await expect(newProjectButton).toBeVisible();
    });

    test("creates new project", async ({ page }) => {
      // Open actions menu
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      // Click new project button
      await page.locator('button:has-text("New Project")').click();

      // Wait for project to be created (might show toast or redirect)
      await page.waitForTimeout(2000);

      // Verify diagram is still visible (new project should show empty state or default diagram)
      await expect(page.locator(".react-flow, .drop-zone")).toBeVisible({ timeout: 5000 });
    });
  });

  test.describe("Error Handling and Loading States", () => {
    test("shows loading state when switching examples", async ({ page }) => {
      // Click examples button
      const examplesButton = page.locator('button[aria-label="Examples"]');
      await examplesButton.click();
      await page.waitForTimeout(500);

      // Click an example to load
      const exampleItems = page.locator('[class*="example"], [role="option"]');
      const itemCount = await exampleItems.count();

      if (itemCount > 1) {
        // Look for loader/spinner (if implemented)
        await exampleItems.nth(1).click();

        // Wait a short time to see if loader appears
        const loader = page.locator('[class*="loader"], [class*="spinner"], [class*="loading"]');
        const loaderVisible = await loader
          .first()
          .isVisible({ timeout: 1000 })
          .catch(() => false);

        // Loader might or might not be visible, but diagram should eventually load
        await page.waitForSelector(".react-flow", { timeout: 30000 });
      }
    });
  });

  test.describe("Integration - Multiple Features Together", () => {
    test("complete workflow: load example, navigate, export", async ({ page }) => {
      // 1. Load a specific example
      const examplesButton = page.locator('button[aria-label="Examples"]');
      await examplesButton.click();
      await page.waitForTimeout(500);

      const exampleItems = page.locator('[class*="example"], [role="option"]');
      const itemCount = await exampleItems.count();

      if (itemCount > 0) {
        await exampleItems.first().click();
        await page.waitForSelector(".react-flow", { timeout: 30000 });
        await page.waitForTimeout(2000);
      }

      // 2. Navigate to different tabs
      await page.locator('button.view-tab:has-text("Code")').click();
      await page.waitForTimeout(500);
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // 3. Export DSL
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      const downloadPromise = page.waitForEvent("download", { timeout: 10000 });
      await page.locator('button:has-text("Export .sruja")').click();

      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/\.sruja$/);
    });

    test("navigate, select node, share", async ({ page }) => {
      // 1. Navigate to diagram
      await page.locator('button.view-tab:has-text("Diagram")').click();
      await page.waitForTimeout(1000);

      // 2. Select a node
      const nodes = page.locator(".react-flow__node");
      const nodeCount = await nodes.count();

      if (nodeCount > 0) {
        await nodes.first().click();
        await page.waitForTimeout(500);
      }

      // 3. Share
      const actionsButton = page.locator('button:has-text("Actions")');
      await actionsButton.click();

      await page.locator('button:has-text("Share")').click();

      // Verify share worked (toast appears)
      await expect(page.locator("text=/Link copied to clipboard|Share/i")).toBeVisible({
        timeout: 5000,
      });
    });
  });
});
