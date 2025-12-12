// packages/ui/src/components/ThemeProvider.tsx
import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { ThemeMode, getTheme, type ThemeColors } from '../design-system/theme';

interface ThemeContextType {
  theme: ThemeColors;
  mode: ThemeMode;
  setMode: (mode: ThemeMode) => void;
  toggle: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

const THEME_STORAGE_KEY = 'sruja-theme-mode';

export interface ThemeProviderProps {
  children: ReactNode;
  defaultMode?: ThemeMode;
  storageKey?: string;
}

export function ThemeProvider({
  children,
  defaultMode = 'system',
  storageKey = THEME_STORAGE_KEY,
}: ThemeProviderProps) {
  const [mode, setModeState] = useState<ThemeMode>(() => {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem(storageKey) as ThemeMode | null;
      if (stored && ['light', 'dark', 'system'].includes(stored)) {
        return stored;
      }
    }
    return defaultMode;
  });

  const [theme, setTheme] = useState<ThemeColors>(() => getTheme(mode));

  useEffect(() => {
    const currentTheme = getTheme(mode);
    setTheme(currentTheme);

    // Apply theme to document root
    if (typeof document !== 'undefined') {
      const root = document.documentElement;
      const isDark = mode === 'dark' || (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
      
      root.style.setProperty('--color-background', currentTheme.background);
      root.style.setProperty('--color-surface', currentTheme.surface);
      root.style.setProperty('--color-border', currentTheme.border);
      root.style.setProperty('--color-text-primary', currentTheme.text.primary);
      root.style.setProperty('--color-text-secondary', currentTheme.text.secondary);
      root.style.setProperty('--color-text-tertiary', currentTheme.text.tertiary);
      root.style.setProperty('--color-primary', currentTheme.primary.default);
      root.style.setProperty('--color-primary-hover', currentTheme.primary.hover);
      root.style.setProperty('--color-primary-active', currentTheme.primary.active);
      root.style.setProperty('--color-success-500', currentTheme.semantic.success);
      root.style.setProperty('--color-error-500', currentTheme.semantic.error);
      root.style.setProperty('--color-warning-500', currentTheme.semantic.warning);
      root.style.setProperty('--color-info-500', currentTheme.semantic.info);
      
      root.classList.toggle('dark', isDark);
      root.classList.toggle('light', !isDark);
      root.setAttribute('data-theme', isDark ? 'dark' : 'light');
    }

    // Store preference
    if (typeof window !== 'undefined') {
      localStorage.setItem(storageKey, mode);
      // Dispatch custom event for cross-component theme synchronization
      window.dispatchEvent(new CustomEvent('theme-change'));
    }
  }, [mode, storageKey]);

  // Listen for system theme changes
  useEffect(() => {
    if (mode !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => {
      setTheme(getTheme('system'));
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [mode]);

  const setMode = (newMode: ThemeMode) => {
    setModeState(newMode);
  };

  const toggle = () => {
    if (mode === 'light') {
      setMode('dark');
    } else if (mode === 'dark') {
      setMode('light');
    } else {
      // If system, toggle to opposite of current system preference
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      setMode(prefersDark ? 'light' : 'dark');
    }
  };

  return (
    <ThemeContext.Provider value={{ theme, mode, setMode, toggle }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}













