import { test, expect } from "@playwright/test";

test.describe("Examples Loading", () => {
  test("open menu and load first example", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
    const examplesButton = page.locator('button[aria-label="Examples"]');
    await examplesButton.click();
    const firstExample = page.locator(".example-item").first();
    await firstExample.waitFor({ timeout: 10000 });
    await firstExample.click();
    await page.waitForSelector(".react-flow", { timeout: 30000 });
    // Check for diagram content (SVG elements)
    await page.waitForSelector(".react-flow svg", { timeout: 10000 });
    const diagramContent = await page.locator(".react-flow svg").count();
    expect(diagramContent).toBeGreaterThan(0);
  });
});
