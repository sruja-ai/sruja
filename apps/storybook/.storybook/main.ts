import type { StorybookConfig } from '@storybook/react-vite'
import path from 'path'
import { fileURLToPath } from 'url'

const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(ts|tsx)'],
  addons: ['@storybook/addon-vitest', '@storybook/addon-docs', '@storybook/addon-viewport', '@chromatic-com/storybook', '@storybook/addon-a11y'],
  framework: {
    name: '@storybook/react-vite',
    options: {}
  },
  viteFinal: async (cfg) => {
    const dirname = path.dirname(fileURLToPath(import.meta.url))
    cfg.resolve = cfg.resolve || {}
    cfg.resolve.alias = {
      ...(cfg.resolve.alias || {}),
      '@sruja/shared': path.resolve(dirname, '../../../packages/shared/src'),
      '@sruja/viewer': path.resolve(dirname, '../../../packages/viewer/src'),
      '@sruja/viewer-core': path.resolve(dirname, '../../viewer-core/app'),
      '@sruja/architecture-visualizer': path.resolve(dirname, '../../architecture-visualizer/src'),
      '@sruja/layout': path.resolve(dirname, '../../../packages/layout/src'),
    }
    cfg.optimizeDeps = cfg.optimizeDeps || {}
    cfg.optimizeDeps.exclude = [
      ...((cfg.optimizeDeps as any).exclude || []),
      'monaco-languageclient',
      '@codingame/monaco-jsonrpc',
      '@codingame/monaco-vscode-api',
      '@codingame/monaco-vscode-api/*',
      '@codingame/*'
    ]
    cfg.build = cfg.build || {}
    cfg.build.rollupOptions = {
      ...cfg.build.rollupOptions,
      external: [
        ...((cfg.build.rollupOptions?.external as any) || []),
        '@codingame/monaco-vscode-api/*',
        '@codingame/*'
      ]
    }
    return cfg
  }
}

export default config
