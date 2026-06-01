import { test, expect } from "@playwright/test";

/**
 * E2E: "Show diagram" in the book.
 * Assumes the book is served (e.g. make book-serve) with WASM copied (book/copy-wasm.sh).
 * Page under test: docs/e2e-show-diagram.md – minimal ```sruja block (no stdlib import).
 */
test("book Show diagram renders diagram for sruja code block", async ({ page }) => {
  await page.goto("/docs/e2e-show-diagram.html");

  // Find the first "Show diagram" button (Sruja code block toolbar)
  const showDiagramBtn = page.getByRole("button", { name: "Show diagram" }).first();
  await expect(showDiagramBtn).toBeVisible();
  await showDiagramBtn.click();

  // Preview container becomes visible
  const preview = page.locator(".sruja-preview").first();
  await expect(preview).toBeVisible();

  // Wait for success: button becomes "Hide diagram" (WASM ran and Mermaid was invoked)
  const wasmTimeout = process.env.CI ? 60_000 : 20_000;
  await expect(page.getByRole("button", { name: "Hide diagram" }).first()).toBeVisible({
    timeout: wasmTimeout,
  });

  // Diagram rendered as SVG (Mermaid output)
  await expect(preview.locator("svg")).toBeVisible({ timeout: 5_000 });

  // No WASM/load error in the preview
  await expect(preview).not.toContainText("WASM not available");
});
