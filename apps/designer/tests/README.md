# E2E Tests for Designer App

## Overview

This directory contains end-to-end tests for the designer app using Playwright. Tests are configured to run against the **built production code** by default to ensure we validate the actual production build.

## Test Structure

### Core Tests

- **`app.spec.ts`** - Core app functionality:
  - App loading and empty state
  - Demo architecture loading
  - View tabs visibility and navigation (including Governance tab)
  - URL state persistence
  - Examples menu functionality
  - **Updated**: Fixed selectors to match actual implementation (ReactFlow instead of LikeC4 canvas)

- **`builder.spec.ts`** - Builder wizard:
  - Wizard steps display
  - Step navigation
  - DSL preview functionality

- **`diagram.spec.ts`** - Diagram view:
  - SVG rendering
  - Canvas interactivity
  - Level parameter handling

- **`details.spec.ts`** - Details view:
  - Details view display
  - Content sections

- **`code-panel.spec.ts`** - Code panel:
  - Code panel display
  - Code tabs (DSL, JSON, Markdown)
  - Content rendering

- **`navigation.spec.ts`** - Navigation:
  - Header visibility
  - Examples button
  - Navigation panel toggle

- **`import-export.spec.ts`** - Import/Export:
  - Export menu accessibility
  - Import button accessibility

- **`user-features.spec.ts`** - Comprehensive user-facing features:
  - Export functionality (DSL, PNG, SVG)
  - Import functionality
  - Share functionality
  - Tab navigation
  - Level navigation (breadcrumb, drill-down)
  - Node selection
  - Example selection
  - Project creation
  - **Updated**: Fixed base path and improved selectors

- **`integration-workflows.spec.ts`** - End-to-end integration tests:
  - Complete user journeys
  - Multi-step workflows (load → navigate → export)
  - Mode switching (view ↔ edit)
  - Error handling scenarios
  - Performance validation
  - **New**: Comprehensive workflow testing

- **`ecommerce-quality.spec.ts`** - Quality measurement (specialized test)

### Archived Tests

See `archive/README.md` for details on archived tests that were consolidated or may not be relevant to the current app state.

## Running Tests

### Run against built code (default)

```bash
npm run test:e2e
```

This will:

1. Clean the dist directory
2. Build the app with `BASE_PATH=/` for testing
3. Start the preview server
4. Run all Playwright tests

### Run against dev server (faster iteration)

```bash
npm run test:e2e:dev
```

### Run specific test file

```bash
npm run test:e2e -- tests/app.spec.ts
```

### Run specific test

```bash
npm run test:e2e -- tests/app.spec.ts -g "loads demo architecture"
```

## Configuration

Tests are configured in `playwright.config.ts`:

- Default port: `4173`
- Default host: `127.0.0.1`
- Tests run against preview server (built code) by default
- Set `PLAYWRIGHT_TEST_USE_DEV=true` to use dev server

## Environment Variables

- `PLAYWRIGHT_TEST_SERVER_CMD` - Custom server start command
- `PLAYWRIGHT_TEST_BASE_URL` - Custom base URL if server runs elsewhere
- `PLAYWRIGHT_TEST_NO_SERVER` - Skip starting a webServer (expects externally running app)
- `PLAYWRIGHT_TEST_USE_DEV` - Set to "true" to use dev server instead of preview
- `PLAYWRIGHT_TEST_PORT` - Custom port (default: 4173)
- `PLAYWRIGHT_TEST_HOST` - Custom host (default: 127.0.0.1)

## Test Best Practices

1. **Always load demo first** - Most tests require an architecture to be loaded
2. **Use proper selectors** - Prefer data-testid, aria-labels, or semantic selectors
3. **Wait for elements** - Use `waitForSelector` with appropriate timeouts
4. **Handle conditional UI** - Use `.isVisible().catch(() => false)` for optional elements
5. **Test production build** - Default to testing built code, not dev server

## Recent Improvements

See `E2E_IMPROVEMENTS.md` for detailed information about recent test improvements, including:

- Fixed selectors to match actual component implementation
- Corrected base path for test environment
- Added comprehensive integration workflow tests
- Improved error handling and timeout management

## Notes

- The build process requires file system permissions that may not be available in sandboxed environments
- Tests are designed to be resilient to UI changes by using multiple selector strategies
- All tests pass Codacy quality checks
- **Important**: Tests use `BASE_PATH=/` for test builds (see `build:test` script in package.json)
- Playwright browsers must be installed: `npx playwright install chromium`
