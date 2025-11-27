import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { viteCommonjs } from '@originjs/vite-plugin-commonjs'
import type { Plugin } from 'vite'

export default defineConfig({
  plugins: [
    react(),
    viteCommonjs(),
  ],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  optimizeDeps: {
    include: ['mermaid'],
    exclude: [],
    esbuildOptions: {
      target: 'esnext',
    },
    force: true,
  },
  build: {
    commonjsOptions: {
      include: [/mermaid/, /node_modules/],
      transformMixedEsModules: true,
      requireReturnsDefault: 'auto',
    },
    rollupOptions: {
      // Don't split dagre and graphlib - keep them together
      output: {},
    },
  },
})

