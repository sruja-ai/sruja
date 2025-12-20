import { test, expect } from "@playwright/test";

test.describe("Share & Export", () => {
  test("copy shareable URL writes to clipboard (mocked)", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Mock clipboard in page context
    await page.evaluate(() => {
      (window as any).__COPIED__ = "";
      const orig = navigator.clipboard.writeText;
      (navigator.clipboard as any).writeText = async (text: string) => {
        (window as any).__COPIED__ = text;
        if (orig) return orig.call(navigator.clipboard, text);
      };
    });

    // Ensure demo is loaded
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    }

    // Prefer Share Panel modal for reliable copy
    // Switch to Builder tab and open preview sidebar (contains Share button)
    const builderTab = page.locator('button.view-tab:has-text("Builder")');
    await builderTab.click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });
    const toggleBtn = page.locator(".preview-toggle-btn");
    await toggleBtn.click();

    // Open Share panel and click Copy
    await page.locator(".share-btn").click();
    await page.waitForSelector(".share-panel-modal", { timeout: 5000 });
    const copyBtn = page.locator(".share-copy-btn");
    await copyBtn.click();

    // Validate captured text
    await expect
      .poll(async () => page.evaluate(() => (window as any).__COPIED__ as string), {
        timeout: 10000,
      })
      .toMatch(/\bshare=/);
  });

  test("export downloads a .sruja file", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Ensure demo is loaded
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
    }

    // Open Actions menu and trigger export; assert download event
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      (async () => {
        const actionsBtn = page.locator('button[aria-label="Actions"]');
        await actionsBtn.click();
        await page.locator('button[aria-label="Export .sruja file"]').click();
      })(),
    ]);
    const suggestedName = await download.suggestedFilename();
    expect(suggestedName.endsWith(".sruja")).toBeTruthy();
  });
});
