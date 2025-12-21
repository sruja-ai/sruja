// @sruja/website - ESLint configuration for Astro + React
import reactConfig from '@sruja/eslint-config/react';

export default [
  ...reactConfig,
  {
    rules: {
      'no-empty': 'off'
    }
  },
  {
    ignores: [
      'public/',
      'scripts/**/*.mjs',
      'astro.config.mjs'
    ]
  }
];
