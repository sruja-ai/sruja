// apps/viewer-core/vite.config.ts
// Development server configuration
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@sruja/viewer': resolve(__dirname, '../../packages/viewer/src'),
      '@sruja/ui': resolve(__dirname, '../../packages/ui/src'),
      '@sruja/shared': resolve(__dirname, '../../packages/shared/src'),
    },
  },
  server: {
    port: 5173,
    open: true,
  },
});
