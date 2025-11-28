# E2E Testing Assessment for Learn App

**Date:** 2025-11-28  
**Purpose:** Assess end-to-end testing strategy for Hugo static site

## Executive Summary

**Verdict: ✅ YES, E2E testing is highly recommended and feasible**

The learn app has significant client-side functionality that requires E2E testing:
- Playground with WASM compilation
- Dynamic navigation and theme switching
- Code block enhancements
- Search functionality
- Course state management

**Recommended Framework:** **Playwright** (best fit for static sites, fast, reliable)  
**Alternative:** Cypress (good, but slower for static sites)  
**Effort:** Medium (2-3 weeks for comprehensive coverage)  
**Benefits:** Very High (catch regressions, ensure functionality, confidence in deployments)

---

## Testable Features Analysis

### 1. **Playground Functionality** ⭐⭐⭐⭐⭐ (Critical)

**Features to Test:**
- WASM loading and initialization
- Code compilation (Sruja → SVG)
- Example dropdown selection
- Run button functionality
- Error handling and display
- SVG output rendering
- Zoom controls (in, out, reset)
- Expand modal functionality
- Toolbar visibility

**Test Scenarios:**
```typescript
- Load playground page
- Wait for WASM to initialize
- Select example from dropdown
- Click Run button
- Verify SVG is rendered
- Test zoom controls
- Test expand modal
- Test error handling with invalid code
```

### 2. **Navigation System** ⭐⭐⭐⭐⭐ (Critical)

**Features to Test:**
- Top navigation bar rendering
- Section detection (playground, docs, courses, etc.)
- Active link highlighting
- Mobile menu toggle
- Dropdown menus (Resources)
- Navigation links functionality
- Search bar integration

**Test Scenarios:**
```typescript
- Navigate to different sections
- Verify active link highlighting
- Test mobile menu toggle
- Test dropdown menus
- Verify navigation links work
- Test search functionality
```

### 3. **Theme Toggle** ⭐⭐⭐⭐ (Important)

**Features to Test:**
- Theme toggle button
- Dark/light mode switching
- Theme persistence (localStorage)
- CSS class application
- Icon changes

**Test Scenarios:**
```typescript
- Click theme toggle
- Verify theme changes
- Reload page, verify persistence
- Test in different sections
```

### 4. **Code Block Enhancements** ⭐⭐⭐⭐ (Important)

**Features to Test:**
- Code block toolbar rendering
- Copy button functionality
- Edit button (toggle editor)
- Run button (WASM compilation)
- Output display
- Error handling

**Test Scenarios:**
```typescript
- Find code blocks with .language-sruja
- Test copy functionality
- Test edit toggle
- Test run functionality
- Verify output rendering
```

### 5. **Search Functionality** ⭐⭐⭐ (Nice to Have)

**Features to Test:**
- Search input rendering
- Search suggestions display
- Search results
- Empty state handling

### 6. **Course State Management** ⭐⭐⭐ (Nice to Have)

**Features to Test:**
- Page visit tracking
- localStorage persistence
- Progress UI updates

### 7. **Responsive Design** ⭐⭐⭐⭐ (Important)

**Features to Test:**
- Mobile navigation
- Tablet layout
- Desktop layout
- Breakpoint behavior

---

## Framework Comparison

### Option 1: Playwright ⭐⭐⭐⭐⭐ (Recommended)

**Pros:**
- ✅ Fast and reliable
- ✅ Excellent for static sites
- ✅ Multi-browser support (Chromium, Firefox, WebKit)
- ✅ Auto-waiting (no flaky tests)
- ✅ Great debugging tools
- ✅ Screenshot/video recording
- ✅ TypeScript support
- ✅ CI/CD friendly
- ✅ Can test WASM functionality

**Cons:**
- ⚠️ Requires Node.js setup
- ⚠️ Learning curve (moderate)

**Best For:**
- Static sites (like Hugo)
- WASM testing
- Cross-browser testing
- CI/CD integration

### Option 2: Cypress ⭐⭐⭐⭐

**Pros:**
- ✅ Great developer experience
- ✅ Time-travel debugging
- ✅ Good documentation
- ✅ Large community

**Cons:**
- ⚠️ Slower than Playwright
- ⚠️ Limited browser support (Chromium-based)
- ⚠️ Can be flaky with static sites
- ⚠️ WASM testing can be challenging

**Best For:**
- SPAs (Single Page Applications)
- Teams already using Cypress

### Option 3: Puppeteer ⭐⭐⭐

**Pros:**
- ✅ Direct Chrome DevTools Protocol
- ✅ Good for automation

**Cons:**
- ⚠️ Chrome/Chromium only
- ⚠️ More low-level API
- ⚠️ Less developer-friendly

**Best For:**
- Chrome-specific testing
- Advanced automation needs

### Recommendation: **Playwright** ✅

**Rationale:**
1. Best performance for static sites
2. Excellent WASM support
3. Multi-browser testing out of the box
4. Great CI/CD integration
5. TypeScript support (aligns with our migration plan)

---

## Testing Strategy

### Test Structure

```
learn/
├── e2e/
│   ├── tests/
│   │   ├── playground.spec.ts
│   │   ├── navigation.spec.ts
│   │   ├── theme.spec.ts
│   │   ├── code-blocks.spec.ts
│   │   ├── search.spec.ts
│   │   └── responsive.spec.ts
│   ├── fixtures/
│   │   ├── test-data.ts
│   │   └── sruja-examples.ts
│   ├── utils/
│   │   ├── helpers.ts
│   │   └── selectors.ts
│   ├── playwright.config.ts
│   └── package.json
```

### Test Categories

**1. Critical Path Tests (Smoke Tests)**
- Playground: Load, compile, render
- Navigation: Basic navigation works
- Theme: Toggle works

**2. Feature Tests**
- All playground features
- All navigation features
- Code block enhancements
- Search functionality

**3. Regression Tests**
- Previously fixed bugs
- Edge cases
- Error scenarios

**4. Visual Regression (Optional)**
- Screenshot comparisons
- Layout verification

---

## Implementation Plan

### Phase 1: Setup (Week 1)

#### 1.1 Install Playwright

```bash
cd learn
npm init -y
npm install --save-dev @playwright/test
npx playwright install
```

#### 1.2 Create Configuration

**playwright.config.ts:**
```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e/tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:1313',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
  ],

  webServer: {
    command: 'cd .. && hugo server -D --port 1313',
    url: 'http://localhost:1313',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
```

#### 1.3 Create Test Utilities

**e2e/utils/helpers.ts:**
```typescript
import { Page, expect } from '@playwright/test';

export async function waitForWasmReady(page: Page): Promise<void> {
  // Wait for WASM to be ready
  await page.waitForFunction(() => {
    return window.srujaWasmReady === true;
  }, { timeout: 30000 });
  
  // Wait for compileSruja function to be available
  await page.waitForFunction(() => {
    return typeof window.compileSruja === 'function';
  }, { timeout: 30000 });
}

export async function waitForSVG(page: Page, selector: string): Promise<void> {
  await page.waitForSelector(`${selector} svg`, { timeout: 10000 });
}

export async function getPlaygroundStatus(page: Page): Promise<string> {
  const status = await page.locator('#status').textContent();
  return status || '';
}
```

**e2e/utils/selectors.ts:**
```typescript
export const Selectors = {
  // Playground
  playground: {
    exampleSelect: '#example-select',
    runButton: '#run-btn',
    status: '#status',
    input: '#sruja-input',
    output: '#d2-output',
    errorOutput: '#error-output',
    zoomIn: '#zoom-in',
    zoomOut: '#zoom-out',
    zoomReset: '#zoom-reset',
    expandPreview: '#expand-preview',
  },
  
  // Navigation
  nav: {
    topNav: '.site-top-nav',
    navToggle: '.nav-toggle',
    mobileMenu: '.nav-mobile-menu',
    themeToggle: '.theme-toggle',
    searchInput: '#book-search-input',
  },
  
  // Code blocks
  codeBlocks: {
    wrapper: '.sruja-code-wrapper',
    copyButton: '.sruja-btn-copy',
    editButton: '.sruja-btn-edit',
    runButton: '.sruja-btn-run',
    output: '.sruja-run-output',
  },
};
```

### Phase 2: Core Tests (Week 2)

#### 2.1 Playground Tests

**e2e/tests/playground.spec.ts:**
```typescript
import { test, expect } from '@playwright/test';
import { waitForWasmReady, waitForSVG, getPlaygroundStatus } from '../utils/helpers';
import { Selectors } from '../utils/selectors';

test.describe('Playground', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/playground/');
    await waitForWasmReady(page);
  });

  test('should load playground page', async ({ page }) => {
    await expect(page.locator(Selectors.playground.input)).toBeVisible();
    await expect(page.locator(Selectors.playground.runButton)).toBeVisible();
  });

  test('should show Ready status after WASM loads', async ({ page }) => {
    const status = await getPlaygroundStatus(page);
    expect(status).toBe('Ready');
  });

  test('should compile and render SVG', async ({ page }) => {
    // Click Run button
    await page.click(Selectors.playground.runButton);
    
    // Wait for SVG to render
    await waitForSVG(page, Selectors.playground.output);
    
    // Verify SVG is present
    const svg = page.locator(`${Selectors.playground.output} svg`);
    await expect(svg).toBeVisible();
    
    // Verify toolbar is visible
    const toolbar = page.locator('.preview-toolbar');
    await expect(toolbar).toBeVisible();
  });

  test('should handle example selection', async ({ page }) => {
    const select = page.locator(Selectors.playground.exampleSelect);
    
    // Select "Basic Example"
    await select.selectOption('Basic Example');
    
    // Verify input content changed
    const input = page.locator(Selectors.playground.input);
    const content = await input.inputValue();
    expect(content).toContain('Basic Example');
  });

  test('should display error for invalid code', async ({ page }) => {
    const input = page.locator(Selectors.playground.input);
    
    // Enter invalid Sruja code
    await input.fill('invalid syntax {');
    
    // Click Run
    await page.click(Selectors.playground.runButton);
    
    // Verify error is displayed
    const errorOutput = page.locator(Selectors.playground.errorOutput);
    await expect(errorOutput).toBeVisible();
    await expect(errorOutput).toContainText('error', { ignoreCase: true });
  });

  test('should handle zoom controls', async ({ page }) => {
    // First compile code
    await page.click(Selectors.playground.runButton);
    await waitForSVG(page, Selectors.playground.output);
    
    // Test zoom in
    await page.click(Selectors.playground.zoomIn);
    
    // Test zoom out
    await page.click(Selectors.playground.zoomOut);
    
    // Test reset
    await page.click(Selectors.playground.zoomReset);
    
    // Verify controls are functional (no errors)
    const toolbar = page.locator('.preview-toolbar');
    await expect(toolbar).toBeVisible();
  });

  test('should open expand modal', async ({ page }) => {
    // Compile code first
    await page.click(Selectors.playground.runButton);
    await waitForSVG(page, Selectors.playground.output);
    
    // Click expand button
    await page.click(Selectors.playground.expandPreview);
    
    // Verify modal is open
    const modal = page.locator('.sruja-modal');
    await expect(modal).toBeVisible();
    
    // Close modal
    const closeButton = page.locator('.sruja-modal-close');
    await closeButton.click();
    
    // Verify modal is closed
    await expect(modal).not.toBeVisible();
  });
});
```

#### 2.2 Navigation Tests

**e2e/tests/navigation.spec.ts:**
```typescript
import { test, expect } from '@playwright/test';
import { Selectors } from '../utils/selectors';

test.describe('Navigation', () => {
  test('should render top navigation', async ({ page }) => {
    await page.goto('/');
    
    const nav = page.locator(Selectors.nav.topNav);
    await expect(nav).toBeVisible();
  });

  test('should highlight active section', async ({ page }) => {
    await page.goto('/playground/');
    
    const playgroundLink = page.locator('a[href="/playground/"]');
    await expect(playgroundLink).toHaveClass(/active/);
  });

  test('should toggle mobile menu', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    
    const toggle = page.locator(Selectors.nav.navToggle);
    const mobileMenu = page.locator(Selectors.nav.mobileMenu);
    
    // Menu should be hidden initially
    await expect(mobileMenu).not.toHaveClass(/active/);
    
    // Click toggle
    await toggle.click();
    
    // Menu should be visible
    await expect(mobileMenu).toHaveClass(/active/);
    
    // Click toggle again
    await toggle.click();
    
    // Menu should be hidden
    await expect(mobileMenu).not.toHaveClass(/active/);
  });

  test('should navigate to different sections', async ({ page }) => {
    await page.goto('/');
    
    // Navigate to docs
    await page.click('a[href="/docs/"]');
    await expect(page).toHaveURL(/\/docs\//);
    
    // Navigate to courses
    await page.click('a[href="/courses/"]');
    await expect(page).toHaveURL(/\/courses\//);
  });
});
```

#### 2.3 Theme Tests

**e2e/tests/theme.spec.ts:**
```typescript
import { test, expect } from '@playwright/test';
import { Selectors } from '../utils/selectors';

test.describe('Theme Toggle', () => {
  test('should toggle theme', async ({ page }) => {
    await page.goto('/');
    
    const themeToggle = page.locator(Selectors.nav.themeToggle);
    const html = page.locator('html');
    
    // Get initial theme
    const initialTheme = await html.evaluate((el) => {
      return el.classList.contains('theme-dark') ? 'dark' : 'light';
    });
    
    // Toggle theme
    await themeToggle.click();
    
    // Wait for theme change
    await page.waitForTimeout(100);
    
    // Verify theme changed
    const newTheme = await html.evaluate((el) => {
      return el.classList.contains('theme-dark') ? 'dark' : 'light';
    });
    
    expect(newTheme).not.toBe(initialTheme);
  });

  test('should persist theme in localStorage', async ({ page }) => {
    await page.goto('/');
    
    const themeToggle = page.locator(Selectors.nav.themeToggle);
    
    // Set theme to dark
    await themeToggle.click();
    await page.waitForTimeout(100);
    
    // Reload page
    await page.reload();
    
    // Verify theme persisted
    const html = page.locator('html');
    const theme = await html.evaluate((el) => {
      return el.classList.contains('theme-dark') ? 'dark' : 'light';
    });
    
    expect(theme).toBe('dark');
  });
});
```

### Phase 3: Advanced Tests (Week 3)

#### 3.1 Code Block Tests

**e2e/tests/code-blocks.spec.ts:**
```typescript
import { test, expect } from '@playwright/test';
import { waitForWasmReady } from '../utils/helpers';
import { Selectors } from '../utils/selectors';

test.describe('Code Block Enhancements', () => {
  test.beforeEach(async ({ page }) => {
    await waitForWasmReady(page);
  });

  test('should enhance code blocks with toolbar', async ({ page }) => {
    // Navigate to a page with code blocks
    await page.goto('/docs/getting-started/');
    
    // Wait for code blocks to be enhanced
    await page.waitForSelector(Selectors.codeBlocks.wrapper, { timeout: 5000 });
    
    // Verify toolbar buttons exist
    const copyButton = page.locator(Selectors.codeBlocks.copyButton).first();
    await expect(copyButton).toBeVisible();
  });

  test('should copy code to clipboard', async ({ page, context }) => {
    await page.goto('/docs/getting-started/');
    await page.waitForSelector(Selectors.codeBlocks.wrapper);
    
    // Grant clipboard permissions
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    
    const copyButton = page.locator(Selectors.codeBlocks.copyButton).first();
    await copyButton.click();
    
    // Verify button shows success state
    await expect(copyButton).toHaveClass(/success/);
  });
});
```

#### 3.2 Responsive Tests

**e2e/tests/responsive.spec.ts:**
```typescript
import { test, expect } from '@playwright/test';

test.describe('Responsive Design', () => {
  const viewports = [
    { name: 'Mobile', width: 375, height: 667 },
    { name: 'Tablet', width: 768, height: 1024 },
    { name: 'Desktop', width: 1920, height: 1080 },
  ];

  for (const viewport of viewports) {
    test(`should render correctly on ${viewport.name}`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      await page.goto('/');
      
      // Verify navigation is visible
      const nav = page.locator('.site-top-nav');
      await expect(nav).toBeVisible();
      
      // Verify content is visible
      const main = page.locator('main');
      await expect(main).toBeVisible();
    });
  }
});
```

---

## CI/CD Integration

### GitHub Actions Workflow

**`.github/workflows/e2e-tests.yml`:**
```yaml
name: E2E Tests

on:
  push:
    branches: [main]
    paths:
      - 'learn/**'
  pull_request:
    branches: [main]
    paths:
      - 'learn/**'
  workflow_dispatch:

jobs:
  e2e:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout
        uses: actions/checkout@v6
        with:
          submodules: true
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: learn/package-lock.json
      
      - name: Setup Hugo
        uses: peaceiris/actions-hugo@v3
        with:
          hugo-version: '0.152.2'
          extended: true
      
      - name: Install Dart Sass
        run: sudo snap install dart-sass
      
      - name: Setup Go
        uses: actions/setup-go@v5
        with:
          go-version: '1.25'
      
      - name: Install dependencies
        run: |
          go mod download
          cd learn && npm ci
      
      - name: Build WASM
        run: make build-docs
      
      - name: Install Playwright
        run: cd learn && npx playwright install --with-deps
      
      - name: Run E2E tests
        run: cd learn && npx playwright test
        env:
          CI: true
      
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: learn/playwright-report/
          retention-days: 30
```

---

## Test Data & Fixtures

### Test Fixtures

**e2e/fixtures/sruja-examples.ts:**
```typescript
export const SrujaExamples = {
  valid: {
    quickStart: `architecture "Quick Start" {
  person User "User"
  system Web "Web App" {
    container Frontend "Frontend"
    datastore DB "Database"
  }
  User -> Frontend "Visits"
  Frontend -> DB "Reads/Writes"
}`,
    basic: `architecture "Basic Example" {
  person User "User"
  system SoftwareSystem "Software System" {
    container WebApp "Web Application" { }
    container Database "Database" { }
  }
  User -> WebApp "Uses"
  WebApp -> Database "Reads/Writes"
}`,
  },
  invalid: {
    syntaxError: 'invalid syntax {',
    missingBrace: 'architecture "Test" {',
    empty: '',
  },
};
```

---

## Best Practices

### 1. **Test Organization**
- Group related tests in describe blocks
- Use descriptive test names
- Follow AAA pattern (Arrange, Act, Assert)

### 2. **Selectors**
- Use data-testid attributes (recommended)
- Or use stable CSS selectors
- Avoid XPath when possible

### 3. **Waiting**
- Use Playwright's auto-waiting
- Avoid fixed timeouts when possible
- Use `waitForSelector` for dynamic content

### 4. **WASM Testing**
- Wait for WASM to be ready before testing
- Handle WASM loading errors gracefully
- Test both success and error paths

### 5. **Maintenance**
- Keep tests independent
- Use fixtures for test data
- Clean up after tests (localStorage, etc.)

---

## Coverage Goals

### Critical Path (Must Have)
- ✅ Playground: Load, compile, render
- ✅ Navigation: Basic navigation
- ✅ Theme: Toggle functionality

### Important Features (Should Have)
- ✅ Playground: All controls
- ✅ Navigation: All features
- ✅ Code blocks: Basic functionality
- ✅ Responsive: Key breakpoints

### Nice to Have
- ⚪ Search functionality
- ⚪ Course state management
- ⚪ Visual regression tests

---

## Estimated Effort

### Setup (Week 1)
- [ ] Install Playwright
- [ ] Create configuration
- [ ] Set up test utilities
- [ ] Create CI/CD workflow

### Core Tests (Week 2)
- [ ] Playground tests (2-3 days)
- [ ] Navigation tests (1 day)
- [ ] Theme tests (1 day)

### Advanced Tests (Week 3)
- [ ] Code block tests (1-2 days)
- [ ] Responsive tests (1 day)
- [ ] Edge cases and error handling (1 day)

**Total Estimated Effort:** 2-3 weeks

---

## Benefits

### Immediate Benefits
1. **Regression Prevention** ⭐⭐⭐⭐⭐
   - Catch bugs before deployment
   - Prevent breaking changes

2. **Confidence** ⭐⭐⭐⭐⭐
   - Deploy with confidence
   - Know what works

3. **Documentation** ⭐⭐⭐⭐
   - Tests serve as documentation
   - Show how features work

### Long-term Benefits
1. **Maintainability** ⭐⭐⭐⭐⭐
   - Easier refactoring
   - Safer changes
   - Better code quality

2. **Team Collaboration** ⭐⭐⭐⭐
   - Clear expectations
   - Shared understanding
   - Faster onboarding

---

## Challenges & Mitigation

### Challenge 1: WASM Loading Time
**Issue:** WASM can take time to load  
**Mitigation:** Use `waitForWasmReady` helper with timeout

### Challenge 2: Flaky Tests
**Issue:** Tests may be flaky  
**Mitigation:** Use Playwright's auto-waiting, avoid fixed timeouts

### Challenge 3: Test Maintenance
**Issue:** Tests need to be updated with code changes  
**Mitigation:** Use stable selectors, keep tests simple

### Challenge 4: CI/CD Performance
**Issue:** E2E tests can be slow  
**Mitigation:** Run tests in parallel, use test sharding

---

## Recommendation

### ✅ **Proceed with Playwright E2E Testing**

**Rationale:**
1. **Critical Functionality:** Playground and navigation need testing
2. **WASM Support:** Playwright handles WASM well
3. **CI/CD Ready:** Easy to integrate
4. **TypeScript Support:** Aligns with migration plan
5. **High Value:** Prevents regressions, ensures quality

### Implementation Priority

**Phase 1 (Critical):**
1. Playground: Load, compile, render
2. Navigation: Basic functionality
3. Theme: Toggle

**Phase 2 (Important):**
4. Playground: All controls
5. Code blocks: Basic functionality
6. Responsive: Key breakpoints

**Phase 3 (Nice to Have):**
7. Search functionality
8. Visual regression
9. Performance testing

---

## Next Steps

1. **Review this assessment**
2. **Set up Playwright** (Week 1)
3. **Write critical path tests** (Week 2)
4. **Integrate with CI/CD** (Week 2)
5. **Expand test coverage** (Week 3)

---

## References

- [Playwright Documentation](https://playwright.dev/)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Testing WASM with Playwright](https://playwright.dev/docs/api/class-page#page-evaluate)

---

**Document Version:** 1.0  
**Last Updated:** 2025-11-28

