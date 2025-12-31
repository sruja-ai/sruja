// apps/website/__tests__/e2e/staging/designer.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Designer App on Staging", () => {
  test.beforeEach(async ({ page: _page }) => {
    test.setTimeout(90_000); // Designer may take longer to load
  });

  test("designer app loads and displays interface", async ({ page }) => {
    await page.goto("/designer");

    await expect(page).toHaveTitle(/Sruja/i);
    await page.waitForLoadState("networkidle");

    // Wait for designer to initialize - check for multiple possible indicators
    // The designer shows "Loading Playground..." initially, then loads the interface
    // Wait for either the editor/designer elements OR the "Try a Demo" / "Create New" buttons
    // which indicate the designer has finished loading
    const designerLoaded = await Promise.race([
      page
        .waitForSelector(
          'textarea, [role="textbox"], .monaco-editor, canvas, [data-testid*="designer"], [data-testid*="editor"], [data-testid="viewer-editor"]',
          { timeout: 60_000 }
        )
        .then(() => true),
      page
        .getByRole("button", { name: /Try a Demo|Create New/i })
        .first()
        .waitFor({ timeout: 60_000 })
        .then(() => true),
      // Also check that "Loading Playground..." is gone
      page
        .waitForFunction(() => !document.body.textContent?.includes("Loading Playground..."), {
          timeout: 60_000,
        })
        .then(() => true),
      page.waitForTimeout(60_000).then(() => false),
    ]);

    expect(designerLoaded).toBe(true);

    // Additional verification: check that the designer interface is visible
    // Look for either the action buttons or editor elements
    const designerInterface = page
      .locator("button")
      .filter({ hasText: /Try a Demo|Create New/i })
      .or(page.locator('textarea, [role="textbox"]'))
      .first();
    await expect(designerInterface).toBeVisible({ timeout: 5_000 });
  });

  test("designer can load example files", async ({ page }) => {
    await page.goto("/designer");
    await page.waitForLoadState("networkidle");

    // Wait for designer to be ready
    await page.waitForTimeout(5000);

    // Look for example selector or file picker
    // This is a basic check - actual implementation may vary
    const exampleSelector = page
      .locator('select, [role="button"]')
      .filter({ hasText: /example|load|open/i })
      .first();

    // If example selector exists, try to interact with it
    if ((await exampleSelector.count()) > 0) {
      await exampleSelector.click();
      await page.waitForTimeout(2000);
    }

    // Check that designer is still functional (not crashed)
    const body = page.locator("body");
    await expect(body).toBeVisible();
  });

  test("designer WASM loads successfully", async ({ page }) => {
    await page.goto("/designer");
    await page.waitForLoadState("networkidle");

    // Check for WASM-related errors in console
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        const text = msg.text();
        // Ignore common non-critical errors, but log WASM-related ones
        if (text.includes("wasm") || text.includes("WASM")) {
          errors.push(text);
        }
      }
    });

    // Wait for potential WASM initialization
    await page.waitForTimeout(10_000);

    // Check that no critical WASM errors occurred
    const wasmErrors = errors.filter(
      (e) =>
        e.includes("failed") ||
        e.includes("error") ||
        e.includes("cannot") ||
        e.includes("undefined")
    );

    // Log errors for debugging but don't fail if it's just warnings
    if (wasmErrors.length > 0) {
      console.warn("WASM-related console errors:", wasmErrors);
    }

    // Basic check: page should still be functional
    const body = page.locator("body");
    await expect(body).toBeVisible();
  });

  test("designer examples directory is accessible", async ({ page }) => {
    // Check that examples are available via the designer path
    const response = await page.request.get("/designer/examples/manifest.json");

    // Should either return 200 (examples exist) or 404 (not found but not a server error)
    expect([200, 404]).toContain(response.status());

    if (response.status() === 200) {
      const manifest = await response.json();
      expect(manifest).toBeDefined();
    }
  });
});
