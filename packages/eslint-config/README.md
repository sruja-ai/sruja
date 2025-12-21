# @sruja/eslint-config

Shared ESLint configuration for the Sruja monorepo using ESLint v9 flat config format.

## Usage

### Base Configuration (TypeScript)

For packages/apps with TypeScript only:

```javascript
// eslint.config.js
import baseConfig from '@sruja/eslint-config';

export default baseConfig;
```

### React Configuration

For packages/apps with React/TSX:

```javascript
// eslint.config.js
import reactConfig from '@sruja/eslint-config/react';

export default reactConfig;
```

## Features

- ESLint v9 with flat config format
- TypeScript support via `typescript-eslint`
- React support with `eslint-plugin-react` and `eslint-plugin-react-hooks`
- Consistent rules across all packages and apps
- Shared ignore patterns

## Rules

- TypeScript recommended rules
- React recommended rules (for React projects)
- Custom rules:
  - `@typescript-eslint/no-unused-vars`: Warn (with `_` prefix ignored)
  - `@typescript-eslint/no-explicit-any`: Warn
  - `no-console`: Error (allows warn, error, info, debug)

## Ignored Patterns

- `dist/`, `build/`, `node_modules/`
- `*.js`, `*.d.ts`
- `coverage/`, `.turbo/`
- `storybook-static/`, `.astro/`, `out/`

