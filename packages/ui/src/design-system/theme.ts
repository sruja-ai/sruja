// packages/ui/src/design-system/theme.ts
// Theme system with dark/light mode support

export type ThemeMode = 'light' | 'dark' | 'system';

export interface ThemeColors {
  background: string;
  surface: string;
  border: string;
  text: {
    primary: string;
    secondary: string;
    tertiary: string;
  };
  primary: {
    default: string;
    hover: string;
    active: string;
  };
  neutral: {
    50: string;
    100: string;
    200: string;
    300: string;
    400: string;
    500: string;
    600: string;
    700: string;
    800: string;
    900: string;
  };
  semantic: {
    success: string;
    error: string;
    warning: string;
    info: string;
  };
}

export const lightTheme: ThemeColors = {
  background: '#ffffff',
  surface: '#f8fafc',
  border: '#e2e8f0',
  text: {
    primary: '#0f172a',
    secondary: '#475569',
    tertiary: '#64748b',
  },
  primary: {
    default: '#7c3aed',
    hover: '#6d28d9',
    active: '#5b21b6',
  },
  neutral: {
    50: '#f8fafc',
    100: '#f1f5f9',
    200: '#e2e8f0',
    300: '#cbd5e1',
    400: '#94a3b8',
    500: '#64748b',
    600: '#475569',
    700: '#334155',
    800: '#1e293b',
    900: '#0f172a',
  },
  semantic: {
    success: '#22c55e',
    error: '#ef4444',
    warning: '#f59e0b',
    info: '#3b82f6',
  },
};

export const darkTheme: ThemeColors = {
  background: '#0f172a',
  surface: '#1e293b',
  border: '#334155',
  text: {
    primary: '#f1f5f9',
    secondary: '#cbd5e1',
    tertiary: '#94a3b8',
  },
  primary: {
    default: '#a78bfa',
    hover: '#c4b5fd',
    active: '#8b5cf6',
  },
  neutral: {
    50: '#020617',
    100: '#0f172a',
    200: '#1e293b',
    300: '#334155',
    400: '#475569',
    500: '#64748b',
    600: '#94a3b8',
    700: '#cbd5e1',
    800: '#e2e8f0',
    900: '#f1f5f9',
  },
  semantic: {
    success: '#22c55e',
    error: '#f87171',
    warning: '#fbbf24',
    info: '#60a5fa',
  },
};

export function getTheme(mode: ThemeMode = 'system'): ThemeColors {
  if (mode === 'system') {
    if (typeof window !== 'undefined') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      return prefersDark ? darkTheme : lightTheme;
    }
    return lightTheme;
  }
  return mode === 'dark' ? darkTheme : lightTheme;
}






















