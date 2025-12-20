import type { StorybookConfig } from '@storybook/react-vite'
import path, { dirname } from 'path';
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const storybookRoot = path.resolve(__dirname, '..')

const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(ts|tsx)'],
  addons: [
    getAbsolutePath("@storybook/addon-vitest"),
    getAbsolutePath("@storybook/addon-docs"),
    getAbsolutePath("@chromatic-com/storybook"),
    getAbsolutePath("@storybook/addon-a11y")
  ],
  framework: {
    name: getAbsolutePath("@storybook/react-vite"),
    options: {}
  },
  viteFinal: async (cfg) => {
    // Don't override root - let Storybook handle it
    // Only set base path if not already set
    if (!cfg.base) {
      cfg.base = '/'
    }
    
    // Configure server to handle paths correctly
    cfg.server = cfg.server || {}
    cfg.server.fs = cfg.server.fs || {}
    const workspaceRoot = path.resolve(storybookRoot, '../../..')
    cfg.server.fs.allow = [
      ...(cfg.server.fs.allow || []),
      storybookRoot,
      path.resolve(storybookRoot, '..'),
      workspaceRoot,
      path.resolve(workspaceRoot, 'apps/designer'),
    ]
    
    cfg.resolve = cfg.resolve || {}
    cfg.resolve.alias = {
      ...(cfg.resolve.alias || {}),
      '@sruja/shared': path.resolve(__dirname, '../../../packages/shared/src'),
      '@sruja/ui': path.resolve(__dirname, '../../../packages/ui/src'),
    }
    // Ensure designer app dependencies can be resolved
    cfg.resolve.conditions = cfg.resolve.conditions || []
    if (!cfg.resolve.conditions.includes('import')) {
      cfg.resolve.conditions.push('import')
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

function getAbsolutePath(value: string): any {
  return dirname(fileURLToPath(import.meta.resolve(`${value}/package.json`)));
}
