// Theme management utilities
import { getStorageItem, setStorageItem } from './storage';

export type Theme = 'dark' | 'light';

const THEME_STORAGE_KEY = 'sruja_theme';

export function applyTheme(theme: Theme): void {
  const rootEl = document.documentElement;
  rootEl.classList.remove('theme-dark', 'theme-light', 'dark');
  if (theme === 'dark') rootEl.classList.add('theme-dark', 'dark');
  else if (theme === 'light') rootEl.classList.add('theme-light');

  const editorBg = theme === 'dark' ? '#0f172a' : '#ffffff';
  const editorFg = theme === 'dark' ? '#e2e8f0' : '#0f172a';
  const editorBorder = theme === 'dark' ? '#475569' : '#cbd5e1';
  rootEl.style.setProperty('--sruja-editor-bg', editorBg);
  rootEl.style.setProperty('--sruja-editor-fg', editorFg);
  rootEl.style.setProperty('--sruja-editor-border', editorBorder);
  
  // Use safe storage utility
  setStorageItem(THEME_STORAGE_KEY, theme);
  
  const btn = document.querySelector('.theme-toggle');
  if (btn) btn.textContent = theme === 'dark' ? '☀️' : '🌙';
}

export function initTheme(): void {
  const savedTheme = getStorageItem(THEME_STORAGE_KEY) as Theme | null;
  if (savedTheme === 'dark' || savedTheme === 'light') {
    applyTheme(savedTheme);
  }
}

export function toggleTheme(): void {
  const current = getStorageItem(THEME_STORAGE_KEY);
  const next: Theme = (current === 'dark') ? 'light' : 'dark';
  applyTheme(next);
}
