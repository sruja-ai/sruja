import { create } from 'storybook/theming'

export const srujaLightTheme = create({
  base: 'light',
  brandTitle: 'Sruja UI',
  brandUrl: 'https://github.com/dilipkola/sruja-lang',
  brandImage: undefined,
  fontBase:
    '-apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu", "Cantarell", "Fira Sans", "Droid Sans", "Helvetica Neue", sans-serif',
  fontCode:
    'Monaco, Menlo, "Ubuntu Mono", Consolas, "Courier New", monospace',
  colorPrimary: '#7c3aed',
  colorSecondary: '#2563EB',
  appBg: '#f8fafc',
  appContentBg: '#ffffff',
  appBorderColor: '#e2e8f0',
  appBorderRadius: 8,
  textColor: '#0f172a',
  barTextColor: '#475569',
  barSelectedColor: '#7c3aed',
  barBg: '#ffffff',
})

export const srujaDarkTheme = create({
  base: 'dark',
  brandTitle: 'Sruja UI',
  brandUrl: 'https://github.com/dilipkola/sruja-lang',
  brandImage: undefined,
  fontBase:
    '-apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu", "Cantarell", "Fira Sans", "Droid Sans", "Helvetica Neue", sans-serif',
  fontCode:
    'Monaco, Menlo, "Ubuntu Mono", Consolas, "Courier New", monospace',
  colorPrimary: '#38bdf8',
  colorSecondary: '#93c5fd',
  appBg: '#0f172a',
  appContentBg: '#1e293b',
  appBorderColor: '#334155',
  appBorderRadius: 8,
  textColor: '#f1f5f9',
  barTextColor: '#cbd5e1',
  barSelectedColor: '#38bdf8',
  barBg: '#0f172a',
})

