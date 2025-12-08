// @sruja/eslint-config - Base ESLint configuration for TypeScript projects
// ESLint v9 flat config format

import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  // Base recommended rules
  js.configs.recommended,
  
  // TypeScript recommended rules
  ...tseslint.configs.recommended,
  
  // Custom rules
  {
    files: ['**/*.ts', '**/*.tsx', '**/*.mts'],
    rules: {
      '@typescript-eslint/no-unused-vars': [
        'warn',
        { argsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      'no-console': [
        'error',
        {
          allow: ['warn', 'error', 'info', 'debug'],
        },
      ],
    },
  },
  
  // Allow console in scripts (CLI tools)
  {
    files: ['**/scripts/**/*.ts', '**/scripts/**/*.mts'],
    rules: {
      'no-console': 'off', // Scripts are CLI tools, console is appropriate
    },
  },
  
  // Ignore patterns
  {
    ignores: [
      'dist/',
      'build/',
      'node_modules/',
      '*.js',
      '*.d.ts',
      'coverage/',
      '.turbo/',
      'pkg/export/html/*.html',
      'storybook-static/',
      '.astro/',
      'out/',
    ],
  },
);

