# @sruja/eslint-config

FAANG-quality ESLint configuration for TypeScript and React projects.

This package provides a comprehensive set of ESLint rules that enforce best practices, type safety, and code quality standards used at top-tier tech companies.

## 🚀 Features

- **TypeScript Strict Mode**: Enforces strict type checking with zero `any` usage
- **React Best Practices**: Optimized for React 18+ with Hooks rules
- **Accessibility**: Full A11y rule set for inclusive components
- **Performance**: Rules to prevent performance issues
- **Code Quality**: Consistent style and organization
- **Test-Specific Rules**: Relaxed rules for test files
- **Monorepo Ready**: Configured for Turborepo and other monorepos

## 📦 Installation

```bash
npm install --save-dev @sruja/eslint-config eslint typescript
```

For React projects, also install:

```bash
npm install --save-dev @sruja/eslint-config eslint typescript eslint-plugin-react eslint-plugin-react-hooks eslint-plugin-jsx-a11y eslint-plugin-react-perf
```

## 🛠️ Usage

### Base TypeScript Configuration

For non-React TypeScript projects:

```typescript
// eslint.config.ts
import srujaEslintConfig from '@sruja/eslint-config';

export default [
  ...srujaEslintConfig,
  {
    // Add your project-specific overrides
    rules: {
      '@typescript-eslint/no-magic-numbers': 'off',
    },
  },
];
```

### React Configuration

For React projects:

```typescript
// eslint.config.ts
import srujaReactConfig from '@sruja/eslint-config/react';

export default [
  ...srujaReactConfig,
  {
    // Add your project-specific overrides
    rules: {
      'react/jsx-no-literals': 'off',
    },
  },
];
```

### Next.js Configuration

For Next.js projects:

```typescript
// eslint.config.ts
import srujaReactConfig from '@sruja/eslint-config/react';

export default [
  ...srujaReactConfig,
  {
    files: ['**/pages/**/*.{ts,tsx}', '**/app/**/*.{ts,tsx}'],
    rules: {
      'react/jsx-key': 'warn',
      'react/no-danger': 'off',
    },
  },
];
```

## 📋 Configuration Options

### TypeScript Rules

The base configuration includes comprehensive TypeScript rules:

**Type Safety (Critical)**
- `no-explicit-any`: Disallows use of `any` type
- `no-unsafe-assignment`: Prevents unsafe type assignments
- `no-unsafe-call`: Prevents unsafe function calls
- `no-unsafe-member-access`: Prevents unsafe property access
- `strict-boolean-expressions`: Enforces strict boolean checks
- `no-floating-promises`: Requires proper promise handling

**Code Quality (High)**
- `ban-ts-comment`: Restricts TypeScript suppressions
- `no-magic-numbers`: Prevents magic numbers (with sensible defaults)
- `no-duplicate-enum-values`: Prevents duplicate enum values
- `no-empty-interface`: Enforces meaningful interfaces

**Best Practices (High)**
- `naming-convention`: Enforces consistent naming
- `consistent-type-definitions`: Prefers interfaces over types
- `consistent-type-imports`: Enforces `import type` for types
- `no-unused-vars`: Prevents unused variables

### React Rules

The React configuration adds React-specific rules:

**Core Rules (Critical)**
- `react-hooks/rules-of-hooks`: Enforces Hooks rules
- `react-hooks/exhaustive-deps`: Ensures dependencies are correct

**Component Design (Critical)**
- `prefer-stateless-function`: Prefers functional components
- `no-danger`: Warns against using `dangerouslySetInnerHTML`
- `no-string-refs`: Disallows string refs

**JSX Quality (High)**
- `jsx-key`: Requires unique keys for lists
- `jsx-no-leaked-render`: Prevents leaked render behavior
- `jsx-sort-props`: Sorts props for consistency
- `jsx-boolean-value`: Enforces consistent boolean props

**Accessibility (High)**
- `jsx-a11y/*`: Full accessibility rule set
- `anchor-has-content`: Requires anchor content
- `control-has-associated-label`: Requires labels for controls

**Performance (High)**
- `react-perf/jsx-no-jsx-as-prop`: Prevents JSX as props
- `react-perf/jsx-no-new-array-as-prop`: Prevents new arrays as props
- `react-perf/jsx-no-new-function-as-prop`: Prevents new functions as props

## 🎯 Rule Enforcement

### Severity Levels

- **Error**: Must be fixed before merging
- **Warn**: Should be fixed, allows exceptions
- **Off**: Rule disabled, documented reason required

### Project-Specific Overrides

Different projects may need different strictness levels:

```typescript
export default [
  ...srujaReactConfig,
  
  // Library package - strictest rules
  {
    files: ['packages/shared/**/*.{ts,tsx}'],
    rules: {
      '@typescript-eslint/explicit-module-boundary-types': 'error',
      'react/function-component-definition': ['error', { namedComponents: 'function-declaration' }],
    },
  },
  
  // Application code - slightly relaxed
  {
    files: ['apps/**/*.{ts,tsx}'],
    rules: {
      '@typescript-eslint/explicit-module-boundary-types': 'off',
    },
  },
  
  // Test files - most relaxed
  {
    files: ['**/*.test.{ts,tsx}', '**/__tests__/**/*.{ts,tsx}'],
    rules: {
      '@typescript-eslint/no-magic-numbers': 'off',
      '@typescript-eslint/no-explicit-any': 'warn',
      'react/jsx-no-literals': 'off',
    },
  },
];
```

## 🔄 Migration Guide

### From Existing Config

1. **Remove old config**:

```bash
rm .eslintrc.js .eslintrc.json .eslintrc.yml
```

2. **Create new config**:

```typescript
// eslint.config.ts
import srujaReactConfig from '@sruja/eslint-config/react';

export default [
  ...srujaReactConfig,
];
```

3. **Fix errors incrementally**:

```bash
# See all errors
npm run lint

# Fix auto-fixable errors
npm run lint:fix
```

### Common Migration Issues

#### Issue: Too many errors on existing codebase

**Solution**: Add temporary overrides:

```typescript
export default [
  ...srujaReactConfig,
  {
    files: ['src/legacy/**/*.{ts,tsx}'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unused-vars': 'warn',
      'react-hooks/exhaustive-deps': 'off',
    },
  },
];
```

#### Issue: Magic numbers rule is too strict

**Solution**: Add additional allowed values:

```typescript
{
  rules: {
    '@typescript-eslint/no-magic-numbers': [
      'warn',
      {
        ignore: [-1, 0, 1, 2, 100, 1000, 60, 24, 365],
        ignoreArrayIndexes: true,
        ignoreDefaultValues: true,
        ignoreEnums: true,
        ignoreNumericLiteralTypes: true,
      },
    ],
  },
}
```

#### Issue: React components need specific prop types

**Solution**: Disable for specific components:

```typescript
{
  overrides: [
    {
      files: ['src/components/Chart/**/*.{ts,tsx}'],
      rules: {
        'react/prop-types': 'off',
        '@typescript-eslint/explicit-module-boundary-types': 'off',
      },
    },
  ],
}
```

## 📚 Best Practices

### Type Safety

1. **Never use `any`** - Use `unknown` when type is truly unknown
2. **Prefer `readonly`** for immutable data
3. **Use branded types** for domain values
4. **Leverage discriminated unions** for state machines

### React Components

1. **Prefer functional components** with hooks
2. **Use `React.memo`** for expensive components
3. **Properly handle async** with loading/error states
4. **Test accessibility** with A11y rules

### Error Handling

1. **Use Result type** for expected errors
2. **Throw only for unexpected errors**
3. **Handle errors** at appropriate boundaries
4. **Log errors** with proper context

## 🔧 Customization

### Adding Project-Specific Rules

```typescript
// eslint.config.ts
import srujaReactConfig from '@sruja/eslint-config/react';

export default [
  ...srujaReactConfig,
  
  // Add your rules
  {
    rules: {
      'no-console': ['warn', { allow: ['warn', 'error'] }],
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
    },
  },
];
```

### Creating Custom Rule Sets

```typescript
// eslint.config.ts
import srujaReactConfig from '@sruja/eslint-config/react';

const strictRules = {
  '@typescript-eslint/explicit-module-boundary-types': 'error',
  '@typescript-eslint/no-implicit-any': 'error',
  'react/prop-types': 'error',
};

const relaxedRules = {
  '@typescript-eslint/explicit-module-boundary-types': 'off',
  '@typescript-eslint/no-implicit-any': 'off',
};

export default [
  ...srujaReactConfig,
  
  // Apply to library code
  {
    files: ['packages/**/*.{ts,tsx}'],
    rules: strictRules,
  },
  
  // Apply to app code
  {
    files: ['apps/**/*.{ts,tsx}'],
    rules: relaxedRules,
  },
];
```

## 🐛 Troubleshooting

### ESLint doesn't recognize config

**Problem**: ESLint ignores your config file

**Solution**:
1. Ensure file is named `eslint.config.ts` (not `.js` or `.mjs`)
2. Make sure you're using ESLint 9+
3. Check that dependencies are installed:
   ```bash
   npm install --save-dev @sruja/eslint-config
   ```

### TypeScript errors in ESLint

**Problem**: ESLint can't find types

**Solution**: Ensure TypeScript is installed and `tsconfig.json` exists:
```bash
npm install --save-dev typescript
```

### Rules conflict with project setup

**Problem**: Rules don't match your project's needs

**Solution**: Add overrides in your config:
```typescript
export default [
  ...srujaReactConfig,
  {
    rules: {
      // Override specific rules
      'rule-name': 'off',
    },
  },
];
```

## 📖 Related Documentation

- [TypeScript Best Practices](./src/docs/TYPESCRIPT_BEST_PRACTICES.md)
- [Testing Standards](./src/testing/TESTING_STANDARDS.md)
- [ESLint Documentation](https://eslint.org/docs/latest/)
- [TypeScript ESLint](https://typescript-eslint.io/)
- [React ESLint Plugin](https://github.com/jsx-eslint/eslint-plugin-react)

## 🤝 Contributing

When contributing to this package:

1. Follow the existing code style
2. Add tests for new rules
3. Update documentation
4. Run linting and tests:
   ```bash
   npm run lint
   npm test
   ```

## 📝 License

Apache-2.0 - See LICENSE file for details

## 🎉 Acknowledgments

This configuration draws inspiration from:
- Airbnb JavaScript Style Guide
- TypeScript ESLint recommended rules
- React ESLint plugin
- FAANG company engineering standards