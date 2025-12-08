import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: [],
    globals: true,
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    exclude: [
      '**/node_modules/**',
      '**/dist/**',
      '**/.{idea,git,cache,output,temp}/**',
      '**/{karma,rollup,webpack,vite,vitest,jest,ava,babel,nyc,cypress,tsup,build,eslint,prettier}.config.*',
      'tests/**',
    ],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: [
        'src/utils/**/*.{ts,tsx}',
        'src/handlers/**/*.{ts,tsx}',
        'src/context/**/*.{ts,tsx}',
      ],
      exclude: [
        'src/components/**',
        'src/handlers/explorerHandlers.ts',
        'src/handlers/modalHandlers.ts',
        'src/handlers/monacoHandlers.ts',
        'src/handlers/pasteHandler.ts',
        'src/utils/exportUtils.ts',
        'src/utils/viewerUtils.ts',
        'src/utils/commands.tsx',
        '**/__tests__/**',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        '**/node_modules/**',
        '**/dist/**',
        '**/*.config.{ts,js}',
        'tests/**',
      ],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
