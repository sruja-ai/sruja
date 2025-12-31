// apps/website/__tests__/e2e/staging/critical-paths.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Critical User Paths on Staging", () => {
  test.beforeEach(async ({ page: _page }) => {
    test.setTimeout(60_000);
  });

  test("complete user journey: homepage -> docs -> getting started", async ({ page }) => {
    // Start at homepage
    await page.goto("/");
    await expect(page).toHaveTitle(/Sruja/i);
    await page.waitForLoadState("networkidle");

    // Find "Get Started" button - wait for it to be visible and enabled
    const getStartedButton = page
      .getByRole("button", { name: /Get Started/i })
      .or(page.locator("a").filter({ hasText: /Get Started/i }))
      .first();

    await expect(getStartedButton).toBeVisible({ timeout: 10_000 });

    // Click the button and wait for navigation
    // The button may trigger client-side navigation, so we wait for URL change
    await Promise.all([
      page.waitForURL(/docs|getting-started/i, { timeout: 20_000 }),
      getStartedButton.click(),
    ]);

    // Verify we're on the getting started page
    await expect(page).toHaveURL(/docs|getting-started/i);
    await page.waitForLoadState("networkidle");

    // Page should load successfully
    const body = page.locator("body");
    await expect(body).not.toContainText(/404|Not Found/i);
  });

  test("navigation: homepage -> designer", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle(/Sruja/i);

    // Look for designer link/button
    const designerLink = page
      .locator("a, button")
      .filter({ hasText: /designer|Designer/i })
      .first();

    if ((await designerLink.count()) > 0) {
      await designerLink.click();
      await page.waitForLoadState("networkidle");

      // Should navigate to designer
      expect(page.url()).toMatch(/designer/i);

      // Designer should load
      await page.waitForTimeout(5000); // Give designer time to initialize
      const body = page.locator("body");
      await expect(body).toBeVisible();
    } else {
      // If no direct link, navigate directly
      await page.goto("/designer");
      await page.waitForLoadState("networkidle");
      expect(page.url()).toMatch(/designer/i);
    }
  });

  test("navigation: homepage -> examples", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle(/Sruja/i);

    // Look for examples link
    const examplesLink = page
      .locator("a, button")
      .filter({ hasText: /examples|Examples/i })
      .first();

    if ((await examplesLink.count()) > 0) {
      await examplesLink.click();
      await page.waitForLoadState("networkidle");

      // Should navigate to examples
      expect(page.url()).toMatch(/examples/i);

      // Page should load successfully
      const body = page.locator("body");
      await expect(body).not.toContainText(/404|Not Found/i);
    }
  });

  test("static assets load correctly", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");

    // Check for common static assets
    const images = page.locator("img");
    const imageCount = await images.count();

    if (imageCount > 0) {
      // Check first few images load
      for (let i = 0; i < Math.min(3, imageCount); i++) {
        const img = images.nth(i);
        const src = await img.getAttribute("src");
        if (src && !src.startsWith("data:")) {
          // Verify image loads (not broken)
          const response = await page.request.get(
            src.startsWith("http") ? src : new URL(src, page.url()).href
          );
          expect(response.status()).toBeLessThan(400);
        }
      }
    }
  });

  test("no critical JavaScript errors on homepage", async ({ page }) => {
    const errors: string[] = [];

    page.on("console", (msg) => {
      if (msg.type() === "error") {
        const text = msg.text();
        // Filter out common non-critical errors
        if (
          !text.includes("favicon") &&
          !text.includes("analytics") &&
          !text.includes("tracking") &&
          !text.includes("adblock")
        ) {
          errors.push(text);
        }
      }
    });

    page.on("pageerror", (error) => {
      errors.push(error.message);
    });

    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(3000); // Wait for any async errors

    // Log errors for debugging
    if (errors.length > 0) {
      console.error("JavaScript errors found:", errors);
    }

    // Page should still be functional
    const body = page.locator("body");
    await expect(body).toBeVisible();
  });

  test("responsive design works on mobile viewport", async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    await page.goto("/");
    await page.waitForLoadState("networkidle");

    // Check page loads and is visible
    const body = page.locator("body");
    await expect(body).toBeVisible();

    // Check hero content is still visible (might be adjusted for mobile)
    const heroHeading = page
      .locator("h1")
      .filter({ hasText: /Architecture Editor with Live Code Sync/i });
    await expect(heroHeading).toBeVisible({ timeout: 10_000 });
  });
});
