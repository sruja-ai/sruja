// apps/viewer-core/vite.config.core.ts
// Build configuration for viewer-core.js (Cytoscape engine)
import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  resolve: {
    alias: {
      '@sruja/shared': resolve(__dirname, '../../packages/shared/src'),
      '@sruja/viewer': resolve(__dirname, '../../packages/viewer/src'),
      '@sruja/ui': resolve(__dirname, '../../packages/ui/src'),
    },
  },
  build: {
    lib: {
      entry: resolve(__dirname, 'core/index.ts'),
      name: 'SrujaViewerCore',
      formats: ['iife'],
      fileName: 'viewer-core',
    },
    rollupOptions: {
      external: [], // Bundle everything including cytoscape
      output: {
        globals: {},
      },
    },
    outDir: 'dist',
    emptyOutDir: false, // Don't clear dist when building app
  },
});
