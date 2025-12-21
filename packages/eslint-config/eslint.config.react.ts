// @sruja/eslint-config/react - ESLint configuration for React projects
// ESLint v9 flat config format

import baseConfig from './eslint.config';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';

export default [
  ...baseConfig,
  
  // React-specific configuration
  {
    files: ['**/*.tsx', '**/*.jsx'],
    plugins: {
      react,
      'react-hooks': reactHooks,
    },
    settings: {
      react: {
        version: 'detect',
      },
    },
    rules: {
      ...react.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      'react/react-in-jsx-scope': 'off', // Not needed with React 17+
      'react/prop-types': 'off', // Using TypeScript for prop validation
    },
  },
];

