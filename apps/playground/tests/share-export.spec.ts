import { test, expect } from "@playwright/test";

test.describe("Share & Export", () => {
  test("copy shareable URL writes to clipboard (mocked)", async ({ page }) => {
    await page.addInitScript(() => {
      // @ts-ignore
      window.__COPIED__ = "";
      const orig = navigator.clipboard.writeText;
      // @ts-ignore
      navigator.clipboard.writeText = async (text) => {
        // @ts-ignore
        window.__COPIED__ = text;
        if (orig) return orig.call(navigator.clipboard, text);
      };
    });
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Ensure demo is loaded
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Click share button
    const shareBtn = page.locator('button[aria-label="Copy shareable URL"]');
    await shareBtn.click();

    // Validate captured text
    await expect
      .poll(async () => page.evaluate(() => (window as any).__COPIED__ as string))
      .toMatch(/\bshare=/);
    const text = await page.evaluate(() => (window as any).__COPIED__ as string);
    expect(text).toMatch(/\btab=/);
  });

  test("export downloads a .sruja file", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Ensure demo is loaded
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }

    // Trigger export and assert download event
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.locator('button[aria-label="Export to .sruja file"]').click(),
    ]);
    const suggestedName = await download.suggestedFilename();
    expect(suggestedName.endsWith(".sruja")).toBeTruthy();
  });
});
