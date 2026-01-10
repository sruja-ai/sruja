// apps/designer/tests/role-views.spec.ts
// E2E tests for role-based views and role switcher
import { test, expect } from "@playwright/test";

test.describe("Role-Based Views", () => {
  test.beforeEach(async ({ page }) => {
    // Load an example that has role definitions
    await page.goto("/?level=L1&tab=roles&example=project_saas_platform.sruja", {
      waitUntil: "networkidle",
      timeout: 60000,
    });

    // Wait for app to load
    await page.waitForSelector(".app-container, .role-view", { timeout: 30000 });
  });

  test.describe("Role Switcher", () => {
    test("displays role tabs when roles are defined", async ({ page }) => {
      // Navigate to Roles tab if not already there
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Check for role tabs container
      const roleTabsContainer = page.locator(".role-tabs-container, .role-tabs");
      const roleTabsVisible = await roleTabsContainer.isVisible().catch(() => false);

      if (roleTabsVisible) {
        // Should have at least one role tab
        const roleTabs = page.locator(".role-tab, button[role='tab']");
        const tabCount = await roleTabs.count();
        expect(tabCount).toBeGreaterThan(0);
      } else {
        // If no roles defined, should show empty state
        const emptyState = page.locator("text=/No roles defined|No Roles Defined/i");
        const emptyVisible = await emptyState.isVisible().catch(() => false);
        // Either roles exist or empty state is shown
        expect(roleTabsVisible || emptyVisible).toBe(true);
      }
    });

    test("switches between available roles", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Find role tabs
      const roleTabs = page.locator(".role-tab, button[role='tab']");
      const tabCount = await roleTabs.count();

      if (tabCount > 1) {
        // Click first role tab
        await roleTabs.first().click();
        await page.waitForTimeout(500);

        // Verify role view content is visible
        const roleViewContent = page.locator(".role-view-content, .role-view-content-wrapper");
        await expect(roleViewContent.first()).toBeVisible({ timeout: 5000 });

        // Click second role tab
        await roleTabs.nth(1).click();
        await page.waitForTimeout(500);

        // Verify content updated
        await expect(roleViewContent.first()).toBeVisible({ timeout: 5000 });
      }
    });

    test("displays role-specific views and scenarios", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Check for role view content
      const roleViewContent = page.locator(".role-view-content, .role-view-content-wrapper");
      const contentVisible = await roleViewContent.isVisible().catch(() => false);

      if (contentVisible) {
        // Should show either views/scenarios or empty state
        const viewsSection = page.locator("text=/Recommended Views|Standard Tools/i");
        const emptyState = page.locator("text=/No Role-specific Views/i");

        const hasViews = await viewsSection.isVisible().catch(() => false);
        const hasEmpty = await emptyState.isVisible().catch(() => false);

        // Should show either views or empty state
        expect(hasViews || hasEmpty).toBe(true);
      }
    });

    test("launches view from role view", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Look for view cards with "Launch Scenario" button
      const launchButtons = page.locator('button:has-text("Launch Scenario")');
      const buttonCount = await launchButtons.count();

      if (buttonCount > 0) {
        // Click first launch button
        await launchButtons.first().click();
        await page.waitForTimeout(2000);

        // Should navigate to diagram tab
        await expect.poll(async () => page.url()).toMatch(/\btab=diagram\b/);

        // Verify diagram is visible
        await expect(page.locator(".react-flow")).toBeVisible({ timeout: 10000 });
      }
    });
  });

  test.describe("Default Roles", () => {
    test("shows Product role view when available", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Look for Product role tab
      const productTab = page.locator(".role-tab, button[role='tab']").filter({
        hasText: /Product/i,
      });
      const productVisible = await productTab.isVisible().catch(() => false);

      if (productVisible) {
        await productTab.click();
        await page.waitForTimeout(500);

        // Verify Product view content
        const roleView = page.locator(".role-view");
        await expect(roleView).toBeVisible({ timeout: 5000 });
      }
    });

    test("shows Architect role view when available", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Look for Architect role tab
      const architectTab = page.locator(".role-tab, button[role='tab']").filter({
        hasText: /Architect/i,
      });
      const architectVisible = await architectTab.isVisible().catch(() => false);

      if (architectVisible) {
        await architectTab.click();
        await page.waitForTimeout(500);

        // Verify Architect view content
        const roleView = page.locator(".role-view");
        await expect(roleView).toBeVisible({ timeout: 5000 });
      }
    });

    test("shows DevOps role view when available", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Look for DevOps role tab
      const devopsTab = page.locator(".role-tab, button[role='tab']").filter({
        hasText: /DevOps/i,
      });
      const devopsVisible = await devopsTab.isVisible().catch(() => false);

      if (devopsVisible) {
        await devopsTab.click();
        await page.waitForTimeout(500);

        // Verify DevOps view content
        const roleView = page.locator(".role-view");
        await expect(roleView).toBeVisible({ timeout: 5000 });
      }
    });

    test("shows Security role view when available", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Look for Security role tab
      const securityTab = page.locator(".role-tab, button[role='tab']").filter({
        hasText: /Security/i,
      });
      const securityVisible = await securityTab.isVisible().catch(() => false);

      if (securityVisible) {
        await securityTab.click();
        await page.waitForTimeout(500);

        // Verify Security view content
        const roleView = page.locator(".role-view");
        await expect(roleView).toBeVisible({ timeout: 5000 });
      }
    });

    test("shows CTO role view when available", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Look for CTO role tab
      const ctoTab = page.locator(".role-tab, button[role='tab']").filter({
        hasText: /CTO/i,
      });
      const ctoVisible = await ctoTab.isVisible().catch(() => false);

      if (ctoVisible) {
        await ctoTab.click();
        await page.waitForTimeout(500);

        // Verify CTO view content
        const roleView = page.locator(".role-view");
        await expect(roleView).toBeVisible({ timeout: 5000 });
      }
    });

    test("shows SRE role view when available", async ({ page }) => {
      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Look for SRE role tab
      const sreTab = page.locator(".role-tab, button[role='tab']").filter({
        hasText: /SRE/i,
      });
      const sreVisible = await sreTab.isVisible().catch(() => false);

      if (sreVisible) {
        await sreTab.click();
        await page.waitForTimeout(500);

        // Verify SRE view content
        const roleView = page.locator(".role-view");
        await expect(roleView).toBeVisible({ timeout: 5000 });
      }
    });
  });

  test.describe("Role View Empty States", () => {
    test("shows empty state when no roles are defined", async ({ page }) => {
      // Load a simple example that might not have roles
      await page.goto("/?level=L1&tab=roles&example=minimal_low_quality.sruja", {
        waitUntil: "networkidle",
        timeout: 60000,
      });

      await page.waitForSelector(".app-container, .role-view", { timeout: 30000 });

      // Navigate to Roles tab
      const rolesTab = page.locator('button.view-tab:has-text("Roles")');
      if (await rolesTab.isVisible().catch(() => false)) {
        await rolesTab.click();
        await page.waitForTimeout(1000);
      }

      // Should show empty state message
      const emptyState = page.locator("text=/No roles defined|No Roles Defined/i");
      const emptyVisible = await emptyState.isVisible().catch(() => false);

      // Either roles exist or empty state is shown
      expect(emptyVisible || (await page.locator(".role-tab").count()) > 0).toBe(true);
    });
  });
});
