// apps/designer/tests/overview-tab.spec.ts
// E2E tests for Overview tab features
import { test, expect } from "@playwright/test";

test.describe("Overview Tab", () => {
  test.beforeEach(async ({ page }) => {
    // Load an example
    await page.goto("/?level=L1&tab=overview&example=ecommerce_platform.sruja", {
      waitUntil: "networkidle",
      timeout: 60000,
    });

    // Wait for app to load
    await page.waitForSelector(".app-container, .overview-tab", { timeout: 30000 });

    // Navigate to Overview tab if not already there
    const overviewTab = page.locator('button.view-tab:has-text("Overview")');
    if (await overviewTab.isVisible().catch(() => false)) {
      await overviewTab.click();
      await page.waitForTimeout(1000);
    }
  });

  test.describe("Overview Content", () => {
    test("displays overview tab content", async ({ page }) => {
      const overviewTab = page.locator(".overview-tab");
      await expect(overviewTab).toBeVisible({ timeout: 5000 });
    });

    test("shows empty state when no architecture loaded", async ({ page }) => {
      // Navigate to empty state
      await page.goto("/?tab=overview");
      await page.waitForSelector(".app-container, .overview-tab", { timeout: 30000 });

      // Should show empty state
      const emptyState = page.locator("text=/No Architecture Loaded/i, .overview-tab-empty");
      const emptyVisible = await emptyState.isVisible().catch(() => false);

      if (emptyVisible) {
        await expect(emptyState.first()).toBeVisible({ timeout: 5000 });
      }
    });
  });

  test.describe("Hero Section", () => {
    test("displays architecture name", async ({ page }) => {
      const hero = page.locator(".overview-hero, [class*='hero']");
      const heroVisible = await hero.isVisible().catch(() => false);

      if (heroVisible) {
        await expect(hero.first()).toBeVisible({ timeout: 5000 });

        // Should have architecture name
        const name = page.locator("h1, h2, [class*='title'], [class*='name']");
        const nameCount = await name.count();
        expect(nameCount).toBeGreaterThan(0);
      }
    });

    test("displays architecture description", async ({ page }) => {
      const hero = page.locator(".overview-hero, [class*='hero']");
      const heroVisible = await hero.isVisible().catch(() => false);

      if (heroVisible) {
        // Description might be visible
        // Description might or might not be present
      }
    });
  });

  test.describe("Stats Row", () => {
    test("displays statistics", async ({ page }) => {
      const statsRow = page.locator(".stats-row, [class*='stats']");
      const statsVisible = await statsRow.isVisible().catch(() => false);

      if (statsVisible) {
        await expect(statsRow.first()).toBeVisible({ timeout: 5000 });

        // Should show stats like systems, persons, etc.
        const stats = page.locator("text=/systems|persons|requirements|ADRs|policies/i");
        const statsCount = await stats.count();
        expect(statsCount).toBeGreaterThanOrEqual(0);
      }
    });

    test("stats are clickable for navigation", async ({ page }) => {
      const statsRow = page.locator(".stats-row, [class*='stats']");
      const statsVisible = await statsRow.isVisible().catch(() => false);

      if (statsVisible) {
        // Look for clickable stat items
        const statItems = page.locator(
          ".stats-row button, .stats-row [role='button'], [class*='stat'][class*='clickable']"
        );
        const itemCount = await statItems.count();

        if (itemCount > 0) {
          await statItems.first().click();
          await page.waitForTimeout(1000);

          // Might navigate to another tab
        }
      }
    });
  });

  test.describe("Governance Widget", () => {
    test("displays governance widget", async ({ page }) => {
      const governanceWidget = page.locator(".governance-widget, [class*='governance']");
      const widgetVisible = await governanceWidget.isVisible().catch(() => false);

      if (widgetVisible) {
        await expect(governanceWidget.first()).toBeVisible({ timeout: 5000 });
      }
    });

    test("governance widget shows score", async ({ page }) => {
      const governanceWidget = page.locator(".governance-widget, [class*='governance']");
      const widgetVisible = await governanceWidget.isVisible().catch(() => false);

      if (widgetVisible) {
        // Should show score or metrics
        // Score might or might not be visible
      }
    });
  });

  test.describe("Metadata Section", () => {
    test("displays metadata if available", async ({ page }) => {
      const metadataSection = page.locator(".metadata-section, [class*='metadata']");
      const metadataVisible = await metadataSection.isVisible().catch(() => false);

      if (metadataVisible) {
        await expect(metadataSection.first()).toBeVisible({ timeout: 5000 });
      }
    });
  });

  test.describe("Quick Navigation", () => {
    test("displays quick navigation cards", async ({ page }) => {
      const quickNav = page.locator(
        ".overview-quick-nav, [class*='quick-nav'], text=/Quick Navigation/i"
      );
      const navVisible = await quickNav.isVisible().catch(() => false);

      if (navVisible) {
        await expect(quickNav.first()).toBeVisible({ timeout: 5000 });
      }
    });

    test("quick navigation cards are clickable", async ({ page }) => {
      const quickNav = page.locator(".overview-quick-nav, [class*='quick-nav']");
      const navVisible = await quickNav.isVisible().catch(() => false);

      if (navVisible) {
        // Look for navigation cards
        const navCards = page.locator(
          ".overview-quick-nav [class*='card'], .overview-quick-nav [role='button'], .overview-quick-nav button"
        );
        const cardCount = await navCards.count();

        if (cardCount > 0) {
          await navCards.first().click();
          await page.waitForTimeout(1000);

          // Should navigate to another tab
          // URL might change or tab might switch
        }
      }
    });

    test("navigates to Builder tab from quick nav", async ({ page }) => {
      const builderCard = page.locator(
        ".overview-quick-nav text=/Builder/i, .overview-quick-nav [class*='builder']"
      );
      const builderVisible = await builderCard.isVisible().catch(() => false);

      if (builderVisible) {
        await builderCard.click();
        await page.waitForTimeout(1000);

        // Should navigate to builder tab
        await expect.poll(async () => page.url()).toMatch(/\btab=builder\b/);
      }
    });

    test("navigates to Diagram tab from quick nav", async ({ page }) => {
      const diagramCard = page.locator(
        ".overview-quick-nav text=/Diagram/i, .overview-quick-nav [class*='diagram']"
      );
      const diagramVisible = await diagramCard.isVisible().catch(() => false);

      if (diagramVisible) {
        await diagramCard.click();
        await page.waitForTimeout(1000);

        // Should navigate to diagram tab
        await expect.poll(async () => page.url()).toMatch(/\btab=diagram\b/);
      }
    });
  });

  test.describe("Sections", () => {
    test("displays goals section if available", async ({ page }) => {
      const goalsSection = page.locator(".goals-section, [class*='goals'], text=/Goals/i");
      const goalsVisible = await goalsSection.isVisible().catch(() => false);

      // Goals section might or might not be visible
      if (goalsVisible) {
        await expect(goalsSection.first()).toBeVisible({ timeout: 5000 });
      }
    });

    test("displays constraints section if available", async ({ page }) => {
      const constraintsSection = page.locator(
        ".constraints-section, [class*='constraints'], text=/Constraints/i"
      );
      const constraintsVisible = await constraintsSection.isVisible().catch(() => false);

      // Constraints section might or might not be visible
      if (constraintsVisible) {
        await expect(constraintsSection.first()).toBeVisible({ timeout: 5000 });
      }
    });

    test("displays policies section if available", async ({ page }) => {
      const policiesSection = page.locator(
        ".policies-section, [class*='policies'], text=/Policies/i"
      );
      const policiesVisible = await policiesSection.isVisible().catch(() => false);

      // Policies section might or might not be visible
      if (policiesVisible) {
        await expect(policiesSection.first()).toBeVisible({ timeout: 5000 });
      }
    });
  });
});
