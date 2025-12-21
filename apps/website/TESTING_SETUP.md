# Testing Setup Guide

## ✅ Completed Setup

### Test Configuration Files
- `vitest.config.ts` - Unit test configuration with path aliases
- `playwright.config.ts` - E2E test configuration
- `src/shared/__tests__/setup.ts` - Global test setup with mocks

### Test Utilities
- `src/shared/__tests__/utils/render.tsx` - Custom render with providers
- `src/shared/__tests__/mocks/wasm.ts` - WASM API mocks
- `src/shared/__tests__/fixtures/` - Test data fixtures

### Initial Tests
- `src/shared/utils/storage.test.ts` - Storage utility tests
- `src/shared/utils/date.test.ts` - Date utility tests
- `src/shared/utils/analytics.test.ts` - Analytics utility tests
- `__tests__/e2e/viewer.spec.ts` - E2E viewer test

## 📦 Installation

To use the testing infrastructure, install the dependencies:

```bash
npm install
```

The following testing packages are included in `package.json`:
- `vitest` - Unit testing framework
- `@vitest/ui` - Test UI
- `@vitest/coverage-v8` - Coverage reports
- `@testing-library/react` - React component testing
- `@testing-library/jest-dom` - DOM matchers
- `@testing-library/user-event` - User interaction testing
- `@playwright/test` - E2E testing
- `jsdom` - DOM environment for tests

## 🚀 Running Tests

### Unit Tests (Vitest)
```bash
# Run all unit tests
npm run test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage

# Run only unit tests (single run)
npm run test:unit
```

**Note**: Vitest is configured to exclude E2E tests in `__tests__/e2e/` directory.

### E2E Tests
```bash
# Run E2E tests
npm run test:e2e

# Run E2E tests with UI
npm run test:e2e:ui
```

### All Tests
```bash
# Run both unit and E2E tests
npm run test:all
```

## 📁 Test Organization

Tests follow the feature-based structure:

```
src/
├── shared/
│   ├── __tests__/
│   │   ├── setup.ts          # Global setup
│   │   ├── utils/            # Test utilities
│   │   ├── mocks/            # Mock implementations
│   │   └── fixtures/         # Test data
│   └── utils/
│       └── *.test.ts         # Co-located tests
│
└── features/
    └── [feature]/
        ├── components/
        │   └── *.test.tsx    # Component tests
        └── __tests__/        # Integration tests

__tests__/
└── e2e/
    └── *.spec.ts             # E2E tests
```

## 🧪 Writing Tests

### Unit Test Example
```typescript
// src/shared/utils/my-util.test.ts
import { describe, it, expect } from 'vitest';
import { myUtil } from './my-util';

describe('myUtil', () => {
  it('should do something', () => {
    expect(myUtil()).toBe('expected');
  });
});
```

### Component Test Example
```typescript
// src/features/viewer/components/MyComponent.test.tsx
import { render, screen } from '@/shared/__tests__/utils/render';
import { MyComponent } from './MyComponent';

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(<MyComponent />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });
});
```

### E2E Test Example
```typescript
// __tests__/e2e/my-feature.spec.ts
import { test, expect } from '@playwright/test';

test('my feature works', async ({ page }) => {
  await page.goto('/my-feature');
  await expect(page.locator('h1')).toContainText('My Feature');
});
```

## 📊 Coverage

Coverage reports are generated in the `coverage/` directory after running:

```bash
npm run test:coverage
```

Open `coverage/index.html` in your browser to view the coverage report.

## 🎯 Next Steps

1. Install dependencies: `npm install`
2. Run initial tests: `npm run test`
3. Add more tests as you develop features
4. Set up CI/CD to run tests automatically
