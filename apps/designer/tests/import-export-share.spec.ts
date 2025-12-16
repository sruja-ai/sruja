import { test, expect } from "@playwright/test";

const demoDsl = `architecture "Demo" {
  persons { user "User" }
  systems { web "WebApp" }
  relations { user -> web "uses" }
}`;

test.describe("Import, Export, Share", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });
  });

  test("import .sruja file", async ({ page }) => {
    const fileInput = page.locator('input[type="file"][accept=".sruja"]');
    await fileInput.setInputFiles({
      name: "demo.sruja",
      mimeType: "text/plain",
      buffer: Buffer.from(demoDsl),
    });
    await page.waitForSelector(".react-flow", { timeout: 30000 });
    const nodes = await page.locator(".react-flow__node").count();
    expect(nodes).toBeGreaterThan(0);
  });

  test("export .sruja file", async ({ page }) => {
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      (async () => {
        const actionsBtn = page.locator('button[aria-label="Actions"]');
        await actionsBtn.click();
        await page.locator('button[aria-label="Export .sruja file"]').click();
      })(),
    ]);
    const path = await download.path();
    expect(path).toBeTruthy();
  });

  test("share button clickable", async ({ page }) => {
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".react-flow", { timeout: 30000 });
    }
    // Open Actions and click Share
    const actionsBtn = page.locator('button[aria-label="Actions"]');
    await actionsBtn.click();
    const shareBtn = page.locator('button[aria-label="Copy shareable URL"]');
    await shareBtn.click();
    await expect(actionsBtn).toBeVisible();
  });
});
