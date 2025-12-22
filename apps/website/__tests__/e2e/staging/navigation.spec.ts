// apps/website/__tests__/e2e/staging/navigation.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Staging Website Navigation", () => {
  test.beforeEach(async ({ page }) => {
    // Set a longer timeout for staging site
    test.setTimeout(60_000);
  });

  test("homepage loads and displays hero content", async ({ page }) => {
    await page.goto("/");

    // Check page title
    await expect(page).toHaveTitle(/Sruja/i);

    // Check hero heading
    const heroHeading = page.locator("h1").filter({ hasText: /Build Better Software Systems/i });
    await expect(heroHeading).toBeVisible({ timeout: 10_000 });

    // Check hero description contains "Sruja"
    const heroDescription = page.locator(".hero p").first();
    await expect(heroDescription).toContainText(/Sruja/i);

    // Check for audience cards (should be 4 cards in 2 columns)
    const audienceCards = page.locator(".audience-card");
    await expect(audienceCards).toHaveCount(4);

    // Check for action buttons
    const getStartedButton = page
      .locator("button, a")
      .filter({ hasText: /Get Started/i })
      .first();
    await expect(getStartedButton).toBeVisible();
  });

  test("navigation links work correctly", async ({ page }) => {
    await page.goto("/");

    // Test "Get Started" link
    const getStartedLink = page
      .locator("a, button")
      .filter({ hasText: /Get Started/i })
      .first();
    if ((await getStartedLink.count()) > 0) {
      const href = await getStartedLink.getAttribute("href");
      if (href && !href.startsWith("#")) {
        await getStartedLink.click();
        await page.waitForLoadState("networkidle");
        // Should navigate to docs/getting-started or similar
        expect(page.url()).toMatch(/docs|getting-started/i);
      }
    }
  });

  test("docs page is accessible", async ({ page }) => {
    await page.goto("/docs");

    await expect(page).toHaveTitle(/Sruja/i);
    await page.waitForLoadState("networkidle");

    // Check page loaded successfully (not 404)
    const body = page.locator("body");
    await expect(body).not.toContainText(/404|Not Found/i);
  });

  test("designer page is accessible", async ({ page }) => {
    await page.goto("/designer");

    await expect(page).toHaveTitle(/Sruja/i);
    await page.waitForLoadState("networkidle");

    // Check page loaded successfully
    const body = page.locator("body");
    await expect(body).not.toContainText(/404|Not Found/i);
  });

  test("courses page is accessible", async ({ page }) => {
    await page.goto("/courses");

    await expect(page).toHaveTitle(/Sruja/i);
    await page.waitForLoadState("networkidle");

    // Check page loaded successfully
    const body = page.locator("body");
    await expect(body).not.toContainText(/404|Not Found/i);
  });

  test("tutorials page is accessible", async ({ page }) => {
    await page.goto("/tutorials");

    await expect(page).toHaveTitle(/Sruja/i);
    await page.waitForLoadState("networkidle");

    // Check page loaded successfully
    const body = page.locator("body");
    await expect(body).not.toContainText(/404|Not Found/i);
  });

  test("examples page is accessible", async ({ page }) => {
    await page.goto("/docs/examples");

    await expect(page).toHaveTitle(/Sruja/i);
    await page.waitForLoadState("networkidle");

    // Check page loaded successfully
    const body = page.locator("body");
    await expect(body).not.toContainText(/404|Not Found/i);
  });

  test("viewer page loads", async ({ page }) => {
    await page.goto("/viewer");

    await expect(page).toHaveTitle(/Sruja/i);
    await page.waitForLoadState("networkidle");

    // Check viewer interface loads
    const viewerContainer = page
      .locator('[data-testid="viewer"]')
      .or(page.locator(".viewer-container"))
      .or(page.locator('textarea, [role="textbox"]'))
      .first();
    await expect(viewerContainer).toBeVisible({ timeout: 15_000 });
  });
});
