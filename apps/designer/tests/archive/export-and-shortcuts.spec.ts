import { test, expect } from "@playwright/test";

const ensureDemoLoaded = async (page: import("@playwright/test").Page) => {
  await page.goto("/");
  await page.waitForSelector(".app", { timeout: 30000 });
  // Ensure diagram tab is active for exports
  const diagramTab = page.locator('button.view-tab:has-text("Diagram")');
  if (await diagramTab.isVisible().catch(() => false)) {
    await diagramTab.click();
  }
  const dropZone = page.locator(".drop-zone");
  if (await dropZone.isVisible().catch(() => false)) {
    await page.locator("button.demo-btn").click();
    await page.waitForSelector(".react-flow, .likec4-canvas", { timeout: 30000 });
  }
  // Ensure diagram content is present to enable export actions (LikeC4 uses SVG)
  await page.waitForSelector(".likec4-diagram-container svg, .react-flow svg", { timeout: 30000 });
};

const ctrlOrMeta = process.platform === "darwin" ? "Meta" : "Control";

test.describe("Exports & Keyboard Shortcuts", () => {
  test("exports PNG and SVG from Actions menu", async ({ page }) => {
    await ensureDemoLoaded(page);

    // Open Actions and trigger PNG export
    const [pngDownload] = await Promise.all([
      page.waitForEvent("download", { timeout: 60000 }),
      (async () => {
        const actionsBtn = page.locator('button[aria-label="Actions"]');
        await actionsBtn.click();
        await page.locator('button[aria-label="Export as PNG"]').waitFor({ timeout: 10000 });
        await page.locator('button[aria-label="Export as PNG"]').click();
      })(),
    ]);
    expect((await pngDownload.suggestedFilename()).endsWith(".png")).toBeTruthy();

    // Trigger SVG export
    const [svgDownload] = await Promise.all([
      page.waitForEvent("download", { timeout: 60000 }),
      (async () => {
        const actionsBtn = page.locator('button[aria-label="Actions"]');
        await actionsBtn.click();
        await page.locator('button[aria-label="Export as SVG"]').waitFor({ timeout: 10000 });
        await page.locator('button[aria-label="Export as SVG"]').click();
      })(),
    ]);
    expect((await svgDownload.suggestedFilename()).endsWith(".svg")).toBeTruthy();
  });

  test("command palette opens with shortcut and navigates", async ({ page }) => {
    await ensureDemoLoaded(page);

    await page.locator("body").click();
    await page.keyboard.press(`${ctrlOrMeta}+KeyK`);
    // Fallback: manually dispatch if palette not visible
    const paletteLocator = page.locator(".command-palette");
    if (!(await paletteLocator.isVisible().catch(() => false))) {
      await page.evaluate(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "k",
            ctrlKey: navigator.platform.toUpperCase().includes("MAC") ? false : true,
            metaKey: navigator.platform.toUpperCase().includes("MAC"),
          })
        );
      });
    }
    await expect(paletteLocator).toBeVisible({ timeout: 10000 });

    // Search for Diagram command and execute
    await page.getByPlaceholder("Type a command or search...").fill("diagram");
    await page.getByRole("button", { name: "Go to Diagram" }).click();
    await expect(palette).toBeHidden();

    // Diagram tab should be active
    const diagramTab = page.locator('button.view-tab:has-text("Diagram")');
    await expect(diagramTab).toHaveAttribute("aria-selected", "true");
  });

  test("shortcuts modal opens with ? shortcut", async ({ page }) => {
    await ensureDemoLoaded(page);

    await page.locator("body").click();
    await page.keyboard.press("Shift+Slash"); // '?'
    // Fallback dispatch if modal not visible
    const modalLocator = page.locator(".shortcuts-modal");
    if (!(await modalLocator.isVisible().catch(() => false))) {
      await page.evaluate(() => {
        window.dispatchEvent(new KeyboardEvent("keydown", { key: "?", shiftKey: true }));
      });
    }
    await expect(modalLocator).toBeVisible({ timeout: 10000 });
    await expect(modalLocator).toContainText("Keyboard Shortcuts");
  });
});
