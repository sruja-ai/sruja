// apps/viewer-core/vite.config.embed.ts
// Build configuration for embed-viewer.js (UMD/IIFE bundle for HTML export)
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
      entry: resolve(__dirname, 'app/embed.tsx'),
      name: 'SrujaViewer',
      formats: ['iife'],
      fileName: 'embed-viewer',
    },
    rollupOptions: {
      // Bundle EVERYTHING including React - no externals
      external: [],
      output: {
        // Inline all dependencies
        inlineDynamicImports: true,
        globals: {},
        // Extract CSS to separate file
        assetFileNames: (assetInfo) => {
          if (assetInfo.name === 'style.css') {
            return 'embed-viewer.css';
          }
          return assetInfo.name || 'asset';
        },
      },
    },
    outDir: 'dist',
    emptyOutDir: false,
    cssCodeSplit: false, // Single CSS file
    commonjsOptions: {
      include: [/node_modules/],
      transformMixedEsModules: true,
    },
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: false, // Keep console for debugging
      },
    },
  },
  resolve: {
    alias: {
      '@sruja/viewer': resolve(__dirname, '../../packages/viewer/src'),
      '@sruja/ui': resolve(__dirname, '../../packages/ui/src'),
      '@sruja/shared': resolve(__dirname, '../../packages/shared/src'),
    },
  },
});

