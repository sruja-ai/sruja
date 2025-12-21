// scripts/test-fixes.ts
// Standalone script to test fixes using Playwright
import { chromium, type Browser, type Page } from "playwright";

async function testFixes() {
  console.log("🚀 Starting fix verification tests...\n");

  const browser: Browser = await chromium.launch({ headless: false });
  const page: Page = await browser.newPage();

  try {
    // Test 1: App loads
    console.log("Test 1: App loads...");
    await page.goto("http://localhost:5173");
    await page.waitForSelector(".app", { timeout: 30000 });
    console.log("✅ App loaded\n");

    // Test 2: Load demo
    console.log("Test 2: Load demo...");
    const dropZone = page.locator(".drop-zone");
    if (await dropZone.isVisible().catch(() => false)) {
      await page.locator("button.demo-btn").click();
      await page.waitForSelector(".likec4-canvas", { timeout: 30000 });
      console.log("✅ Demo loaded\n");
    } else {
      console.log("⚠️  Drop zone not visible, demo may already be loaded\n");
    }

    // Test 3: Diagram renders
    console.log("Test 3: Diagram renders...");
    await page.waitForSelector(".likec4-canvas", { timeout: 10000 });
    const canvasVisible = await page.locator(".likec4-canvas").isVisible();
    if (canvasVisible) {
      await page.waitForSelector(".likec4-diagram-container svg", { timeout: 10000 });
      const svgCount = await page.locator(".likec4-diagram-container svg").count();
      if (svgCount > 0) {
        console.log(`✅ Diagram rendered with ${svgCount} SVG element(s)\n`);
      } else {
        console.log("❌ Diagram canvas visible but no SVG elements found\n");
      }
    } else {
      console.log("❌ Diagram canvas not visible\n");
    }

    // Test 4: Code tab shows content
    console.log("Test 4: Code tab shows content...");
    await page.locator('button.view-tab:has-text("Code")').click();
    await page.waitForSelector(".code-panel-container", { timeout: 10000 });
    
    const dslPanel = page.locator(".dsl-panel");
    const isVisible = await dslPanel.isVisible().catch(() => false);
    if (isVisible) {
      const editor = page.locator(".monaco-editor, .dsl-panel-content");
      const content = await editor.first().textContent().catch(() => "");
      if (content && content.trim().length > 0) {
        console.log(`✅ Code tab shows content (${content.trim().length} chars)\n`);
      } else {
        console.log("❌ Code tab visible but no content\n");
      }
    } else {
      console.log("❌ DSL panel not visible\n");
    }

    // Test 5: Builder wizard
    console.log("Test 5: Builder wizard...");
    await page.locator('button.view-tab:has-text("Builder")').click();
    await page.waitForSelector(".builder-wizard", { timeout: 10000 });
    
    const wizardVisible = await page.locator(".builder-wizard").isVisible();
    if (wizardVisible) {
      const steps = page.locator(".wizard-step");
      const stepCount = await steps.count();
      console.log(`✅ Builder wizard visible with ${stepCount} step(s)\n`);
    } else {
      console.log("❌ Builder wizard not visible\n");
    }

    // Test 6: Template gallery
    console.log("Test 6: Template gallery...");
    const templateBtn = page.locator('button:has-text("Start from a Template"), .template-prompt-btn');
    const templateBtnVisible = await templateBtn.isVisible().catch(() => false);
    
    if (templateBtnVisible) {
      await templateBtn.click();
      await page.waitForSelector(".template-gallery-modal", { timeout: 5000 });
      
      const modalVisible = await page.locator(".template-gallery-modal").isVisible();
      if (modalVisible) {
        const templates = page.locator(".template-card");
        const templateCount = await templates.count();
        console.log(`✅ Template gallery opened with ${templateCount} template(s)\n`);
        
        // Close modal
        await page.locator(".template-close-btn, .template-cancel-btn").first().click();
        await page.waitForTimeout(500);
      } else {
        console.log("❌ Template gallery modal not visible\n");
      }
    } else {
      console.log("⚠️  Template button not visible (may need to be on Goals step)\n");
    }

    console.log("✅ All tests completed!");
    
    // Keep browser open for manual inspection
    console.log("\n📝 Browser will stay open for 10 seconds for manual inspection...");
    await page.waitForTimeout(10000);

  } catch (error) {
    console.error("❌ Test failed:", error);
    throw error;
  } finally {
    await browser.close();
  }
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  testFixes().catch((error) => {
    console.error("Fatal error:", error);
    process.exit(1);
  });
}

export { testFixes };

