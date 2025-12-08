// apps/viewer-core/vite.config.app.ts
// Build configuration for viewer-app.js (React UI)
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

import { nodePolyfills } from 'vite-plugin-node-polyfills';

export default defineConfig({
  plugins: [
    react(),
    nodePolyfills({
      include: ['buffer', 'process', 'stream', 'util', 'path', 'events'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
    }),
  ],
  define: {
    'process.env.NODE_ENV': '"production"',
  },
  build: {
    lib: {
      entry: resolve(__dirname, 'app/index.tsx'),
      name: 'SrujaViewerApp',
      formats: ['iife'],
      fileName: 'viewer-app',
    },
    rollupOptions: {
      external: ['cytoscape'],
      output: {
        globals: {
          'cytoscape': 'cytoscape',
        },
      },
    },
    outDir: 'dist',
    emptyOutDir: false, // Don't clear dist when building core
    commonjsOptions: {
      include: [/node_modules/],
      transformMixedEsModules: true,
    },
  },
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
    ],
    exclude: ['@sruja/shared', '@sruja/viewer', '@sruja/ui'],
  },
  resolve: {
    alias: {
      '@sruja/viewer': resolve(__dirname, '../../packages/viewer/src'),
      '@sruja/ui': resolve(__dirname, '../../packages/ui/src'),
      '@sruja/shared': resolve(__dirname, '../../packages/shared/src'),
    },
  },
});
