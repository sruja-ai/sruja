import { build } from 'esbuild';

await build({
  entryPoints: ['src/extension.ts'],
  bundle: true,
  platform: 'node',
  format: 'cjs',
  target: ['node18'],
  sourcemap: true,
  external: ['vscode'],
  outfile: 'dist/extension.js',
  logLevel: 'info',
});

