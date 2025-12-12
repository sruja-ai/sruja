# Test Coverage Setup

## Overview

Test coverage reporting is configured for TypeScript/JavaScript packages using Vitest and Codecov.

## Coverage Thresholds

### @sruja/shared

- Lines: 60%
- Functions: 60%
- Branches: 50%
- Statements: 60%

### @sruja/ui

- Lines: 50%
- Functions: 50%
- Branches: 40%
- Statements: 50%

## Running Coverage Locally

```bash
# Run coverage for all packages
npm run test:coverage

# Run coverage for specific package
npm run test:coverage --filter=@sruja/shared
npm run test:coverage --filter=@sruja/ui
```

## CI Integration

Coverage is automatically:

1. Generated during CI test runs
2. Uploaded to Codecov
3. Displayed in PR comments
4. Tracked over time

## Codecov Setup

1. Sign up at https://codecov.io
2. Add repository
3. Get token from Codecov dashboard
4. Add token to GitHub secrets as `CODECOV_TOKEN`
5. Coverage will automatically upload on each CI run

## Viewing Coverage

- **Local**: Open `coverage/index.html` in browser after running `npm run test:coverage`
- **CI**: View in Codecov dashboard or PR comments
- **Badge**: Coverage badge in README shows current coverage

## Improving Coverage

1. Run `npm run test:coverage` locally
2. Open `coverage/index.html` to see uncovered lines
3. Add tests for uncovered code
4. Re-run to verify improvement
