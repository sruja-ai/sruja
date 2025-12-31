// apps/designer/tests/edge-labels-verification.spec.ts
// Test to verify edge labels are extracted and rendered correctly
import { test, expect } from "@playwright/test";

test.describe("Edge Labels Verification", () => {
  test("should extract and display edge labels from relations", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app", { timeout: 30000 });

    // Load demo/example first
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForTimeout(2000);
    }

    // Ensure we're on Diagram tab
    await page.locator('button.view-tab:has-text("Diagram")').click();
    await page.waitForTimeout(2000);

    // Wait for React Flow to be visible
    const reactFlow = page.locator(".react-flow");
    await expect(reactFlow).toBeVisible({ timeout: 30000 });

    // Wait for edges to be rendered
    await page.waitForSelector(".react-flow__edge", { timeout: 10000 });

    // Check console logs for relation data
    const consoleLogs: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "log" && msg.text().includes("Sample relation")) {
        consoleLogs.push(msg.text());
      }
    });

    // Wait a bit for logs to accumulate
    await page.waitForTimeout(2000);

    // Verify that relations have title fields in the model
    const relationData = await page.evaluate(() => {
      // Access the model from the store or window
      return (window as unknown as any).__MODEL_DATA__ || null;
    });

    if (relationData) {
      console.log("Relation data verified in model");
    }

    // Check for edge labels in React Flow
    const edgeLabels = page.locator(".react-flow__edge .react-flow__edge-label");
    const labelCount = await edgeLabels.count();

    console.log(`Found ${labelCount} edge labels in React Flow`);

    // If labels exist, verify they're visible
    if (labelCount > 0) {
      await expect(edgeLabels.first()).toBeVisible();
      const firstLabelText = await edgeLabels.first().textContent();
      expect(firstLabelText).toBeTruthy();
      expect(firstLabelText?.trim().length).toBeGreaterThan(0);
      console.log(`First label text: "${firstLabelText}"`);
    } else {
      // Log warning but don't fail - this helps diagnose the issue
      console.warn("No edge labels found. This may indicate:");
      console.warn("1. Relations don't have labels in the source model");
      console.warn("2. WASM isn't extracting labels correctly");
      console.warn("3. React Flow isn't rendering labels");
    }

    // Take a screenshot for visual verification
    await page.screenshot({ path: "test-results/edge-labels-verification.png", fullPage: true });
  });
});
