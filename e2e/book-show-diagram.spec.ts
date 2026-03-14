import { test, expect } from "@playwright/test";

/**
 * E2E: "Show diagram" in the book.
 * Assumes the book is served (e.g. make book-serve) with WASM copied (book/copy-wasm.sh).
 * Page under test: How Sruja works – has a ```sruja block with the platform architecture.
 */
test("book Show diagram renders diagram for sruja code block", async ({ page }) => {
  await page.goto("/docs/how-sruja-works.html");

  // Find the first "Show diagram" button (Sruja code block toolbar)
  const showDiagramBtn = page.getByRole("button", { name: "Show diagram" }).first();
  await expect(showDiagramBtn).toBeVisible();
  await showDiagramBtn.click();

  // Preview container becomes visible
  const preview = page.locator(".sruja-preview").first();
  await expect(preview).toBeVisible();

  // Wait for success: button becomes "Hide diagram" (WASM ran and Mermaid was invoked)
  await expect(page.getByRole("button", { name: "Hide diagram" }).first()).toBeVisible({
    timeout: 20_000,
  });

  // Diagram rendered as SVG (Mermaid output)
  await expect(preview.locator("svg")).toBeVisible({ timeout: 5_000 });

  // No WASM/load error in the preview
  await expect(preview).not.toContainText("WASM not available");
});
