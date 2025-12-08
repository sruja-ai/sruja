// packages/shared/vitest.config.ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'jsdom', // Use jsdom for browser APIs like localStorage and IndexedDB
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'json-summary', 'html', 'lcov'],
      exclude: [
        'node_modules/',
        'dist/',
        '**/*.config.*',
        '**/scripts/**',
        '**/*.test.*',
        '**/*.spec.*',
      ],
      thresholds: {
        lines: 55,
        functions: 60,
        branches: 40,
        statements: 55,
      },
    },
  },
});

