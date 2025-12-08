// apps/website/astro.config.mjs
import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import mdx from '@astrojs/mdx';

import tailwindcss from '@tailwindcss/vite';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// https://astro.build/config
export default defineConfig({
  site: process.env.SITE_URL || 'https://sruja.ai',
  base: process.env.BASE_URL || '/',
  markdown: {
    syntaxHighlight: 'shiki',
    shikiConfig: {
      wrap: true,
      themes: {
        light: 'github-light',
        dark: 'github-dark',
      },
      langs: [
        { id: 'sruja', scopeName: 'source.sruja', path: path.resolve(__dirname, './syntaxes/sruja.tmLanguage.json') }
      ],
    },
  },
  integrations: [
    react(),
    mdx(),
  ],
  output: 'static',
  vite: {
    plugins: [
      tailwindcss(),
    ],
    define: {
      'global': 'globalThis',
    },
    optimizeDeps: {
      include: ['react', 'react-dom', 'monaco-editor', 'buffer', 'algoliasearch/lite'],
      exclude: ['@sruja/shared', '@sruja/viewer', '@sruja/ui', '@sruja/studio-core'],
    },
    ssr: {
      // Add all Monaco/vscode packages to noExternal so Vite processes CSS imports
      // They will only run client-side via client:only directive, preventing SSR execution
      noExternal: [
        '@sruja/ui',
        '@sruja/shared',
        '@sruja/studio-core',
        '@sruja/viewer',
        'monaco-editor'
      ],
    },
    resolve: {
      conditions: ['import', 'module', 'browser', 'default'],
      // Ensure CSS files are resolved as raw assets, not modules
      dedupe: ['react', 'react-dom', '@sruja/ui'],
      // Explicitly handle CSS imports from packages
      alias: {
        // Map CSS import to actual file path
        'node:buffer': 'buffer',
        // Feature-based path aliases
        '@': path.resolve(__dirname, './src'),
        '@/features': path.resolve(__dirname, './src/features'),
        '@/shared': path.resolve(__dirname, './src/shared'),
      },
    },
    // Default CSS handling; Tailwind v4 Vite plugin processes imports
  },
});

