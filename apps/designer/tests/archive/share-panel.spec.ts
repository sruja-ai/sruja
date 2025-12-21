import { test, expect } from "@playwright/test";

test.describe("Share Panel", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Load demo first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    }

    // Switch to Builder tab
    const builderTab = page.locator('button.view-tab:has-text("Builder")');
    await builderTab.click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });
    // Ensure preview sidebar is visible
    const toggleBtn = page.locator(".preview-toggle-btn");
    await toggleBtn.click();
  });

  test("opens SharePanel modal", async ({ page }) => {
    const shareBtn = page.locator(".share-btn");
    await shareBtn.click();

    const modal = page.locator(".share-panel-modal");
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("Share & Export");
  });

  test("displays shareable URL", async ({ page }) => {
    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });

    const urlInput = page.locator(".share-url-row input");
    await expect(urlInput).toBeVisible();

    const url = await urlInput.inputValue();
    expect(url).toContain("?share=");
  });

  test("copies URL to clipboard", async ({ page }) => {
    // Mock clipboard in page context
    await page.evaluate(() => {
      (window as any).__COPIED__ = "";
      const orig = navigator.clipboard.writeText;
      (navigator.clipboard as any).writeText = async (text: string) => {
        (window as any).__COPIED__ = text;
        if (orig) return orig.call(navigator.clipboard, text);
      };
    });

    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });

    const copyBtn = page.locator(".share-copy-btn");
    await copyBtn.click();

    // Button should show "Copied!"
    await expect(copyBtn).toContainText("Copied!");

    // Verify clipboard content
    const copied = await page.evaluate(() => (window as any).__COPIED__);
    expect(copied).toContain("?share=");
  });

  test("shows export options for DSL, JSON, Markdown", async ({ page }) => {
    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });

    const exportGrid = page.locator(".export-grid");
    await expect(exportGrid).toBeVisible();

    // Check for 3 export cards
    const cards = page.locator(".export-card");
    await expect(cards).toHaveCount(3);

    await expect(page.locator(".export-card")).toContainText(["Sruja DSL", "JSON", "Markdown"]);
  });

  test("downloads DSL file", async ({ page }) => {
    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });

    const dslCard = page.locator(".export-card").filter({ hasText: "Sruja DSL" });
    const downloadBtn = dslCard.locator('button[aria-label="Download DSL file"]');

    const [download] = await Promise.all([page.waitForEvent("download"), downloadBtn.click()]);

    expect(download.suggestedFilename()).toMatch(/\.sruja$/);
  });

  test("downloads JSON file", async ({ page }) => {
    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });

    const jsonCard = page.locator(".export-card").filter({ hasText: "JSON" });
    const downloadBtn = jsonCard.locator('button[aria-label="Download JSON file"]');

    const [download] = await Promise.all([page.waitForEvent("download"), downloadBtn.click()]);

    expect(download.suggestedFilename()).toMatch(/\.json$/);
  });

  test("closes modal on overlay click", async ({ page }) => {
    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });

    // Click overlay (outside modal)
    await page.locator(".share-panel-overlay").click({ position: { x: 10, y: 10 } });

    await expect(page.locator(".share-panel-modal")).not.toBeVisible();
  });

  test("closes modal on X button", async ({ page }) => {
    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });

    await page.locator(".share-close-btn").click();

    await expect(page.locator(".share-panel-modal")).not.toBeVisible();
  });
});

test.describe("Share URL Loading", () => {
  test("loads architecture from share URL parameter", async ({ page }) => {
    // First get a valid share URL
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    }

    // Go to builder and get share URL
    await page.locator('button.view-tab:has-text("Builder")').click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });
    const toggleBtn = page.locator(".preview-toggle-btn");
    await toggleBtn.click();
    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });

    const shareUrl = await page.locator(".share-url-row input").inputValue();
    await page.locator(".share-close-btn").click();

    // Navigate to the share URL
    await page.goto(shareUrl);
    await page.waitForSelector(".app", { timeout: 30000 });

    // Should load the diagram
    await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    await page.waitForSelector(".likec4-diagram-container svg, .react-flow svg", { timeout: 10000 });
    const diagramContent = await page.locator(".likec4-diagram-container svg, .react-flow svg").count();
    expect(diagramContent).toBeGreaterThan(0);
  });
});
