// Theme management utilities

export type Theme = 'dark' | 'light';

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
  localStorage.setItem('sruja_theme', theme);
  const btn = document.querySelector('.theme-toggle');
  if (btn) btn.textContent = theme === 'dark' ? '☀️' : '🌙';
}

export function initTheme(): void {
  const savedTheme = localStorage.getItem('sruja_theme') as Theme | null;
  if (savedTheme) applyTheme(savedTheme);
}

export function toggleTheme(): void {
  const next: Theme = (localStorage.getItem('sruja_theme') === 'dark') ? 'light' : 'dark';
  applyTheme(next);
}
